//! NetherZXMesh binary format (.nczxanim)
//!
//! ZX console animation clip format containing sampled bone transforms.
//! POD format with minimal header - no magic bytes.
//!
//! # Layout
//! ```text
//! Header (4 bytes):
//! 0x00: bone_count u8        - Number of bones per frame (max 255)
//! 0x01: flags u8             - Reserved, must be 0
//! 0x02: frame_count u16 LE   - Total number of frames (max 65535)
//!
//! Frame Data (frame_count × bone_count × 16 bytes):
//! Each bone transform is stored in 16 bytes:
//! - rotation: u32 (smallest-three packed quaternion)
//! - position: [u16; 3] (f16 × 3)
//! - scale: [u16; 3] (f16 × 3)
//! ```
//!
//! Frame data is stored sequentially: [frame0_bone0, frame0_bone1, ..., frame1_bone0, ...]

mod encoding;
mod header;
mod types;

#[cfg(test)]
mod tests;

// Re-export public API
pub use encoding::{
    decode_bone_transform, decode_quat_smallest_three, encode_bone_transform,
    encode_quat_smallest_three, f16_to_f32, f32_to_f16,
};
pub use header::NetherZXAnimationHeader;
pub use types::{
    BoneTransform, PlatformBoneKeyframe, BONE_TRANSFORM_SIZE, PLATFORM_BONE_KEYFRAME_SIZE,
};
