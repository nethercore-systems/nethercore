//! Unit tests for EPU Rust API (128-bit format)

use super::*;
use glam::Vec3;

// =============================================================================
// Bit Packing Tests (128-bit format)
// =============================================================================

#[test]
fn test_epu_layer_encode_nop() {
    let layer = EpuLayer::nop();
    let [hi, _lo] = layer.encode();

    // NOP should have opcode 0 and mostly zeros
    let opcode = (hi >> 59) & 0x1F;
    assert_eq!(opcode, 0);
}

#[test]
fn test_epu_layer_encode_opcode_position() {
    // Test that opcode is in bits 63..59 of hi word
    let layer = EpuLayer {
        opcode: EpuOpcode::Flow, // 0xB
        ..EpuLayer::nop()
    };
    let [hi, _lo] = layer.encode();

    let opcode = (hi >> 59) & 0x1F;
    assert_eq!(opcode, 0xB);
}

#[test]
fn test_epu_layer_encode_region_position() {
    // Test that region is in bits 58..56 of hi word
    let layer = EpuLayer {
        region_mask: REGION_FLOOR, // 0b001
        ..EpuLayer::nop()
    };
    let [hi, _lo] = layer.encode();

    let region = (hi >> 56) & 0x7;
    assert_eq!(region, 0b001);
}

#[test]
fn test_epu_layer_encode_blend_position() {
    // Test that blend is in bits 55..53 of hi word
    let layer = EpuLayer {
        blend: EpuBlend::Lerp, // 3
        ..EpuLayer::nop()
    };
    let [hi, _lo] = layer.encode();

    let blend = (hi >> 53) & 0x7;
    assert_eq!(blend, 3);
}

#[test]
fn test_epu_layer_encode_meta5_position() {
    // meta5 occupies bits 52..48 of the hi word.
    let layer = EpuLayer {
        meta5: 0b1_0101,
        ..EpuLayer::nop()
    };
    let [hi, _lo] = layer.encode();

    let meta5 = (hi >> 48) & 0x1F;
    assert_eq!(meta5, 0b1_0101);
}

#[test]
fn test_epu_layer_encode_color_a() {
    let layer = EpuLayer {
        color_a: [0xAB, 0xCD, 0xEF],
        ..EpuLayer::nop()
    };
    let [hi, _lo] = layer.encode();

    // color_a is in bits 47..24 of hi word (RGB24)
    let color_a = (hi >> 24) & 0xFFFFFF;
    assert_eq!(color_a, 0xABCDEF);
}

#[test]
fn test_epu_layer_encode_color_b() {
    let layer = EpuLayer {
        color_b: [0x12, 0x34, 0x56],
        ..EpuLayer::nop()
    };
    let [hi, _lo] = layer.encode();

    // color_b is in bits 23..0 of hi word (RGB24)
    let color_b = hi & 0xFFFFFF;
    assert_eq!(color_b, 0x123456);
}

#[test]
fn test_epu_layer_encode_intensity() {
    let layer = EpuLayer {
        intensity: 0xCD,
        ..EpuLayer::nop()
    };
    let [_hi, lo] = layer.encode();

    // intensity is in bits 63..56 of lo word
    let intensity = (lo >> 56) & 0xFF;
    assert_eq!(intensity, 0xCD);
}

#[test]
fn test_epu_layer_encode_params() {
    let layer = EpuLayer {
        param_a: 0x11,
        param_b: 0x22,
        param_c: 0x33,
        param_d: 0x44,
        ..EpuLayer::nop()
    };
    let [_hi, lo] = layer.encode();

    let param_a = (lo >> 48) & 0xFF;
    let param_b = (lo >> 40) & 0xFF;
    let param_c = (lo >> 32) & 0xFF;
    let param_d = (lo >> 24) & 0xFF;

    assert_eq!(param_a, 0x11);
    assert_eq!(param_b, 0x22);
    assert_eq!(param_c, 0x33);
    assert_eq!(param_d, 0x44);
}

#[test]
fn test_epu_layer_encode_direction() {
    let layer = EpuLayer {
        direction: 0xBEEF,
        ..EpuLayer::nop()
    };
    let [_hi, lo] = layer.encode();

    // direction is in bits 23..8 of lo word
    let direction = (lo >> 8) & 0xFFFF;
    assert_eq!(direction, 0xBEEF);
}

