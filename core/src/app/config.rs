//! Configuration management (~/.emberware/config.toml)
//!
//! Handles loading, saving, and providing defaults for application settings.
//! Settings are stored in TOML format in the platform-specific config directory.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::app::input::InputConfig;

/// Application configuration.
///
/// Contains all user-configurable settings organized into sections.
/// Serialized to/from TOML format for persistence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Config {
    /// Video/graphics settings
    #[serde(default)]
    pub video: VideoConfig,
    /// Audio settings
    #[serde(default)]
    pub audio: AudioConfig,
    /// Input/controller settings
    #[serde(default)]
    pub input: InputConfig,
}

/// Scaling mode for render target to window
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScaleMode {
    /// Stretch to fill window (may distort aspect ratio)
    Stretch,
    /// Integer scaling for pixel-perfect rendering (adds black bars)
    PixelPerfect,
}

impl Default for ScaleMode {
    fn default() -> Self {
        ScaleMode::Stretch
    }
}

/// Video and graphics configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoConfig {
    /// Whether to run in fullscreen mode (default: false)
    #[serde(default)]
    pub fullscreen: bool,
    /// Whether to enable vertical sync (default: true)
    #[serde(default = "default_true")]
    pub vsync: bool,
    /// Resolution scale multiplier (default: 2, range: 1-4)
    #[serde(default = "default_scale")]
    pub resolution_scale: u32,
    /// Scaling mode for game framebuffer (default: Stretch)
    #[serde(default)]
    pub scale_mode: ScaleMode,
}

/// Audio configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Master volume level (default: 0.8, range: 0.0-1.0)
    #[serde(default = "default_volume")]
    pub master_volume: f32,
}

fn default_true() -> bool {
    true
}
fn default_scale() -> u32 {
    2
}
fn default_volume() -> f32 {
    0.8
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            fullscreen: false,
            vsync: true,
            resolution_scale: 2,
            scale_mode: ScaleMode::default(),
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self { master_volume: 0.8 }
    }
}

/// Returns the platform-specific configuration directory.
///
/// On Windows: `%APPDATA%\emberware\emberware\config`
/// On macOS: `~/Library/Application Support/io.emberware.emberware`
/// On Linux: `~/.config/emberware`
///
/// Returns `None` if the home directory cannot be determined.
pub fn config_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("io", "emberware", "emberware")
        .map(|dirs| dirs.config_dir().to_path_buf())
}

/// Returns the platform-specific data directory for game storage.
///
/// On Windows: `%APPDATA%\emberware\emberware\data`
/// On macOS: `~/Library/Application Support/io.emberware.emberware`
/// On Linux: `~/.local/share/emberware`
///
/// This is where downloaded games and save data are stored.
/// Returns `None` if the home directory cannot be determined.
pub fn data_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("io", "emberware", "emberware")
        .map(|dirs| dirs.data_dir().to_path_buf())
}

/// Loads the configuration from disk.
///
/// Reads `config.toml` from the platform's configuration directory.
/// Returns default values if the file doesn't exist or cannot be parsed.
pub fn load() -> Config {
    config_dir()
        .and_then(|dir| std::fs::read_to_string(dir.join("config.toml")).ok())
        .and_then(|content| toml::from_str(&content).ok())
        .unwrap_or_default()
}

