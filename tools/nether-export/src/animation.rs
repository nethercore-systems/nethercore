//! Animation converter (glTF -> .nczxanim)
//!
//! Extracts and samples animation clips from glTF files.
//! Outputs the new compressed platform format (16 bytes per bone).

use anyhow::{bail, Context, Result};
use hashbrown::HashMap;
use std::fs::File;
use std::io::{BufWriter, Cursor};
use std::path::Path;

use crate::formats::write_nether_animation;

/// Default sample rate for animations (frames per second)
const DEFAULT_FRAME_RATE: f32 = 30.0;

/// Bone transform (TRS) for a single bone in a single frame
#[derive(Clone, Copy, Debug)]
pub struct BoneTRS {
    /// Quaternion rotation [x, y, z, w]
    pub rotation: [f32; 4],
    /// Position/translation
    pub position: [f32; 3],
    /// Scale
    pub scale: [f32; 3],
}

impl Default for BoneTRS {
    fn default() -> Self {
        Self {
            rotation: [0.0, 0.0, 0.0, 1.0], // Identity quaternion
            position: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

/// Result of in-memory animation conversion
#[derive(Debug, Clone)]
pub struct ConvertedAnimation {
    /// Number of bones per frame
    pub bone_count: u8,
    /// Number of frames
    pub frame_count: u16,
    /// Packed animation data (frame_count × bone_count × 16 bytes)
    /// Does NOT include the 4-byte header
    pub data: Vec<u8>,
}

/// Convert glTF animation to in-memory format (for direct ROM packing)
///
/// # Arguments
/// * `input` - Path to the glTF/GLB file
/// * `animation_name` - Optional animation name to select (uses first animation if None)
/// * `skin_name` - Optional skin name to select (uses first skin if None)
/// * `frame_rate` - Optional frame rate for sampling (defaults to 30 FPS)
pub fn convert_gltf_animation_to_memory(
    input: &Path,
    animation_name: Option<&str>,
    skin_name: Option<&str>,
    frame_rate: Option<f32>,
) -> Result<ConvertedAnimation> {
    let (document, buffers, _images) =
        gltf::import(input).with_context(|| format!("Failed to load glTF: {:?}", input))?;

    // Find skin by name or use first
    let skin = if let Some(name) = skin_name {
        document
            .skins()
            .find(|s| s.name() == Some(name))
            .with_context(|| format!("Skin '{}' not found in glTF", name))?
    } else {
        document
            .skins()
            .next()
            .context("No skins found in glTF file")?
    };

    // Find animation by name or use first
    let animation = if let Some(name) = animation_name {
        document
            .animations()
            .find(|a| a.name() == Some(name))
            .with_context(|| {
                let available: Vec<_> = document.animations().filter_map(|a| a.name()).collect();
                format!(
                    "Animation '{}' not found in glTF. Available animations: {:?}",
                    name, available
                )
            })?
    } else {
        document
            .animations()
            .next()
            .context("No animations found in glTF file")?
    };

    let frame_rate = frame_rate.unwrap_or(DEFAULT_FRAME_RATE);

    // Sample the animation (returns TRS per bone per frame)
    let (frames, bone_count) = sample_animation(&animation, &skin, &buffers, frame_rate)?;

    if frames.is_empty() {
        bail!("Animation produced no frames");
    }

    // Validate bone count
    if bone_count > 255 {
        bail!(
            "Animation has {} bones, maximum is 255 for new format",
            bone_count
        );
    }

    // Validate frame count
    if frames.len() > 65535 {
        bail!(
            "Animation has {} frames, maximum is 65535 for new format",
            frames.len()
        );
    }

    // Write to in-memory buffer
    let mut buffer = Cursor::new(Vec::new());
    write_nether_animation(&mut buffer, bone_count as u8, &frames)?;

    // Extract data (skip 4-byte header)
    let full_data = buffer.into_inner();
    let data = full_data[4..].to_vec();

    Ok(ConvertedAnimation {
        bone_count: bone_count as u8,
        frame_count: frames.len() as u16,
        data,
    })
}

/// Convert glTF animation to NetherAnimation format
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

    // Sample the animation (returns TRS per bone per frame)
    let (frames, bone_count) = sample_animation(&animation, &skin, &buffers, frame_rate)?;

    if frames.is_empty() {
        bail!("Animation produced no frames");
    }

    // Validate bone count
    if bone_count > 255 {
        bail!(
            "Animation has {} bones, maximum is 255 for new format",
            bone_count
        );
    }

    // Validate frame count
    if frames.len() > 65535 {
        bail!(
            "Animation has {} frames, maximum is 65535 for new format",
            frames.len()
        );
    }

    // Write output
    let file =
        File::create(output).with_context(|| format!("Failed to create output: {:?}", output))?;
    let mut writer = BufWriter::new(file);

    write_nether_animation(&mut writer, bone_count as u8, &frames)?;

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
///
/// Returns (frames, bone_count) where each frame is Vec<BoneTRS> of bone transforms.
fn sample_animation(
    animation: &gltf::Animation,
    skin: &gltf::Skin,
    buffers: &[gltf::buffer::Data],
    frame_rate: f32,
) -> Result<(Vec<Vec<BoneTRS>>, u32)> {
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

    // Initialize frames with node rest poses (not identity defaults)
    // This ensures bones without animation channels keep their bind pose
    let mut frames: Vec<Vec<BoneTRS>> = (0..frame_count)
        .map(|_| {
            joints
                .iter()
                .map(|joint| {
                    let (t, r, s) = joint.transform().decomposed();
                    BoneTRS {
                        position: t,
                        rotation: r,
                        scale: s,
                    }
                })
                .collect()
        })
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

        #[allow(clippy::needless_range_loop)]
        match target.property() {
            gltf::animation::Property::Translation => {
                let values = read_accessor_vec3(&output_accessor, buffers)?;
                for frame_idx in 0..frame_count {
                    let t = frame_idx as f32 / frame_rate;
                    let value = interpolate_vec3(&times, &values, t, interpolation);
                    frames[frame_idx][bone_index].position = value;
                }
            }
            gltf::animation::Property::Rotation => {
                let values = read_accessor_quat(&output_accessor, buffers)?;
                for frame_idx in 0..frame_count {
                    let t = frame_idx as f32 / frame_rate;
                    let value = interpolate_quat(&times, &values, t, interpolation);
                    frames[frame_idx][bone_index].rotation = value;
                }
            }
            gltf::animation::Property::Scale => {
                let values = read_accessor_vec3(&output_accessor, buffers)?;
                for frame_idx in 0..frame_count {
                    let t = frame_idx as f32 / frame_rate;
                    let value = interpolate_vec3(&times, &values, t, interpolation);
                    frames[frame_idx][bone_index].scale = value;
                }
            }
            _ => {} // Ignore weights/morph targets
        }
    }

    Ok((frames, bone_count as u32))
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