#[test]
fn test_epu_layer_encode_alpha() {
    let layer = EpuLayer {
        alpha_a: 0xA,
        alpha_b: 0xB,
        ..EpuLayer::nop()
    };
    let [_hi, lo] = layer.encode();

    // alpha_a is in bits 7..4 of lo word
    // alpha_b is in bits 3..0 of lo word
    let alpha_a = (lo >> 4) & 0xF;
    let alpha_b = lo & 0xF;

    assert_eq!(alpha_a, 0xA);
    assert_eq!(alpha_b, 0xB);
}

#[test]
fn test_epu_layer_encode_full() {
    // Test a fully populated layer
    let layer = EpuLayer {
        opcode: EpuOpcode::Decal, // 0x8
        region_mask: REGION_SKY,  // 0b100
        blend: EpuBlend::Add,     // 0
        meta5: 0x1B,
        color_a: [0xFF, 0x00, 0x00], // red
        color_b: [0x00, 0xFF, 0x00], // green
        alpha_a: 15,
        alpha_b: 8,
        intensity: 200,
        param_a: 0x12,
        param_b: 0x34,
        param_c: 0x56,
        param_d: 0x78,
        direction: 0x7890,
    };
    let [hi, lo] = layer.encode();

    // Verify hi word fields
    assert_eq!((hi >> 59) & 0x1F, 0x8); // opcode
    assert_eq!((hi >> 56) & 0x7, 0b100); // region (SKY)
    assert_eq!((hi >> 53) & 0x7, 0); // blend (ADD)
    assert_eq!((hi >> 48) & 0x1F, 0x1B); // meta5
    assert_eq!((hi >> 24) & 0xFFFFFF, 0xFF0000); // color_a (red)
    assert_eq!(hi & 0xFFFFFF, 0x00FF00); // color_b (green)

    // Verify lo word fields
    assert_eq!((lo >> 56) & 0xFF, 200); // intensity
    assert_eq!((lo >> 48) & 0xFF, 0x12); // param_a
    assert_eq!((lo >> 40) & 0xFF, 0x34); // param_b
    assert_eq!((lo >> 32) & 0xFF, 0x56); // param_c
    assert_eq!((lo >> 24) & 0xFF, 0x78); // param_d
    assert_eq!((lo >> 8) & 0xFFFF, 0x7890); // direction
    assert_eq!((lo >> 4) & 0xF, 15); // alpha_a
    assert_eq!(lo & 0xF, 8); // alpha_b
}

// =============================================================================
// Direction Encoding Tests
// =============================================================================

#[test]
fn test_encode_direction_u16_up() {
    let encoded = encode_direction_u16(Vec3::Y);

    let u = (encoded & 0xFF) as u8;
    let v = ((encoded >> 8) & 0xFF) as u8;

    // u should be near center (127 or 128), v should be 255
    assert!((125..=130).contains(&u), "u = {u}, expected ~127-128");
    assert_eq!(v, 255, "v = {v}, expected 255 for +Y");
}

#[test]
fn test_encode_direction_u16_forward() {
    let encoded = encode_direction_u16(Vec3::Z);

    let u = (encoded & 0xFF) as u8;
    let v = ((encoded >> 8) & 0xFF) as u8;

    // Both should be near center (127 or 128)
    assert!((125..=130).contains(&u), "u = {u}, expected ~127-128");
    assert!((125..=130).contains(&v), "v = {v}, expected ~127-128");
}

#[test]
fn test_encode_direction_u16_neg_z() {
    let encoded = encode_direction_u16(-Vec3::Z);

    let u = (encoded & 0xFF) as u8;
    let v = ((encoded >> 8) & 0xFF) as u8;

    // Both should be at extremes due to octahedral wrapping
    assert!(u == 0 || u == 255 || (125..=130).contains(&u), "u = {u}");
    assert!(v == 0 || v == 255 || (125..=130).contains(&v), "v = {v}");
}

#[test]
fn test_encode_direction_u16_right() {
    let encoded = encode_direction_u16(Vec3::X);

    let u = (encoded & 0xFF) as u8;
    let v = ((encoded >> 8) & 0xFF) as u8;

    // +X should be offset from center
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

    for [hi, _lo] in config.layers {
        // NOP opcode is 0
        let opcode = (hi >> 59) & 0x1F;
        assert_eq!(opcode, 0, "Default layer should have NOP opcode");
    }
}

