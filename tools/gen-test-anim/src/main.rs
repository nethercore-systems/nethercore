//! Generate test animation files for animation-demo example
//!
//! Creates a simple 3-bone wave animation for testing the keyframe system.

use std::f32::consts::TAU;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use zx_common::formats::animation::{encode_bone_transform, NetherZXAnimationHeader};

fn main() {
    let output_path = PathBuf::from("examples/4-animation/animation-demo/assets/wave.nczxanim");

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create output directory");
    }

    // Generate a simple 3-bone, 30-frame wave animation
    let bone_count: u8 = 3;
    let frame_count: u16 = 30;

    let header = NetherZXAnimationHeader::new(bone_count, frame_count);

    let mut file = File::create(&output_path).expect("Failed to create output file");

    // Write header (4 bytes)
    file.write_all(&header.to_bytes())
        .expect("Failed to write header");

    // Write frame data
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

            // Write 16-byte keyframe
            file.write_all(&keyframe.to_bytes())
                .expect("Failed to write keyframe");
        }
    }

    let file_size = 4 + (frame_count as usize * bone_count as usize * 16);
    println!(
        "Generated {} ({} bones, {} frames, {} bytes)",
        output_path.display(),
        bone_count,
        frame_count,
        file_size
    );
}
