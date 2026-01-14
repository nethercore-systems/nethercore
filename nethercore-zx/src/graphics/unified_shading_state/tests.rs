#[cfg(test)]
mod tests {
    use super::super::*;
    use glam::Vec3;
    use zx_common::encode_octahedral;

    #[test]
    fn test_packed_sizes() {
        assert_eq!(std::mem::size_of::<PackedLight>(), 12); // 12 bytes for point light support
        assert_eq!(std::mem::size_of::<PackedEnvironmentState>(), 48); // 4 (header) + 44 (data)
        assert_eq!(std::mem::size_of::<PackedUnifiedShadingState>(), 80); // 16 (header) + 48 (lights) + 16 (animation/env)
    }

    #[test]
    fn test_quantization() {
        assert_eq!(pack_unorm8(0.0), 0);
        assert_eq!(pack_unorm8(1.0), 255);
        assert_eq!(pack_unorm8(0.5), 128);

        assert_eq!(pack_snorm16(0.0), 0);
        assert_eq!(pack_snorm16(1.0), 32767);
        assert_eq!(pack_snorm16(-1.0), -32767);
    }

    #[test]
    fn test_octahedral_cardinals() {
        // Test that cardinal directions encode/decode correctly
        let tests = [
            Vec3::new(1.0, 0.0, 0.0),  // +X
            Vec3::new(-1.0, 0.0, 0.0), // -X
            Vec3::new(0.0, 1.0, 0.0),  // +Y
            Vec3::new(0.0, -1.0, 0.0), // -Y
            Vec3::new(0.0, 0.0, 1.0),  // +Z
            Vec3::new(0.0, 0.0, -1.0), // -Z
        ];

        for dir in &tests {
            let (u, v) = encode_octahedral(*dir);
            assert!((-1.0..=1.0).contains(&u), "u out of range for {:?}", dir);
            assert!((-1.0..=1.0).contains(&v), "v out of range for {:?}", dir);

            // Verify packing doesn't panic and produces valid output
            let packed = zx_common::pack_octahedral_u32(*dir);
            assert_ne!(packed, 0xFFFFFFFF, "invalid pack for {:?}", dir);
        }
    }

    #[test]
    fn test_octahedral_zero_vector() {
        let zero = Vec3::new(0.0, 0.0, 0.0);
        let (u, v) = encode_octahedral(zero);
        assert_eq!(u, 0.0);
        assert_eq!(v, 0.0);
    }

    #[test]
    fn test_octahedral_diagonal() {
        // Test diagonal directions (challenging for octahedral)
        let diag = Vec3::new(0.577, 0.577, 0.577).normalize();
        let (u, v) = encode_octahedral(diag);
        assert!((-1.0..=1.0).contains(&u));
        assert!((-1.0..=1.0).contains(&v));
    }

    #[test]
    fn test_pack_rgba8() {
        // Format: 0xRRGGBBAA (R in highest byte, A in lowest)
        let packed = pack_rgba8(1.0, 0.5, 0.25, 1.0);
        assert_eq!((packed >> 24) & 0xFF, 255); // R
        assert_eq!((packed >> 16) & 0xFF, 128); // G
        assert_eq!((packed >> 8) & 0xFF, 64); // B
        assert_eq!(packed & 0xFF, 255); // A
    }

    #[test]
    fn test_disabled_light() {
        let light = PackedLight::disabled();
        assert_eq!(light.data0, 0);
        assert_eq!(light.data1, 0);
        assert_eq!(light.data2, 0);
        assert!(!light.is_enabled());
    }

    #[test]
    fn test_directional_light_roundtrip() {
        let dir = Vec3::new(0.5, -0.7, 0.3).normalize();
        let color = Vec3::new(1.0, 0.5, 0.25);
        let intensity = 2.5;

        let light = PackedLight::directional(dir, color, intensity, true);

        assert_eq!(light.get_type(), LightType::Directional);
        assert!(light.is_enabled());

        let unpacked_dir = light.get_direction();
        assert!((unpacked_dir[0] - dir.x).abs() < 0.01);
        assert!((unpacked_dir[1] - dir.y).abs() < 0.01);
        assert!((unpacked_dir[2] - dir.z).abs() < 0.01);

        let unpacked_color = light.get_color();
        assert!((unpacked_color[0] - color.x).abs() < 0.01);
        assert!((unpacked_color[1] - color.y).abs() < 0.01);
        assert!((unpacked_color[2] - color.z).abs() < 0.01);

        // Intensity with 7-bit precision in 0-8 range
        let unpacked_intensity = light.get_intensity();
        assert!((unpacked_intensity - intensity).abs() < 0.1);
    }