#[test]
fn test_builder_ramp_bounds() {
    let mut builder = epu_begin();
    builder.ramp_bounds(RampParams {
        up: Vec3::Y,
        wall_color: [200, 180, 150],
        sky_color: [100, 150, 220],
        floor_color: [80, 140, 80],
        ceil_q: 10,
        floor_q: 5,
        softness: 180,
    });
    let config = epu_finish(builder);

    let [hi, lo] = config.layers[0];

    // Check opcode is RAMP
    let opcode = (hi >> 59) & 0x1F;
    assert_eq!(opcode, EpuOpcode::Ramp as u64);

    // Check meta5 is 0
    let meta5 = (hi >> 48) & 0x1F;
    assert_eq!(meta5, 0);

    // Check sky color (color_a)
    let color_a = (hi >> 24) & 0xFFFFFF;
    assert_eq!(color_a, 0x6496DC); // [100, 150, 220]

    // Check floor color (color_b)
    let color_b = hi & 0xFFFFFF;
    assert_eq!(color_b, 0x508C50); // [80, 140, 80]

    // Check softness (intensity)
    let softness = (lo >> 56) & 0xFF;
    assert_eq!(softness, 180);

    // Check thresholds (param_d)
    let param_d = (lo >> 24) & 0xFF;
    let ceil_q = (param_d >> 4) & 0x0F;
    let floor_q = param_d & 0x0F;
    assert_eq!(ceil_q, 10);
    assert_eq!(floor_q, 5);
}

#[test]
fn test_builder_sector_bounds() {
    let mut builder = epu_begin();
    builder.ramp_bounds(RampParams::default());
    builder.sector_bounds(SectorParams {
        up: Vec3::Y,
        sky_color: [255, 200, 100],
        wall_color: [10, 20, 30],
        strength: 180,
        center_u01: 64,
        width: 128,
        variant_id: 2,
    });
    let config = epu_finish(builder);

    let [hi, lo] = config.layers[1];

    let opcode = (hi >> 59) & 0x1F;
    assert_eq!(opcode, EpuOpcode::Sector as u64);

    // Check meta5 (domain=0, variant=2)
    let meta5 = (hi >> 48) & 0x1F;
    assert_eq!(meta5, pack_meta5(0, 2) as u64);

    // Check color
    let color_a = (hi >> 24) & 0xFFFFFF;
    assert_eq!(color_a, 0xFFC864); // [255, 200, 100]

    let intensity = (lo >> 56) & 0xFF;
    assert_eq!(intensity, 180);

    let center_u01 = (lo >> 48) & 0xFF;
    assert_eq!(center_u01, 64);

    let width = (lo >> 40) & 0xFF;
    assert_eq!(width, 128);
}

#[test]
fn test_builder_lobe_radiance() {
    let mut builder = epu_begin();
    builder.lobe_radiance(LobeRadianceParams {
        region: EpuRegion::All,
        blend: EpuBlend::Add,
        dir: Vec3::Y,
        color: [255, 200, 100],
        edge_color: [0, 0, 0],
        intensity: 180,
        exponent: 32,
        falloff: 10,
        waveform: PhaseWaveform::Sine,
        phase: 10,
        alpha: 15,
    });
    let config = epu_finish(builder);

    let [hi, _lo] = config.layers[4];
    let opcode = (hi >> 59) & 0x1F;
    assert_eq!(opcode, EpuOpcode::LobeRadiance as u64);
}

#[test]
fn test_builder_band_radiance() {
    let mut builder = epu_begin();
    builder.band_radiance(BandRadianceParams {
        region: EpuRegion::All,
        blend: EpuBlend::Add,
        axis: Vec3::Y,
        color: [255, 128, 64],
        edge_color: [0, 0, 0],
        intensity: 200,
        width: 64,
        offset: 128,
        softness: 0,
        phase: 50,
        alpha: 15,
    });
    let config = epu_finish(builder);

    let [hi, _lo] = config.layers[4];
    let opcode = (hi >> 59) & 0x1F;
    assert_eq!(opcode, EpuOpcode::BandRadiance as u64);
}

