//! Public types for settings UI actions and tab selection

use crate::app::config::{Config, ScaleMode};

/// Actions returned from the settings UI
#[derive(Debug, Clone)]
pub enum SettingsAction {
    /// No action this frame
    None,
    /// Close the settings panel
    Close,
    /// Save settings to disk
    Save(Box<Config>),
    /// Reset to defaults
    ResetDefaults,
    /// Preview scale mode change (apply immediately for feedback)
    PreviewScaleMode(ScaleMode),
    /// Toggle fullscreen (apply immediately)
    ToggleFullscreen(bool),
    /// Set volume (apply immediately for preview)
    SetVolume(f32),
}

/// Settings tab selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Video,
    Audio,
    Controls,
    Hotkeys,
}
