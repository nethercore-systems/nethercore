//! Material and shading state update methods for ZXFFIState

use super::ZXFFIState;

impl ZXFFIState {
    /// Update material property in current shading state (with quantization check)
    /// Metallic is stored in uniform_set_0 byte 0
    pub fn update_material_metallic(&mut self, value: f32) {
        use crate::graphics::{pack_unorm8, update_u32_byte};
        let quantized = pack_unorm8(value);
        let current_byte = (self.current_shading_state.uniform_set_0 & 0xFF) as u8;
        if current_byte != quantized {
            self.current_shading_state.uniform_set_0 =
                update_u32_byte(self.current_shading_state.uniform_set_0, 0, quantized);
            self.shading_state_dirty = true;
        }
    }

    /// Roughness is stored in uniform_set_0 byte 1
    pub fn update_material_roughness(&mut self, value: f32) {
        use crate::graphics::{pack_unorm8, update_u32_byte};
        let quantized = pack_unorm8(value);
        let current_byte = ((self.current_shading_state.uniform_set_0 >> 8) & 0xFF) as u8;
        if current_byte != quantized {
            self.current_shading_state.uniform_set_0 =
                update_u32_byte(self.current_shading_state.uniform_set_0, 1, quantized);
            self.shading_state_dirty = true;
        }
    }

    /// Emissive is stored in uniform_set_0 byte 2
    pub fn update_material_emissive(&mut self, value: f32) {
        use crate::graphics::{pack_unorm8, update_u32_byte};
        let quantized = pack_unorm8(value);
        let current_byte = ((self.current_shading_state.uniform_set_0 >> 16) & 0xFF) as u8;
        if current_byte != quantized {
            self.current_shading_state.uniform_set_0 =
                update_u32_byte(self.current_shading_state.uniform_set_0, 2, quantized);
            self.shading_state_dirty = true;
        }
    }

    /// Rim intensity is stored in uniform_set_0 byte 3
    pub fn update_material_rim_intensity(&mut self, value: f32) {
        use crate::graphics::{pack_unorm8, update_u32_byte};
        let quantized = pack_unorm8(value);
        let current_byte = ((self.current_shading_state.uniform_set_0 >> 24) & 0xFF) as u8;
        if current_byte != quantized {
            self.current_shading_state.uniform_set_0 =
                update_u32_byte(self.current_shading_state.uniform_set_0, 3, quantized);
            self.shading_state_dirty = true;
        }
    }

    /// Rim power is stored in uniform_set_1 byte 0 (low byte)
    pub fn update_material_rim_power(&mut self, value: f32) {
        use crate::graphics::{pack_unorm8, update_u32_byte};
        // Rim power is [0-1] → [0-32] in shader, so we pack [0-1] as u8
        let quantized = pack_unorm8(value / 32.0); // Normalize from [0-32] to [0-1]
        let current_byte = (self.current_shading_state.uniform_set_1 & 0xFF) as u8;
        if current_byte != quantized {
            self.current_shading_state.uniform_set_1 =
                update_u32_byte(self.current_shading_state.uniform_set_1, 0, quantized);
            self.shading_state_dirty = true;
        }
    }

    /// Update a directional light in current shading state (with quantization)
    pub fn update_light(
        &mut self,
        index: usize,
        direction: [f32; 3],
        color: [f32; 3],
        intensity: f32,
        enabled: bool,
    ) {
        use crate::graphics::PackedLight;
        use glam::Vec3;

        let new_light = PackedLight::directional(
            Vec3::from_slice(&direction),
            Vec3::from_slice(&color),
            intensity,
            enabled,
        );

        if self.current_shading_state.lights[index] != new_light {
            self.current_shading_state.lights[index] = new_light;
            self.shading_state_dirty = true;
        }
    }

    /// Update a point light in current shading state (with quantization)
    pub fn update_point_light(
        &mut self,
        index: usize,
        position: [f32; 3],
        color: [f32; 3],
        intensity: f32,
        range: f32,
        enabled: bool,
    ) {
        use crate::graphics::PackedLight;
        use glam::Vec3;

        let new_light = PackedLight::point(
            Vec3::from_slice(&position),
            Vec3::from_slice(&color),
            intensity,
            range,
            enabled,
        );

        if self.current_shading_state.lights[index] != new_light {
            self.current_shading_state.lights[index] = new_light;
            self.shading_state_dirty = true;
        }
    }

