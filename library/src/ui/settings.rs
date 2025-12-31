//! Settings UI for configuring video, audio, and input
//!
//! This wraps the shared settings UI from core and converts actions
//! to library-specific UiAction variants.

use super::UiAction;
use eframe::egui::Context;
use nethercore_core::app::config::Config;
use nethercore_core::app::ui::{SettingsAction, SharedSettingsUi};
use winit::keyboard::KeyCode;

/// Settings UI state (wraps the shared implementation)
pub struct SettingsUi {
    inner: SharedSettingsUi,
}

impl SettingsUi {
    pub fn new(config: &Config) -> Self {
        Self {
            inner: SharedSettingsUi::new(config),
        }
    }

    /// Update temp config from current config
    pub fn update_temp_config(&mut self, config: &Config) {
        self.inner.update_temp_config(config);
    }

    /// Handle key press for remapping.
    /// Returns true if the key was consumed (for remapping), false otherwise.
    pub fn handle_key_press(&mut self, key: KeyCode) -> bool {
        self.inner.handle_key_press(key)
    }

    /// Show the settings UI and return an action if needed
    pub fn show(&mut self, ctx: &Context) -> Option<UiAction> {
        // Make the settings UI visible when shown via library
        self.inner.visible = true;

        match self.inner.show_as_panel(ctx, "Back to Library") {
            SettingsAction::None => None,
            SettingsAction::Close => Some(UiAction::OpenSettings), // Toggle back to library
            SettingsAction::Save(config) => Some(UiAction::SaveSettings(*config)),
            SettingsAction::ResetDefaults => None, // Applied internally, no library action needed
            SettingsAction::PreviewScaleMode(mode) => Some(UiAction::SetScaleMode(mode)),
            SettingsAction::ToggleFullscreen(_) => None, // Not exposed to library
            SettingsAction::SetVolume(_) => None,        // Not exposed to library
        }
    }
}
