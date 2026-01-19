//! Unit tests for EPU Rust API

use super::*;
use glam::Vec3;

// =============================================================================
// Bit Packing Tests
// =============================================================================

#[test]
fn test_epu_layer_encode_nop() {
    let layer = EpuLayer::nop();
    let encoded = layer.encode();

    // NOP should be all zeros
    assert_eq!(encoded, 0);
}

#[test]
fn test_epu_layer_encode_opcode_position() {
    // Test that opcode is in bits 63..60
    let layer = EpuLayer {
        opcode: EpuOpcode::Flow, // 0x8
        ..EpuLayer::nop()
    };
    let encoded = layer.encode();

    let opcode = (encoded >> 60) & 0xF;
    assert_eq!(opcode, 0x8);
}

#[test]
fn test_epu_layer_encode_region_position() {
    // Test that region is in bits 59..58
    let layer = EpuLayer {
        region: EpuRegion::Floor, // 0b11
        ..EpuLayer::nop()
    };
    let encoded = layer.encode();

    let region = (encoded >> 58) & 0x3;
    assert_eq!(region, 0b11);
}

#[test]
fn test_epu_layer_encode_blend_position() {
    // Test that blend is in bits 57..56
    let layer = EpuLayer {
        blend: EpuBlend::Lerp, // 0b11
        ..EpuLayer::nop()
    };
    let encoded = layer.encode();

    let blend = (encoded >> 56) & 0x3;
    assert_eq!(blend, 0b11);
}

#[test]
fn test_epu_layer_encode_color_index() {
    let layer = EpuLayer {
        color_index: 0xAB,
        ..EpuLayer::nop()
    };
    let encoded = layer.encode();

    let color = (encoded >> 48) & 0xFF;
    assert_eq!(color, 0xAB);
}

#[test]
fn test_epu_layer_encode_intensity() {
    let layer = EpuLayer {
        intensity: 0xCD,
        ..EpuLayer::nop()
    };
    let encoded = layer.encode();

    let intensity = (encoded >> 40) & 0xFF;
    assert_eq!(intensity, 0xCD);
}

#[test]
fn test_epu_layer_encode_params() {
    let layer = EpuLayer {
        param_a: 0x11,
        param_b: 0x22,
        param_c: 0x33,
        ..EpuLayer::nop()
    };
    let encoded = layer.encode();

    let param_a = (encoded >> 32) & 0xFF;
    let param_b = (encoded >> 24) & 0xFF;
    let param_c = (encoded >> 16) & 0xFF;

    assert_eq!(param_a, 0x11);
    assert_eq!(param_b, 0x22);
    assert_eq!(param_c, 0x33);
}

#[test]
fn test_epu_layer_encode_direction() {
    let layer = EpuLayer {
        direction: 0xBEEF,
        ..EpuLayer::nop()
    };
    let encoded = layer.encode();

    let direction = encoded & 0xFFFF;
    assert_eq!(direction, 0xBEEF);
}

#[test]
fn test_epu_layer_encode_full() {
    // Test a fully populated layer
    let layer = EpuLayer {
        opcode: EpuOpcode::Decal, // 0x5
        region: EpuRegion::Sky,   // 0b01
        blend: EpuBlend::Add,     // 0b00
        color_index: 15,
        intensity: 255,
        param_a: 0x12,
        param_b: 0x34,
        param_c: 0x56,
        direction: 0x7890,
    };
    let encoded = layer.encode();

    // Verify all fields
    assert_eq!((encoded >> 60) & 0xF, 0x5); // opcode
    assert_eq!((encoded >> 58) & 0x3, 0b01); // region
    assert_eq!((encoded >> 56) & 0x3, 0b00); // blend
    assert_eq!((encoded >> 48) & 0xFF, 15); // color_index
    assert_eq!((encoded >> 40) & 0xFF, 255); // intensity
    assert_eq!((encoded >> 32) & 0xFF, 0x12); // param_a
    assert_eq!((encoded >> 24) & 0xFF, 0x34); // param_b
    assert_eq!((encoded >> 16) & 0xFF, 0x56); // param_c
    assert_eq!(encoded & 0xFFFF, 0x7890); // direction
}

// =============================================================================
// Direction Encoding Tests
// =============================================================================