    /// Update color in current shading state (no quantization - already u32 RGBA8)
    pub fn update_color(&mut self, color: u32) {
        if self.current_shading_state.color_rgba8 != color {
            self.current_shading_state.color_rgba8 = color;
            self.shading_state_dirty = true;
        }
    }

    /// Update EPU environment index (`env_id`) in current shading state.
    pub fn update_environment_index(&mut self, env_id: u32) {
        if self.current_shading_state.environment_index != env_id {
            self.current_shading_state.environment_index = env_id;
            self.shading_state_dirty = true;
        }
    }

    /// Update a single matcap blend mode slot in current shading state
    /// Matcap blend modes are stored in uniform_set_0 (Mode 1 only)
    pub fn update_matcap_blend_mode(
        &mut self,
        slot: usize,
        mode: crate::graphics::MatcapBlendMode,
    ) {
        use crate::graphics::{pack_matcap_blend_modes, unpack_matcap_blend_modes};

        // Unpack current modes, modify one slot, repack
        let mut modes = unpack_matcap_blend_modes(self.current_shading_state.uniform_set_0);
        modes[slot] = mode;
        let packed = pack_matcap_blend_modes(modes);

        if self.current_shading_state.uniform_set_0 != packed {
            self.current_shading_state.uniform_set_0 = packed;
            self.shading_state_dirty = true;
        }
    }

    /// Update specular color in current shading state (Mode 3 only)
    /// Specular RGB is stored in uniform_set_1 using 0xRRGGBBRP format (big-endian, same as color_rgba8)
    pub fn update_specular_color(&mut self, r: f32, g: f32, b: f32) {
        use crate::graphics::pack_unorm8;

        let r_u8 = pack_unorm8(r);
        let g_u8 = pack_unorm8(g);
        let b_u8 = pack_unorm8(b);

        // Keep rim_power in byte 0 (low byte), update bytes 3-1 (high bytes) with RGB
        let rim_power_byte = (self.current_shading_state.uniform_set_1 & 0xFF) as u8;
        let new_packed = ((r_u8 as u32) << 24)
            | ((g_u8 as u32) << 16)
            | ((b_u8 as u32) << 8)
            | (rim_power_byte as u32);

        if self.current_shading_state.uniform_set_1 != new_packed {
            self.current_shading_state.uniform_set_1 = new_packed;
            self.shading_state_dirty = true;
        }
    }

    /// Update skinning mode in current shading state
    /// - false: raw mode (bone matrices used as-is)
    /// - true: inverse bind mode (GPU applies inverse bind matrices)
    pub fn update_skinning_mode(&mut self, inverse_bind: bool) {
        use crate::graphics::FLAG_SKINNING_MODE;

        let new_flags = if inverse_bind {
            self.current_shading_state.flags | FLAG_SKINNING_MODE
        } else {
            self.current_shading_state.flags & !FLAG_SKINNING_MODE
        };

        if self.current_shading_state.flags != new_flags {
            self.current_shading_state.flags = new_flags;
            self.shading_state_dirty = true;
        }
    }

    /// Update texture filter mode in current shading state
    /// - false/0: nearest (pixelated)
    /// - true/1: linear (smooth)
    pub fn update_texture_filter(&mut self, linear: bool) {
        use crate::graphics::FLAG_TEXTURE_FILTER_LINEAR;

        let new_flags = if linear {
            self.current_shading_state.flags | FLAG_TEXTURE_FILTER_LINEAR
        } else {
            self.current_shading_state.flags & !FLAG_TEXTURE_FILTER_LINEAR
        };

        if self.current_shading_state.flags != new_flags {
            self.current_shading_state.flags = new_flags;
            self.shading_state_dirty = true;
        }
    }

