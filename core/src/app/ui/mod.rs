//! Shared UI components for settings and configuration
//!
//! This module provides reusable UI components that work in both
//! the library app and standalone player.

mod keycode_display;
pub mod settings;

pub use keycode_display::keycode_to_display_string;
pub use settings::{SettingsAction, SettingsTab, SharedSettingsUi};
