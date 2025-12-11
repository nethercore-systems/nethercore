//! Animation converter (glTF -> .ewzanim)
//!
//! Extracts and samples animation clips from glTF files.

use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use crate::formats::write_ember_animation;

/// Default sample rate for animations (frames per second)
const DEFAULT_FRAME_RATE: f32 = 30.0;

/// Convert glTF animation to EmberAnimation format
pub fn convert_gltf_animation(
    input: &Path,
    output: &Path,
    animation_index: Option<usize>,
    skin_index: Option<usize>,
    frame_rate: Option<f32>,
) -> Result<()> {
    let (document, buffers, _images) =
        gltf::import(input).with_context(|| format!("Failed to load glTF: {:?}", input))?;

    // Get the skin (needed for bone order)
    let skin = if let Some(idx) = skin_index {
        document
            .skins()
            .nth(idx)
            .with_context(|| format!("Skin index {} not found in glTF", idx))?
    } else {
        document
            .skins()
            .next()
            .context("No skins found in glTF file")?
    };

    // Get the animation
    let animation = if let Some(idx) = animation_index {
        document
            .animations()
            .nth(idx)
            .with_context(|| format!("Animation index {} not found in glTF", idx))?
    } else {
        document
            .animations()
            .next()
            .context("No animations found in glTF file")?
    };

    let frame_rate = frame_rate.unwrap_or(DEFAULT_FRAME_RATE);

    // Sample the animation
    let (frames, bone_count) = sample_animation(&animation, &skin, &buffers, frame_rate)?;

    if frames.is_empty() {
        bail!("Animation produced no frames");
    }

    // Write output
    let file =
        File::create(output).with_context(|| format!("Failed to create output: {:?}", output))?;
    let mut writer = BufWriter::new(file);

    write_ember_animation(&mut writer, bone_count, frame_rate, &frames)?;

    tracing::info!(
        "Exported animation '{}': {} bones, {} frames at {} fps ({:.2}s)",
        animation.name().unwrap_or("unnamed"),
        bone_count,
        frames.len(),
        frame_rate,
        frames.len() as f32 / frame_rate
    );

    Ok(())
}

/// Sample animation channels at regular intervals
fn sample_animation(
    animation: &gltf::Animation,
    skin: &gltf::Skin,
    buffers: &[gltf::buffer::Data],
    frame_rate: f32,
) -> Result<(Vec<Vec<[f32; 12]>>, u32)> {
    // Build joint index map (node index -> bone index)
    let joints: Vec<_> = skin.joints().collect();
    let bone_count = joints.len();
    let joint_map: HashMap<usize, usize> = joints
        .iter()
        .enumerate()
        .map(|(i, j)| (j.index(), i))
        .collect();

    // Find animation duration
    let mut max_time = 0.0f32;
    for channel in animation.channels() {
        let sampler = channel.sampler();
        let input = sampler.input();
        let times = read_accessor_f32(&input, buffers)?;
        if let Some(&t) = times.last() {
            max_time = max_time.max(t);
        }
    }

    if max_time <= 0.0 {
        bail!("Animation has zero duration");
    }

    // Calculate frame count
    let frame_count = (max_time * frame_rate).ceil() as usize;
    if frame_count == 0 {
        bail!("Animation too short for given frame rate");
    }

    // Initialize frames with identity transforms
    let identity: [f32; 12] = [
        1.0, 0.0, 0.0, // col0 (x-axis)
        0.0, 1.0, 0.0, // col1 (y-axis)
        0.0, 0.0, 1.0, // col2 (z-axis)
        0.0, 0.0, 0.0, // col3 (translation)
    ];
    let mut frames: Vec<Vec<[f32; 12]>> = (0..frame_count)
        .map(|_| vec![identity; bone_count])
        .collect();

    // Per-bone transform components (accumulated per frame)
    // We store separate T, R, S and compose them at the end
    let mut translations: Vec<Vec<[f32; 3]>> = (0..frame_count)
        .map(|_| vec![[0.0, 0.0, 0.0]; bone_count])
        .collect();
    let mut rotations: Vec<Vec<[f32; 4]>> = (0..frame_count)
        .map(|_| vec![[0.0, 0.0, 0.0, 1.0]; bone_count]) // Identity quaternion
        .collect();
    let mut scales: Vec<Vec<[f32; 3]>> = (0..frame_count)
        .map(|_| vec![[1.0, 1.0, 1.0]; bone_count])
        .collect();

    // Sample each channel
    for channel in animation.channels() {
        let target = channel.target();
        let node_index = target.node().index();

        // Skip if not a joint in our skin
        let Some(&bone_index) = joint_map.get(&node_index) else {
            continue;
        };

        let sampler = channel.sampler();
        let input_accessor = sampler.input();
        let output_accessor = sampler.output();

        let times = read_accessor_f32(&input_accessor, buffers)?;
        let interpolation = sampler.interpolation();

        match target.property() {
            gltf::animation::Property::Translation => {
                let values = read_accessor_vec3(&output_accessor, buffers)?;
                for frame_idx in 0..frame_count {
                    let t = frame_idx as f32 / frame_rate;
                    let value = interpolate_vec3(&times, &values, t, interpolation);
                    translations[frame_idx][bone_index] = value;
                }
            }
            gltf::animation::Property::Rotation => {
                let values = read_accessor_quat(&output_accessor, buffers)?;
                for frame_idx in 0..frame_count {
                    let t = frame_idx as f32 / frame_rate;
                    let value = interpolate_quat(&times, &values, t, interpolation);
                    rotations[frame_idx][bone_index] = value;
                }
            }
            gltf::animation::Property::Scale => {
                let values = read_accessor_vec3(&output_accessor, buffers)?;
                for frame_idx in 0..frame_count {
                    let t = frame_idx as f32 / frame_rate;
                    let value = interpolate_vec3(&times, &values, t, interpolation);
                    scales[frame_idx][bone_index] = value;
                }
            }
            _ => {} // Ignore weights/morph targets
        }
    }

    // Compose TRS into 3x4 matrices
    for frame_idx in 0..frame_count {
        for bone_idx in 0..bone_count {
            let t = translations[frame_idx][bone_idx];
            let r = rotations[frame_idx][bone_idx];
            let s = scales[frame_idx][bone_idx];
            frames[frame_idx][bone_idx] = compose_trs_matrix(t, r, s);
        }
    }

    Ok((frames, bone_count as u32))
}

