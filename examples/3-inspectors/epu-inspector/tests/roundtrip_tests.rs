//! Roundtrip tests for EPU mapping functions.
//!
//! These tests verify that raw <-> macro value conversions are consistent
//! and that pack/unpack operations preserve data.
//!
//! The mapping functions only use `libm` math, so they can be tested on the host.

use std::f32::consts::PI;

// ============================================================================
// Mapping Functions (copied from lib.rs for testing)
// ============================================================================

/// Convert f32 [0.0, 1.0] to u8 [0, 255]
#[inline]
fn f32_to_u8_01(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0) as u8
}

/// Convert u8 [0, 255] to f32 [0.0, 1.0]
#[inline]
fn u8_01_to_f32(v: u8) -> f32 {
    v as f32 / 255.0
}

/// Convert f32 [min, max] to u8 [0, 255] via linear interpolation
#[inline]
fn f32_to_u8_lerp(v: f32, min: f32, max: f32) -> u8 {
    if max <= min {
        return 0;
    }
    let t = (v - min) / (max - min);
    (t.clamp(0.0, 1.0) * 255.0) as u8
}

/// Convert u8 [0, 255] to f32 [min, max] via linear interpolation
#[inline]
fn u8_lerp_to_f32(v: u8, min: f32, max: f32) -> f32 {
    let t = v as f32 / 255.0;
    min + t * (max - min)
}

/// Convert f32 [0.0, 1.0] to u4 [0, 15]
#[inline]
fn f32_to_u4_01(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 15.0) as u8
}

/// Convert u4 [0, 15] to f32 [0.0, 1.0]
#[inline]
fn u4_01_to_f32(v: u8) -> f32 {
    (v & 0xF) as f32 / 15.0
}

/// Convert octahedral-encoded direction to (azimuth, elevation) in degrees
fn octahedral_to_angles(dir16: u16) -> (f32, f32) {
    let u_byte = (dir16 & 0xFF) as f32;
    let v_byte = ((dir16 >> 8) & 0xFF) as f32;

    let u = u_byte / 127.5 - 1.0;
    let v = v_byte / 127.5 - 1.0;

    let z = 1.0 - u.abs() - v.abs();
    let (x, y) = if z >= 0.0 {
        (u, v)
    } else {
        let sign_u = if u >= 0.0 { 1.0 } else { -1.0 };
        let sign_v = if v >= 0.0 { 1.0 } else { -1.0 };
        ((1.0 - v.abs()) * sign_u, (1.0 - u.abs()) * sign_v)
    };

    let len = (x * x + y * y + z * z).sqrt();
    let (nx, ny, nz) = if len > 0.0001 {
        (x / len, y / len, z / len)
    } else {
        (0.0, 0.0, 1.0)
    };

    let elevation = nz.clamp(-1.0, 1.0).asin() * 180.0 / PI;
    let azimuth = ny.atan2(nx) * 180.0 / PI;
    let azimuth = if azimuth < 0.0 { azimuth + 360.0 } else { azimuth };

    (azimuth, elevation)
}

/// Convert (azimuth, elevation) in degrees to octahedral encoding
fn angles_to_octahedral(azimuth: f32, elevation: f32) -> u16 {
    let az_rad = azimuth * PI / 180.0;
    let el_rad = elevation * PI / 180.0;

    let cos_el = el_rad.cos();
    let x = az_rad.cos() * cos_el;
    let y = az_rad.sin() * cos_el;
    let z = el_rad.sin();

    let sum = x.abs() + y.abs() + z.abs();
    let (mut u, mut v) = if sum > 0.0001 {
        (x / sum, y / sum)
    } else {
        (0.0, 0.0)
    };

    if z < 0.0 {
        let sign_u = if u >= 0.0 { 1.0 } else { -1.0 };
        let sign_v = if v >= 0.0 { 1.0 } else { -1.0 };
        let new_u = (1.0 - v.abs()) * sign_u;
        let new_v = (1.0 - u.abs()) * sign_v;
        u = new_u;
        v = new_v;
    }

    let u_byte = ((u + 1.0) * 127.5).clamp(0.0, 255.0) as u8;
    let v_byte = ((v + 1.0) * 127.5).clamp(0.0, 255.0) as u8;

    (u_byte as u16) | ((v_byte as u16) << 8)
}

