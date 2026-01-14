//! Shared settings UI for configuring video, audio, and input
//!
//! This module provides a reusable settings UI that works in both
//! the library app (as a full panel) and the standalone player (as a popup window).

mod input_mapping;
mod types;
mod ui;

pub use types::{SettingsAction, SettingsTab};
pub use ui::SharedSettingsUi;
