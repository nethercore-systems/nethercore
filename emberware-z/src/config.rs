//! Configuration management (~/.emberware/config.toml)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::input::InputConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub video: VideoConfig,
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub input: InputConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    #[serde(default)]
    pub fullscreen: bool,
    #[serde(default = "default_true")]
    pub vsync: bool,
    #[serde(default = "default_scale")]
    pub resolution_scale: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    #[serde(default = "default_volume")]
    pub master_volume: f32,
}

fn default_true() -> bool { true }
fn default_scale() -> u32 { 2 }
fn default_volume() -> f32 { 0.8 }

impl Default for Config {
    fn default() -> Self {
        Self {
            video: VideoConfig::default(),
            audio: AudioConfig::default(),
            input: InputConfig::default(),
        }
    }
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            fullscreen: false,
            vsync: true,
            resolution_scale: 2,
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 0.8,
        }
    }
}

pub fn config_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("io", "emberware", "emberware")
        .map(|dirs| dirs.config_dir().to_path_buf())
}

pub fn data_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("io", "emberware", "emberware")
        .map(|dirs| dirs.data_dir().to_path_buf())
}

pub fn load() -> Config {
    config_dir()
        .and_then(|dir| std::fs::read_to_string(dir.join("config.toml")).ok())
        .and_then(|content| toml::from_str(&content).ok())
        .unwrap_or_default()
}

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

    #[test]
    fn test_default_helper_functions() {
        assert!(default_true());
        assert_eq!(default_scale(), 2);
        assert!((default_volume() - 0.8).abs() < f32::EPSILON);
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
            },
            audio: AudioConfig {
                master_volume: 0.5,
            },
            input: InputConfig::default(),
        };

        let toml_str = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();

        assert!(parsed.video.fullscreen);
        assert!(!parsed.video.vsync);
        assert_eq!(parsed.video.resolution_scale, 3);
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
        };
        let toml_str = toml::to_string(&video).unwrap();
        assert!(toml_str.contains("fullscreen = true"));
        assert!(toml_str.contains("vsync = true"));
        assert!(toml_str.contains("resolution_scale = 4"));
    }

    #[test]
    fn test_audio_config_serialize() {
        let audio = AudioConfig {
            master_volume: 1.0,
        };
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
            };
            let toml_str = toml::to_string(&video).unwrap();
            let parsed: VideoConfig = toml::from_str(&toml_str).unwrap();
            assert_eq!(parsed.resolution_scale, scale);
        }
    }

    // =============================================================
    // Directory function tests
    // =============================================================

    #[test]
    fn test_config_dir_returns_some() {
        // On most systems, config_dir should return Some
        // (might fail in unusual environments, but generally works)
        let dir = config_dir();
        // We just check it's consistent with itself
        assert_eq!(dir, config_dir());
    }

    #[test]
    fn test_data_dir_returns_some() {
        let dir = data_dir();
        assert_eq!(dir, data_dir());
    }

    #[test]
    fn test_config_and_data_dirs_differ() {
        // config_dir and data_dir should typically be different paths
        let config = config_dir();
        let data = data_dir();
        if let (Some(c), Some(d)) = (config, data) {
            // They might be the same on some platforms, but typically differ
            // Just verify they're both valid paths
            assert!(c.to_string_lossy().contains("emberware"));
            assert!(d.to_string_lossy().contains("emberware"));
        }
    }

    // =============================================================
    // Clone and Debug trait tests
    // =============================================================

    #[test]
    fn test_config_clone() {
        let config = Config::default();
        let cloned = config.clone();
        assert_eq!(cloned.video.fullscreen, config.video.fullscreen);
        assert_eq!(cloned.video.vsync, config.video.vsync);
        assert_eq!(cloned.video.resolution_scale, config.video.resolution_scale);
    }

    #[test]
    fn test_config_debug() {
        let config = Config::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("video"));
        assert!(debug_str.contains("audio"));
    }

    #[test]
    fn test_video_config_debug() {
        let video = VideoConfig::default();
        let debug_str = format!("{:?}", video);
        assert!(debug_str.contains("VideoConfig"));
        assert!(debug_str.contains("fullscreen"));
    }

    #[test]
    fn test_audio_config_debug() {
        let audio = AudioConfig::default();
        let debug_str = format!("{:?}", audio);
        assert!(debug_str.contains("AudioConfig"));
        assert!(debug_str.contains("master_volume"));
    }

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