/// Compose translation, rotation, scale into a 3x4 matrix (column-major)
fn compose_trs_matrix(t: [f32; 3], q: [f32; 4], s: [f32; 3]) -> [f32; 12] {
    // Convert quaternion to rotation matrix
    let [qx, qy, qz, qw] = q;

    let xx = qx * qx;
    let yy = qy * qy;
    let zz = qz * qz;
    let xy = qx * qy;
    let xz = qx * qz;
    let yz = qy * qz;
    let wx = qw * qx;
    let wy = qw * qy;
    let wz = qw * qz;

    // Rotation matrix elements (column-major)
    let r00 = 1.0 - 2.0 * (yy + zz);
    let r01 = 2.0 * (xy + wz);
    let r02 = 2.0 * (xz - wy);

    let r10 = 2.0 * (xy - wz);
    let r11 = 1.0 - 2.0 * (xx + zz);
    let r12 = 2.0 * (yz + wx);

    let r20 = 2.0 * (xz + wy);
    let r21 = 2.0 * (yz - wx);
    let r22 = 1.0 - 2.0 * (xx + yy);

    // Apply scale and compose
    [
        r00 * s[0],
        r01 * s[0],
        r02 * s[0], // col0 (scaled x-axis)
        r10 * s[1],
        r11 * s[1],
        r12 * s[1], // col1 (scaled y-axis)
        r20 * s[2],
        r21 * s[2],
        r22 * s[2], // col2 (scaled z-axis)
        t[0],
        t[1],
        t[2], // col3 (translation)
    ]
}

// ============================================================================
// glTF accessor readers
// ============================================================================

fn read_accessor_f32(
    accessor: &gltf::Accessor,
    buffers: &[gltf::buffer::Data],
) -> Result<Vec<f32>> {
    let view = accessor.view().context("Accessor has no buffer view")?;
    let buffer = &buffers[view.buffer().index()];
    let offset = view.offset() + accessor.offset();
    let count = accessor.count();
    let stride = view.stride().unwrap_or(4);

    let mut values = Vec::with_capacity(count);
    for i in 0..count {
        let byte_offset = offset + i * stride;
        let bytes = [
            buffer[byte_offset],
            buffer[byte_offset + 1],
            buffer[byte_offset + 2],
            buffer[byte_offset + 3],
        ];
        values.push(f32::from_le_bytes(bytes));
    }
    Ok(values)
}

fn read_accessor_vec3(
    accessor: &gltf::Accessor,
    buffers: &[gltf::buffer::Data],
) -> Result<Vec<[f32; 3]>> {
    let view = accessor.view().context("Accessor has no buffer view")?;
    let buffer = &buffers[view.buffer().index()];
    let offset = view.offset() + accessor.offset();
    let count = accessor.count();
    let stride = view.stride().unwrap_or(12);

    let mut values = Vec::with_capacity(count);
    for i in 0..count {
        let byte_offset = offset + i * stride;
        let mut v = [0.0f32; 3];
        for (j, float) in v.iter_mut().enumerate() {
            let bo = byte_offset + j * 4;
            let bytes = [buffer[bo], buffer[bo + 1], buffer[bo + 2], buffer[bo + 3]];
            *float = f32::from_le_bytes(bytes);
        }
        values.push(v);
    }
    Ok(values)
}

fn read_accessor_quat(
    accessor: &gltf::Accessor,
    buffers: &[gltf::buffer::Data],
) -> Result<Vec<[f32; 4]>> {
    let view = accessor.view().context("Accessor has no buffer view")?;
    let buffer = &buffers[view.buffer().index()];
    let offset = view.offset() + accessor.offset();
    let count = accessor.count();
    let stride = view.stride().unwrap_or(16);

    let mut values = Vec::with_capacity(count);
    for i in 0..count {
        let byte_offset = offset + i * stride;
        let mut v = [0.0f32; 4];
        for (j, float) in v.iter_mut().enumerate() {
            let bo = byte_offset + j * 4;
            let bytes = [buffer[bo], buffer[bo + 1], buffer[bo + 2], buffer[bo + 3]];
            *float = f32::from_le_bytes(bytes);
        }
        values.push(v);
    }
    Ok(values)
}