#[test]
fn test_builder_atmosphere_absorption() {
    let mut builder = epu_begin();
    builder.atmosphere(AtmosphereParams {
        region: EpuRegion::All,
        blend: EpuBlend::Multiply,
        zenith_color: [200, 200, 220],
        horizon_color: [200, 200, 220],
        intensity: 128,
        falloff_exponent: 100,
        horizon_y: 128,
        mie_concentration: 0,
        mie_exponent: 0,
        sun_dir: Vec3::Y,
        alpha: 15,
        variant_id: 0, // ABSORPTION
    });
    let config = epu_finish(builder);

    let [hi, _lo] = config.layers[4];

    let opcode = (hi >> 59) & 0x1F;
    assert_eq!(opcode, EpuOpcode::Atmosphere as u64);

    // Absorption is typically used with MULTIPLY blend.
    let blend = (hi >> 53) & 0x7;
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
        color: [255, 255, 255],
        color_b: [200, 180, 120],
        intensity: 255,
        softness_q: 2,
        size: 12,
        glow_softness: 64,
        phase: 0,
        alpha: 15,
    });
    let config = epu_finish(builder);

    // Feature should be in slot 4
    let [hi, lo] = config.layers[4];

    let opcode = (hi >> 59) & 0x1F;
    assert_eq!(opcode, EpuOpcode::Decal as u64);

    let region = (hi >> 56) & 0x7;
    assert_eq!(region, REGION_SKY as u64);

    // Check meta5 is 0
    let meta5 = (hi >> 48) & 0x1F;
    assert_eq!(meta5, 0);

    // Check alpha_a
    let alpha_a = (lo >> 4) & 0xF;
    assert_eq!(alpha_a, 15);

    // Check shape and softness in param_a
    let param_a = (lo >> 48) & 0xFF;
    let shape = (param_a >> 4) & 0x0F;
    let softness = param_a & 0x0F;

    assert_eq!(shape, DecalShape::Disk as u64);
    assert_eq!(softness, 2);

    let size = (lo >> 40) & 0xFF;
    assert_eq!(size, 12);
}

#[test]
fn test_builder_scatter() {
    let mut builder = epu_begin();
    builder.scatter(ScatterParams {
        region: EpuRegion::All,
        blend: EpuBlend::Add,
        color: [255, 255, 255],
        intensity: 255,
        density: 200,
        size: 20,
        twinkle_q: 8,
        seed: 3,
    });
    let config = epu_finish(builder);

    let [hi, lo] = config.layers[4];

    let opcode = (hi >> 59) & 0x1F;
    assert_eq!(opcode, EpuOpcode::Scatter as u64);

    // Check param_c packing (high nibble only; low nibble reserved)
    let param_c = (lo >> 32) & 0xFF;
    let twinkle = (param_c >> 4) & 0x0F;
    let reserved = param_c & 0x0F;

    assert_eq!(twinkle, 8);
    assert_eq!(reserved, 0);

    // Seed is stored in param_d
    let seed = (lo >> 24) & 0xFF;
    assert_eq!(seed, 3);
}

#[test]
fn test_builder_grid() {
    let mut builder = epu_begin();
    builder.grid(GridParams {
        region: EpuRegion::Walls,
        blend: EpuBlend::Add,
        color: [64, 64, 64],
        intensity: 128,
        scale: 32,
        thickness: 20,
        pattern: GridPattern::Grid,
        scroll_q: 5,
        phase: 0,
    });
    let config = epu_finish(builder);

    let [hi, lo] = config.layers[4];

    let opcode = (hi >> 59) & 0x1F;
    assert_eq!(opcode, EpuOpcode::Grid as u64);

    let region = (hi >> 56) & 0x7;
    assert_eq!(region, REGION_WALLS as u64);

    // Check param_c packing
    let param_c = (lo >> 32) & 0xFF;
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
        color: [200, 200, 255],
        intensity: 60,
        scale: 32,
        phase: 20,
        octaves: 2,
        pattern: FlowPattern::Caustic,
        turbulence: 0,
    });
    let config = epu_finish(builder);

    let [hi, lo] = config.layers[4];

    let opcode = (hi >> 59) & 0x1F;
    assert_eq!(opcode, EpuOpcode::Flow as u64);

    let blend = (hi >> 53) & 0x7;
    assert_eq!(blend, EpuBlend::Lerp as u64);

    // Check meta5 is 0
    let meta5 = (hi >> 48) & 0x1F;
    assert_eq!(meta5, 0);

    // Check param_c packing
    let param_c = (lo >> 32) & 0xFF;
    let octaves = (param_c >> 4) & 0x0F;
    let pattern = param_c & 0x0F;

    assert_eq!(octaves, 2);
    assert_eq!(pattern, FlowPattern::Caustic as u64);
}

