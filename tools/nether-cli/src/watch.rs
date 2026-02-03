//! File watching for dev mode hot reload/relaunch
//!
//! Watches source files and assets, triggering rebuilds when changes are detected.
//! Currently implements Phase 1: full rebuild + relaunch on any change.

use anyhow::{Context, Result};
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use crate::manifest::{AssetsSection, NetherManifest};

/// Source file extensions to watch
const SOURCE_EXTENSIONS: &[&str] = &["rs", "zig", "c", "cpp", "h", "hpp", "toml"];

/// Directories to exclude from source watching
const EXCLUDED_DIRS: &[&str] = &["target", "zig-out", "build", "node_modules", ".git"];

/// Debounce duration for file changes (batches rapid saves)
const DEBOUNCE_DURATION: Duration = Duration::from_millis(100);

/// Collects all paths that should be watched for a project.
pub fn collect_watch_paths(project_dir: &Path, manifest: &NetherManifest) -> WatchPaths {
    let mut asset_files = Vec::new();
    collect_asset_paths(project_dir, &manifest.assets, &mut asset_files);

    WatchPaths {
        manifest: project_dir.join("nether.toml"),
        source_dirs: vec![project_dir.to_path_buf()],
        asset_files,
    }
}

/// Collected paths to watch
#[derive(Debug, Default)]
pub struct WatchPaths {
    /// The nether.toml manifest file
    pub manifest: PathBuf,
    /// Source directories to watch recursively
    pub source_dirs: Vec<PathBuf>,
    /// Individual asset files to watch
    pub asset_files: Vec<PathBuf>,
}

impl WatchPaths {
    /// Get total number of watch targets
    pub fn count(&self) -> usize {
        1 + self.source_dirs.len() + self.asset_files.len()
    }
}

/// Collect asset file paths from the assets section
fn collect_asset_paths(project_dir: &Path, assets: &AssetsSection, out: &mut Vec<PathBuf>) {
    let add_entries = |entries: &[crate::manifest::AssetEntry], out: &mut Vec<PathBuf>| {
        for entry in entries {
            let path = project_dir.join(&entry.path);
            if path.exists() {
                out.push(path);
            }
        }
    };

    add_entries(&assets.textures, out);
    add_entries(&assets.meshes, out);
    add_entries(&assets.skeletons, out);
    add_entries(&assets.keyframes, out);
    add_entries(&assets.animations, out);
    add_entries(&assets.sounds, out);
    add_entries(&assets.trackers, out);
    add_entries(&assets.data, out);
}

/// Check if a path should be watched (source file filtering)
fn should_watch_path(path: &Path) -> bool {
    // Check if in excluded directory
    for component in path.components() {
        if let std::path::Component::Normal(name) = component {
            if let Some(name_str) = name.to_str() {
                if EXCLUDED_DIRS.contains(&name_str) {
                    return false;
                }
            }
        }
    }

    // Check extension for source files
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        SOURCE_EXTENSIONS.contains(&ext)
    } else {
        false
    }
}

/// Event received from the file watcher
#[derive(Debug)]
pub enum WatchEvent {
    /// Files changed - rebuild needed
    FilesChanged(Vec<PathBuf>),
    /// Manifest changed - need full re-watch
    ManifestChanged,
    /// Watch error occurred
    Error(String),
}

/// File watcher for dev mode
pub struct FileWatcher {
    /// Receiver for debounced events
    rx: mpsc::Receiver<Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>>,
    /// The debouncer (holds the watcher)
    _debouncer: notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>,
    /// Manifest path for detecting manifest changes
    manifest_path: PathBuf,
    /// Set of asset paths for quick lookup
    asset_paths: HashSet<PathBuf>,
}

impl FileWatcher {
    /// Create a new file watcher for the given paths
    pub fn new(paths: &WatchPaths) -> Result<Self> {
        let (tx, rx) = mpsc::channel();

        let mut debouncer =
            new_debouncer(DEBOUNCE_DURATION, tx).context("Failed to create file watcher")?;

        let watcher = debouncer.watcher();

        // Watch manifest file
        if paths.manifest.exists() {
            watcher
                .watch(&paths.manifest, RecursiveMode::NonRecursive)
                .with_context(|| {
                    format!("Failed to watch manifest: {}", paths.manifest.display())
                })?;
        }

        // Watch source directories recursively
        for dir in &paths.source_dirs {
            if dir.exists() {
                watcher
                    .watch(dir, RecursiveMode::Recursive)
                    .with_context(|| format!("Failed to watch directory: {}", dir.display()))?;
            }
        }

        // Watch individual asset files
        for file in &paths.asset_files {
            if file.exists() {
                // Watch the parent directory for the file (notify doesn't watch individual files well)
                if let Some(parent) = file.parent() {
                    // Only watch if we haven't already watched this directory
                    watcher
                        .watch(parent, RecursiveMode::NonRecursive)
                        .with_context(|| format!("Failed to watch asset: {}", file.display()))?;
                }
            }
        }

        let asset_paths: HashSet<PathBuf> = paths
            .asset_files
            .iter()
            .filter_map(|p| p.canonicalize().ok())
            .collect();

        let manifest_path = paths
            .manifest
            .canonicalize()
            .unwrap_or_else(|_| paths.manifest.clone());

        Ok(Self {
            rx,
            _debouncer: debouncer,
            manifest_path,
            asset_paths,
        })
    }