    #[test]
    fn test_point_light_roundtrip() {
        let pos = Vec3::new(10.5, -5.25, 100.0);
        let color = Vec3::new(0.8, 0.6, 0.4);
        let intensity = 4.0;
        let range = 25.0;

        let light = PackedLight::point(pos, color, intensity, range, true);

        assert_eq!(light.get_type(), LightType::Point);
        assert!(light.is_enabled());

        let unpacked_pos = light.get_position();
        // f16 precision is about 3 decimal digits
        assert!((unpacked_pos[0] - pos.x).abs() < 0.1);
        assert!((unpacked_pos[1] - pos.y).abs() < 0.1);
        assert!((unpacked_pos[2] - pos.z).abs() < 1.0);

        let unpacked_range = light.get_range();
        assert!((unpacked_range - range).abs() < 0.5);
    }

    #[test]
    fn test_f16_packing() {
        let values = [0.0, 1.0, -1.0, 100.0, 0.001, 65504.0];
        for v in values {
            let packed = pack_f16(v);
            let unpacked = unpack_f16(packed);
            let error = (unpacked - v).abs() / v.abs().max(1.0);
            assert!(
                error < 0.01,
                "f16 roundtrip failed for {}: got {}",
                v,
                unpacked
            );
        }
    }

    #[test]
    fn test_f16x2_packing() {
        let (x, y) = (42.5, -17.25);
        let packed = pack_f16x2(x, y);
        let (ux, uy) = unpack_f16x2(packed);
        assert!((ux - x).abs() < 0.1);
        assert!((uy - y).abs() < 0.1);
    }

    #[test]
    fn test_intensity_range() {
        // Test intensity at various points in 0-8 range
        for intensity in [0.0, 1.0, 2.0, 4.0, 7.9] {
            let light =
                PackedLight::directional(Vec3::new(0.0, -1.0, 0.0), Vec3::ONE, intensity, true);
            let unpacked = light.get_intensity();
            assert!(
                (unpacked - intensity).abs() < 0.1,
                "intensity {} unpacked to {}",
                intensity,
                unpacked
            );
        }
    }

    #[test]
    fn test_disabled_directional_light() {
        let light = PackedLight::directional(
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::ONE,
            1.0,
            false, // disabled
        );
        assert!(!light.is_enabled());
        assert_eq!(light.get_intensity(), 0.0);
    }

    #[test]
    fn test_texture_filter_flag() {
        let mut state = PackedUnifiedShadingState::default();
        // Default: nearest (flag not set)
        assert_eq!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);

