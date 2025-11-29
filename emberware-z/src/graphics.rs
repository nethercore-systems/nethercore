//! Emberware Z graphics backend (wgpu)
//!
//! Implements the `Graphics` trait from emberware-core with a wgpu-based
//! renderer featuring PS1/N64 aesthetic (vertex jitter, affine textures).
//!
//! # Vertex Buffer Architecture
//!
//! Each vertex format gets its own buffer to avoid padding waste:
//! - FORMAT_UV (1): Has UV coordinates
//! - FORMAT_COLOR (2): Has per-vertex color (RGB, 3 floats)
//! - FORMAT_NORMAL (4): Has normals
//! - FORMAT_SKINNED (8): Has bone indices/weights
//!
//! All 16 combinations are supported (8 base + 8 skinned variants).
//!
//! # Command Buffer Pattern
//!
//! Immediate-mode draws are buffered on the CPU side and flushed once per frame
//! to minimize draw calls. Retained meshes are stored separately.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use glam::{Mat4, Vec4};
use wgpu::util::DeviceExt;
use winit::window::Window;

use emberware_core::console::Graphics;

use crate::console::VRAM_LIMIT;

// ============================================================================
// Vertex Format Flags
// ============================================================================

/// Vertex format flag: Has UV coordinates (2 floats)
pub const FORMAT_UV: u8 = 1;
/// Vertex format flag: Has per-vertex color (RGB, 3 floats)
pub const FORMAT_COLOR: u8 = 2;
/// Vertex format flag: Has normals (3 floats)
pub const FORMAT_NORMAL: u8 = 4;
/// Vertex format flag: Has bone indices/weights for skinning (4 u8 + 4 floats)
pub const FORMAT_SKINNED: u8 = 8;

/// All format flags combined
pub const FORMAT_ALL: u8 = FORMAT_UV | FORMAT_COLOR | FORMAT_NORMAL | FORMAT_SKINNED;

/// Number of vertex format permutations (16: 0-15)
pub const VERTEX_FORMAT_COUNT: usize = 16;

/// Calculate vertex stride in bytes for a given format
#[inline]
pub const fn vertex_stride(format: u8) -> u32 {
    let mut stride = 3 * 4; // Position: 3 floats = 12 bytes

    if format & FORMAT_UV != 0 {
        stride += 2 * 4; // UV: 2 floats = 8 bytes
    }
    if format & FORMAT_COLOR != 0 {
        stride += 3 * 4; // Color: 3 floats = 12 bytes
    }
    if format & FORMAT_NORMAL != 0 {
        stride += 3 * 4; // Normal: 3 floats = 12 bytes
    }
    if format & FORMAT_SKINNED != 0 {
        stride += 4 + 4 * 4; // Bone indices (4 u8 = 4 bytes) + weights (4 floats = 16 bytes) = 20 bytes
    }

    stride
}

/// Vertex format information for creating vertex buffer layouts
#[derive(Debug, Clone)]
pub struct VertexFormatInfo {
    /// Format flags (combination of FORMAT_* constants)
    pub format: u8,
    /// Stride in bytes
    pub stride: u32,
    /// Human-readable name for debugging
    pub name: &'static str,
}

impl VertexFormatInfo {
    /// Get vertex format info for a format index
    pub const fn for_format(format: u8) -> Self {
        let name = match format {
            0 => "POS",
            1 => "POS_UV",
            2 => "POS_COLOR",
            3 => "POS_UV_COLOR",
            4 => "POS_NORMAL",
            5 => "POS_UV_NORMAL",
            6 => "POS_COLOR_NORMAL",
            7 => "POS_UV_COLOR_NORMAL",
            8 => "POS_SKINNED",
            9 => "POS_UV_SKINNED",
            10 => "POS_COLOR_SKINNED",
            11 => "POS_UV_COLOR_SKINNED",
            12 => "POS_NORMAL_SKINNED",
            13 => "POS_UV_NORMAL_SKINNED",
            14 => "POS_COLOR_NORMAL_SKINNED",
            15 => "POS_UV_COLOR_NORMAL_SKINNED",
            _ => "UNKNOWN",
        };

        Self {
            format,
            stride: vertex_stride(format),
            name,
        }
    }

    /// Check if this format has UV coordinates
    #[inline]
    pub const fn has_uv(&self) -> bool {
        self.format & FORMAT_UV != 0
    }

    /// Check if this format has per-vertex color
    #[inline]
    pub const fn has_color(&self) -> bool {
        self.format & FORMAT_COLOR != 0
    }

    /// Check if this format has normals
    #[inline]
    pub const fn has_normal(&self) -> bool {
        self.format & FORMAT_NORMAL != 0
    }

    /// Check if this format has skinning data
    #[inline]
    pub const fn has_skinned(&self) -> bool {
        self.format & FORMAT_SKINNED != 0
    }

    /// Create wgpu vertex buffer layout for this format
    pub fn vertex_buffer_layout(&self) -> wgpu::VertexBufferLayout<'static> {
        // Build attribute list based on format
        let attributes = Self::build_attributes(self.format);

        wgpu::VertexBufferLayout {
            array_stride: self.stride as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes,
        }
    }

    /// Build vertex attributes for a format (returns static slice)
    fn build_attributes(format: u8) -> &'static [wgpu::VertexAttribute] {
        // Pre-computed attribute arrays for each format
        // Position is always at location 0
        match format {
            0 => &[
                // POS only
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
            ],
            1 => &[
                // POS_UV
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
            ],
            2 => &[
                // POS_COLOR
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 2,
                },
            ],
            3 => &[
                // POS_UV_COLOR
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 20,
                    shader_location: 2,
                },
            ],
            4 => &[
                // POS_NORMAL
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 3,
                },
            ],
            5 => &[
                // POS_UV_NORMAL
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 20,
                    shader_location: 3,
                },
            ],
            6 => &[
                // POS_COLOR_NORMAL
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 24,
                    shader_location: 3,
                },
            ],
            7 => &[
                // POS_UV_COLOR_NORMAL
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 20,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 32,
                    shader_location: 3,
                },
            ],
            8 => &[
                // POS_SKINNED
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint8x4,
                    offset: 12,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 16,
                    shader_location: 5,
                },
            ],
            9 => &[
                // POS_UV_SKINNED
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint8x4,
                    offset: 20,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 24,
                    shader_location: 5,
                },
            ],
            10 => &[
                // POS_COLOR_SKINNED
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint8x4,
                    offset: 24,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 28,
                    shader_location: 5,
                },
            ],
            11 => &[
                // POS_UV_COLOR_SKINNED
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 20,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint8x4,
                    offset: 32,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 36,
                    shader_location: 5,
                },
            ],
            12 => &[
                // POS_NORMAL_SKINNED
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint8x4,
                    offset: 24,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 28,
                    shader_location: 5,
                },
            ],
            13 => &[
                // POS_UV_NORMAL_SKINNED
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 20,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint8x4,
                    offset: 32,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 36,
                    shader_location: 5,
                },
            ],
            14 => &[
                // POS_COLOR_NORMAL_SKINNED
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 24,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint8x4,
                    offset: 36,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 40,
                    shader_location: 5,
                },
            ],
            15 => &[
                // POS_UV_COLOR_NORMAL_SKINNED
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 20,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 32,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint8x4,
                    offset: 44,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 48,
                    shader_location: 5,
                },
            ],
            _ => &[
                // Fallback: POS only
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
            ],
        }
    }
}

