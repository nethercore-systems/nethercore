//! Generate test animation files for animation-demo example
//!
//! Creates a simple 3-bone wave animation for testing the keyframe system.

use std::f32::consts::TAU;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use zx_common::formats::animation::{encode_bone_transform, NetherZXAnimationHeader};

fn main() {
    // Output to shared examples/assets folder with anim-demo- prefix
    let output_path = PathBuf::from("examples/assets/anim-demo-wave.nczxanim");

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create output directory");
    }

    // Generate a simple 3-bone, 30-frame wave animation
    let bone_count: u8 = 3;
    let frame_count: u16 = 30;

    write_wave_animation(&output_path, bone_count, frame_count);

    let file_size = 4 + (frame_count as usize * bone_count as usize * 16);
    println!(
        "Generated {} ({} bones, {} frames, {} bytes)",
        output_path.display(),
        bone_count,
        frame_count,
        file_size
    );
}

fn write_wave_animation(path: &Path, bone_count: u8, frame_count: u16) {
    let data = generate_wave_animation_bytes(bone_count, frame_count);
    let mut file = File::create(path).expect("Failed to create output file");
    file.write_all(&data).expect("Failed to write animation");
}

fn generate_wave_animation_bytes(bone_count: u8, frame_count: u16) -> Vec<u8> {
    let header = NetherZXAnimationHeader::new(bone_count, frame_count);
    let mut bytes = Vec::with_capacity(header.file_size());
    bytes.extend_from_slice(&header.to_bytes());

    for frame in 0..frame_count {
        let t = (frame as f32 / frame_count as f32) * TAU;

        for bone in 0..bone_count {
            // Calculate rotation for this bone at this frame
            // Each bone rotates with a phase offset
            let phase = bone as f32 * 0.5;
            let angle = (t + phase).sin() * 0.3; // ±0.3 radians

            // Convert angle to quaternion (rotation around Z axis)
            let half_angle = angle * 0.5;
            let qx = 0.0f32;
            let qy = 0.0f32;
            let qz = half_angle.sin();
            let qw = half_angle.cos();

            // Position: bone offset along Y axis
            let py = bone as f32 * 1.5 - 1.5; // -1.5, 0.0, 1.5

            // Encode to platform format
            let keyframe = encode_bone_transform([qx, qy, qz, qw], [0.0, py, 0.0], [1.0, 1.0, 1.0]);
            bytes.extend_from_slice(&keyframe.to_bytes());
        }
    }

    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use zx_common::formats::animation::NetherZXAnimationHeader;

    #[test]
    fn wave_animation_bytes_have_valid_header_and_expected_size() {
        let bytes = generate_wave_animation_bytes(3, 30);

        let header = NetherZXAnimationHeader::from_bytes(&bytes[0..4]).unwrap();
        assert!(header.validate());
        assert_eq!(header.bone_count, 3);
        assert_eq!(header.frame_count, 30);

        assert_eq!(bytes.len(), header.file_size());
        assert!(bytes[NetherZXAnimationHeader::SIZE..]
            .iter()
            .any(|b| *b != 0));
    }

    #[test]
    fn write_wave_animation_writes_output_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("anim.nczxanim");
        write_wave_animation(&path, 3, 30);
        assert!(path.is_file());
        assert!(std::fs::metadata(&path).unwrap().len() > 4);
    }
}
