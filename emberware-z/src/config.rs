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
