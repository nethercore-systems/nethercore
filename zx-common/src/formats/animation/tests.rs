//! Tests for animation format

use super::*;

// ========================================================================
// Header Tests
// ========================================================================

#[test]
fn test_animation_header_roundtrip() {
    let header = NetherZXAnimationHeader::new(25, 90);
    assert_eq!(header.bone_count, 25);
    assert_eq!(header.frame_count, 90);
    assert_eq!(header.flags, 0);

    let bytes = header.to_bytes();
    assert_eq!(bytes.len(), NetherZXAnimationHeader::SIZE);

    let parsed = NetherZXAnimationHeader::from_bytes(&bytes).unwrap();
    assert_eq!(parsed.bone_count, header.bone_count);
    assert_eq!(parsed.frame_count, header.frame_count);
    assert_eq!(parsed.flags, header.flags);
}

#[test]
fn test_animation_header_size() {
    assert_eq!(NetherZXAnimationHeader::SIZE, 4);
    assert_eq!(PLATFORM_BONE_KEYFRAME_SIZE, 16);
    assert_eq!(BONE_TRANSFORM_SIZE, 40);
}

#[test]
fn test_animation_file_size() {
    // Example from spec: 40 bones, 60 frames
    let header = NetherZXAnimationHeader::new(40, 60);
    // file_size = 4 + (frame_count x bone_count x 16) = 4 + 38400 = 38404
    assert_eq!(header.file_size(), 38404);
}

#[test]
fn test_animation_header_from_short_bytes() {
    let short_bytes = [0u8; 2];
    assert!(NetherZXAnimationHeader::from_bytes(&short_bytes).is_none());
}

#[test]
fn test_header_validation() {
    let valid = NetherZXAnimationHeader::new(10, 100);
    assert!(valid.validate());

    let invalid_bones = NetherZXAnimationHeader::new(0, 100);
    assert!(!invalid_bones.validate());

    let invalid_frames = NetherZXAnimationHeader::new(10, 0);
    assert!(!invalid_frames.validate());
}

// ========================================================================
// Smallest-Three Quaternion Encoding Tests
// ========================================================================

#[test]
fn test_quat_identity_roundtrip() {
    // Identity quaternion: [0, 0, 0, 1]
    let q = [0.0, 0.0, 0.0, 1.0];
    let encoded = encode_quat_smallest_three(q);
    let decoded = decode_quat_smallest_three(encoded);

    // Verify idx is 3 (w is largest)
    assert_eq!(
        encoded & 0x3,
        3,
        "Identity quaternion should drop w (idx=3)"
    );

    // Verify roundtrip (dot product should be close to 1)
    let dot = q[0] * decoded[0] + q[1] * decoded[1] + q[2] * decoded[2] + q[3] * decoded[3];
    assert!(
        dot.abs() > 0.999,
        "Identity roundtrip failed: dot = {}",
        dot
    );
}

#[test]
fn test_quat_90_x_roundtrip() {
    // 90° X rotation (axis-angle): [1, 0, 0, 0]
    let q = [1.0, 0.0, 0.0, 0.0];
    let encoded = encode_quat_smallest_three(q);
    let decoded = decode_quat_smallest_three(encoded);

    // Verify idx is 0 (x is largest)
    assert_eq!(encoded & 0x3, 0, "90° X rotation should drop x (idx=0)");

    // Verify roundtrip
    let dot = q[0] * decoded[0] + q[1] * decoded[1] + q[2] * decoded[2] + q[3] * decoded[3];
    assert!(dot.abs() > 0.999, "90° X roundtrip failed: dot = {}", dot);
}

#[test]
fn test_quat_90_y_roundtrip() {
    // 90° Y rotation: [0, 1, 0, 0]
    let q = [0.0, 1.0, 0.0, 0.0];
    let encoded = encode_quat_smallest_three(q);
    let decoded = decode_quat_smallest_three(encoded);

    // Verify idx is 1 (y is largest)
    assert_eq!(encoded & 0x3, 1, "90° Y rotation should drop y (idx=1)");

    // Verify roundtrip
    let dot = q[0] * decoded[0] + q[1] * decoded[1] + q[2] * decoded[2] + q[3] * decoded[3];
    assert!(dot.abs() > 0.999, "90° Y roundtrip failed: dot = {}", dot);
}

