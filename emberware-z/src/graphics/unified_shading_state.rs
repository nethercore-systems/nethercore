//! Unified Shading State System
//!
//! Quantizes all per-draw material state into a hashable POD structure,
//! implements interning to deduplicate identical materials, and enables
//! better batching/sorting.

use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};
use hashbrown::HashMap;
use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device, Queue};

use super::render_state::MatcapBlendMode;
use crate::state::LightState;

/// Quantized sky data for GPU upload
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedSky {
    pub horizon_color: u32,           // RGBA8 packed
    pub zenith_color: u32,            // RGBA8 packed
    pub sun_direction: [i16; 4],      // snorm16x4 (w unused)
    pub sun_color_and_sharpness: u32, // RGB8 + sharpness u8
}

/// One packed light
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedLight {
    pub direction: [i16; 4],      // snorm16x4 (w = enabled flag)
    pub color_and_intensity: u32, // RGB8 + intensity u8
}

/// Unified per-draw shading state (~96 bytes, POD, hashable)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedUnifiedShadingState {
    // PBR params (4 bytes)
    pub metallic: u8,
    pub roughness: u8,
    pub emissive: u8,
    pub pad0: u8,

    pub color_rgba8: u32,     // Base color (4 bytes)
    pub blend_modes: u32,     // 4× u8 packed (4 bytes)

    pub sky: PackedSky,       // 16 bytes
    pub lights: [PackedLight; 4], // 64 bytes
}

/// Handle to interned shading state
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UnifiedShadingStateHandle(pub u32);

impl UnifiedShadingStateHandle {
    pub const INVALID: Self = Self(0);
}

// ============================================================================
// Quantization Helpers
// ============================================================================

/// Convert f32 (0.0-1.0) to u8 (0-255)
fn quantize_f32_to_u8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Convert Vec3 to snorm16x4 (normalized direction)
fn quantize_vec3_to_snorm16(v: Vec3) -> [i16; 4] {
    let normalized = v.normalize_or_zero();
    [
        (normalized.x * 32767.0).round() as i16,
        (normalized.y * 32767.0).round() as i16,
        (normalized.z * 32767.0).round() as i16,
        0,
    ]
}

/// Convert Vec3 to snorm16x4 with enabled flag in w component
fn quantize_vec3_to_snorm16_with_flag(v: Vec3, enabled: bool) -> [i16; 4] {
    let normalized = v.normalize_or_zero();
    [
        (normalized.x * 32767.0).round() as i16,
        (normalized.y * 32767.0).round() as i16,
        (normalized.z * 32767.0).round() as i16,
        if enabled { 32767 } else { 0 },
    ]
}

/// Pack Vec4 color to RGBA8 u32
fn pack_rgba8_from_vec4(color: Vec4) -> u32 {
    let r = (color.x.clamp(0.0, 1.0) * 255.0).round() as u32;
    let g = (color.y.clamp(0.0, 1.0) * 255.0).round() as u32;
    let b = (color.z.clamp(0.0, 1.0) * 255.0).round() as u32;
    let a = (color.w.clamp(0.0, 1.0) * 255.0).round() as u32;
    (r << 24) | (g << 16) | (b << 8) | a
}

/// Pack Vec3 color and scalar to u32 (RGB8 + u8)
fn pack_color_and_scalar(color: Vec3, scalar: f32) -> u32 {
    let r = (color.x.clamp(0.0, 1.0) * 255.0).round() as u32;
    let g = (color.y.clamp(0.0, 1.0) * 255.0).round() as u32;
    let b = (color.z.clamp(0.0, 1.0) * 255.0).round() as u32;
    let s = (scalar.clamp(0.0, 1.0) * 255.0).round() as u32;
    (r << 24) | (g << 16) | (b << 8) | s
}

/// Pack 4 blend modes into u32
fn pack_blend_modes(modes: &[MatcapBlendMode; 4]) -> u32 {
    (modes[0] as u32) << 24
        | (modes[1] as u32) << 16
        | (modes[2] as u32) << 8
        | (modes[3] as u32)
}

// ============================================================================
// PackedSky Implementation
// ============================================================================

/// Sky state for quantization
pub struct Sky {
    pub horizon_color: Vec4,
    pub zenith_color: Vec4,
    pub sun_direction: Vec3,
    pub sun_color: Vec3,
    pub sun_sharpness: f32,
}

impl PackedSky {
    pub fn from_sky(sky: &Sky) -> Self {
        Self {
            horizon_color: pack_rgba8_from_vec4(sky.horizon_color),
            zenith_color: pack_rgba8_from_vec4(sky.zenith_color),
            sun_direction: quantize_vec3_to_snorm16(sky.sun_direction),
            sun_color_and_sharpness: pack_color_and_scalar(
                sky.sun_color,
                sky.sun_sharpness,
            ),
        }
    }
}