#[test]
fn test_encode_direction_u16_up() {
    let encoded = encode_direction_u16(Vec3::Y);

    // Y = (0, 1, 0) in octahedral projection:
    // denom = |0| + |1| + |0| = 1
    // p = (0, 1) / 1 = (0, 1)
    // Since z >= 0, no wrapping
    // Map [-1, 1] to [0, 255]: u = (0*0.5+0.5)*255 = 127.5 -> 128
    //                          v = (1*0.5+0.5)*255 = 255
    let u = (encoded & 0xFF) as u8;
    let v = ((encoded >> 8) & 0xFF) as u8;

    // u should be near center (127 or 128), v should be 255
    assert!((125..=130).contains(&u), "u = {u}, expected ~127-128");
    assert_eq!(v, 255, "v = {v}, expected 255 for +Y");
}

#[test]
fn test_encode_direction_u16_forward() {
    let encoded = encode_direction_u16(Vec3::Z);

    // Z = (0, 0, 1) in octahedral projection:
    // denom = |0| + |0| + |1| = 1
    // p = (0, 0) / 1 = (0, 0)
    // Since z >= 0, no wrapping
    // Map [-1, 1] to [0, 255]: u = (0*0.5+0.5)*255 = 127.5 -> 128
    //                          v = (0*0.5+0.5)*255 = 127.5 -> 128
    let u = (encoded & 0xFF) as u8;
    let v = ((encoded >> 8) & 0xFF) as u8;

    // Both should be near center (127 or 128)
    assert!((125..=130).contains(&u), "u = {u}, expected ~127-128");
    assert!((125..=130).contains(&v), "v = {v}, expected ~127-128");
}

#[test]
fn test_encode_direction_u16_neg_z() {
    let encoded = encode_direction_u16(-Vec3::Z);

    // -Z should encode to lower hemisphere with wrapping
    let u = (encoded & 0xFF) as u8;
    let v = ((encoded >> 8) & 0xFF) as u8;

    // Both should be at extremes due to octahedral wrapping
    // For -Z: oct coords wrap to corners
    assert!(u == 0 || u == 255 || (125..=130).contains(&u), "u = {u}");
    assert!(v == 0 || v == 255 || (125..=130).contains(&v), "v = {v}");
}

#[test]
fn test_encode_direction_u16_right() {
    let encoded = encode_direction_u16(Vec3::X);

    let u = (encoded & 0xFF) as u8;
    let v = ((encoded >> 8) & 0xFF) as u8;

    // +X should be offset from center
    // In octahedral, +X maps to (1, 0) in oct space -> (1.0, 0.5) in [0,1] -> (255, 127-128)
    assert!(u > 200, "u = {u}, expected high value for +X");
    assert!((125..=130).contains(&v), "v = {v}, expected ~127-128");
}

#[test]
fn test_encode_direction_u16_normalized() {
    // Non-unit vector should be normalized
    let unnormalized = Vec3::new(10.0, 0.0, 0.0);
    let normalized = Vec3::X;

    let encoded_unnorm = encode_direction_u16(unnormalized);
    let encoded_norm = encode_direction_u16(normalized);

    assert_eq!(encoded_unnorm, encoded_norm);
}

#[test]
fn test_encode_direction_u16_zero_vector() {
    // Zero vector should default to +Y
    let encoded_zero = encode_direction_u16(Vec3::ZERO);
    let encoded_y = encode_direction_u16(Vec3::Y);

    assert_eq!(encoded_zero, encoded_y);
}

// =============================================================================
// Threshold Packing Tests
// =============================================================================

#[test]
fn test_pack_thresholds() {
    let packed = pack_thresholds(0xA, 0xB);

    let ceil = (packed >> 4) & 0x0F;
    let floor = packed & 0x0F;

    assert_eq!(ceil, 0xA);
    assert_eq!(floor, 0xB);
}

#[test]
fn test_pack_thresholds_clamped() {
    // Values > 15 should be masked
    let packed = pack_thresholds(0xFF, 0xEE);

    let ceil = (packed >> 4) & 0x0F;
    let floor = packed & 0x0F;

    assert_eq!(ceil, 0x0F);
    assert_eq!(floor, 0x0E);
}

// =============================================================================
// Builder Tests
// =============================================================================

#[test]
fn test_builder_default_is_all_nop() {
    let builder = epu_begin();
    let config = epu_finish(builder);

    for layer in config.layers {
        assert_eq!(layer, 0, "Default layer should be NOP (all zeros)");
    }
}