#[test]
fn test_quat_90_z_roundtrip() {
    // 90° Z rotation: [0, 0, 1, 0]
    let q = [0.0, 0.0, 1.0, 0.0];
    let encoded = encode_quat_smallest_three(q);
    let decoded = decode_quat_smallest_three(encoded);

    // Verify idx is 2 (z is largest)
    assert_eq!(encoded & 0x3, 2, "90° Z rotation should drop z (idx=2)");

    // Verify roundtrip
    let dot = q[0] * decoded[0] + q[1] * decoded[1] + q[2] * decoded[2] + q[3] * decoded[3];
    assert!(dot.abs() > 0.999, "90° Z roundtrip failed: dot = {}", dot);
}

#[test]
fn test_quat_120_rotation_roundtrip() {
    // 120° rotation around [1,1,1]: [0.5, 0.5, 0.5, 0.5]
    let q = [0.5, 0.5, 0.5, 0.5];
    let encoded = encode_quat_smallest_three(q);
    let decoded = decode_quat_smallest_three(encoded);

    // Verify roundtrip
    let dot = q[0] * decoded[0] + q[1] * decoded[1] + q[2] * decoded[2] + q[3] * decoded[3];
    assert!(
        dot.abs() > 0.999,
        "120° [1,1,1] roundtrip failed: dot = {}",
        dot
    );
}

#[test]
fn test_quat_half_angle_90_x_roundtrip() {
    // 90° X (half-angle form): [0.707107, 0, 0, 0.707107]
    let sqrt2_inv = std::f32::consts::FRAC_1_SQRT_2;
    let q = [sqrt2_inv, 0.0, 0.0, sqrt2_inv];
    let encoded = encode_quat_smallest_three(q);
    let decoded = decode_quat_smallest_three(encoded);

    // Verify roundtrip
    let dot = q[0] * decoded[0] + q[1] * decoded[1] + q[2] * decoded[2] + q[3] * decoded[3];
    assert!(
        dot.abs() > 0.999,
        "Half-angle 90° X roundtrip failed: dot = {}",
        dot
    );
}

#[test]
fn test_quat_sign_flip_roundtrip() {
    // Sign flip: [-0.5, -0.5, -0.5, 0.5]
    // Should produce same rotation as [0.5, 0.5, 0.5, -0.5]
    let q = [-0.5, -0.5, -0.5, 0.5];
    let encoded = encode_quat_smallest_three(q);
    let decoded = decode_quat_smallest_three(encoded);

    // For q and -q representing same rotation, dot can be positive or negative
    let dot = q[0] * decoded[0] + q[1] * decoded[1] + q[2] * decoded[2] + q[3] * decoded[3];
    assert!(
        dot.abs() > 0.999,
        "Sign flip roundtrip failed: dot = {}",
        dot
    );
}

#[test]
fn test_quat_roundtrip_precision() {
    // Test roundtrip precision for arbitrary quaternion
    let q = [0.270598, 0.0, 0.0, 0.962728]; // ~31.4° X rotation
    let encoded = encode_quat_smallest_three(q);
    let decoded = decode_quat_smallest_three(encoded);

    // Compute dot product (should be > 0.9999 for < 0.1° error)
    let dot = q[0] * decoded[0] + q[1] * decoded[1] + q[2] * decoded[2] + q[3] * decoded[3];
    assert!(
        dot.abs() > 0.9999,
        "Quaternion roundtrip precision failed: dot = {}",
        dot
    );
}

// ========================================================================
// Half-Float (f16) Tests (from spec)
// ========================================================================

#[test]
fn test_f16_zero() {
    assert_eq!(f32_to_f16(0.0), 0x0000);
    assert_eq!(f16_to_f32(0x0000), 0.0);
}

#[test]
fn test_f16_negative_zero() {
    assert_eq!(f32_to_f16(-0.0), 0x8000);
    // Note: -0.0 == 0.0 in Rust
}

#[test]
fn test_f16_one() {
    assert_eq!(f32_to_f16(1.0), 0x3C00);
    assert_eq!(f16_to_f32(0x3C00), 1.0);
}

#[test]
fn test_f16_negative_one() {
    assert_eq!(f32_to_f16(-1.0), 0xBC00);
    assert_eq!(f16_to_f32(0xBC00), -1.0);
}