// ============================================================================
// Interpolation
// ============================================================================

fn interpolate_vec3(
    times: &[f32],
    values: &[[f32; 3]],
    t: f32,
    _interp: gltf::animation::Interpolation,
) -> [f32; 3] {
    if times.is_empty() || values.is_empty() {
        return [0.0, 0.0, 0.0];
    }

    // Find keyframes
    let mut i = 0;
    while i < times.len() - 1 && times[i + 1] < t {
        i += 1;
    }

    if i >= times.len() - 1 {
        return values[values.len() - 1];
    }

    // Linear interpolation (simplified - ignores cubic spline)
    let t0 = times[i];
    let t1 = times[i + 1];
    let factor = if t1 > t0 { (t - t0) / (t1 - t0) } else { 0.0 };
    let factor = factor.clamp(0.0, 1.0);

    let v0 = values[i];
    let v1 = values[i + 1];

    [
        v0[0] + (v1[0] - v0[0]) * factor,
        v0[1] + (v1[1] - v0[1]) * factor,
        v0[2] + (v1[2] - v0[2]) * factor,
    ]
}

fn interpolate_quat(
    times: &[f32],
    values: &[[f32; 4]],
    t: f32,
    _interp: gltf::animation::Interpolation,
) -> [f32; 4] {
    if times.is_empty() || values.is_empty() {
        return [0.0, 0.0, 0.0, 1.0]; // Identity quaternion
    }

    // Find keyframes
    let mut i = 0;
    while i < times.len() - 1 && times[i + 1] < t {
        i += 1;
    }

    if i >= times.len() - 1 {
        return values[values.len() - 1];
    }

    // Spherical linear interpolation (slerp)
    let t0 = times[i];
    let t1 = times[i + 1];
    let factor = if t1 > t0 { (t - t0) / (t1 - t0) } else { 0.0 };
    let factor = factor.clamp(0.0, 1.0);

    slerp(values[i], values[i + 1], factor)
}

fn slerp(q0: [f32; 4], q1: [f32; 4], t: f32) -> [f32; 4] {
    let mut dot = q0[0] * q1[0] + q0[1] * q1[1] + q0[2] * q1[2] + q0[3] * q1[3];

    // Ensure shortest path
    let mut q1 = q1;
    if dot < 0.0 {
        q1 = [-q1[0], -q1[1], -q1[2], -q1[3]];
        dot = -dot;
    }

    // If quaternions are very close, use linear interpolation
    if dot > 0.9995 {
        let result = [
            q0[0] + t * (q1[0] - q0[0]),
            q0[1] + t * (q1[1] - q0[1]),
            q0[2] + t * (q1[2] - q0[2]),
            q0[3] + t * (q1[3] - q0[3]),
        ];
        return normalize_quat(result);
    }

    let theta_0 = dot.acos();
    let theta = theta_0 * t;
    let sin_theta = theta.sin();
    let sin_theta_0 = theta_0.sin();

    let s0 = (theta_0 - theta).cos() - dot * sin_theta / sin_theta_0;
    let s1 = sin_theta / sin_theta_0;

    [
        s0 * q0[0] + s1 * q1[0],
        s0 * q0[1] + s1 * q1[1],
        s0 * q0[2] + s1 * q1[2],
        s0 * q0[3] + s1 * q1[3],
    ]
}

fn normalize_quat(q: [f32; 4]) -> [f32; 4] {
    let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if len > 0.0 {
        [q[0] / len, q[1] / len, q[2] / len, q[3] / len]
    } else {
        [0.0, 0.0, 0.0, 1.0]
    }
}

/// List available animations in a glTF file
pub fn list_animations(input: &Path) -> Result<()> {
    let (document, buffers, _images) =
        gltf::import(input).with_context(|| format!("Failed to load glTF: {:?}", input))?;

    let animations: Vec<_> = document.animations().collect();
    if animations.is_empty() {
        tracing::info!("No animations found in {:?}", input);
        return Ok(());
    }

    tracing::info!("Animations in {:?}:", input);
    for (i, anim) in animations.iter().enumerate() {
        let name = anim.name().unwrap_or("unnamed");
        let channel_count = anim.channels().count();

        // Calculate duration
        let mut max_time = 0.0f32;
        for channel in anim.channels() {
            let sampler = channel.sampler();
            let input = sampler.input();
            if let Ok(times) = read_accessor_f32(&input, &buffers) {
                if let Some(&t) = times.last() {
                    max_time = max_time.max(t);
                }
            }
        }

        tracing::info!(
            "  [{}] '{}': {} channels, {:.2}s",
            i,
            name,
            channel_count,
            max_time
        );
    }

    Ok(())
}