#[test]
fn test_builder_ramp_enclosure() {
    let mut builder = epu_begin();
    builder.ramp_enclosure(RampParams {
        up: Vec3::Y,
        wall_color: 24,
        sky_color: 40,
        floor_color: 52,
        ceil_q: 10,
        floor_q: 5,
        softness: 180,
    });
    let config = epu_finish(builder);

    let layer0 = config.layers[0];

    // Check opcode is RAMP
    let opcode = (layer0 >> 60) & 0xF;
    assert_eq!(opcode, EpuOpcode::Ramp as u64);

    // Check colors
    let wall_color = (layer0 >> 48) & 0xFF;
    let sky_color = (layer0 >> 32) & 0xFF;
    let floor_color = (layer0 >> 24) & 0xFF;

    assert_eq!(wall_color, 24);
    assert_eq!(sky_color, 40);
    assert_eq!(floor_color, 52);

    // Check thresholds
    let param_c = (layer0 >> 16) & 0xFF;
    let ceil_q = (param_c >> 4) & 0x0F;
    let floor_q = param_c & 0x0F;

    assert_eq!(ceil_q, 10);
    assert_eq!(floor_q, 5);

    // Check softness
    let softness = (layer0 >> 40) & 0xFF;
    assert_eq!(softness, 180);
}

#[test]
fn test_builder_lobe() {
    let mut builder = epu_begin();
    let sun_dir = Vec3::new(0.5, 0.7, 0.3).normalize();
    builder.lobe(sun_dir, 20, 180, 32, 10, 1);
    let config = epu_finish(builder);

    // Lobe should be in bounds slot (slot 0 since no ramp was added)
    let layer = config.layers[0];

    let opcode = (layer >> 60) & 0xF;
    assert_eq!(opcode, EpuOpcode::Lobe as u64);

    let color = (layer >> 48) & 0xFF;
    assert_eq!(color, 20);

    let intensity = (layer >> 40) & 0xFF;
    assert_eq!(intensity, 180);

    let exponent = (layer >> 32) & 0xFF;
    assert_eq!(exponent, 32);

    let anim_speed = (layer >> 24) & 0xFF;
    assert_eq!(anim_speed, 10);

    let anim_mode = (layer >> 16) & 0xFF;
    assert_eq!(anim_mode, 1);
}

#[test]
fn test_builder_band() {
    let mut builder = epu_begin();
    builder.band(Vec3::Y, 30, 200, 64, 128, 50);
    let config = epu_finish(builder);

    let layer = config.layers[0];
    let opcode = (layer >> 60) & 0xF;
    assert_eq!(opcode, EpuOpcode::Band as u64);
}

#[test]
fn test_builder_fog() {
    let mut builder = epu_begin();
    builder.fog(Vec3::Y, 80, 128, 128, 100);
    let config = epu_finish(builder);

    let layer = config.layers[0];

    let opcode = (layer >> 60) & 0xF;
    assert_eq!(opcode, EpuOpcode::Fog as u64);

    // Fog should use MULTIPLY blend
    let blend = (layer >> 56) & 0x3;
    assert_eq!(blend, EpuBlend::Multiply as u64);
}

#[test]
fn test_builder_decal() {
    let mut builder = epu_begin();
    builder.decal(DecalParams {
        region: EpuRegion::Sky,
        blend: EpuBlend::Add,
        shape: DecalShape::Disk,
        dir: Vec3::Y,
        color: 15,
        intensity: 255,
        softness_q: 2,
        size: 12,
        pulse_speed: 0,
    });
    let config = epu_finish(builder);

    // Feature should be in slot 4
    let layer = config.layers[4];

    let opcode = (layer >> 60) & 0xF;
    assert_eq!(opcode, EpuOpcode::Decal as u64);

    let region = (layer >> 58) & 0x3;
    assert_eq!(region, EpuRegion::Sky as u64);

    // Check shape and softness in param_a
    let param_a = (layer >> 32) & 0xFF;
    let shape = (param_a >> 4) & 0x0F;
    let softness = param_a & 0x0F;

    assert_eq!(shape, DecalShape::Disk as u64);
    assert_eq!(softness, 2);

    let size = (layer >> 24) & 0xFF;
    assert_eq!(size, 12);
}