#[test]
fn test_f16_half() {
    assert_eq!(f32_to_f16(0.5), 0x3800);
    assert_eq!(f16_to_f32(0x3800), 0.5);
}

#[test]
fn test_f16_two() {
    assert_eq!(f32_to_f16(2.0), 0x4000);
    assert_eq!(f16_to_f32(0x4000), 2.0);
}

#[test]
fn test_f16_max_normal() {
    assert_eq!(f32_to_f16(65504.0), 0x7BFF);
    assert_eq!(f16_to_f32(0x7BFF), 65504.0);
}

#[test]
fn test_f16_min_normal() {
    assert_eq!(f32_to_f16(-65504.0), 0xFBFF);
    assert_eq!(f16_to_f32(0xFBFF), -65504.0);
}

// ========================================================================
// Full Roundtrip Tests (from spec)
// ========================================================================

#[test]
fn test_identity_transform_roundtrip() {
    let input = BoneTransform {
        rotation: [0.0, 0.0, 0.0, 1.0],
        position: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    };

    let encoded = encode_bone_transform(input.rotation, input.position, input.scale);

    // Verify idx=3 (w is dropped as largest component)
    assert_eq!(
        encoded.rotation & 0x3,
        3,
        "Identity rotation should drop w (idx=3)"
    );
    assert_eq!(
        encoded.position,
        [0x0000, 0x0000, 0x0000],
        "Zero position encoding"
    );
    assert_eq!(
        encoded.scale,
        [0x3C00, 0x3C00, 0x3C00],
        "Unit scale encoding"
    );

    let decoded = decode_bone_transform(&encoded);

    assert!((decoded.rotation[3] - 1.0).abs() < 0.002, "w ≈ 1");
    assert!(
        decoded.position.iter().all(|&v| v.abs() < 0.001),
        "position ≈ 0"
    );
    assert!(
        decoded.scale.iter().all(|&v| (v - 1.0).abs() < 0.001),
        "scale ≈ 1"
    );
}

#[test]
fn test_typical_animation_pose_roundtrip() {
    let input = BoneTransform {
        rotation: [0.270598, 0.0, 0.0, 0.962728], // 31.4° X rotation
        position: [1.5, 2.25, -0.75],
        scale: [1.0, 1.0, 1.0],
    };

    let encoded = encode_bone_transform(input.rotation, input.position, input.scale);
    let decoded = decode_bone_transform(&encoded);

    // Verify rotation (angular error < 0.1°)
    let dot = input.rotation[0] * decoded.rotation[0]
        + input.rotation[1] * decoded.rotation[1]
        + input.rotation[2] * decoded.rotation[2]
        + input.rotation[3] * decoded.rotation[3];
    assert!(
        dot.abs() > 0.9999,
        "Nearly identical rotation: dot = {}",
        dot
    );

    // Verify position (f16 precision)
    assert!((decoded.position[0] - 1.5).abs() < 0.01);
    assert!((decoded.position[1] - 2.25).abs() < 0.01);
    assert!((decoded.position[2] - (-0.75)).abs() < 0.01);

    // Verify scale
    assert!(decoded.scale.iter().all(|&v| (v - 1.0).abs() < 0.001));
}

#[test]
fn test_extreme_values_roundtrip() {
    let input = BoneTransform {
        rotation: [
            std::f32::consts::FRAC_1_SQRT_2,
            0.0,
            std::f32::consts::FRAC_1_SQRT_2,
            0.0,
        ], // 180° around [1,0,1]
        position: [1000.0, -500.0, 0.001],
        scale: [0.5, 1.0, 2.5], // Non-uniform scale
    };

    let encoded = encode_bone_transform(input.rotation, input.position, input.scale);
    let decoded = decode_bone_transform(&encoded);

    // Position precision degrades at large values (f16 limitation)
    assert!((decoded.position[0] - 1000.0).abs() < 1.0);
    assert!((decoded.position[1] - (-500.0)).abs() < 0.5);
    assert!((decoded.position[2] - 0.001).abs() < 0.001);

    // Verify non-uniform scale (XYZ)
    assert!((decoded.scale[0] - 0.5).abs() < 0.01);
    assert!((decoded.scale[1] - 1.0).abs() < 0.01);
    assert!((decoded.scale[2] - 2.5).abs() < 0.01);
}