// ============================================================================
// Growable Buffer
// ============================================================================

/// Initial buffer size (64KB)
const INITIAL_BUFFER_SIZE: u64 = 64 * 1024;

/// Growth factor when buffer needs to expand (2x)
const BUFFER_GROWTH_FACTOR: u64 = 2;

/// Auto-growing GPU buffer for vertex/index data
///
/// Grows dynamically during init phase when more data is needed.
/// Avoids frequent reallocation by doubling capacity on growth.
pub struct GrowableBuffer {
    /// The wgpu buffer
    buffer: wgpu::Buffer,
    /// Buffer usage flags
    usage: wgpu::BufferUsages,
    /// Current capacity in bytes
    capacity: u64,
    /// Current used size in bytes
    used: u64,
    /// Debug label
    label: String,
}

impl GrowableBuffer {
    /// Create a new growable buffer with initial capacity
    pub fn new(device: &wgpu::Device, usage: wgpu::BufferUsages, label: &str) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: INITIAL_BUFFER_SIZE,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            usage,
            capacity: INITIAL_BUFFER_SIZE,
            used: 0,
            label: label.to_string(),
        }
    }

    /// Ensure the buffer has enough capacity for additional bytes
    ///
    /// If the buffer needs to grow, creates a new larger buffer and returns true.
    /// The old buffer contents are NOT preserved (call this before writing).
    pub fn ensure_capacity(&mut self, device: &wgpu::Device, additional_bytes: u64) -> bool {
        let required = self.used + additional_bytes;
        if required <= self.capacity {
            return false;
        }

        // Calculate new capacity (at least double, or enough for required)
        let mut new_capacity = self.capacity * BUFFER_GROWTH_FACTOR;
        while new_capacity < required {
            new_capacity *= BUFFER_GROWTH_FACTOR;
        }

        tracing::debug!(
            "Growing buffer '{}': {} -> {} bytes",
            self.label,
            self.capacity,
            new_capacity
        );

        // Create new buffer
        self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&self.label),
            size: new_capacity,
            usage: self.usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.capacity = new_capacity;
        // Reset used since data needs to be re-uploaded
        self.used = 0;

        true
    }

    /// Write data to the buffer at the current position
    ///
    /// Returns the byte offset where data was written.
    /// Panics if there's not enough capacity (call ensure_capacity first).
    pub fn write(&mut self, queue: &wgpu::Queue, data: &[u8]) -> u64 {
        let offset = self.used;
        assert!(
            offset + data.len() as u64 <= self.capacity,
            "Buffer overflow: {} + {} > {}",
            offset,
            data.len(),
            self.capacity
        );

        queue.write_buffer(&self.buffer, offset, data);
        self.used += data.len() as u64;

        offset
    }

    /// Reset the used counter (for per-frame immediate mode buffers)
    pub fn reset(&mut self) {
        self.used = 0;
    }

    /// Get the underlying wgpu buffer
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Get current used bytes
    pub fn used(&self) -> u64 {
        self.used
    }

    /// Get current capacity in bytes
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    /// Get a buffer slice for the used portion
    pub fn slice(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(0..self.used)
    }
}

// ============================================================================
// Mesh Handle
// ============================================================================

/// Handle to a retained mesh
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshHandle(pub u32);

impl MeshHandle {
    /// Invalid/null mesh handle
    pub const INVALID: MeshHandle = MeshHandle(0);
}

/// Stored mesh data for retained mode drawing
#[derive(Debug)]
pub struct RetainedMesh {
    /// Vertex format flags
    pub format: u8,
    /// Number of vertices
    pub vertex_count: u32,
    /// Number of indices (0 for non-indexed)
    pub index_count: u32,
    /// Byte offset into the format's vertex buffer
    pub vertex_offset: u64,
    /// Byte offset into the format's index buffer (if indexed)
    pub index_offset: u64,
}

// ============================================================================
// Draw Commands
// ============================================================================

/// A draw command for batching
#[derive(Debug, Clone)]
pub struct DrawCommand {
    /// Vertex format
    pub format: u8,
    /// Model transform matrix
    pub transform: Mat4,
    /// Number of vertices to draw
    pub vertex_count: u32,
    /// Number of indices (0 for non-indexed)
    pub index_count: u32,
    /// Base vertex index in the immediate buffer
    pub base_vertex: u32,
    /// First index in the immediate index buffer
    pub first_index: u32,
    /// Texture slots bound for this draw
    pub texture_slots: [TextureHandle; 4],
    /// Uniform color
    pub color: u32,
    /// Render state at time of draw
    pub depth_test: bool,
    pub cull_mode: CullMode,
    pub blend_mode: BlendMode,
}

/// Command buffer for batching immediate-mode draws
pub struct CommandBuffer {
    /// Draw commands accumulated this frame
    commands: Vec<DrawCommand>,
    /// Per-format immediate vertex data (CPU side)
    vertex_data: [Vec<u8>; VERTEX_FORMAT_COUNT],
    /// Per-format immediate index data (CPU side)
    index_data: [Vec<u32>; VERTEX_FORMAT_COUNT],
    /// Per-format vertex counts for base_vertex calculation
    vertex_counts: [u32; VERTEX_FORMAT_COUNT],
    /// Per-format index counts
    index_counts: [u32; VERTEX_FORMAT_COUNT],
}