// ============================================================================
// PackedLight Implementation
// ============================================================================

impl PackedLight {
    pub fn from_light(light: &LightState) -> Self {
        let direction = Vec3::from_array(light.direction);
        let color = Vec3::from_array(light.color);
        
        Self {
            direction: quantize_vec3_to_snorm16_with_flag(
                direction,
                light.enabled,
            ),
            color_and_intensity: pack_color_and_scalar(
                color,
                light.intensity,
            ),
        }
    }
}

// ============================================================================
// PackedUnifiedShadingState Implementation
// ============================================================================

impl PackedUnifiedShadingState {
    /// Construct from unquantized render state
    pub fn from_render_state(
        color: u32,
        metallic: f32,
        roughness: f32,
        emissive: f32,
        matcap_blend_modes: &[MatcapBlendMode; 4],
        sky: &Sky,
        lights: &[LightState; 4],
    ) -> Self {
        Self {
            metallic: quantize_f32_to_u8(metallic),
            roughness: quantize_f32_to_u8(roughness),
            emissive: quantize_f32_to_u8(emissive),
            pad0: 0,

            color_rgba8: color,
            blend_modes: pack_blend_modes(matcap_blend_modes),

            sky: PackedSky::from_sky(sky),
            lights: [
                PackedLight::from_light(&lights[0]),
                PackedLight::from_light(&lights[1]),
                PackedLight::from_light(&lights[2]),
                PackedLight::from_light(&lights[3]),
            ],
        }
    }
}

// ============================================================================
// Shading State Cache
// ============================================================================

pub struct ShadingStateCache {
    // Map: packed state → handle
    cache: HashMap<PackedUnifiedShadingState, UnifiedShadingStateHandle>,

    // All unique states (indexed by handle)
    states: Vec<PackedUnifiedShadingState>,

    // GPU buffer
    states_buffer: Buffer,
    buffer_capacity: usize,
    dirty: bool,
}

impl ShadingStateCache {
    pub fn new(device: &Device) -> Self {
        let initial_capacity = 256;
        let states_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Shading States"),
            size: (initial_capacity * std::mem::size_of::<PackedUnifiedShadingState>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            cache: HashMap::new(),
            states: Vec::new(),
            states_buffer,
            buffer_capacity: initial_capacity,
            dirty: false,
        }
    }

    /// Intern a shading state, returning its handle
    pub fn intern(&mut self, state: PackedUnifiedShadingState) -> UnifiedShadingStateHandle {
        // Check if already cached
        if let Some(&handle) = self.cache.get(&state) {
            return handle;
        }

        // Allocate new handle
        let handle = UnifiedShadingStateHandle(self.states.len() as u32);
        self.states.push(state);
        self.cache.insert(state, handle);
        self.dirty = true;

        tracing::trace!("Interned new shading state: {:?}", handle);
        handle
    }

    /// Upload dirty states to GPU
    pub fn upload(&mut self, device: &Device, queue: &Queue) {
        if !self.dirty {
            return;
        }

        // Grow buffer if needed
        if self.states.len() > self.buffer_capacity {
            let new_capacity = (self.states.len() * 2).next_power_of_two();
            tracing::debug!(
                "Growing shading state buffer: {} → {} states",
                self.buffer_capacity,
                new_capacity
            );

            self.states_buffer = device.create_buffer(&BufferDescriptor {
                label: Some("Shading States"),
                size: (new_capacity * std::mem::size_of::<PackedUnifiedShadingState>()) as u64,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.buffer_capacity = new_capacity;
        }

        // Upload all states
        let data = bytemuck::cast_slice(&self.states);
        queue.write_buffer(&self.states_buffer, 0, data);

        self.dirty = false;
        tracing::debug!("Uploaded {} shading states to GPU", self.states.len());
    }

    /// Get GPU buffer
    pub fn buffer(&self) -> &Buffer {
        &self.states_buffer
    }

    /// Get state by handle (for pipeline extraction)
    pub fn get(&self, handle: UnifiedShadingStateHandle) -> Option<&PackedUnifiedShadingState> {
        self.states.get(handle.0 as usize)
    }

    /// Clear cache (optional, for per-frame reset)
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.cache.clear();
        self.states.clear();
        self.dirty = true;
    }

    /// Get stats for debugging
    pub fn stats(&self) -> (usize, usize) {
        (self.states.len(), self.buffer_capacity)
    }
}
