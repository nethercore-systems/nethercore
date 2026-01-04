//! Texture management for Nethercore ZX graphics.
//!
//! Handles texture loading, VRAM tracking, and fallback textures.
//! Supports RGBA8 (uncompressed), BC7 (compressed RGBA), and BC5 (compressed RG for normal maps).

use hashbrown::HashMap;

use anyhow::Result;
use wgpu::util::DeviceExt;

use crate::console::VRAM_LIMIT;
use zx_common::TextureFormat;

pub use super::render_state::TextureHandle;

/// Internal texture data
///
/// Fields tracked for debugging and VRAM accounting.
#[allow(dead_code)]
pub(crate) struct TextureEntry {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
    /// Size in bytes (for VRAM tracking)
    pub size_bytes: usize,
}

/// Manages game textures, VRAM budget, and fallback textures.
///
/// This struct owns all texture resources and handles:
/// - Loading textures from RGBA8 pixel data
/// - VRAM budget tracking and enforcement
/// - Fallback textures (checkerboard, white, font)
pub struct TextureManager {
    textures: HashMap<u32, TextureEntry>,
    next_texture_id: u32,
    vram_used: usize,

    // Fallback textures
    fallback_checkerboard: TextureHandle,
    fallback_white: TextureHandle,

    // Built-in font texture
    font_texture: TextureHandle,
}

impl TextureManager {
    /// Create a new TextureManager with fallback textures.
    ///
    /// This creates the checkerboard, white, and font textures.
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Self> {
        let mut manager = Self {
            textures: HashMap::new(),
            next_texture_id: 1, // 0 is reserved for INVALID
            vram_used: 0,
            fallback_checkerboard: TextureHandle::INVALID,
            fallback_white: TextureHandle::INVALID,
            font_texture: TextureHandle::INVALID,
        };

        manager.create_fallback_textures(device, queue)?;

        Ok(manager)
    }

    /// Create fallback textures (checkerboard, white, and font)
    fn create_fallback_textures(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<()> {
        // 8x8 magenta/black checkerboard for missing textures
        let mut checkerboard_data = vec![0u8; 8 * 8 * 4];
        for y in 0..8 {
            for x in 0..8 {
                let idx = (y * 8 + x) * 4;
                let is_magenta = (x + y) % 2 == 0;
                if is_magenta {
                    checkerboard_data[idx] = 255; // R
                    checkerboard_data[idx + 1] = 0; // G
                    checkerboard_data[idx + 2] = 255; // B
                    checkerboard_data[idx + 3] = 255; // A
                } else {
                    checkerboard_data[idx] = 0; // R
                    checkerboard_data[idx + 1] = 0; // G
                    checkerboard_data[idx + 2] = 0; // B
                    checkerboard_data[idx + 3] = 255; // A
                }
            }
        }
        self.fallback_checkerboard = self
            .load_texture_internal(device, queue, 8, 8, &checkerboard_data, false)
            .map_err(|e| {
                anyhow::anyhow!("Failed to create checkerboard fallback texture: {}", e)
            })?;

        // 1x1 white texture for untextured draws
        let white_data = [255u8, 255, 255, 255];
        self.fallback_white = self
            .load_texture_internal(device, queue, 1, 1, &white_data, false)
            .map_err(|e| anyhow::anyhow!("Failed to create white fallback texture: {}", e))?;

        // Load built-in font texture
        use crate::font;
        let font_atlas = font::generate_font_atlas();
        self.font_texture = self
            .load_texture_internal(
                device,
                queue,
                font::ATLAS_WIDTH,
                font::ATLAS_HEIGHT,
                &font_atlas,
                false,
            )
            .map_err(|e| anyhow::anyhow!("Failed to create font texture: {}", e))?;

        tracing::debug!(
            "Created font texture: {}x{}",
            font::ATLAS_WIDTH,
            font::ATLAS_HEIGHT
        );
        Ok(())
    }

    /// Load a texture from RGBA8 pixel data.
    ///
    /// Returns a TextureHandle or an error if VRAM budget is exceeded.
    pub fn load_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Result<TextureHandle> {
        self.load_texture_internal(device, queue, width, height, pixels, true)
    }

    /// Load a texture with explicit format (RGBA8, BC7, or BC5).
    ///
    /// This is the main entry point for loading textures from ROM data packs.
    /// BC7 textures provide 4× compression compared to RGBA8.
    /// BC5 textures are 2-channel (RG) for normal maps where Z is reconstructed.
    pub fn load_texture_with_format(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        data: &[u8],
        format: TextureFormat,
    ) -> Result<TextureHandle> {
        match format {
            TextureFormat::Rgba8 => {
                self.load_texture_internal(device, queue, width, height, data, true)
            }
            TextureFormat::Bc7 => self.load_texture_bc_internal(
                device,
                queue,
                width,
                height,
                data,
                wgpu::TextureFormat::Bc7RgbaUnorm,
                "BC7",
                true,
            ),
            TextureFormat::Bc5 => self.load_texture_bc_internal(
                device,
                queue,
                width,
                height,
                data,
                wgpu::TextureFormat::Bc5RgUnorm,
                "BC5",
                true,
            ),
        }
    }

    /// Internal block-compressed texture loading (BC7 or BC5)
    ///
    /// Both BC7 and BC5 use 4×4 blocks with 16 bytes per block.
    fn load_texture_bc_internal(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        data: &[u8],
        wgpu_format: wgpu::TextureFormat,
        format_name: &str,
        track_vram: bool,
    ) -> Result<TextureHandle> {
        // BC7/BC5: 4×4 blocks, 16 bytes per block
        let blocks_x = width.div_ceil(4);
        let blocks_y = height.div_ceil(4);
        let expected_size = (blocks_x * blocks_y * 16) as usize;

        if data.len() != expected_size {
            anyhow::bail!(
                "{} data size mismatch: expected {} bytes for {}x{} ({}x{} blocks), got {}",
                format_name,
                expected_size,
                width,
                height,
                blocks_x,
                blocks_y,
                data.len()
            );
        }

        // VRAM size is the compressed size
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

        // Create texture with block-compressed format
        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some(format_name),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu_format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            data,
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
            "Loaded {} texture {}: {}x{}, {} bytes (VRAM: {}/{})",
            format_name,
            handle.0,
            width,
            height,
            size_bytes,
            self.vram_used,
            VRAM_LIMIT
        );

        Ok(handle)
    }