#[test]
fn test_builder_slot_allocation() {
    let mut builder = epu_begin();

    // Add bounds layers (slots 0-3)
    builder.ramp_bounds(RampParams::default());
    builder.sector_bounds(SectorParams::default());
    builder.split_bounds(SplitParams::default());
    builder.cell_bounds(CellParams::default());

    // Add feature layers (slots 4-7)
    builder.decal(DecalParams::default());
    builder.grid(GridParams::default());
    builder.scatter(ScatterParams::default());
    builder.flow(FlowParams::default());

    let config = epu_finish(builder);

    // Verify bounds slots
    assert_eq!((config.layers[0][0] >> 59) & 0x1F, EpuOpcode::Ramp as u64);
    assert_eq!((config.layers[1][0] >> 59) & 0x1F, EpuOpcode::Sector as u64);
    assert_eq!((config.layers[2][0] >> 59) & 0x1F, EpuOpcode::Split as u64);
    assert_eq!((config.layers[3][0] >> 59) & 0x1F, EpuOpcode::Cell as u64);

    // Verify feature slots
    assert_eq!((config.layers[4][0] >> 59) & 0x1F, EpuOpcode::Decal as u64);
    assert_eq!((config.layers[5][0] >> 59) & 0x1F, EpuOpcode::Grid as u64);
    assert_eq!(
        (config.layers[6][0] >> 59) & 0x1F,
        EpuOpcode::Scatter as u64
    );
    assert_eq!((config.layers[7][0] >> 59) & 0x1F, EpuOpcode::Flow as u64);
}

#[test]
fn test_builder_bounds_overflow_ignored() {
    let mut builder = epu_begin();

    // Add 5 bounds layers (only 4 slots available)
    builder.sector_bounds(SectorParams {
        sky_color: [255, 0, 0],
        ..SectorParams::default()
    });
    builder.sector_bounds(SectorParams {
        sky_color: [0, 255, 0],
        ..SectorParams::default()
    });
    builder.sector_bounds(SectorParams {
        sky_color: [0, 0, 255],
        ..SectorParams::default()
    });
    builder.sector_bounds(SectorParams {
        sky_color: [255, 255, 0],
        ..SectorParams::default()
    });
    builder.sector_bounds(SectorParams {
        sky_color: [255, 0, 255],
        ..SectorParams::default()
    }); // This should be ignored

    let config = epu_finish(builder);

    // 5th layer should not appear anywhere
    for (i, [hi, _lo]) in config.layers.iter().enumerate() {
        if i < 4 {
            // Bounds slots should have sectors
            let opcode = (hi >> 59) & 0x1F;
            assert_eq!(opcode, EpuOpcode::Sector as u64);
        } else {
            // Feature slots should be empty (NOP)
            let opcode = (hi >> 59) & 0x1F;
            assert_eq!(opcode, 0, "Feature slot {i} should be NOP");
        }
    }
}

#[test]
fn test_builder_feature_overflow_ignored() {
    let mut builder = epu_begin();

    // Add 5 feature layers (only 4 slots available)
    for i in 0..5u8 {
        builder.decal(DecalParams {
            color: [i * 50, i * 40, i * 30],
            ..DecalParams::default()
        });
    }

    let config = epu_finish(builder);

    // Only first 4 should appear in slots 4-7
    for i in 4..8 {
        let opcode = (config.layers[i][0] >> 59) & 0x1F;
        assert_eq!(opcode, EpuOpcode::Decal as u64);
    }
}

// =============================================================================
// Config Size Test
// =============================================================================

#[test]
fn test_epu_config_size() {
    // EpuConfig must be exactly 128 bytes
    assert_eq!(
        std::mem::size_of::<EpuConfig>(),
        128,
        "EpuConfig must be exactly 128 bytes"
    );
}

// =============================================================================
// Example Config Tests
// =============================================================================

#[test]
fn test_void_with_stars() {
    let mut e = epu_begin();

    // Fully closed "void": make everything black, minimal softness.
    e.ramp_bounds(RampParams {
        up: Vec3::Y,
        wall_color: [0, 0, 0],
        sky_color: [0, 0, 0],
        floor_color: [0, 0, 0],
        ceil_q: 15,
        floor_q: 0,
        softness: 10,
    });

    // Stars.
    e.scatter(ScatterParams {
        region: EpuRegion::All,
        blend: EpuBlend::Add,
        color: [255, 255, 255],
        intensity: 255,
        density: 200,
        size: 20,
        twinkle_q: 8,
        seed: 3,
    });

    let config = epu_finish(e);

    // Verify RAMP in slot 0
    assert_eq!((config.layers[0][0] >> 59) & 0x1F, EpuOpcode::Ramp as u64);

    // Verify SCATTER in slot 4
    assert_eq!(
        (config.layers[4][0] >> 59) & 0x1F,
        EpuOpcode::Scatter as u64
    );

    // Verify meta5 is 0
    let meta5 = (config.layers[4][0] >> 48) & 0x1F;
    assert_eq!(meta5, 0);
}

