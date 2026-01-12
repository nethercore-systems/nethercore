use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::manifest::NetherManifest;

pub struct ManifestContext {
    pub manifest: NetherManifest,
    pub project_dir: PathBuf,
}

pub fn load_manifest(manifest_path: &Path) -> Result<ManifestContext> {
    let manifest = NetherManifest::load(manifest_path)?;
    manifest.validate()?;

    let project_dir = manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    Ok(ManifestContext {
        manifest,
        project_dir,
    })
}

pub fn resolve_wasm_path(ctx: &ManifestContext, override_path: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = override_path {
        Ok(path)
    } else {
        ctx.manifest.find_wasm(&ctx.project_dir, false)
    }
}
