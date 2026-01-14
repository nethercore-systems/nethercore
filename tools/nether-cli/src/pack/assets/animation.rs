//! Animation keyframes loading.

use anyhow::{Context, Result};
use zx_common::{NetherZXAnimationHeader, PackedKeyframes};

/// Load keyframes from file
///
/// Supports:
/// - .nczxanim (Nethercore animation format) - direct load
/// - .gltf / .glb (glTF 2.0) - auto-converted via nether-export
pub fn load_keyframes(
    id: &str,
    path: &std::path::Path,
    animation_name: Option<&str>,
    skin_name: Option<&str>,
) -> Result<PackedKeyframes> {
    // Detect format by extension
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        // Native format - direct load
        "nczxanim" => load_keyframes_native(id, path),

        // glTF/GLB - convert via nether-export
        "gltf" | "glb" => {
            let converted = nether_export::convert_gltf_animation_to_memory(
                path,
                animation_name,
                skin_name,
                None, // Use default 30 FPS
            )
            .with_context(|| format!("Failed to convert glTF animation: {}", path.display()))?;

            Ok(PackedKeyframes {
                id: id.to_string(),
                bone_count: converted.bone_count,
                frame_count: converted.frame_count,
                data: converted.data,
            })
        }

        _ => anyhow::bail!(
            "Unsupported animation format '{}': {} (use .nczxanim, .gltf, or .glb)",
            ext,
            path.display()
        ),
    }
}

/// Load keyframes from native .nczxanim file
fn load_keyframes_native(id: &str, path: &std::path::Path) -> Result<PackedKeyframes> {
    let data = std::fs::read(path)
        .with_context(|| format!("Failed to load keyframes: {}", path.display()))?;

    // Parse header
    let header = NetherZXAnimationHeader::from_bytes(&data)
        .context("Failed to parse keyframes header - file may be corrupted or wrong format")?;

    // Copy values from packed struct to avoid alignment issues
    let bone_count = header.bone_count;
    let frame_count = header.frame_count;
    let flags = header.flags;

    // Validate header
    if !header.validate() {
        anyhow::bail!(
            "Invalid keyframes header (bone_count={}, frame_count={}, flags={}): {}",
            bone_count,
            frame_count,
            flags,
            path.display()
        );
    }

    // Validate data size
    let expected_size = header.file_size();
    if data.len() < expected_size {
        anyhow::bail!(
            "Keyframes data too small: {} bytes, expected {} (bones: {}, frames: {})",
            data.len(),
            expected_size,
            bone_count,
            frame_count
        );
    }

    // Extract frame data (skip header)
    let frame_data = data[NetherZXAnimationHeader::SIZE..expected_size].to_vec();

    Ok(PackedKeyframes {
        id: id.to_string(),
        bone_count,
        frame_count,
        data: frame_data,
    })
}