// ============================================================================
// u8_01 Roundtrip Tests
// ============================================================================

#[test]
fn test_u8_01_roundtrip_zero() {
    let raw = 0u8;
    let macro_val = u8_01_to_f32(raw);
    let back = f32_to_u8_01(macro_val);
    assert_eq!(raw, back, "raw -> macro -> raw: {} -> {} -> {}", raw, macro_val, back);
    assert!((macro_val - 0.0).abs() < 0.001);
}

#[test]
fn test_u8_01_roundtrip_half() {
    // 127 should map to ~0.498, and 0.5 should map to 127
    let raw = 127u8;
    let macro_val = u8_01_to_f32(raw);
    let back = f32_to_u8_01(macro_val);
    assert_eq!(raw, back, "raw -> macro -> raw: {} -> {} -> {}", raw, macro_val, back);
    assert!((macro_val - 0.498).abs() < 0.01);
}

#[test]
fn test_u8_01_roundtrip_max() {
    let raw = 255u8;
    let macro_val = u8_01_to_f32(raw);
    let back = f32_to_u8_01(macro_val);
    assert_eq!(raw, back, "raw -> macro -> raw: {} -> {} -> {}", raw, macro_val, back);
    assert!((macro_val - 1.0).abs() < 0.001);
}

#[test]
fn test_u8_01_macro_to_raw_boundary() {
    // 0.0 -> 0
    assert_eq!(f32_to_u8_01(0.0), 0);
    // 0.5 -> 127 (truncated from 127.5)
    assert_eq!(f32_to_u8_01(0.5), 127);
    // 1.0 -> 255
    assert_eq!(f32_to_u8_01(1.0), 255);
}

#[test]
fn test_u8_01_clamping() {
    // Values below 0 should clamp
    assert_eq!(f32_to_u8_01(-0.5), 0);
    // Values above 1 should clamp
    assert_eq!(f32_to_u8_01(1.5), 255);
}

#[test]
fn test_u8_01_all_values_roundtrip() {
    // Every u8 value should roundtrip exactly
    for raw in 0u8..=255u8 {
        let macro_val = u8_01_to_f32(raw);
        let back = f32_to_u8_01(macro_val);
        assert_eq!(raw, back, "raw {} did not roundtrip (got {})", raw, back);
    }
}

// ============================================================================
// u8_lerp Roundtrip Tests
// ============================================================================

#[test]
fn test_u8_lerp_roundtrip_simple_range() {
    let min = 0.0f32;
    let max = 10.0f32;

    // 0 -> 0.0 -> 0
    let raw = 0u8;
    let macro_val = u8_lerp_to_f32(raw, min, max);
    let back = f32_to_u8_lerp(macro_val, min, max);
    assert_eq!(raw, back);
    assert!((macro_val - 0.0).abs() < 0.001);

    // 255 -> 10.0 -> 255
    let raw = 255u8;
    let macro_val = u8_lerp_to_f32(raw, min, max);
    let back = f32_to_u8_lerp(macro_val, min, max);
    assert_eq!(raw, back);
    assert!((macro_val - 10.0).abs() < 0.001);
}

#[test]
fn test_u8_lerp_roundtrip_negative_range() {
    let min = -1.0f32;
    let max = 1.0f32;

    // 0 -> -1.0 -> 0
    let raw = 0u8;
    let macro_val = u8_lerp_to_f32(raw, min, max);
    let back = f32_to_u8_lerp(macro_val, min, max);
    assert_eq!(raw, back);
    assert!((macro_val - (-1.0)).abs() < 0.001);

    // 127 -> ~0.0 -> 127
    let raw = 127u8;
    let macro_val = u8_lerp_to_f32(raw, min, max);
    let back = f32_to_u8_lerp(macro_val, min, max);
    assert_eq!(raw, back);
    assert!(macro_val.abs() < 0.02); // ~0.0

    // 255 -> 1.0 -> 255
    let raw = 255u8;
    let macro_val = u8_lerp_to_f32(raw, min, max);
    let back = f32_to_u8_lerp(macro_val, min, max);
    assert_eq!(raw, back);
    assert!((macro_val - 1.0).abs() < 0.001);
}