impl CommandBuffer {
    /// Create a new command buffer
    pub fn new() -> Self {
        Self {
            commands: Vec::with_capacity(1024),
            vertex_data: std::array::from_fn(|_| Vec::with_capacity(64 * 1024)),
            index_data: std::array::from_fn(|_| Vec::with_capacity(16 * 1024)),
            vertex_counts: [0; VERTEX_FORMAT_COUNT],
            index_counts: [0; VERTEX_FORMAT_COUNT],
        }
    }

    /// Add vertices for immediate drawing (non-indexed)
    ///
    /// Returns the base vertex index for this batch.
    pub fn add_vertices(&mut self, format: u8, vertices: &[f32], transform: Mat4, state: &RenderState) -> u32 {
        let format_idx = format as usize;
        let stride = vertex_stride(format) as usize;
        let vertex_count = (vertices.len() * 4) / stride;
        let base_vertex = self.vertex_counts[format_idx];

        // Append vertex data
        let byte_data = bytemuck::cast_slice(vertices);
        self.vertex_data[format_idx].extend_from_slice(byte_data);
        self.vertex_counts[format_idx] += vertex_count as u32;

        // Record draw command
        self.commands.push(DrawCommand {
            format,
            transform,
            vertex_count: vertex_count as u32,
            index_count: 0,
            base_vertex,
            first_index: 0,
            texture_slots: state.texture_slots,
            color: state.color,
            depth_test: state.depth_test,
            cull_mode: state.cull_mode,
            blend_mode: state.blend_mode,
        });

        base_vertex
    }

    /// Add indexed vertices for immediate drawing
    ///
    /// Returns (base_vertex, first_index) for this batch.
    pub fn add_vertices_indexed(
        &mut self,
        format: u8,
        vertices: &[f32],
        indices: &[u32],
        transform: Mat4,
        state: &RenderState,
    ) -> (u32, u32) {
        let format_idx = format as usize;
        let stride = vertex_stride(format) as usize;
        let vertex_count = (vertices.len() * 4) / stride;
        let base_vertex = self.vertex_counts[format_idx];
        let first_index = self.index_counts[format_idx];

        // Append vertex data
        let byte_data = bytemuck::cast_slice(vertices);
        self.vertex_data[format_idx].extend_from_slice(byte_data);
        self.vertex_counts[format_idx] += vertex_count as u32;

        // Append index data
        self.index_data[format_idx].extend_from_slice(indices);
        self.index_counts[format_idx] += indices.len() as u32;

        // Record draw command
        self.commands.push(DrawCommand {
            format,
            transform,
            vertex_count: vertex_count as u32,
            index_count: indices.len() as u32,
            base_vertex,
            first_index,
            texture_slots: state.texture_slots,
            color: state.color,
            depth_test: state.depth_test,
            cull_mode: state.cull_mode,
            blend_mode: state.blend_mode,
        });

        (base_vertex, first_index)
    }

    /// Get accumulated commands
    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    /// Get vertex data for a format
    pub fn vertex_data(&self, format: u8) -> &[u8] {
        &self.vertex_data[format as usize]
    }

    /// Get index data for a format
    pub fn index_data(&self, format: u8) -> &[u32] {
        &self.index_data[format as usize]
    }

    /// Reset the command buffer for the next frame
    pub fn reset(&mut self) {
        self.commands.clear();
        for data in &mut self.vertex_data {
            data.clear();
        }
        for data in &mut self.index_data {
            data.clear();
        }
        self.vertex_counts = [0; VERTEX_FORMAT_COUNT];
        self.index_counts = [0; VERTEX_FORMAT_COUNT];
    }
}

impl Default for CommandBuffer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Texture Handle
// ============================================================================

/// Handle to a loaded texture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub u32);

impl TextureHandle {
    /// Invalid/null texture handle
    pub const INVALID: TextureHandle = TextureHandle(0);
}

// ============================================================================
// Render State Enums
// ============================================================================

/// Cull mode for face culling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum CullMode {
    /// No face culling
    #[default]
    None = 0,
    /// Cull back faces
    Back = 1,
    /// Cull front faces
    Front = 2,
}

impl CullMode {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => CullMode::None,
            1 => CullMode::Back,
            2 => CullMode::Front,
            _ => CullMode::None,
        }
    }

    pub fn to_wgpu(self) -> Option<wgpu::Face> {
        match self {
            CullMode::None => None,
            CullMode::Back => Some(wgpu::Face::Back),
            CullMode::Front => Some(wgpu::Face::Front),
        }
    }
}

/// Blend mode for alpha blending
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum BlendMode {
    /// No blending (opaque)
    #[default]
    None = 0,
    /// Standard alpha blending
    Alpha = 1,
    /// Additive blending
    Additive = 2,
    /// Multiply blending
    Multiply = 3,
}

impl BlendMode {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => BlendMode::None,
            1 => BlendMode::Alpha,
            2 => BlendMode::Additive,
            3 => BlendMode::Multiply,
            _ => BlendMode::None,
        }
    }

    pub fn to_wgpu(self) -> Option<wgpu::BlendState> {
        match self {
            BlendMode::None => None,
            BlendMode::Alpha => Some(wgpu::BlendState::ALPHA_BLENDING),
            BlendMode::Additive => Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
            BlendMode::Multiply => Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::Dst,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::DstAlpha,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
        }
    }
}

/// Texture filter mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum TextureFilter {
    /// Nearest neighbor (pixelated)
    #[default]
    Nearest = 0,
    /// Linear interpolation (smooth)
    Linear = 1,
}

impl TextureFilter {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => TextureFilter::Nearest,
            1 => TextureFilter::Linear,
            _ => TextureFilter::Nearest,
        }
    }

    pub fn to_wgpu(self) -> wgpu::FilterMode {
        match self {
            TextureFilter::Nearest => wgpu::FilterMode::Nearest,
            TextureFilter::Linear => wgpu::FilterMode::Linear,
        }
    }
}

// ============================================================================
// Render State
// ============================================================================