    /// Internal texture loading (optionally tracks VRAM)
    fn load_texture_internal(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
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
        let texture = device.create_texture_with_data(
            queue,
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
                format: wgpu::TextureFormat::Rgba8Unorm,
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

    /// Get font texture handle
    pub fn font_texture(&self) -> TextureHandle {
        self.font_texture
    }

    /// Get white fallback texture handle
    pub fn white_texture(&self) -> TextureHandle {
        self.fallback_white
    }

    /// Get font texture view
    pub fn get_font_texture_view(&self) -> &wgpu::TextureView {
        &self.textures[&self.font_texture.0].view
    }

    /// Get VRAM usage in bytes
    pub fn vram_used(&self) -> usize {
        self.vram_used
    }

    /// Get VRAM limit in bytes
    pub fn vram_limit(&self) -> usize {
        VRAM_LIMIT
    }

    /// Clear all game textures (preserves font and fallback textures)
    ///
    /// This implements the "clear-on-init" pattern - clearing at the start of
    /// loading a new game rather than when exiting. This handles crashes/failed
    /// init gracefully since the next game load will clear stale state.
    pub fn clear_game_textures(&mut self) {
        // Keep only the built-in textures (checkerboard, white, font)
        let checkerboard_id = self.fallback_checkerboard.0;
        let white_id = self.fallback_white.0;
        let font_id = self.font_texture.0;

        self.textures
            .retain(|&id, _| id == checkerboard_id || id == white_id || id == font_id);

        // Reset next_texture_id to after the built-in textures
        // The built-in textures have IDs 1, 2, 3 (0 is INVALID)
        self.next_texture_id = 4;

        // Recalculate vram_used from remaining textures
        self.vram_used = self.textures.values().map(|e| e.size_bytes).sum();

        tracing::debug!(
            "Cleared game textures, {} built-in textures remain, VRAM: {} bytes",
            self.textures.len(),
            self.vram_used
        );
    }
}