#[test]
fn test_sunny_meadow() {
    let sun_dir = Vec3::new(0.5, 0.7, 0.3).normalize();

    let mut e = epu_begin();

    // Open-ish sky bounds.
    e.ramp_bounds(RampParams {
        up: Vec3::Y,
        wall_color: [200, 180, 150],
        sky_color: [100, 150, 220],
        floor_color: [80, 140, 80],
        ceil_q: 10,
        floor_q: 5,
        softness: 180,
    });

    e.lobe_radiance(LobeRadianceParams {
        dir: sun_dir,
        color: [255, 240, 200],
        intensity: 180,
        exponent: 32,
        ..LobeRadianceParams::default()
    });

    // Sun disk.
    e.decal(DecalParams {
        region: EpuRegion::Sky,
        blend: EpuBlend::Add,
        shape: DecalShape::Disk,
        dir: sun_dir,
        color: [255, 255, 255],
        color_b: [255, 220, 180],
        intensity: 255,
        softness_q: 2,
        size: 12,
        glow_softness: 64,
        phase: 0,
        alpha: 15,
    });

    let config = epu_finish(e);

    // Verify structure
    assert_eq!((config.layers[0][0] >> 59) & 0x1F, EpuOpcode::Ramp as u64);
    assert_eq!(
        (config.layers[4][0] >> 59) & 0x1F,
        EpuOpcode::LobeRadiance as u64
    );
    assert_eq!((config.layers[5][0] >> 59) & 0x1F, EpuOpcode::Decal as u64);
}

// =============================================================================
// State Hash Tests
// =============================================================================

#[test]
fn test_state_hash_stability() {
    // Same config should produce same hash
    let config1 = EpuConfig {
        layers: [[1, 2]; 8],
    };
    let config2 = EpuConfig {
        layers: [[1, 2]; 8],
    };

    assert_eq!(config1.state_hash(), config2.state_hash());
}

#[test]
fn test_state_hash_differs_for_different_configs() {
    let config1 = EpuConfig {
        layers: [
            [1, 2],
            [3, 4],
            [5, 6],
            [7, 8],
            [9, 10],
            [11, 12],
            [13, 14],
            [15, 16],
        ],
    };
    let config2 = EpuConfig {
        layers: [
            [1, 2],
            [3, 4],
            [5, 6],
            [7, 8],
            [9, 10],
            [11, 12],
            [13, 14],
            [15, 17],
        ], // Different
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
// Region Mask Tests
// =============================================================================

#[test]
fn test_region_mask_constants() {
    assert_eq!(REGION_SKY, 0b100);
    assert_eq!(REGION_WALLS, 0b010);
    assert_eq!(REGION_FLOOR, 0b001);
    assert_eq!(REGION_ALL, 0b111);
    assert_eq!(REGION_NONE, 0b000);
}

#[test]
fn test_region_enum_to_mask() {
    assert_eq!(EpuRegion::All.to_mask(), REGION_ALL);
    assert_eq!(EpuRegion::Sky.to_mask(), REGION_SKY);
    assert_eq!(EpuRegion::Walls.to_mask(), REGION_WALLS);
    assert_eq!(EpuRegion::Floor.to_mask(), REGION_FLOOR);
}

// =============================================================================
// Blend Mode Tests
// =============================================================================

#[test]
fn test_blend_mode_values() {
    assert_eq!(EpuBlend::Add as u8, 0);
    assert_eq!(EpuBlend::Multiply as u8, 1);
    assert_eq!(EpuBlend::Max as u8, 2);
    assert_eq!(EpuBlend::Lerp as u8, 3);
    assert_eq!(EpuBlend::Screen as u8, 4);
    assert_eq!(EpuBlend::HsvMod as u8, 5);
    assert_eq!(EpuBlend::Min as u8, 6);
    assert_eq!(EpuBlend::Overlay as u8, 7);
}