/// Saves the configuration to disk.
///
/// Writes `config.toml` to the platform's configuration directory.
/// Creates the directory if it doesn't exist.
///
/// # Errors
///
/// Returns an error if the directory cannot be created or the file
/// cannot be written.
pub fn save(config: &Config) -> std::io::Result<()> {
    if let Some(dir) = config_dir() {
        std::fs::create_dir_all(&dir)?;
        let content = toml::to_string_pretty(config).unwrap();
        std::fs::write(dir.join("config.toml"), content)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // =============================================================
    // Default value tests
    // =============================================================

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(!config.video.fullscreen);
        assert!(config.video.vsync);
        assert_eq!(config.video.resolution_scale, 2);
        assert!((config.audio.master_volume - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_video_config_default() {
        let video = VideoConfig::default();
        assert!(!video.fullscreen);
        assert!(video.vsync);
        assert_eq!(video.resolution_scale, 2);
    }

    #[test]
    fn test_audio_config_default() {
        let audio = AudioConfig::default();
        assert!((audio.master_volume - 0.8).abs() < f32::EPSILON);
    }

    // =============================================================
    // TOML serialization tests
    // =============================================================

    #[test]
    fn test_config_serialize_roundtrip() {
        let config = Config {
            video: VideoConfig {
                fullscreen: true,
                vsync: false,
                resolution_scale: 3,
                scale_mode: ScaleMode::PixelPerfect,
            },
            audio: AudioConfig { master_volume: 0.5 },
            input: InputConfig::default(),
        };

        let toml_str = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();

        assert!(parsed.video.fullscreen);
        assert!(!parsed.video.vsync);
        assert_eq!(parsed.video.resolution_scale, 3);
        assert_eq!(parsed.video.scale_mode, ScaleMode::PixelPerfect);
        assert!((parsed.audio.master_volume - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_config_deserialize_empty() {
        // Empty TOML should produce defaults
        let config: Config = toml::from_str("").unwrap();
        assert!(!config.video.fullscreen);
        assert!(config.video.vsync);
        assert_eq!(config.video.resolution_scale, 2);
        assert!((config.audio.master_volume - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_config_deserialize_partial_video() {
        // Only set fullscreen, rest should default
        let toml_str = r#"
[video]
fullscreen = true
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.video.fullscreen);
        assert!(config.video.vsync); // default
        assert_eq!(config.video.resolution_scale, 2); // default
    }

    #[test]
    fn test_config_deserialize_partial_audio() {
        let toml_str = r#"
[audio]
master_volume = 0.3
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!((config.audio.master_volume - 0.3).abs() < f32::EPSILON);
        // video should be default
        assert!(!config.video.fullscreen);
    }

    #[test]
    fn test_video_config_serialize() {
        let video = VideoConfig {
            fullscreen: true,
            vsync: true,
            resolution_scale: 4,
            scale_mode: ScaleMode::Stretch,
        };
        let toml_str = toml::to_string(&video).unwrap();
        assert!(toml_str.contains("fullscreen = true"));
        assert!(toml_str.contains("vsync = true"));
        assert!(toml_str.contains("resolution_scale = 4"));
    }

    #[test]
    fn test_audio_config_serialize() {
        let audio = AudioConfig { master_volume: 1.0 };
        let toml_str = toml::to_string(&audio).unwrap();
        assert!(toml_str.contains("master_volume = 1.0"));
    }

    // =============================================================
    // Edge case tests
    // =============================================================

    #[test]
    fn test_audio_volume_zero() {
        let audio = AudioConfig { master_volume: 0.0 };
        let toml_str = toml::to_string(&audio).unwrap();
        let parsed: AudioConfig = toml::from_str(&toml_str).unwrap();
        assert!((parsed.master_volume - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_audio_volume_max() {
        let audio = AudioConfig { master_volume: 1.0 };
        let toml_str = toml::to_string(&audio).unwrap();
        let parsed: AudioConfig = toml::from_str(&toml_str).unwrap();
        assert!((parsed.master_volume - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_resolution_scale_values() {
        for scale in [1, 2, 3, 4] {
            let video = VideoConfig {
                fullscreen: false,
                vsync: true,
                resolution_scale: scale,
                scale_mode: ScaleMode::default(),
            };
            let toml_str = toml::to_string(&video).unwrap();
            let parsed: VideoConfig = toml::from_str(&toml_str).unwrap();
            assert_eq!(parsed.resolution_scale, scale);
        }
    }

    // =============================================================
    // Directory function tests
    // =============================================================

    // =============================================================
    // Load function tests (without filesystem access)
    // =============================================================

    #[test]
    fn test_load_returns_default_when_no_file() {
        // load() should return defaults if the file doesn't exist
        // We can't easily test this without mocking, but we can verify
        // the function doesn't panic and returns a valid config
        let config = load();
        // Should get defaults or whatever is in the actual config file
        // Just verify it's a valid Config struct
        assert!(config.video.resolution_scale > 0);
        assert!(config.audio.master_volume >= 0.0);
    }
}
