//! Packed EPU cubemap-face environment loading.

use anyhow::{bail, Context, Result};
use zx_common::formats::PackedEpuEnvironmentFaces;
use zx_common::TextureFormat;

use crate::manifest::EpuEnvironmentEntry;

struct LoadedFace {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

fn load_face(path: &std::path::Path) -> Result<LoadedFace> {
    let img = image::open(path)
        .with_context(|| format!("Failed to load EPU face image: {}", path.display()))?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(LoadedFace {
        width,
        height,
        data: rgba.into_raw(),
    })
}

/// Load a six-face cubemap environment as a packed EPU source asset.
pub fn load_epu_environment(
    id: &str,
    project_dir: &std::path::Path,
    entry: &EpuEnvironmentEntry,
) -> Result<PackedEpuEnvironmentFaces> {
    let px = load_face(&project_dir.join(&entry.px))?;
    let nx = load_face(&project_dir.join(&entry.nx))?;
    let py = load_face(&project_dir.join(&entry.py))?;
    let ny = load_face(&project_dir.join(&entry.ny))?;
    let pz = load_face(&project_dir.join(&entry.pz))?;
    let nz = load_face(&project_dir.join(&entry.nz))?;

    let width = px.width;
    let height = px.height;
    for (name, face) in [
        ("nx", &nx),
        ("py", &py),
        ("ny", &ny),
        ("pz", &pz),
        ("nz", &nz),
    ] {
        if face.width != width || face.height != height {
            bail!(
                "EPU environment '{}' face '{}' dimensions {}x{} do not match {}x{}",
                id,
                name,
                face.width,
                face.height,
                width,
                height
            );
        }
    }

    if width != height {
        bail!(
            "EPU environment '{}' must use square faces, got {}x{}",
            id,
            width,
            height
        );
    }

    let packed = PackedEpuEnvironmentFaces {
        id: id.to_string(),
        width: width as u16,
        height: height as u16,
        format: TextureFormat::Rgba8,
        px: px.data,
        nx: nx.data,
        py: py.data,
        ny: ny.data,
        pz: pz.data,
        nz: nz.data,
    };

    if !packed.validate() {
        bail!("EPU environment '{}' failed packed validation", id);
    }

    Ok(packed)
}
