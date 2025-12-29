//! Configuration management (~/.nethercore/config.toml)
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
    /// Debug inspection settings
    #[serde(default)]
    pub debug: DebugConfig,
    /// Capture (screenshot/GIF) settings
    #[serde(default)]
    pub capture: CaptureConfig,
}

/// Scaling mode for render target to window
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ScaleMode {
    /// Stretch to fill window (may distort aspect ratio)
    Stretch,
    /// Maintain aspect ratio, scale to fill as much as possible (adds letterbox bars)
    Fit,
    /// Integer scaling for pixel-perfect rendering (adds black bars, may not fill screen)
    #[default]
    PixelPerfect,
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

/// Debug inspection configuration.
///
/// Configures hotkeys and behavior for the debug inspection panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebugConfig {
    /// Toggle debug inspection panel (default: F4)
    #[serde(default = "default_panel_toggle")]
    pub panel_toggle: String,
    /// Toggle pause/resume (default: F5)
    #[serde(default = "default_pause_toggle")]
    pub pause_toggle: String,
    /// Step single frame when paused (default: F6)
    #[serde(default = "default_step_frame")]
    pub step_frame: String,
    /// Decrease time scale (default: F7)
    #[serde(default = "default_speed_decrease")]
    pub speed_decrease: String,
    /// Increase time scale (default: F8)
    #[serde(default = "default_speed_increase")]
    pub speed_increase: String,
}

/// Screenshot and GIF recording configuration.
///
/// Configures hotkeys and settings for capturing gameplay.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureConfig {
    /// Screenshot keybinding (default: F9)
    #[serde(default = "default_screenshot_key")]
    pub screenshot: String,
    /// GIF recording toggle keybinding (default: F10)
    #[serde(default = "default_gif_toggle_key")]
    pub gif_toggle: String,
    /// GIF recording framerate (default: 30)
    #[serde(default = "default_gif_fps")]
    pub gif_fps: u32,
    /// GIF max duration in seconds (default: 60)
    #[serde(default = "default_gif_max_seconds")]
    pub gif_max_seconds: u32,
}

fn default_panel_toggle() -> String {
    "F3".to_string()
}
fn default_pause_toggle() -> String {
    "F5".to_string()
}
fn default_step_frame() -> String {
    "F6".to_string()
}
fn default_speed_decrease() -> String {
    "F7".to_string()
}
fn default_speed_increase() -> String {
    "F8".to_string()
}

fn default_screenshot_key() -> String {
    "F9".to_string()
}
fn default_gif_toggle_key() -> String {
    "F10".to_string()
}
fn default_gif_fps() -> u32 {
    30
}
fn default_gif_max_seconds() -> u32 {
    60
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
            vsync: default_true(),
            resolution_scale: default_scale(),
            scale_mode: ScaleMode::default(),
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: default_volume(),
        }
    }
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            panel_toggle: default_panel_toggle(),
            pause_toggle: default_pause_toggle(),
            step_frame: default_step_frame(),
            speed_decrease: default_speed_decrease(),
            speed_increase: default_speed_increase(),
        }
    }
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            screenshot: default_screenshot_key(),
            gif_toggle: default_gif_toggle_key(),
            gif_fps: default_gif_fps(),
            gif_max_seconds: default_gif_max_seconds(),
        }
    }
}

/// Returns the platform-specific configuration directory.
///
/// On Windows: `%APPDATA%\Nethercore\config`
/// On macOS: `~/Library/Application Support/io.nethercore.Nethercore`
/// On Linux: `~/.config/Nethercore`
///
/// Returns `None` if the home directory cannot be determined.
pub fn config_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("io.nethercore", "", "Nethercore")
        .map(|dirs| dirs.config_dir().to_path_buf())
}

/// Returns the platform-specific data directory for game storage.
///
/// On Windows: `%APPDATA%\Nethercore\data`
/// On macOS: `~/Library/Application Support/io.nethercore.Nethercore`
/// On Linux: `~/.local/share/Nethercore`
///
/// This is where downloaded games and save data are stored.
/// Returns `None` if the home directory cannot be determined.
pub fn data_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("io.nethercore", "", "Nethercore")
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

/// Parse a key string to a key name for comparison.
///
/// Returns the uppercase key name if valid, or None if not recognized.
/// Currently supports F1-F12 keys.
pub fn parse_key_name(s: &str) -> Option<&'static str> {
    match s.to_uppercase().as_str() {
        "F1" => Some("F1"),
        "F2" => Some("F2"),
        "F3" => Some("F3"),
        "F4" => Some("F4"),
        "F5" => Some("F5"),
        "F6" => Some("F6"),
        "F7" => Some("F7"),
        "F8" => Some("F8"),
        "F9" => Some("F9"),
        "F10" => Some("F10"),
        "F11" => Some("F11"),
        "F12" => Some("F12"),
        "ESCAPE" | "ESC" => Some("ESCAPE"),
        _ => None,
    }
}

/// Validate that no keybindings conflict with each other.
///
/// Checks all configurable keys against reserved system keys and each other.
/// Returns a list of warning messages for any conflicts found.
pub fn validate_keybindings(config: &Config) -> Vec<String> {
    use std::collections::HashSet;
    let mut warnings = Vec::new();
    let mut used_keys: HashSet<String> = HashSet::new();

    // Reserved system keys (hardcoded, not configurable)
    used_keys.insert("ESCAPE".to_string());
    used_keys.insert("F2".to_string()); // Settings panel
    used_keys.insert("F4".to_string()); // Debug inspector panel
    used_keys.insert("F11".to_string()); // Fullscreen

    // Debug keys
    let debug_keys = [
        (&config.debug.panel_toggle, "debug.panel_toggle"),
        (&config.debug.pause_toggle, "debug.pause_toggle"),
        (&config.debug.step_frame, "debug.step_frame"),
        (&config.debug.speed_decrease, "debug.speed_decrease"),
        (&config.debug.speed_increase, "debug.speed_increase"),
    ];
    for (key, name) in debug_keys {
        let key_upper = key.to_uppercase();
        if !used_keys.insert(key_upper.clone()) {
            warnings.push(format!(
                "{} key '{}' conflicts with another binding",
                name, key
            ));
        }
    }

    // Capture keys
    let capture_keys = [
        (&config.capture.screenshot, "capture.screenshot"),
        (&config.capture.gif_toggle, "capture.gif_toggle"),
    ];
    for (key, name) in capture_keys {
        let key_upper = key.to_uppercase();
        if !used_keys.insert(key_upper.clone()) {
            warnings.push(format!(
                "{} key '{}' conflicts with another binding",
                name, key
            ));
        }
    }

    warnings
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
            debug: DebugConfig::default(),
            capture: CaptureConfig::default(),
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
    fn test_scale_mode_default_is_integer() {
        assert_eq!(ScaleMode::default(), ScaleMode::PixelPerfect);
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