    /// Wait for the next file change event
    ///
    /// Blocks until files change or an error occurs.
    #[allow(dead_code)] // May be used in future for blocking watch mode
    pub fn wait_for_changes(&self) -> WatchEvent {
        match self.rx.recv() {
            Ok(Ok(events)) => {
                let mut changed_files = Vec::new();
                let mut manifest_changed = false;

                for event in events {
                    // Only process data change events
                    if !matches!(event.kind, DebouncedEventKind::Any) {
                        continue;
                    }

                    let path = event.path;

                    // Check if manifest changed
                    if let Ok(canonical) = path.canonicalize() {
                        if canonical == self.manifest_path {
                            manifest_changed = true;
                            continue;
                        }
                    }

                    // Check if it's a watched asset
                    if let Ok(canonical) = path.canonicalize() {
                        if self.asset_paths.contains(&canonical) {
                            changed_files.push(path);
                            continue;
                        }
                    }

                    // Check if it's a source file we care about
                    if should_watch_path(&path) {
                        changed_files.push(path);
                    }
                }

                if manifest_changed {
                    WatchEvent::ManifestChanged
                } else if !changed_files.is_empty() {
                    WatchEvent::FilesChanged(changed_files)
                } else {
                    // Spurious event, wait for next one
                    self.wait_for_changes()
                }
            }
            Ok(Err(e)) => WatchEvent::Error(format!("Watch error: {}", e)),
            Err(e) => WatchEvent::Error(format!("Channel error: {}", e)),
        }
    }

    /// Non-blocking check for changes
    ///
    /// Returns None if no changes detected yet.
    pub fn try_recv(&self) -> Option<WatchEvent> {
        match self.rx.try_recv() {
            Ok(Ok(events)) => {
                let mut changed_files = Vec::new();
                let mut manifest_changed = false;

                for event in events {
                    if !matches!(event.kind, DebouncedEventKind::Any) {
                        continue;
                    }

                    let path = event.path;

                    if let Ok(canonical) = path.canonicalize() {
                        if canonical == self.manifest_path {
                            manifest_changed = true;
                            continue;
                        }
                        if self.asset_paths.contains(&canonical) {
                            changed_files.push(path);
                            continue;
                        }
                    }

                    if should_watch_path(&path) {
                        changed_files.push(path);
                    }
                }

                if manifest_changed {
                    Some(WatchEvent::ManifestChanged)
                } else if !changed_files.is_empty() {
                    Some(WatchEvent::FilesChanged(changed_files))
                } else {
                    None
                }
            }
            Ok(Err(e)) => Some(WatchEvent::Error(format!("Watch error: {}", e))),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(WatchEvent::Error("Watcher disconnected".to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_should_watch_path_source_files() {
        assert!(should_watch_path(Path::new("src/main.rs")));
        assert!(should_watch_path(Path::new("src/lib.rs")));
        assert!(should_watch_path(Path::new("Cargo.toml")));
        assert!(should_watch_path(Path::new("build.zig")));
        assert!(should_watch_path(Path::new("src/game.c")));
        assert!(should_watch_path(Path::new("include/types.h")));
    }

    #[test]
    fn test_should_watch_path_excluded_dirs() {
        assert!(!should_watch_path(Path::new("target/debug/game.rs")));
        assert!(!should_watch_path(Path::new("zig-out/lib.zig")));
        assert!(!should_watch_path(Path::new(".git/config")));
        assert!(!should_watch_path(Path::new("node_modules/pkg/index.js")));
    }

    #[test]
    fn test_should_watch_path_non_source() {
        assert!(!should_watch_path(Path::new("assets/player.png")));
        assert!(!should_watch_path(Path::new("README.md")));
        assert!(!should_watch_path(Path::new("game.nczx")));
    }

    #[test]
    fn test_collect_watch_paths() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        // Create manifest
        let manifest_content = r#"
[game]
id = "test-game"
title = "Test Game"
author = "Test"
version = "1.0.0"

[[assets.textures]]
id = "player"
path = "assets/player.png"

[[assets.sounds]]
id = "jump"
path = "sounds/jump.wav"
"#;
        fs::write(project_dir.join("nether.toml"), manifest_content).unwrap();

        // Create asset directories and files
        fs::create_dir_all(project_dir.join("assets")).unwrap();
        fs::create_dir_all(project_dir.join("sounds")).unwrap();
        fs::write(project_dir.join("assets/player.png"), b"PNG").unwrap();
        fs::write(project_dir.join("sounds/jump.wav"), b"WAV").unwrap();

        let manifest = NetherManifest::load(&project_dir.join("nether.toml")).unwrap();
        let paths = collect_watch_paths(project_dir, &manifest);

        assert_eq!(paths.manifest, project_dir.join("nether.toml"));
        assert_eq!(paths.source_dirs.len(), 1);
        assert_eq!(paths.source_dirs[0], project_dir);
        assert_eq!(paths.asset_files.len(), 2);
    }
}