#[test]
fn test_u8_lerp_roundtrip_realistic_range() {
    // Pattern scale: 0.5 to 16.0 (from PLANE opcode)
    let min = 0.5f32;
    let max = 16.0f32;

    for raw in [0u8, 50, 100, 127, 200, 255] {
        let macro_val = u8_lerp_to_f32(raw, min, max);
        let back = f32_to_u8_lerp(macro_val, min, max);
        assert_eq!(raw, back, "raw {} did not roundtrip (got {})", raw, back);
    }
}

#[test]
fn test_u8_lerp_all_values_roundtrip() {
    // Use a simple range where roundtrip is exact
    let min = 0.0f32;
    let max = 255.0f32;

    // Every u8 value should roundtrip exactly with this range
    for raw in 0u8..=255u8 {
        let macro_val = u8_lerp_to_f32(raw, min, max);
        let back = f32_to_u8_lerp(macro_val, min, max);
        assert_eq!(raw, back, "raw {} did not roundtrip (got {})", raw, back);
    }
}

#[test]
fn test_u8_lerp_quantization_error() {
    // For small ranges, there's inherent quantization error
    // This test documents that behavior
    let min = 0.01f32;
    let max = 0.5f32;

    for raw in 0u8..=255u8 {
        let macro_val = u8_lerp_to_f32(raw, min, max);
        let back = f32_to_u8_lerp(macro_val, min, max);
        // Allow off-by-one due to floating point rounding
        let diff = (raw as i16 - back as i16).abs();
        assert!(diff <= 1, "raw {} had too much error (got {}, diff={})", raw, back, diff);
    }
}

#[test]
fn test_u8_lerp_clamping() {
    let min = 0.0f32;
    let max = 10.0f32;

    // Values below min should clamp to 0
    assert_eq!(f32_to_u8_lerp(-5.0, min, max), 0);
    // Values above max should clamp to 255
    assert_eq!(f32_to_u8_lerp(15.0, min, max), 255);
}

#[test]
fn test_u8_lerp_degenerate_range() {
    // If max <= min, return 0
    assert_eq!(f32_to_u8_lerp(5.0, 10.0, 10.0), 0);
    assert_eq!(f32_to_u8_lerp(5.0, 10.0, 5.0), 0);
}

// ============================================================================
// u4_01 Roundtrip Tests
// ============================================================================

#[test]
fn test_u4_01_roundtrip_zero() {
    let raw = 0u8;
    let macro_val = u4_01_to_f32(raw);
    let back = f32_to_u4_01(macro_val);
    assert_eq!(raw, back);
    assert!((macro_val - 0.0).abs() < 0.001);
}

#[test]
fn test_u4_01_roundtrip_half() {
    // 7 should map to ~0.467, and 0.5 should map to 7
    let raw = 7u8;
    let macro_val = u4_01_to_f32(raw);
    let back = f32_to_u4_01(macro_val);
    assert_eq!(raw, back);
}

#[test]
fn test_u4_01_roundtrip_max() {
    let raw = 15u8;
    let macro_val = u4_01_to_f32(raw);
    let back = f32_to_u4_01(macro_val);
    assert_eq!(raw, back);
    assert!((macro_val - 1.0).abs() < 0.001);
}

#[test]
fn test_u4_01_all_values_roundtrip() {
    // Every u4 value (0-15) should roundtrip exactly
    for raw in 0u8..=15u8 {
        let macro_val = u4_01_to_f32(raw);
        let back = f32_to_u4_01(macro_val);
        assert_eq!(raw, back, "raw {} did not roundtrip (got {})", raw, back);
    }
}

#[test]
fn test_u4_01_masks_high_bits() {
    // Values > 15 should be masked to low 4 bits
    assert_eq!(u4_01_to_f32(0xFF), 1.0); // 0xFF & 0xF = 15
    assert_eq!(u4_01_to_f32(0x1F), 1.0); // 0x1F & 0xF = 15
}