/// Current render state (tracks what needs pipeline changes)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderState {
    /// Uniform tint color (0xRRGGBBAA)
    pub color: u32,
    /// Depth test enabled
    pub depth_test: bool,
    /// Face culling mode
    pub cull_mode: CullMode,
    /// Blending mode
    pub blend_mode: BlendMode,
    /// Texture filter mode
    pub texture_filter: TextureFilter,
    /// Bound textures per slot (0-3)
    pub texture_slots: [TextureHandle; 4],
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            color: 0xFFFFFFFF, // White, fully opaque
            depth_test: true,
            cull_mode: CullMode::Back,
            blend_mode: BlendMode::None,
            texture_filter: TextureFilter::Nearest,
            texture_slots: [TextureHandle::INVALID; 4],
        }
    }
}

impl RenderState {
    /// Get color as Vec4 (RGBA, 0.0-1.0)
    pub fn color_vec4(&self) -> Vec4 {
        Vec4::new(
            ((self.color >> 24) & 0xFF) as f32 / 255.0,
            ((self.color >> 16) & 0xFF) as f32 / 255.0,
            ((self.color >> 8) & 0xFF) as f32 / 255.0,
            (self.color & 0xFF) as f32 / 255.0,
        )
    }
}

// ============================================================================
// Texture Entry
// ============================================================================

/// Internal texture data
struct TextureEntry {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
    /// Size in bytes (for VRAM tracking)
    size_bytes: usize,
}

// ============================================================================
// ZGraphics
// ============================================================================

/// Emberware Z graphics backend
///
/// Manages wgpu device, textures, render state, and frame presentation.
/// Implements the vertex buffer architecture with one buffer per stride
/// and command buffer pattern for draw batching.
pub struct ZGraphics {
    // Core wgpu objects
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    // Depth buffer
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,

    // Texture management
    textures: HashMap<u32, TextureEntry>,
    next_texture_id: u32,
    vram_used: usize,

    // Fallback textures
    fallback_checkerboard: TextureHandle,
    fallback_white: TextureHandle,

    // Samplers
    sampler_nearest: wgpu::Sampler,
    sampler_linear: wgpu::Sampler,

    // Current render state
    render_state: RenderState,

    // Frame state
    current_frame: Option<wgpu::SurfaceTexture>,
    current_view: Option<wgpu::TextureView>,

    // Vertex buffer architecture
    // Per-format vertex buffers (one for each of 16 vertex formats)
    vertex_buffers: [GrowableBuffer; VERTEX_FORMAT_COUNT],
    // Per-format index buffers
    index_buffers: [GrowableBuffer; VERTEX_FORMAT_COUNT],

    // Retained mesh storage
    retained_meshes: HashMap<u32, RetainedMesh>,
    next_mesh_id: u32,

    // Command buffer for immediate mode draws
    command_buffer: CommandBuffer,

    // Current transform matrix (model transform)
    current_transform: Mat4,
    // Transform stack for push/pop
    transform_stack: Vec<Mat4>,
}

impl ZGraphics {
    /// Create a new ZGraphics instance
    ///
    /// This initializes wgpu with the given window and sets up all core resources.
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        // Create wgpu instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface
        let surface = instance
            .create_surface(window)
            .context("Failed to create surface")?;

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .context("Failed to find suitable GPU adapter")?;

        tracing::info!("Using GPU adapter: {:?}", adapter.get_info().name);

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Emberware Z Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .context("Failed to create GPU device")?;

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create depth buffer
        let (depth_texture, depth_view) = Self::create_depth_texture(&device, width, height);

        // Create samplers
        let sampler_nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler Nearest"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let sampler_linear = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler Linear"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create per-format vertex and index buffers
        let vertex_buffers = std::array::from_fn(|i| {
            let info = VertexFormatInfo::for_format(i as u8);
            GrowableBuffer::new(
                &device,
                wgpu::BufferUsages::VERTEX,
                &format!("Vertex Buffer {}", info.name),
            )
        });

        let index_buffers = std::array::from_fn(|i| {
            let info = VertexFormatInfo::for_format(i as u8);
            GrowableBuffer::new(
                &device,
                wgpu::BufferUsages::INDEX,
                &format!("Index Buffer {}", info.name),
            )
        });

        let mut graphics = Self {
            surface,
            device,
            queue,
            config,
            depth_texture,
            depth_view,
            textures: HashMap::new(),
            next_texture_id: 1, // 0 is reserved for INVALID
            vram_used: 0,
            fallback_checkerboard: TextureHandle::INVALID,
            fallback_white: TextureHandle::INVALID,
            sampler_nearest,
            sampler_linear,
            render_state: RenderState::default(),
            current_frame: None,
            current_view: None,
            vertex_buffers,
            index_buffers,
            retained_meshes: HashMap::new(),
            next_mesh_id: 1, // 0 is reserved for INVALID
            command_buffer: CommandBuffer::new(),
            current_transform: Mat4::IDENTITY,
            transform_stack: Vec::with_capacity(16),
        };

        // Create fallback textures
        graphics.create_fallback_textures();