#[test]
fn test_builder_scatter() {
    let mut builder = epu_begin();
    builder.scatter(ScatterParams {
        region: EpuRegion::All,
        blend: EpuBlend::Add,
        color: 15,
        intensity: 255,
        density: 200,
        size: 20,
        twinkle_q: 8,
        seed: 3,
    });
    let config = epu_finish(builder);

    let layer = config.layers[4];

    let opcode = (layer >> 60) & 0xF;
    assert_eq!(opcode, EpuOpcode::Scatter as u64);

    // Check param_c packing
    let param_c = (layer >> 16) & 0xFF;
    let twinkle = (param_c >> 4) & 0x0F;
    let seed = param_c & 0x0F;

    assert_eq!(twinkle, 8);
    assert_eq!(seed, 3);
}

#[test]
fn test_builder_grid() {
    let mut builder = epu_begin();
    builder.grid(GridParams {
        region: EpuRegion::Walls,
        blend: EpuBlend::Add,
        color: 64,
        intensity: 128,
        scale: 32,
        thickness: 20,
        pattern: GridPattern::Grid,
        scroll_q: 5,
    });
    let config = epu_finish(builder);

    let layer = config.layers[4];

    let opcode = (layer >> 60) & 0xF;
    assert_eq!(opcode, EpuOpcode::Grid as u64);

    let region = (layer >> 58) & 0x3;
    assert_eq!(region, EpuRegion::Walls as u64);

    // Check param_c packing
    let param_c = (layer >> 16) & 0xFF;
    let pattern = (param_c >> 4) & 0x0F;
    let scroll = param_c & 0x0F;

    assert_eq!(pattern, GridPattern::Grid as u64);
    assert_eq!(scroll, 5);
}

#[test]
fn test_builder_flow() {
    let mut builder = epu_begin();
    builder.flow(FlowParams {
        region: EpuRegion::Sky,
        blend: EpuBlend::Lerp,
        dir: Vec3::X,
        color: 15,
        intensity: 60,
        scale: 32,
        speed: 20,
        octaves: 2,
        pattern: FlowPattern::Caustic,
    });
    let config = epu_finish(builder);

    let layer = config.layers[4];

    let opcode = (layer >> 60) & 0xF;
    assert_eq!(opcode, EpuOpcode::Flow as u64);

    let blend = (layer >> 56) & 0x3;
    assert_eq!(blend, EpuBlend::Lerp as u64);

    // Check param_c packing
    let param_c = (layer >> 16) & 0xFF;
    let octaves = (param_c >> 4) & 0x0F;
    let pattern = param_c & 0x0F;

    assert_eq!(octaves, 2);
    assert_eq!(pattern, FlowPattern::Caustic as u64);
}

#[test]
fn test_builder_slot_allocation() {
    let mut builder = epu_begin();

    // Add bounds layers (slots 0-3)
    builder.ramp_enclosure(RampParams {
        up: Vec3::Y,
        wall_color: 0,
        sky_color: 0,
        floor_color: 0,
        ceil_q: 8,
        floor_q: 8,
        softness: 128,
    });
    builder.lobe(Vec3::Y, 1, 100, 32, 0, 0);
    builder.band(Vec3::Y, 2, 100, 64, 128, 0);
    builder.fog(Vec3::Y, 3, 50, 128, 100);

    // Add feature layers (slots 4-7)
    builder.decal(DecalParams::default());
    builder.grid(GridParams::default());
    builder.scatter(ScatterParams::default());
    builder.flow(FlowParams::default());

    let config = epu_finish(builder);

    // Verify bounds slots
    assert_eq!((config.layers[0] >> 60) & 0xF, EpuOpcode::Ramp as u64);
    assert_eq!((config.layers[1] >> 60) & 0xF, EpuOpcode::Lobe as u64);
    assert_eq!((config.layers[2] >> 60) & 0xF, EpuOpcode::Band as u64);
    assert_eq!((config.layers[3] >> 60) & 0xF, EpuOpcode::Fog as u64);

    // Verify feature slots
    assert_eq!((config.layers[4] >> 60) & 0xF, EpuOpcode::Decal as u64);
    assert_eq!((config.layers[5] >> 60) & 0xF, EpuOpcode::Grid as u64);
    assert_eq!((config.layers[6] >> 60) & 0xF, EpuOpcode::Scatter as u64);
    assert_eq!((config.layers[7] >> 60) & 0xF, EpuOpcode::Flow as u64);
}