#[test]
fn test_u4_01_clamping() {
    // Values below 0 should clamp
    assert_eq!(f32_to_u4_01(-0.5), 0);
    // Values above 1 should clamp
    assert_eq!(f32_to_u4_01(1.5), 15);
}

// ============================================================================
// dir16_oct (Octahedral Direction) Roundtrip Tests
// ============================================================================

#[test]
fn test_dir16_oct_roundtrip_up() {
    // Straight up: (any azimuth, elevation=90)
    let (az, el) = (0.0, 90.0);
    let packed = angles_to_octahedral(az, el);
    let (_az_back, el_back) = octahedral_to_angles(packed);

    // Elevation should be close to 90 (azimuth undefined at poles)
    assert!((el_back - 90.0).abs() < 5.0, "elevation: {} -> {} -> {}", el, packed, el_back);
}

#[test]
fn test_dir16_oct_roundtrip_down() {
    // Straight down: (any azimuth, elevation=-90)
    let (az, el) = (0.0, -90.0);
    let packed = angles_to_octahedral(az, el);
    let (_az_back, el_back) = octahedral_to_angles(packed);

    // Elevation should be close to -90 (azimuth undefined at poles)
    assert!((el_back - (-90.0)).abs() < 5.0, "elevation: {} -> {} -> {}", el, packed, el_back);
}

#[test]
fn test_dir16_oct_roundtrip_forward() {
    // Forward: azimuth=0, elevation=0
    let (az, el) = (0.0, 0.0);
    let packed = angles_to_octahedral(az, el);
    let (az_back, el_back) = octahedral_to_angles(packed);

    // Should be close to original
    assert!((el_back - 0.0).abs() < 5.0, "elevation: {} -> {} -> {}", el, packed, el_back);
    // Azimuth might wrap around, but at el=0 it should be close
    assert!(az_back.abs() < 10.0 || (az_back - 360.0).abs() < 10.0, "azimuth: {} -> {} -> {}", az, packed, az_back);
}

#[test]
fn test_dir16_oct_roundtrip_cardinal_directions() {
    // Test cardinal directions on the horizon
    let cardinals = [
        (0.0, 0.0),    // +X (East)
        (90.0, 0.0),   // +Y (North)
        (180.0, 0.0),  // -X (West)
        (270.0, 0.0),  // -Y (South)
    ];

    for (az, el) in cardinals {
        let packed = angles_to_octahedral(az, el);
        let (az_back, el_back) = octahedral_to_angles(packed);

        // Elevation should be close to 0
        assert!((el_back - el).abs() < 5.0, "elevation for ({}, {}): got {}", az, el, el_back);

        // Azimuth should be close (with wrap-around handling)
        let az_diff = (az_back - az).abs();
        let az_diff_wrapped = (360.0 - az_diff).abs();
        assert!(az_diff < 15.0 || az_diff_wrapped < 15.0,
            "azimuth for ({}, {}): got {} (diff={})", az, el, az_back, az_diff.min(az_diff_wrapped));
    }
}

#[test]
fn test_dir16_oct_roundtrip_45_degree_elevation() {
    // Test directions at 45 degrees elevation
    let directions = [
        (0.0, 45.0),
        (90.0, 45.0),
        (180.0, 45.0),
        (270.0, 45.0),
    ];

    for (az, el) in directions {
        let packed = angles_to_octahedral(az, el);
        let (_az_back, el_back) = octahedral_to_angles(packed);

        // Elevation should be reasonably close
        assert!((el_back - el).abs() < 10.0, "elevation for ({}, {}): got {}", az, el, el_back);
    }
}

#[test]
fn test_dir16_oct_roundtrip_negative_elevation() {
    // Test directions below horizon
    let directions = [
        (45.0, -30.0),
        (135.0, -45.0),
        (225.0, -60.0),
        (315.0, -15.0),
    ];

    for (az, el) in directions {
        let packed = angles_to_octahedral(az, el);
        let (_az_back, el_back) = octahedral_to_angles(packed);

        // Elevation should be reasonably close
        assert!((el_back - el).abs() < 15.0, "elevation for ({}, {}): got {}", az, el, el_back);
    }
}