        // Set to linear
        state.flags |= FLAG_TEXTURE_FILTER_LINEAR;
        assert_ne!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);

        // Set back to nearest
        state.flags &= !FLAG_TEXTURE_FILTER_LINEAR;
        assert_eq!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);
    }

    #[test]
    fn test_flags_independence() {
        // Verify texture_filter and skinning_mode flags don't interfere with each other
        let mut state = PackedUnifiedShadingState::default();

        // Set both flags
        state.flags = FLAG_SKINNING_MODE | FLAG_TEXTURE_FILTER_LINEAR;
        assert!(state.skinning_mode());
        assert_ne!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);

        // Clear skinning_mode, texture_filter should remain
        state.flags &= !FLAG_SKINNING_MODE;
        assert!(!state.skinning_mode());
        assert_ne!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);

        // Clear texture_filter, both should be clear
        state.flags &= !FLAG_TEXTURE_FILTER_LINEAR;
        assert!(!state.skinning_mode());
        assert_eq!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);
    }

    #[test]
    fn test_texture_filter_flag_bit_position() {
        // Verify the flag is at bit 1 (value 2)
        assert_eq!(FLAG_TEXTURE_FILTER_LINEAR, 2);
        assert_eq!(FLAG_TEXTURE_FILTER_LINEAR, 1 << 1);

        // Verify it's different from skinning_mode (bit 0)
        assert_ne!(FLAG_TEXTURE_FILTER_LINEAR, FLAG_SKINNING_MODE);
    }

    // ========================================================================
    // Dither Transparency Tests
    // ========================================================================

    #[test]
    fn test_uniform_alpha_packing() {
        // Test all 16 values pack/unpack correctly
        for alpha in 0..=15u32 {
            let flags = alpha << FLAG_UNIFORM_ALPHA_SHIFT;
            let unpacked = (flags & FLAG_UNIFORM_ALPHA_MASK) >> FLAG_UNIFORM_ALPHA_SHIFT;
            assert_eq!(unpacked, alpha);
        }
    }

    #[test]
    fn test_dither_offset_packing() {
        // Test all 16 offset combinations
        for x in 0..=3u32 {
            for y in 0..=3u32 {
                let flags = (x << FLAG_DITHER_OFFSET_X_SHIFT) | (y << FLAG_DITHER_OFFSET_Y_SHIFT);
                let unpacked_x = (flags & FLAG_DITHER_OFFSET_X_MASK) >> FLAG_DITHER_OFFSET_X_SHIFT;
                let unpacked_y = (flags & FLAG_DITHER_OFFSET_Y_MASK) >> FLAG_DITHER_OFFSET_Y_SHIFT;
                assert_eq!(unpacked_x, x);
                assert_eq!(unpacked_y, y);
            }
        }
    }

    #[test]
    fn test_default_flags_are_opaque() {
        let state = PackedUnifiedShadingState::default();
        let alpha = (state.flags & FLAG_UNIFORM_ALPHA_MASK) >> FLAG_UNIFORM_ALPHA_SHIFT;
        assert_eq!(alpha, 15, "Default uniform_alpha must be 15 (opaque)");
    }

    #[test]
    fn test_bayer_threshold_values() {
        // Verify Bayer matrix produces values in expected range
        const BAYER_4X4: [f32; 16] = [
            0.0 / 16.0,
            8.0 / 16.0,
            2.0 / 16.0,
            10.0 / 16.0,
            12.0 / 16.0,
            4.0 / 16.0,
            14.0 / 16.0,
            6.0 / 16.0,
            3.0 / 16.0,
            11.0 / 16.0,
            1.0 / 16.0,
            9.0 / 16.0,
            15.0 / 16.0,
            7.0 / 16.0,
            13.0 / 16.0,
            5.0 / 16.0,
        ];

        for (i, &threshold) in BAYER_4X4.iter().enumerate() {
            assert!(threshold >= 0.0, "Threshold {} is negative", i);
            assert!(threshold < 1.0, "Threshold {} >= 1.0", i);
        }

        // Verify we have 16 unique values
        let mut sorted = BAYER_4X4.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for i in 0..15 {
            assert_ne!(sorted[i], sorted[i + 1], "Duplicate threshold values");
        }
    }

    #[test]
    fn test_dither_flags_independence() {
        // Verify dither flags don't interfere with other flags
        let mut state = PackedUnifiedShadingState::default();

        // Set skinning_mode and texture_filter
        state.flags |= FLAG_SKINNING_MODE | FLAG_TEXTURE_FILTER_LINEAR;

        // Set uniform_alpha to 8 (50% transparency)
        state.flags = (state.flags & !FLAG_UNIFORM_ALPHA_MASK) | (8u32 << FLAG_UNIFORM_ALPHA_SHIFT);

        // Set dither offset to (2, 3)
        state.flags = (state.flags & !FLAG_DITHER_OFFSET_X_MASK & !FLAG_DITHER_OFFSET_Y_MASK)
            | (2u32 << FLAG_DITHER_OFFSET_X_SHIFT)
            | (3u32 << FLAG_DITHER_OFFSET_Y_SHIFT);

        // Verify all flags are independent
        assert!(state.skinning_mode());
        assert_ne!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);
        assert_eq!(
            (state.flags & FLAG_UNIFORM_ALPHA_MASK) >> FLAG_UNIFORM_ALPHA_SHIFT,
            8
        );
        assert_eq!(
            (state.flags & FLAG_DITHER_OFFSET_X_MASK) >> FLAG_DITHER_OFFSET_X_SHIFT,
            2
        );
        assert_eq!(
            (state.flags & FLAG_DITHER_OFFSET_Y_MASK) >> FLAG_DITHER_OFFSET_Y_SHIFT,
            3
        );
    }

    // ========================================================================
    // Normal Mapping Tests
    // ========================================================================

    #[test]
    fn test_skip_normal_map_flag() {
        let mut state = PackedUnifiedShadingState::default();
        // Default: normal map NOT skipped (i.e., normal mapping is enabled)
        assert!(!state.skips_normal_map());

        // Opt-out: skip normal map
        state.set_skip_normal_map(true);
        assert!(state.skips_normal_map());

        // Re-enable normal map (clear skip flag)
        state.set_skip_normal_map(false);
        assert!(!state.skips_normal_map());
    }

    #[test]
    fn test_skip_normal_map_flag_bit_position() {
        // Verify the flag is at bit 16 (value 0x10000)
        assert_eq!(FLAG_SKIP_NORMAL_MAP, 0x10000);
        assert_eq!(FLAG_SKIP_NORMAL_MAP, 1 << 16);

        // Verify it doesn't overlap with other flags
        assert_ne!(
            FLAG_SKIP_NORMAL_MAP & FLAG_SKINNING_MODE,
            FLAG_SKINNING_MODE
        );
        assert_ne!(
            FLAG_SKIP_NORMAL_MAP & FLAG_TEXTURE_FILTER_LINEAR,
            FLAG_TEXTURE_FILTER_LINEAR
        );
        assert_ne!(
            FLAG_SKIP_NORMAL_MAP & FLAG_UNIFORM_ALPHA_MASK,
            FLAG_UNIFORM_ALPHA_MASK
        );
    }

    #[test]
    fn test_skip_normal_map_flag_independence() {
        // Verify skip normal map flag doesn't interfere with other flags
        let mut state = PackedUnifiedShadingState::default();

        // Set multiple flags
        state.flags |= FLAG_SKINNING_MODE | FLAG_TEXTURE_FILTER_LINEAR;
        state.set_skip_normal_map(true);

        // Verify all flags are set correctly
        assert!(state.skinning_mode());
        assert_ne!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);
        assert!(state.skips_normal_map());

        // Clear skip flag, others should remain
        state.set_skip_normal_map(false);
        assert!(state.skinning_mode());
        assert_ne!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);
        assert!(!state.skips_normal_map());
    }

    // ========================================================================
    // Environment System Tests
    // ========================================================================

    #[test]
    fn test_environment_header_packing() {
        // Test all combinations of modes and blend modes
        for base in 0..8u32 {
            for overlay in 0..8u32 {
                for blend in 0..4u32 {
                    let header = PackedEnvironmentState::make_header(base, overlay, blend);
                    let mut env = PackedEnvironmentState::default();
                    env.header = header;
                    assert_eq!(env.base_mode(), base);
                    assert_eq!(env.overlay_mode(), overlay);
                    assert_eq!(env.blend_mode(), blend);
                }
            }
        }
    }

    #[test]
    fn test_environment_mode_setters() {
        let mut env = PackedEnvironmentState::default();

        env.set_base_mode(env_mode::GRADIENT);
        env.set_overlay_mode(env_mode::SCATTER);
        env.set_blend_mode(blend_mode::ADD);

        assert_eq!(env.base_mode(), env_mode::GRADIENT);
        assert_eq!(env.overlay_mode(), env_mode::SCATTER);
        assert_eq!(env.blend_mode(), blend_mode::ADD);

        // Change individual values without affecting others
        env.set_base_mode(env_mode::RINGS);
        assert_eq!(env.base_mode(), env_mode::RINGS);
        assert_eq!(env.overlay_mode(), env_mode::SCATTER); // unchanged
        assert_eq!(env.blend_mode(), blend_mode::ADD); // unchanged
    }

    #[test]
    fn test_environment_gradient_packing() {
        let mut env = PackedEnvironmentState::default();
        env.pack_gradient(GradientConfig {
            offset: 0,
            zenith: 0x3366B2FF,
            sky_horizon: 0xB2D8F2FF,
            ground_horizon: 0x8B7355FF,
            nadir: 0x4A3728FF,
            rotation: 45.0,
            shift: 0.25,
        });

        assert_eq!(env.data[0], 0x3366B2FF);
        assert_eq!(env.data[1], 0xB2D8F2FF);
        assert_eq!(env.data[2], 0x8B7355FF);
        assert_eq!(env.data[3], 0x4A3728FF);

        // Verify f16x2 packing of rotation and shift
        let (rotation, shift) = unpack_f16x2(env.data[4]);
        assert!((rotation - 45.0).abs() < 0.1);
        assert!((shift - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_environment_default_gradient() {
        let env = PackedEnvironmentState::default_gradient();
        assert_eq!(env.base_mode(), env_mode::GRADIENT);
        assert_eq!(env.overlay_mode(), env_mode::GRADIENT);
        assert_eq!(env.blend_mode(), blend_mode::ALPHA);
        // Verify colors are set
        assert_ne!(env.data[0], 0); // zenith
        assert_ne!(env.data[1], 0); // sky_horizon
    }

    #[test]
    fn test_environment_index() {
        assert_eq!(EnvironmentIndex::default(), EnvironmentIndex(0));
        assert_eq!(EnvironmentIndex::INVALID, EnvironmentIndex(u32::MAX));
    }
}