#[test]
fn test_builder_bounds_overflow_ignored() {
    let mut builder = epu_begin();

    // Add 5 bounds layers (only 4 slots available)
    builder.lobe(Vec3::Y, 1, 100, 32, 0, 0);
    builder.lobe(Vec3::X, 2, 100, 32, 0, 0);
    builder.lobe(-Vec3::Y, 3, 100, 32, 0, 0);
    builder.lobe(-Vec3::X, 4, 100, 32, 0, 0);
    builder.lobe(Vec3::Z, 5, 100, 32, 0, 0); // This should be ignored

    let config = epu_finish(builder);

    // 5th lobe should not appear anywhere
    for (i, layer) in config.layers.iter().enumerate() {
        if i < 4 {
            // Bounds slots should have lobes with colors 1-4
            let color = (*layer >> 48) & 0xFF;
            assert_eq!(
                color,
                (i + 1) as u64,
                "Slot {i} should have color {}",
                i + 1
            );
        } else {
            // Feature slots should be empty (NOP)
            assert_eq!(*layer, 0, "Feature slot {i} should be NOP");
        }
    }
}

#[test]
fn test_builder_feature_overflow_ignored() {
    let mut builder = epu_begin();

    // Add 5 feature layers (only 4 slots available)
    for i in 0..5 {
        builder.decal(DecalParams {
            color: i as u8,
            ..DecalParams::default()
        });
    }

    let config = epu_finish(builder);

    // Only first 4 should appear in slots 4-7
    for i in 4..8 {
        let color = (config.layers[i] >> 48) & 0xFF;
        assert_eq!(
            color,
            (i - 4) as u64,
            "Slot {i} should have color {}",
            i - 4
        );
    }
}

// =============================================================================
// Config Size Test
// =============================================================================

#[test]
fn test_epu_config_size() {
    // EpuConfig must be exactly 64 bytes
    assert_eq!(
        std::mem::size_of::<EpuConfig>(),
        64,
        "EpuConfig must be exactly 64 bytes"
    );
}

// =============================================================================
// Example Config Tests (from RFC)
// =============================================================================

#[test]
fn test_void_with_stars() {
    let mut e = epu_begin();

    // Fully closed "void": make everything black, minimal softness.
    e.ramp_enclosure(RampParams {
        up: Vec3::Y,
        wall_color: 0,
        sky_color: 0,
        floor_color: 0,
        ceil_q: 15,
        floor_q: 0,
        softness: 10,
    });

    // Stars are the only light source: emissive by using blend Add.
    e.scatter(ScatterParams {
        region: EpuRegion::All,
        blend: EpuBlend::Add,
        color: 15,
        intensity: 255,
        density: 200,
        size: 20,
        twinkle_q: 8,
        seed: 3,
    });

    let config = epu_finish(e);

    // Verify RAMP in slot 0
    assert_eq!((config.layers[0] >> 60) & 0xF, EpuOpcode::Ramp as u64);

    // Verify SCATTER in slot 4
    assert_eq!((config.layers[4] >> 60) & 0xF, EpuOpcode::Scatter as u64);

    // Verify scatter has ADD blend (emissive)
    assert_eq!((config.layers[4] >> 56) & 0x3, EpuBlend::Add as u64);
}

#[test]
fn test_sunny_meadow() {
    let sun_dir = Vec3::new(0.5, 0.7, 0.3).normalize();

    let mut e = epu_begin();

    // Open-ish sky enclosure.
    e.ramp_enclosure(RampParams {
        up: Vec3::Y,
        wall_color: 24,
        sky_color: 40,
        floor_color: 52,
        ceil_q: 10,
        floor_q: 5,
        softness: 180,
    });

    e.lobe(sun_dir, 20, 180, 32, 0, 0);

    // Sun disk: emissive feature (blend Add).
    e.decal(DecalParams {
        region: EpuRegion::Sky,
        blend: EpuBlend::Add,
        shape: DecalShape::Disk,
        dir: sun_dir,
        color: 15,
        intensity: 255,
        softness_q: 2,
        size: 12,
        pulse_speed: 0,
    });

    let config = epu_finish(e);

    // Verify structure
    assert_eq!((config.layers[0] >> 60) & 0xF, EpuOpcode::Ramp as u64);
    assert_eq!((config.layers[1] >> 60) & 0xF, EpuOpcode::Lobe as u64);
    assert_eq!((config.layers[4] >> 60) & 0xF, EpuOpcode::Decal as u64);
}

// =============================================================================
// State Hash Tests
// =============================================================================