#[test]
fn test_dir16_oct_packed_value_range() {
    // Verify packed values across the full sphere
    // (u16 always fits, but this tests the encoding produces sensible values)
    for az in (0..360).step_by(30) {
        for el in (-90..=90).step_by(30) {
            let packed = angles_to_octahedral(az as f32, el as f32);
            // Verify both bytes are set appropriately
            let u = packed & 0xFF;
            let v = (packed >> 8) & 0xFF;
            // Both components should be in valid range [0, 255]
            assert!(u <= 255 && v <= 255, "packed components out of range for ({}, {}): u={}, v={}", az, el, u, v);
        }
    }
}

// ============================================================================
// Macro -> Raw -> Macro Stability Tests
// ============================================================================

#[test]
fn test_macro_raw_macro_stability_u8_01() {
    // Test that macro -> raw -> macro produces stable values
    // (second roundtrip should equal first roundtrip)
    for initial in [0.0f32, 0.25, 0.5, 0.75, 1.0] {
        let raw1 = f32_to_u8_01(initial);
        let macro1 = u8_01_to_f32(raw1);
        let raw2 = f32_to_u8_01(macro1);
        let macro2 = u8_01_to_f32(raw2);

        assert_eq!(raw1, raw2, "raw values should stabilize: {} -> {} -> {} -> {}", initial, raw1, macro1, raw2);
        assert!((macro1 - macro2).abs() < 0.001, "macro values should stabilize");
    }
}

#[test]
fn test_macro_raw_macro_stability_u8_lerp() {
    let min = 0.5f32;
    let max = 16.0f32;

    for initial in [0.5f32, 4.0, 8.0, 12.0, 16.0] {
        let raw1 = f32_to_u8_lerp(initial, min, max);
        let macro1 = u8_lerp_to_f32(raw1, min, max);
        let raw2 = f32_to_u8_lerp(macro1, min, max);
        let macro2 = u8_lerp_to_f32(raw2, min, max);

        assert_eq!(raw1, raw2, "raw values should stabilize");
        assert!((macro1 - macro2).abs() < 0.001, "macro values should stabilize");
    }
}

#[test]
fn test_macro_raw_macro_stability_u4_01() {
    for initial in [0.0f32, 0.25, 0.5, 0.75, 1.0] {
        let raw1 = f32_to_u4_01(initial);
        let macro1 = u4_01_to_f32(raw1);
        let raw2 = f32_to_u4_01(macro1);
        let macro2 = u4_01_to_f32(raw2);

        assert_eq!(raw1, raw2, "raw values should stabilize");
        assert!((macro1 - macro2).abs() < 0.001, "macro values should stabilize");
    }
}

#[test]
fn test_macro_raw_macro_stability_dir16_oct() {
    // Direction encoding has inherent quantization - test that the output
    // direction remains close after roundtripping (not exact byte equality)
    let directions = [
        (0.0, 0.0),
        (45.0, 30.0),
        (180.0, 60.0),
        (270.0, -45.0),
    ];

    for (az, el) in directions {
        // First conversion
        let packed1 = angles_to_octahedral(az, el);
        let (az1, el1) = octahedral_to_angles(packed1);

        // Second conversion
        let packed2 = angles_to_octahedral(az1, el1);
        let (az2, el2) = octahedral_to_angles(packed2);

        // Third conversion
        let packed3 = angles_to_octahedral(az2, el2);
        let (az3, el3) = octahedral_to_angles(packed3);

        // The angle values should converge to within a small tolerance
        // (the packed values may oscillate by 1 LSB due to quantization)
        assert!((az2 - az3).abs() < 1.0, "azimuth should be close after 2 roundtrips for ({}, {}): {} vs {}", az, el, az2, az3);
        assert!((el2 - el3).abs() < 1.0, "elevation should be close after 2 roundtrips for ({}, {}): {} vs {}", az, el, el2, el3);

        // Packed values should be very close (at most 1 LSB difference per component)
        let u_diff = ((packed2 & 0xFF) as i16 - (packed3 & 0xFF) as i16).abs();
        let v_diff = (((packed2 >> 8) & 0xFF) as i16 - ((packed3 >> 8) & 0xFF) as i16).abs();
        assert!(u_diff <= 1 && v_diff <= 1,
            "packed values should be within 1 LSB after 2 roundtrips for ({}, {}): {} vs {} (u_diff={}, v_diff={})",
            az, el, packed2, packed3, u_diff, v_diff);
    }
}