        Ok(graphics)
    }

    /// Create a new ZGraphics instance (blocking version for sync contexts)
    pub fn new_blocking(window: Arc<Window>) -> Result<Self> {
        pollster::block_on(Self::new(window))
    }

    /// Create depth texture and view
    fn create_depth_texture(device: &wgpu::Device, width: u32, height: u32) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    /// Create fallback textures (checkerboard and white)
    fn create_fallback_textures(&mut self) {
        // 8x8 magenta/black checkerboard for missing textures
        let mut checkerboard_data = vec![0u8; 8 * 8 * 4];
        for y in 0..8 {
            for x in 0..8 {
                let idx = (y * 8 + x) * 4;
                let is_magenta = (x + y) % 2 == 0;
                if is_magenta {
                    checkerboard_data[idx] = 255;     // R
                    checkerboard_data[idx + 1] = 0;   // G
                    checkerboard_data[idx + 2] = 255; // B
                    checkerboard_data[idx + 3] = 255; // A
                } else {
                    checkerboard_data[idx] = 0;       // R
                    checkerboard_data[idx + 1] = 0;   // G
                    checkerboard_data[idx + 2] = 0;   // B
                    checkerboard_data[idx + 3] = 255; // A
                }
            }
        }
        self.fallback_checkerboard = self
            .load_texture_internal(8, 8, &checkerboard_data, false)
            .expect("Failed to create checkerboard fallback texture");

        // 1x1 white texture for untextured draws
        let white_data = [255u8, 255, 255, 255];
        self.fallback_white = self
            .load_texture_internal(1, 1, &white_data, false)
            .expect("Failed to create white fallback texture");
    }

    // ========================================================================
    // Texture Management
    // ========================================================================

    /// Load a texture from RGBA8 pixel data
    ///
    /// Returns a TextureHandle or an error if VRAM budget is exceeded.
    pub fn load_texture(&mut self, width: u32, height: u32, pixels: &[u8]) -> Result<TextureHandle> {
        self.load_texture_internal(width, height, pixels, true)
    }

    /// Internal texture loading (optionally tracks VRAM)
    fn load_texture_internal(
        &mut self,
        width: u32,
        height: u32,
        pixels: &[u8],
        track_vram: bool,
    ) -> Result<TextureHandle> {
        let expected_size = (width * height * 4) as usize;
        if pixels.len() != expected_size {
            anyhow::bail!(
                "Pixel data size mismatch: expected {} bytes, got {}",
                expected_size,
                pixels.len()
            );
        }

        let size_bytes = expected_size;

        // Check VRAM budget
        if track_vram && self.vram_used + size_bytes > VRAM_LIMIT {
            anyhow::bail!(
                "VRAM budget exceeded: {} + {} > {} bytes",
                self.vram_used,
                size_bytes,
                VRAM_LIMIT
            );
        }

        // Create texture
        let texture = self.device.create_texture_with_data(
            &self.queue,
            &wgpu::TextureDescriptor {
                label: Some("Game Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            pixels,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let handle = TextureHandle(self.next_texture_id);
        self.next_texture_id += 1;

        self.textures.insert(
            handle.0,
            TextureEntry {
                texture,
                view,
                width,
                height,
                size_bytes,
            },
        );

        if track_vram {
            self.vram_used += size_bytes;
        }

        tracing::debug!(
            "Loaded texture {}: {}x{}, {} bytes (VRAM: {}/{})",
            handle.0,
            width,
            height,
            size_bytes,
            self.vram_used,
            VRAM_LIMIT
        );

        Ok(handle)
    }

    /// Get texture view by handle
    pub fn get_texture_view(&self, handle: TextureHandle) -> Option<&wgpu::TextureView> {
        self.textures.get(&handle.0).map(|t| &t.view)
    }

    /// Get fallback checkerboard texture view
    pub fn get_fallback_checkerboard_view(&self) -> &wgpu::TextureView {
        &self.textures[&self.fallback_checkerboard.0].view
    }

    /// Get fallback white texture view
    pub fn get_fallback_white_view(&self) -> &wgpu::TextureView {
        &self.textures[&self.fallback_white.0].view
    }

    /// Get texture view for a slot, returning fallback if unbound
    pub fn get_slot_texture_view(&self, slot: usize) -> &wgpu::TextureView {
        let handle = self.render_state.texture_slots.get(slot).copied().unwrap_or(TextureHandle::INVALID);
        if handle == TextureHandle::INVALID {
            self.get_fallback_white_view()
        } else {
            self.get_texture_view(handle)
                .unwrap_or_else(|| self.get_fallback_checkerboard_view())
        }
    }

    /// Get VRAM usage in bytes
    pub fn vram_used(&self) -> usize {
        self.vram_used
    }

    /// Get VRAM limit in bytes
    pub fn vram_limit(&self) -> usize {
        VRAM_LIMIT
    }

    // ========================================================================
    // Render State
    // ========================================================================

    /// Set uniform tint color (0xRRGGBBAA)
    pub fn set_color(&mut self, color: u32) {
        self.render_state.color = color;
    }

    /// Enable or disable depth testing
    pub fn set_depth_test(&mut self, enabled: bool) {
        self.render_state.depth_test = enabled;
    }

    /// Set face culling mode
    pub fn set_cull_mode(&mut self, mode: CullMode) {
        self.render_state.cull_mode = mode;
    }

    /// Set blend mode
    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.render_state.blend_mode = mode;
    }

    /// Set texture filter mode
    pub fn set_texture_filter(&mut self, filter: TextureFilter) {
        self.render_state.texture_filter = filter;
    }

    /// Bind texture to slot 0 (albedo)
    pub fn bind_texture(&mut self, handle: TextureHandle) {
        self.bind_texture_slot(handle, 0);
    }

    /// Bind texture to a specific slot (0-3)
    pub fn bind_texture_slot(&mut self, handle: TextureHandle, slot: usize) {
        if slot < 4 {
            self.render_state.texture_slots[slot] = handle;
        }
    }

    /// Get current render state
    pub fn render_state(&self) -> &RenderState {
        &self.render_state
    }

    /// Get current sampler based on texture filter setting
    pub fn current_sampler(&self) -> &wgpu::Sampler {
        match self.render_state.texture_filter {
            TextureFilter::Nearest => &self.sampler_nearest,
            TextureFilter::Linear => &self.sampler_linear,
        }
    }

    // ========================================================================
    // Transform Stack
    // ========================================================================

    /// Reset transform to identity matrix
    pub fn transform_identity(&mut self) {
        self.current_transform = Mat4::IDENTITY;
    }

    /// Translate the current transform
    pub fn transform_translate(&mut self, x: f32, y: f32, z: f32) {
        self.current_transform = self.current_transform * Mat4::from_translation(glam::vec3(x, y, z));
    }

    /// Rotate the current transform around an axis (angle in degrees)
    pub fn transform_rotate(&mut self, angle_deg: f32, x: f32, y: f32, z: f32) {
        let axis = glam::vec3(x, y, z).normalize();
        let angle_rad = angle_deg.to_radians();
        self.current_transform = self.current_transform * Mat4::from_axis_angle(axis, angle_rad);
    }

    /// Scale the current transform
    pub fn transform_scale(&mut self, x: f32, y: f32, z: f32) {
        self.current_transform = self.current_transform * Mat4::from_scale(glam::vec3(x, y, z));
    }

    /// Push the current transform onto the stack
    ///
    /// Returns false if the stack is full (max 16 entries)
    pub fn transform_push(&mut self) -> bool {
        if self.transform_stack.len() >= 16 {
            tracing::warn!("Transform stack overflow (max 16)");
            return false;
        }
        self.transform_stack.push(self.current_transform);
        true
    }

    /// Pop the transform from the stack
    ///
    /// Returns false if the stack is empty
    pub fn transform_pop(&mut self) -> bool {
        if let Some(transform) = self.transform_stack.pop() {
            self.current_transform = transform;
            true
        } else {
            tracing::warn!("Transform stack underflow");
            false
        }
    }

    /// Set the current transform from a 4x4 matrix (16 floats, column-major)
    pub fn transform_set(&mut self, matrix: &[f32; 16]) {
        self.current_transform = Mat4::from_cols_array(matrix);
    }

    /// Get the current transform matrix
    pub fn current_transform(&self) -> Mat4 {
        self.current_transform
    }

    // ========================================================================
    // Retained Mesh Loading
    // ========================================================================

    /// Load a non-indexed mesh (retained mode)
    ///
    /// The mesh is stored in the appropriate vertex buffer based on format.
    /// Returns a MeshHandle for use with draw_mesh().
    pub fn load_mesh(&mut self, data: &[f32], format: u8) -> Result<MeshHandle> {
        let format_idx = format as usize;
        if format_idx >= VERTEX_FORMAT_COUNT {
            anyhow::bail!("Invalid vertex format: {}", format);
        }

        let stride = vertex_stride(format) as usize;
        let byte_data = bytemuck::cast_slice(data);
        let vertex_count = byte_data.len() / stride;

        if byte_data.len() % stride != 0 {
            anyhow::bail!(
                "Vertex data size {} is not a multiple of stride {}",
                byte_data.len(),
                stride
            );
        }

        // Ensure buffer has capacity
        self.vertex_buffers[format_idx].ensure_capacity(&self.device, byte_data.len() as u64);

        // Write to buffer
        let vertex_offset = self.vertex_buffers[format_idx].write(&self.queue, byte_data);

        // Create mesh handle
        let handle = MeshHandle(self.next_mesh_id);
        self.next_mesh_id += 1;

        self.retained_meshes.insert(
            handle.0,
            RetainedMesh {
                format,
                vertex_count: vertex_count as u32,
                index_count: 0,
                vertex_offset,
                index_offset: 0,
            },
        );

        tracing::debug!(
            "Loaded mesh {}: {} vertices, format {}",
            handle.0,
            vertex_count,
            VertexFormatInfo::for_format(format).name
        );

        Ok(handle)
    }

    /// Load an indexed mesh (retained mode)
    ///
    /// The mesh is stored in the appropriate vertex and index buffers based on format.
    /// Returns a MeshHandle for use with draw_mesh().
    pub fn load_mesh_indexed(
        &mut self,
        data: &[f32],
        indices: &[u32],
        format: u8,
    ) -> Result<MeshHandle> {
        let format_idx = format as usize;
        if format_idx >= VERTEX_FORMAT_COUNT {
            anyhow::bail!("Invalid vertex format: {}", format);
        }

        let stride = vertex_stride(format) as usize;
        let byte_data = bytemuck::cast_slice(data);
        let vertex_count = byte_data.len() / stride;

        if byte_data.len() % stride != 0 {
            anyhow::bail!(
                "Vertex data size {} is not a multiple of stride {}",
                byte_data.len(),
                stride
            );
        }

        // Ensure vertex buffer has capacity
        self.vertex_buffers[format_idx].ensure_capacity(&self.device, byte_data.len() as u64);

        // Ensure index buffer has capacity
        let index_byte_data: &[u8] = bytemuck::cast_slice(indices);
        self.index_buffers[format_idx].ensure_capacity(&self.device, index_byte_data.len() as u64);

        // Write to buffers
        let vertex_offset = self.vertex_buffers[format_idx].write(&self.queue, byte_data);
        let index_offset = self.index_buffers[format_idx].write(&self.queue, index_byte_data);

        // Create mesh handle
        let handle = MeshHandle(self.next_mesh_id);
        self.next_mesh_id += 1;

        self.retained_meshes.insert(
            handle.0,
            RetainedMesh {
                format,
                vertex_count: vertex_count as u32,
                index_count: indices.len() as u32,
                vertex_offset,
                index_offset,
            },
        );

        tracing::debug!(
            "Loaded indexed mesh {}: {} vertices, {} indices, format {}",
            handle.0,
            vertex_count,
            indices.len(),
            VertexFormatInfo::for_format(format).name
        );

        Ok(handle)
    }

    /// Get mesh info by handle
    pub fn get_mesh(&self, handle: MeshHandle) -> Option<&RetainedMesh> {
        self.retained_meshes.get(&handle.0)
    }

    // ========================================================================
    // Immediate Mode Drawing
    // ========================================================================

    /// Draw triangles immediately (non-indexed)
    ///
    /// Vertices are buffered on the CPU and flushed at frame end.
    pub fn draw_triangles(&mut self, vertices: &[f32], format: u8) {
        if format as usize >= VERTEX_FORMAT_COUNT {
            tracing::warn!("Invalid vertex format for draw_triangles: {}", format);
            return;
        }

        self.command_buffer.add_vertices(
            format,
            vertices,
            self.current_transform,
            &self.render_state,
        );
    }

    /// Draw indexed triangles immediately
    ///
    /// Vertices and indices are buffered on the CPU and flushed at frame end.
    pub fn draw_triangles_indexed(&mut self, vertices: &[f32], indices: &[u32], format: u8) {
        if format as usize >= VERTEX_FORMAT_COUNT {
            tracing::warn!("Invalid vertex format for draw_triangles_indexed: {}", format);
            return;
        }

        self.command_buffer.add_vertices_indexed(
            format,
            vertices,
            indices,
            self.current_transform,
            &self.render_state,
        );
    }

    /// Get the command buffer (for flush/rendering)
    pub fn command_buffer(&self) -> &CommandBuffer {
        &self.command_buffer
    }

    /// Get mutable command buffer
    pub fn command_buffer_mut(&mut self) -> &mut CommandBuffer {
        &mut self.command_buffer
    }

    /// Reset the command buffer for the next frame
    ///
    /// Called automatically at the start of begin_frame, but can be called
    /// manually if needed.
    pub fn reset_command_buffer(&mut self) {
        self.command_buffer.reset();
    }

    // ========================================================================
    // Buffer Access (for rendering)
    // ========================================================================

    /// Get vertex buffer for a format
    pub fn vertex_buffer(&self, format: u8) -> &GrowableBuffer {
        &self.vertex_buffers[format as usize]
    }

    /// Get index buffer for a format
    pub fn index_buffer(&self, format: u8) -> &GrowableBuffer {
        &self.index_buffers[format as usize]
    }

    // ========================================================================
    // Device Access
    // ========================================================================

    /// Get wgpu device reference
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Get wgpu queue reference
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Get surface format
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// Get depth format
    pub fn depth_format(&self) -> wgpu::TextureFormat {
        wgpu::TextureFormat::Depth32Float
    }

    /// Get depth texture view
    pub fn depth_view(&self) -> &wgpu::TextureView {
        &self.depth_view
    }

    /// Get current surface dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }
}

impl Graphics for ZGraphics {
    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);

        // Recreate depth buffer
        let (depth_texture, depth_view) = Self::create_depth_texture(&self.device, width, height);
        self.depth_texture = depth_texture;
        self.depth_view = depth_view;

        tracing::debug!("Resized graphics to {}x{}", width, height);
    }

    fn begin_frame(&mut self) {
        // Reset command buffer for new frame
        self.command_buffer.reset();

        // Reset transform to identity
        self.current_transform = Mat4::IDENTITY;
        self.transform_stack.clear();

        // Acquire next frame
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                // Reconfigure surface and try again
                self.surface.configure(&self.device, &self.config);
                match self.surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(e) => {
                        tracing::error!("Failed to acquire frame after reconfigure: {:?}", e);
                        return;
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to acquire frame: {:?}", e);
                return;
            }
        };

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.current_frame = Some(frame);
        self.current_view = Some(view);
    }

    fn end_frame(&mut self) {
        // Present frame
        if let Some(frame) = self.current_frame.take() {
            frame.present();
        }
        self.current_view = None;
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_state_default() {
        let state = RenderState::default();
        assert_eq!(state.color, 0xFFFFFFFF);
        assert!(state.depth_test);
        assert_eq!(state.cull_mode, CullMode::Back);
        assert_eq!(state.blend_mode, BlendMode::None);
        assert_eq!(state.texture_filter, TextureFilter::Nearest);
        assert_eq!(state.texture_slots, [TextureHandle::INVALID; 4]);
    }

    #[test]
    fn test_render_state_color_vec4() {
        let state = RenderState {
            color: 0xFF8040C0,
            ..Default::default()
        };
        let v = state.color_vec4();
        assert!((v.x - 1.0).abs() < 0.01);       // R = 0xFF
        assert!((v.y - 0.502).abs() < 0.01);    // G = 0x80
        assert!((v.z - 0.251).abs() < 0.01);    // B = 0x40
        assert!((v.w - 0.753).abs() < 0.01);    // A = 0xC0
    }

    #[test]
    fn test_cull_mode_conversion() {
        assert_eq!(CullMode::from_u32(0), CullMode::None);
        assert_eq!(CullMode::from_u32(1), CullMode::Back);
        assert_eq!(CullMode::from_u32(2), CullMode::Front);
        assert_eq!(CullMode::from_u32(99), CullMode::None);

        assert!(CullMode::None.to_wgpu().is_none());
        assert_eq!(CullMode::Back.to_wgpu(), Some(wgpu::Face::Back));
        assert_eq!(CullMode::Front.to_wgpu(), Some(wgpu::Face::Front));
    }

    #[test]
    fn test_blend_mode_conversion() {
        assert_eq!(BlendMode::from_u32(0), BlendMode::None);
        assert_eq!(BlendMode::from_u32(1), BlendMode::Alpha);
        assert_eq!(BlendMode::from_u32(2), BlendMode::Additive);
        assert_eq!(BlendMode::from_u32(3), BlendMode::Multiply);
        assert_eq!(BlendMode::from_u32(99), BlendMode::None);

        assert!(BlendMode::None.to_wgpu().is_none());
        assert!(BlendMode::Alpha.to_wgpu().is_some());
        assert!(BlendMode::Additive.to_wgpu().is_some());
        assert!(BlendMode::Multiply.to_wgpu().is_some());
    }

    #[test]
    fn test_texture_filter_conversion() {
        assert_eq!(TextureFilter::from_u32(0), TextureFilter::Nearest);
        assert_eq!(TextureFilter::from_u32(1), TextureFilter::Linear);
        assert_eq!(TextureFilter::from_u32(99), TextureFilter::Nearest);

        assert_eq!(TextureFilter::Nearest.to_wgpu(), wgpu::FilterMode::Nearest);
        assert_eq!(TextureFilter::Linear.to_wgpu(), wgpu::FilterMode::Linear);
    }

    #[test]
    fn test_texture_handle_invalid() {
        assert_eq!(TextureHandle::INVALID, TextureHandle(0));
    }

    #[test]
    fn test_mesh_handle_invalid() {
        assert_eq!(MeshHandle::INVALID, MeshHandle(0));
    }

    // ========================================================================
    // Vertex Format Tests
    // ========================================================================

    #[test]
    fn test_vertex_stride_pos_only() {
        // POS: 3 floats = 12 bytes
        assert_eq!(vertex_stride(0), 12);
    }

    #[test]
    fn test_vertex_stride_pos_uv() {
        // POS + UV: 3 + 2 floats = 20 bytes
        assert_eq!(vertex_stride(FORMAT_UV), 20);
    }

    #[test]
    fn test_vertex_stride_pos_color() {
        // POS + COLOR: 3 + 3 floats = 24 bytes
        assert_eq!(vertex_stride(FORMAT_COLOR), 24);
    }

    #[test]
    fn test_vertex_stride_pos_uv_color() {
        // POS + UV + COLOR: 3 + 2 + 3 floats = 32 bytes
        assert_eq!(vertex_stride(FORMAT_UV | FORMAT_COLOR), 32);
    }

    #[test]
    fn test_vertex_stride_pos_normal() {
        // POS + NORMAL: 3 + 3 floats = 24 bytes
        assert_eq!(vertex_stride(FORMAT_NORMAL), 24);
    }

    #[test]
    fn test_vertex_stride_pos_uv_normal() {
        // POS + UV + NORMAL: 3 + 2 + 3 floats = 32 bytes
        assert_eq!(vertex_stride(FORMAT_UV | FORMAT_NORMAL), 32);
    }

    #[test]
    fn test_vertex_stride_pos_color_normal() {
        // POS + COLOR + NORMAL: 3 + 3 + 3 floats = 36 bytes
        assert_eq!(vertex_stride(FORMAT_COLOR | FORMAT_NORMAL), 36);
    }

    #[test]
    fn test_vertex_stride_pos_uv_color_normal() {
        // POS + UV + COLOR + NORMAL: 3 + 2 + 3 + 3 floats = 44 bytes
        assert_eq!(vertex_stride(FORMAT_UV | FORMAT_COLOR | FORMAT_NORMAL), 44);
    }

    #[test]
    fn test_vertex_stride_pos_skinned() {
        // POS + SKINNED: 3 floats + 4 u8 + 4 floats = 12 + 4 + 16 = 32 bytes
        assert_eq!(vertex_stride(FORMAT_SKINNED), 32);
    }

    #[test]
    fn test_vertex_stride_full() {
        // All flags: POS + UV + COLOR + NORMAL + SKINNED
        // 12 + 8 + 12 + 12 + 20 = 64 bytes
        assert_eq!(vertex_stride(FORMAT_ALL), 64);
    }

    #[test]
    fn test_vertex_format_info_names() {
        assert_eq!(VertexFormatInfo::for_format(0).name, "POS");
        assert_eq!(VertexFormatInfo::for_format(1).name, "POS_UV");
        assert_eq!(VertexFormatInfo::for_format(2).name, "POS_COLOR");
        assert_eq!(VertexFormatInfo::for_format(3).name, "POS_UV_COLOR");
        assert_eq!(VertexFormatInfo::for_format(4).name, "POS_NORMAL");
        assert_eq!(VertexFormatInfo::for_format(5).name, "POS_UV_NORMAL");
        assert_eq!(VertexFormatInfo::for_format(6).name, "POS_COLOR_NORMAL");
        assert_eq!(VertexFormatInfo::for_format(7).name, "POS_UV_COLOR_NORMAL");
        assert_eq!(VertexFormatInfo::for_format(8).name, "POS_SKINNED");
        assert_eq!(VertexFormatInfo::for_format(15).name, "POS_UV_COLOR_NORMAL_SKINNED");
    }

    #[test]
    fn test_vertex_format_info_flags() {
        let format = VertexFormatInfo::for_format(FORMAT_UV | FORMAT_NORMAL);
        assert!(format.has_uv());
        assert!(!format.has_color());
        assert!(format.has_normal());
        assert!(!format.has_skinned());
    }

    #[test]
    fn test_all_16_vertex_formats() {
        // Verify all 16 formats have valid strides
        for i in 0..VERTEX_FORMAT_COUNT {
            let info = VertexFormatInfo::for_format(i as u8);
            assert!(info.stride >= 12, "Format {} has stride {} < 12", i, info.stride);
            assert!(info.stride <= 64, "Format {} has stride {} > 64", i, info.stride);
        }
    }

    // ========================================================================
    // Command Buffer Tests
    // ========================================================================

    #[test]
    fn test_command_buffer_new() {
        let cb = CommandBuffer::new();
        assert!(cb.commands().is_empty());
        for i in 0..VERTEX_FORMAT_COUNT {
            assert!(cb.vertex_data(i as u8).is_empty());
            assert!(cb.index_data(i as u8).is_empty());
        }
    }

    #[test]
    fn test_command_buffer_add_vertices() {
        let mut cb = CommandBuffer::new();
        let state = RenderState::default();

        // A single triangle with POS_COLOR format (3 vertices × 6 floats each)
        let vertices = [
            // pos        color
            0.0, 0.0, 0.0, 1.0, 0.0, 0.0, // vertex 0
            1.0, 0.0, 0.0, 0.0, 1.0, 0.0, // vertex 1
            0.5, 1.0, 0.0, 0.0, 0.0, 1.0, // vertex 2
        ];

        let base = cb.add_vertices(FORMAT_COLOR, &vertices, Mat4::IDENTITY, &state);

        assert_eq!(base, 0);
        assert_eq!(cb.commands().len(), 1);
        assert_eq!(cb.commands()[0].vertex_count, 3);
        assert_eq!(cb.commands()[0].format, FORMAT_COLOR);
    }

    #[test]
    fn test_command_buffer_add_vertices_indexed() {
        let mut cb = CommandBuffer::new();
        let state = RenderState::default();

        // A quad with POS format (4 vertices × 3 floats each)
        let vertices = [
            0.0, 0.0, 0.0, // vertex 0
            1.0, 0.0, 0.0, // vertex 1
            1.0, 1.0, 0.0, // vertex 2
            0.0, 1.0, 0.0, // vertex 3
        ];
        let indices = [0u32, 1, 2, 0, 2, 3];

        let (base_vertex, first_index) = cb.add_vertices_indexed(0, &vertices, &indices, Mat4::IDENTITY, &state);

        assert_eq!(base_vertex, 0);
        assert_eq!(first_index, 0);
        assert_eq!(cb.commands().len(), 1);
        assert_eq!(cb.commands()[0].vertex_count, 4);
        assert_eq!(cb.commands()[0].index_count, 6);
    }

    #[test]
    fn test_command_buffer_reset() {
        let mut cb = CommandBuffer::new();
        let state = RenderState::default();

        let vertices = [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        cb.add_vertices(0, &vertices, Mat4::IDENTITY, &state);

        assert!(!cb.commands().is_empty());

        cb.reset();

        assert!(cb.commands().is_empty());
        assert!(cb.vertex_data(0).is_empty());
    }

    #[test]
    fn test_command_buffer_multiple_batches() {
        let mut cb = CommandBuffer::new();
        let state = RenderState::default();

        // First batch
        let v1 = [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        let base1 = cb.add_vertices(0, &v1, Mat4::IDENTITY, &state);

        // Second batch (same format)
        let v2 = [2.0f32, 0.0, 0.0, 3.0, 0.0, 0.0, 2.5, 1.0, 0.0];
        let base2 = cb.add_vertices(0, &v2, Mat4::IDENTITY, &state);

        assert_eq!(base1, 0);
        assert_eq!(base2, 3); // Should start after first batch
        assert_eq!(cb.commands().len(), 2);
    }
}