#[test]
fn test_state_hash_stability() {
    // Same config should produce same hash
    let config1 = EpuConfig {
        layers: [1, 2, 3, 4, 5, 6, 7, 8],
    };
    let config2 = EpuConfig {
        layers: [1, 2, 3, 4, 5, 6, 7, 8],
    };

    assert_eq!(config1.state_hash(), config2.state_hash());
}

#[test]
fn test_state_hash_differs_for_different_configs() {
    let config1 = EpuConfig {
        layers: [1, 2, 3, 4, 5, 6, 7, 8],
    };
    let config2 = EpuConfig {
        layers: [1, 2, 3, 4, 5, 6, 7, 9], // Last layer different
    };

    assert_ne!(config1.state_hash(), config2.state_hash());
}

#[test]
fn test_state_hash_empty_config() {
    let config = EpuConfig::default();
    // Should not panic and should produce a stable hash
    let hash1 = config.state_hash();
    let hash2 = config.state_hash();
    assert_eq!(hash1, hash2);
}

// =============================================================================
// Time Dependent Tests
// =============================================================================

#[test]
fn test_is_time_dependent_static_config() {
    // Config with no animated features
    let mut e = epu_begin();
    e.ramp_enclosure(RampParams::default());
    e.lobe(Vec3::Y, 20, 180, 32, 0, 0); // anim_speed=0, anim_mode=0
    e.decal(DecalParams {
        pulse_speed: 0, // No pulse
        ..DecalParams::default()
    });
    let config = epu_finish(e);

    assert!(!config.is_time_dependent());
}

#[test]
fn test_is_time_dependent_lobe_animation() {
    let mut e = epu_begin();
    e.lobe(Vec3::Y, 20, 180, 32, 10, 1); // anim_mode=1 (pulse)
    let config = epu_finish(e);

    assert!(config.is_time_dependent());
}

#[test]
fn test_is_time_dependent_band_scroll() {
    let mut e = epu_begin();
    e.band(Vec3::Y, 30, 200, 64, 128, 50); // scroll_speed=50
    let config = epu_finish(e);

    assert!(config.is_time_dependent());
}

#[test]
fn test_is_time_dependent_band_static() {
    let mut e = epu_begin();
    e.band(Vec3::Y, 30, 200, 64, 128, 0); // scroll_speed=0
    let config = epu_finish(e);

    assert!(!config.is_time_dependent());
}

#[test]
fn test_is_time_dependent_decal_pulse() {
    let mut e = epu_begin();
    e.decal(DecalParams {
        pulse_speed: 20, // Pulsing
        ..DecalParams::default()
    });
    let config = epu_finish(e);

    assert!(config.is_time_dependent());
}

#[test]
fn test_is_time_dependent_grid_scroll() {
    let mut e = epu_begin();
    e.grid(GridParams {
        scroll_q: 5, // Scrolling
        ..GridParams::default()
    });
    let config = epu_finish(e);

    assert!(config.is_time_dependent());
}

#[test]
fn test_is_time_dependent_grid_static() {
    let mut e = epu_begin();
    e.grid(GridParams {
        scroll_q: 0, // No scroll
        ..GridParams::default()
    });
    let config = epu_finish(e);

    assert!(!config.is_time_dependent());
}

#[test]
fn test_is_time_dependent_scatter_twinkle() {
    let mut e = epu_begin();
    e.scatter(ScatterParams {
        twinkle_q: 8, // Twinkling
        ..ScatterParams::default()
    });
    let config = epu_finish(e);

    assert!(config.is_time_dependent());
}

#[test]
fn test_is_time_dependent_scatter_static() {
    let mut e = epu_begin();
    e.scatter(ScatterParams {
        twinkle_q: 0, // No twinkle
        ..ScatterParams::default()
    });
    let config = epu_finish(e);

    assert!(!config.is_time_dependent());
}

#[test]
fn test_is_time_dependent_flow_animated() {
    let mut e = epu_begin();
    e.flow(FlowParams {
        speed: 20, // Animated
        ..FlowParams::default()
    });
    let config = epu_finish(e);

    assert!(config.is_time_dependent());
}

#[test]
fn test_is_time_dependent_flow_static() {
    let mut e = epu_begin();
    e.flow(FlowParams {
        speed: 0, // No animation
        ..FlowParams::default()
    });
    let config = epu_finish(e);

    assert!(!config.is_time_dependent());
}

#[test]
fn test_is_time_dependent_empty_config() {
    let config = EpuConfig::default();
    assert!(!config.is_time_dependent());
}