#[test]
fn test_dir16_oct_quantization_bounded() {
    // Test that single-roundtrip quantization error is bounded
    let directions = [
        (0.0, 0.0),
        (45.0, 30.0),
        (90.0, 45.0),
        (180.0, 60.0),
        (270.0, -45.0),
    ];

    for (az, el) in directions {
        let packed1 = angles_to_octahedral(az, el);
        let (az1, el1) = octahedral_to_angles(packed1);
        let packed2 = angles_to_octahedral(az1, el1);

        // Packed values should be close (within a few LSBs)
        let u_diff = ((packed1 & 0xFF) as i16 - (packed2 & 0xFF) as i16).abs();
        let v_diff = (((packed1 >> 8) & 0xFF) as i16 - ((packed2 >> 8) & 0xFF) as i16).abs();

        // Each component should differ by at most 2 (quantization error)
        assert!(u_diff <= 2, "u component changed too much for ({}, {}): diff={}", az, el, u_diff);
        assert!(v_diff <= 2, "v component changed too much for ({}, {}): diff={}", az, el, v_diff);
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_u8_01_edge_values() {
    // Test values very close to boundaries
    assert_eq!(f32_to_u8_01(0.001), 0);  // Small positive
    assert_eq!(f32_to_u8_01(0.999), 254); // Just under 1.0
    assert_eq!(f32_to_u8_01(0.004), 1);   // First value that maps to 1
}

#[test]
fn test_u8_lerp_edge_values() {
    let min = 0.0f32;
    let max = 1.0f32;

    // Mid-range should be approximately 127 or 128
    let mid = f32_to_u8_lerp(0.5, min, max);
    assert!(mid == 127 || mid == 128, "mid value should be 127 or 128, got {}", mid);
}

#[test]
fn test_u4_01_edge_values() {
    // Test boundary values
    assert_eq!(f32_to_u4_01(1.0 / 15.0), 1);  // Exactly 1/15
    assert_eq!(f32_to_u4_01(14.0 / 15.0), 14); // Exactly 14/15
}

#[test]
fn test_dir16_oct_near_singularity() {
    // Test near the poles where azimuth becomes undefined
    let (_, el_up) = octahedral_to_angles(angles_to_octahedral(0.0, 89.0));
    assert!(el_up > 80.0, "near-up should preserve high elevation");

    let (_, el_down) = octahedral_to_angles(angles_to_octahedral(0.0, -89.0));
    assert!(el_down < -80.0, "near-down should preserve low elevation");
}

// ============================================================================
// Common Edit Pattern Tests
// ============================================================================

#[test]
fn test_common_edit_small_increment() {
    // Simulate small UI increments
    let base = 128u8;
    let macro_val = u8_01_to_f32(base);

    // Small increment in macro space
    let incremented = macro_val + 0.01;
    let new_raw = f32_to_u8_01(incremented);

    // Should produce a nearby raw value
    assert!((new_raw as i16 - base as i16).abs() <= 3,
        "small increment should produce nearby value: {} -> {} -> {}", base, macro_val, new_raw);
}

#[test]
fn test_common_edit_lerp_range() {
    // Test editing within a real-world range (pattern scale: 0.5-16.0)
    let min = 0.5f32;
    let max = 16.0f32;

    // User sets value to 8.0 (middle of range)
    let target = 8.0f32;
    let raw = f32_to_u8_lerp(target, min, max);
    let actual = u8_lerp_to_f32(raw, min, max);

    // Should be close to target (within quantization error)
    let error = (actual - target).abs() / (max - min);
    assert!(error < 0.005, "quantization error should be < 0.5%: got {}", error);
}