    /// Update uniform alpha level in current shading state (dither transparency)
    /// - 0: fully transparent (all pixels discarded)
    /// - 15: fully opaque (no pixels discarded, default)
    pub fn update_uniform_alpha(&mut self, alpha: u8) {
        use crate::graphics::{FLAG_UNIFORM_ALPHA_MASK, FLAG_UNIFORM_ALPHA_SHIFT};

        let alpha = alpha.min(15) as u32; // Clamp to 4 bits
        let new_flags = (self.current_shading_state.flags & !FLAG_UNIFORM_ALPHA_MASK)
            | (alpha << FLAG_UNIFORM_ALPHA_SHIFT);

        if self.current_shading_state.flags != new_flags {
            self.current_shading_state.flags = new_flags;
            self.shading_state_dirty = true;
        }
    }

    /// Update dither offset in current shading state
    ///
    /// - x: 0-3 pixel shift in X axis
    /// - y: 0-3 pixel shift in Y axis
    ///
    /// Use different offsets for stacked transparent objects to prevent pattern cancellation
    pub fn update_dither_offset(&mut self, x: u8, y: u8) {
        use crate::graphics::{
            FLAG_DITHER_OFFSET_X_MASK, FLAG_DITHER_OFFSET_X_SHIFT, FLAG_DITHER_OFFSET_Y_MASK,
            FLAG_DITHER_OFFSET_Y_SHIFT,
        };

        let x = (x.min(3) as u32) << FLAG_DITHER_OFFSET_X_SHIFT;
        let y = (y.min(3) as u32) << FLAG_DITHER_OFFSET_Y_SHIFT;
        let new_flags = (self.current_shading_state.flags
            & !FLAG_DITHER_OFFSET_X_MASK
            & !FLAG_DITHER_OFFSET_Y_MASK)
            | x
            | y;

        if self.current_shading_state.flags != new_flags {
            self.current_shading_state.flags = new_flags;
            self.shading_state_dirty = true;
        }
    }

    // =========================================================================
    // Material Override Flag Methods
    // =========================================================================

    /// Internal helper to update an override flag
    fn update_override_flag(&mut self, flag: u32, enabled: bool) {
        let new_flags = if enabled {
            self.current_shading_state.flags | flag
        } else {
            self.current_shading_state.flags & !flag
        };
        if self.current_shading_state.flags != new_flags {
            self.current_shading_state.flags = new_flags;
            self.shading_state_dirty = true;
        }
    }

    /// Update use_uniform_color flag
    pub fn set_use_uniform_color(&mut self, enabled: bool) {
        use crate::graphics::FLAG_USE_UNIFORM_COLOR;
        self.update_override_flag(FLAG_USE_UNIFORM_COLOR, enabled);
    }

    /// Update use_uniform_metallic flag
    pub fn set_use_uniform_metallic(&mut self, enabled: bool) {
        use crate::graphics::FLAG_USE_UNIFORM_METALLIC;
        self.update_override_flag(FLAG_USE_UNIFORM_METALLIC, enabled);
    }

    /// Update use_uniform_roughness flag
    pub fn set_use_uniform_roughness(&mut self, enabled: bool) {
        use crate::graphics::FLAG_USE_UNIFORM_ROUGHNESS;
        self.update_override_flag(FLAG_USE_UNIFORM_ROUGHNESS, enabled);
    }

    /// Update use_uniform_emissive flag
    pub fn set_use_uniform_emissive(&mut self, enabled: bool) {
        use crate::graphics::FLAG_USE_UNIFORM_EMISSIVE;
        self.update_override_flag(FLAG_USE_UNIFORM_EMISSIVE, enabled);
    }

    /// Update use_uniform_specular flag (Mode 3 only)
    pub fn set_use_uniform_specular(&mut self, enabled: bool) {
        use crate::graphics::FLAG_USE_UNIFORM_SPECULAR;
        self.update_override_flag(FLAG_USE_UNIFORM_SPECULAR, enabled);
    }

    /// Update use_matcap_reflection flag (Mode 1 only)
    pub fn set_use_matcap_reflection(&mut self, enabled: bool) {
        use crate::graphics::FLAG_USE_MATCAP_REFLECTION;
        self.update_override_flag(FLAG_USE_MATCAP_REFLECTION, enabled);
    }
}