#[test]
fn test_byte_level_verification() {
    // Test encode→bytes→decode roundtrip at byte level
    let input = BoneTransform {
        rotation: [0.0, 0.0, 0.0, 1.0], // Identity
        position: [1.0, 2.0, -2.0],
        scale: [1.0, 1.0, 1.0],
    };

    let encoded = encode_bone_transform(input.rotation, input.position, input.scale);
    let bytes = encoded.to_bytes();
    let parsed = PlatformBoneKeyframe::from_bytes(&bytes);
    let decoded = decode_bone_transform(&parsed);

    // Verify position and scale are exactly preserved (f16 can represent these exactly)
    assert!((decoded.position[0] - 1.0).abs() < 0.001);
    assert!((decoded.position[1] - 2.0).abs() < 0.001);
    assert!((decoded.position[2] - (-2.0)).abs() < 0.001);
    assert!(decoded.scale.iter().all(|&v| (v - 1.0).abs() < 0.001));

    // Verify rotation roundtrip
    let dot = input.rotation[0] * decoded.rotation[0]
        + input.rotation[1] * decoded.rotation[1]
        + input.rotation[2] * decoded.rotation[2]
        + input.rotation[3] * decoded.rotation[3];
    assert!(
        dot.abs() > 0.999,
        "Rotation roundtrip failed: dot = {}",
        dot
    );
}

// ========================================================================
// Edge Case Tests (from spec)
// ========================================================================

#[test]
fn test_single_frame_animation() {
    // Minimum valid animation: 1 bone, 1 frame (20 bytes total)
    // Build it programmatically to avoid hardcoding implementation-specific values
    let header = NetherZXAnimationHeader::new(1, 1);
    let header_bytes = header.to_bytes();

    // Identity transform: no rotation, zero position, unit scale
    let keyframe = encode_bone_transform(
        [0.0, 0.0, 0.0, 1.0], // Identity quaternion
        [0.0, 0.0, 0.0],      // Zero position
        [1.0, 1.0, 1.0],      // Unit scale
    );
    let keyframe_bytes = keyframe.to_bytes();

    // Combine header + keyframe
    let mut data = Vec::with_capacity(20);
    data.extend_from_slice(&header_bytes);
    data.extend_from_slice(&keyframe_bytes);

    assert_eq!(data.len(), 20); // 4 header + 16 data

    // Verify we can parse it back
    let parsed_header = NetherZXAnimationHeader::from_bytes(&data).unwrap();
    assert_eq!(parsed_header.bone_count, 1);
    assert_eq!(parsed_header.frame_count, 1);
    assert!(parsed_header.validate());

    // Verify the keyframe roundtrips correctly
    let parsed_kf = PlatformBoneKeyframe::from_bytes(&data[4..]);
    let decoded = decode_bone_transform(&parsed_kf);
    assert!((decoded.rotation[3] - 1.0).abs() < 0.002, "w ≈ 1");
    assert!(
        decoded.position.iter().all(|&v| v.abs() < 0.001),
        "position ≈ 0"
    );
    assert!(
        decoded.scale.iter().all(|&v| (v - 1.0).abs() < 0.001),
        "scale ≈ 1"
    );
}

#[test]
fn test_max_bone_count() {
    // Maximum: 255 bones
    let header = NetherZXAnimationHeader::new(255, 1);
    assert!(header.validate());
    assert_eq!(header.file_size(), 4 + 255 * 16);
}

#[test]
fn test_max_frame_count() {
    // Maximum: 65535 frames (at 60fps = ~18 minutes)
    let header = NetherZXAnimationHeader::new(1, 65535);
    assert!(header.validate());
    let expected_size = 4 + (65535 * 16);
    assert_eq!(header.file_size(), expected_size);
    assert_eq!(expected_size, 1048564); // ~1MB for single bone
}

#[test]
fn test_platform_keyframe_roundtrip() {
    // Create a keyframe via encode to ensure valid values
    let kf = encode_bone_transform(
        [0.0, 0.0, 0.0, 1.0], // Identity rotation
        [1.0, 2.0, -2.0],     // Position
        [1.0, 1.0, 1.0],      // Unit scale
    );

    let bytes = kf.to_bytes();
    let parsed = PlatformBoneKeyframe::from_bytes(&bytes);

    assert_eq!(parsed.rotation, kf.rotation);
    assert_eq!(parsed.position, kf.position);
    assert_eq!(parsed.scale, kf.scale);
}
