//! Shared settings UI for configuring video, audio, and input
//!
//! This module provides a reusable settings UI that works in both
//! the library app (as a full panel) and the standalone player (as a popup window).

use egui::{ComboBox, Context, Slider, Ui};
use winit::keyboard::KeyCode;

use crate::app::config::{Config, ScaleMode};
use crate::app::input::KeyboardMapping;

use super::keycode_display::keycode_to_display_string;

/// Actions returned from the settings UI
#[derive(Debug, Clone)]
pub enum SettingsAction {
    /// No action this frame
    None,
    /// Close the settings panel
    Close,
    /// Save settings to disk
    Save(Config),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputButton {
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    ButtonA,
    ButtonB,
    ButtonX,
    ButtonY,
    LeftBumper,
    RightBumper,
    Start,
    Select,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputAxis {
    LeftStickUp,
    LeftStickDown,
    LeftStickLeft,
    LeftStickRight,
    RightStickUp,
    RightStickDown,
    RightStickLeft,
    RightStickRight,
    LeftTrigger,
    RightTrigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WaitingFor {
    Button(InputButton),
    Axis(InputAxis),
}

impl InputButton {
    fn name(&self) -> &'static str {
        match self {
            InputButton::DPadUp => "D-Pad Up",
            InputButton::DPadDown => "D-Pad Down",
            InputButton::DPadLeft => "D-Pad Left",
            InputButton::DPadRight => "D-Pad Right",
            InputButton::ButtonA => "Button A",
            InputButton::ButtonB => "Button B",
            InputButton::ButtonX => "Button X",
            InputButton::ButtonY => "Button Y",
            InputButton::LeftBumper => "Left Bumper",
            InputButton::RightBumper => "Right Bumper",
            InputButton::Start => "Start",
            InputButton::Select => "Select",
        }
    }

    fn get_key(&self, mapping: &KeyboardMapping) -> KeyCode {
        match self {
            InputButton::DPadUp => mapping.dpad_up,
            InputButton::DPadDown => mapping.dpad_down,
            InputButton::DPadLeft => mapping.dpad_left,
            InputButton::DPadRight => mapping.dpad_right,
            InputButton::ButtonA => mapping.button_a,
            InputButton::ButtonB => mapping.button_b,
            InputButton::ButtonX => mapping.button_x,
            InputButton::ButtonY => mapping.button_y,
            InputButton::LeftBumper => mapping.left_bumper,
            InputButton::RightBumper => mapping.right_bumper,
            InputButton::Start => mapping.start,
            InputButton::Select => mapping.select,
        }
    }

    fn set_key(&self, mapping: &mut KeyboardMapping, key: KeyCode) {
        match self {
            InputButton::DPadUp => mapping.dpad_up = key,
            InputButton::DPadDown => mapping.dpad_down = key,
            InputButton::DPadLeft => mapping.dpad_left = key,
            InputButton::DPadRight => mapping.dpad_right = key,
            InputButton::ButtonA => mapping.button_a = key,
            InputButton::ButtonB => mapping.button_b = key,
            InputButton::ButtonX => mapping.button_x = key,
            InputButton::ButtonY => mapping.button_y = key,
            InputButton::LeftBumper => mapping.left_bumper = key,
            InputButton::RightBumper => mapping.right_bumper = key,
            InputButton::Start => mapping.start = key,
            InputButton::Select => mapping.select = key,
        }
    }
}

impl InputAxis {
    fn name(&self) -> &'static str {
        match self {
            InputAxis::LeftStickUp => "Up",
            InputAxis::LeftStickDown => "Down",
            InputAxis::LeftStickLeft => "Left",
            InputAxis::LeftStickRight => "Right",
            InputAxis::RightStickUp => "Up",
            InputAxis::RightStickDown => "Down",
            InputAxis::RightStickLeft => "Left",
            InputAxis::RightStickRight => "Right",
            InputAxis::LeftTrigger => "Left Trigger",
            InputAxis::RightTrigger => "Right Trigger",
        }
    }

    fn get_key(&self, mapping: &KeyboardMapping) -> KeyCode {
        match self {
            InputAxis::LeftStickUp => mapping.left_stick_up,
            InputAxis::LeftStickDown => mapping.left_stick_down,
            InputAxis::LeftStickLeft => mapping.left_stick_left,
            InputAxis::LeftStickRight => mapping.left_stick_right,
            InputAxis::RightStickUp => mapping.right_stick_up,
            InputAxis::RightStickDown => mapping.right_stick_down,
            InputAxis::RightStickLeft => mapping.right_stick_left,
            InputAxis::RightStickRight => mapping.right_stick_right,
            InputAxis::LeftTrigger => mapping.left_trigger,
            InputAxis::RightTrigger => mapping.right_trigger,
        }
    }

    fn set_key(&self, mapping: &mut KeyboardMapping, key: KeyCode) {
        match self {
            InputAxis::LeftStickUp => mapping.left_stick_up = key,
            InputAxis::LeftStickDown => mapping.left_stick_down = key,
            InputAxis::LeftStickLeft => mapping.left_stick_left = key,
            InputAxis::LeftStickRight => mapping.left_stick_right = key,
            InputAxis::RightStickUp => mapping.right_stick_up = key,
            InputAxis::RightStickDown => mapping.right_stick_down = key,
            InputAxis::RightStickLeft => mapping.right_stick_left = key,
            InputAxis::RightStickRight => mapping.right_stick_right = key,
            InputAxis::LeftTrigger => mapping.left_trigger = key,
            InputAxis::RightTrigger => mapping.right_trigger = key,
        }
    }
}

/// Shared settings UI state
///
/// This can be used in both the library app (full panel mode)
/// and the standalone player (popup window mode).
pub struct SharedSettingsUi {
    /// Whether the settings panel is visible (for standalone mode)
    pub visible: bool,
    /// Currently selected tab
    selected_tab: SettingsTab,
    /// Which button or axis is currently being remapped (if any)
    waiting_for_key: Option<WaitingFor>,
    /// Temporary config for editing (not saved until "Apply" is clicked)
    temp_config: Config,
}

impl SharedSettingsUi {
    /// Create a new settings UI with the given config
    pub fn new(config: &Config) -> Self {
        Self {
            visible: false,
            selected_tab: SettingsTab::Video,
            waiting_for_key: None,
            temp_config: config.clone(),
        }
    }

    /// Toggle visibility (for standalone mode)
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Update temp config from current config
    pub fn update_temp_config(&mut self, config: &Config) {
        self.temp_config = config.clone();
    }

    /// Returns true if currently waiting for a key input
    pub fn is_waiting_for_key(&self) -> bool {
        self.waiting_for_key.is_some()
    }

    /// Handle key press for remapping.
    /// Returns true if the key was consumed (for remapping), false otherwise.
    pub fn handle_key_press(&mut self, key: KeyCode) -> bool {
        if let Some(waiting) = self.waiting_for_key {
            // ESC cancels remapping
            if key == KeyCode::Escape {
                self.waiting_for_key = None;
                return true; // Consumed the key
            }

            // Set the new key binding
            match waiting {
                WaitingFor::Button(button) => {
                    button.set_key(&mut self.temp_config.input.keyboard, key);
                }
                WaitingFor::Axis(axis) => {
                    axis.set_key(&mut self.temp_config.input.keyboard, key);
                }
            }
            self.waiting_for_key = None;
            return true; // Consumed the key
        }
        false // Key not consumed
    }

    /// Show as a popup window (for standalone player).
    /// Returns an action if the user interacted with the UI.
    pub fn show_as_window(&mut self, ctx: &Context) -> SettingsAction {
        if !self.visible {
            return SettingsAction::None;
        }

        let mut action = SettingsAction::None;

        egui::Window::new("Settings")
            .collapsible(false)
            .resizable(true)
            .default_width(400.0)
            .min_width(350.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                action = self.render_content(ui, "Close (F2)");
            });

        action
    }

    /// Show as a central panel (for library app).
    /// Returns an action if the user interacted with the UI.
    pub fn show_as_panel(&mut self, ctx: &Context, close_label: &str) -> SettingsAction {
        let mut action = SettingsAction::None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Settings");
            ui.separator();
            action = self.render_content(ui, close_label);
        });

        action
    }

    /// Render the settings content (shared between window and panel modes)
    fn render_content(&mut self, ui: &mut Ui, close_label: &str) -> SettingsAction {
        let mut action = SettingsAction::None;

        // Tab bar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.selected_tab, SettingsTab::Video, "Video");
            ui.selectable_value(&mut self.selected_tab, SettingsTab::Audio, "Audio");
            ui.selectable_value(&mut self.selected_tab, SettingsTab::Controls, "Controls");
            ui.selectable_value(&mut self.selected_tab, SettingsTab::Hotkeys, "Hotkeys");
        });

        ui.separator();
        ui.add_space(10.0);

        // Show selected tab content
        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| match self.selected_tab {
                SettingsTab::Video => {
                    action = self.render_video_tab(ui);
                }
                SettingsTab::Audio => {
                    action = self.render_audio_tab(ui);
                }
                SettingsTab::Controls => {
                    self.render_controls_tab(ui);
                }
                SettingsTab::Hotkeys => {
                    self.render_hotkeys_tab(ui);
                }
            });

        ui.add_space(20.0);
        ui.separator();

        // Bottom buttons
        ui.horizontal(|ui| {
            if ui.button(close_label).clicked() {
                self.visible = false;
                action = SettingsAction::Close;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Apply & Save").clicked() {
                    action = SettingsAction::Save(self.temp_config.clone());
                }

                if ui.button("Reset to Defaults").clicked() {
                    self.temp_config = Config::default();
                    action = SettingsAction::ResetDefaults;
                }
            });
        });

        action
    }

    fn render_video_tab(&mut self, ui: &mut Ui) -> SettingsAction {
        let mut action = SettingsAction::None;
        let video = &mut self.temp_config.video;

        ui.heading("Display Settings");
        ui.add_space(10.0);

        // Fullscreen
        let old_fullscreen = video.fullscreen;
        ui.checkbox(&mut video.fullscreen, "Fullscreen");
        ui.label("   Enable fullscreen mode");
        if video.fullscreen != old_fullscreen {
            action = SettingsAction::ToggleFullscreen(video.fullscreen);
        }
        ui.add_space(5.0);

        // VSync
        ui.checkbox(&mut video.vsync, "V-Sync");
        ui.label("   Synchronize framerate with display refresh rate");
        ui.add_space(15.0);

        // Scale Mode
        ui.heading("Scaling Mode");
        ui.add_space(5.0);

        ui.label("How to scale the game framebuffer to the window:");
        ui.add_space(5.0);

        let old_scale_mode = video.scale_mode;
        ComboBox::from_label("Scale Mode")
            .selected_text(match video.scale_mode {
                ScaleMode::Stretch => "Stretch (Fill Window)",
                ScaleMode::Fit => "Fit (Maintain Aspect Ratio)",
                ScaleMode::PixelPerfect => "Pixel Perfect (Integer Scaling)",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut video.scale_mode,
                    ScaleMode::Fit,
                    "Fit (Maintain Aspect Ratio)",
                );
                ui.selectable_value(
                    &mut video.scale_mode,
                    ScaleMode::Stretch,
                    "Stretch (Fill Window)",
                );
                ui.selectable_value(
                    &mut video.scale_mode,
                    ScaleMode::PixelPerfect,
                    "Pixel Perfect (Integer Scaling)",
                );
            });

        // Show description
        match video.scale_mode {
            ScaleMode::Fit => {
                ui.label("   Scales to fill window while maintaining aspect ratio (letterbox)");
            }
            ScaleMode::Stretch => {
                ui.label("   Stretches to fill window (may distort aspect ratio)");
            }
            ScaleMode::PixelPerfect => {
                ui.label("   Integer scaling with black bars (pixel-perfect display)");
            }
        }

        // If scale mode changed, apply it immediately for preview
        if video.scale_mode != old_scale_mode {
            action = SettingsAction::PreviewScaleMode(video.scale_mode);
        }

        action
    }

    fn render_audio_tab(&mut self, ui: &mut Ui) -> SettingsAction {
        let mut action = SettingsAction::None;
        let audio = &mut self.temp_config.audio;

        ui.heading("Volume");
        ui.add_space(10.0);

        let old_volume = audio.master_volume;
        ui.add(
            Slider::new(&mut audio.master_volume, 0.0..=1.0)
                .text("Master Volume")
                .suffix("%")
                .custom_formatter(|n, _| format!("{:.0}", n * 100.0)),
        );

        ui.add_space(5.0);
        ui.label("   Controls the overall volume level");

        if (audio.master_volume - old_volume).abs() > f32::EPSILON {
            action = SettingsAction::SetVolume(audio.master_volume);
        }

        action
    }

    fn render_controls_tab(&mut self, ui: &mut Ui) {
        ui.heading("Keyboard Controls");
        ui.add_space(5.0);

        if self.waiting_for_key.is_some() {
            ui.colored_label(egui::Color32::YELLOW, "Press any key to rebind...");
            ui.label("Press ESC to cancel");
            ui.add_space(10.0);
        } else {
            ui.label("Click a button to rebind it");
            ui.add_space(10.0);
        }

        let mapping = self.temp_config.input.keyboard.clone();

        // D-Pad
        ui.group(|ui| {
            ui.heading("D-Pad");
            ui.add_space(5.0);

            self.render_button_binding(ui, InputButton::DPadUp, &mapping);
            self.render_button_binding(ui, InputButton::DPadDown, &mapping);
            self.render_button_binding(ui, InputButton::DPadLeft, &mapping);
            self.render_button_binding(ui, InputButton::DPadRight, &mapping);
        });

        ui.add_space(10.0);

        // Face Buttons
        ui.group(|ui| {
            ui.heading("Face Buttons");
            ui.add_space(5.0);

            self.render_button_binding(ui, InputButton::ButtonA, &mapping);
            self.render_button_binding(ui, InputButton::ButtonB, &mapping);
            self.render_button_binding(ui, InputButton::ButtonX, &mapping);
            self.render_button_binding(ui, InputButton::ButtonY, &mapping);
        });

        ui.add_space(10.0);

        // Shoulder Buttons
        ui.group(|ui| {
            ui.heading("Shoulder Buttons");
            ui.add_space(5.0);

            self.render_button_binding(ui, InputButton::LeftBumper, &mapping);
            self.render_button_binding(ui, InputButton::RightBumper, &mapping);
        });

        ui.add_space(10.0);

        // System Buttons
        ui.group(|ui| {
            ui.heading("System Buttons");
            ui.add_space(5.0);

            self.render_button_binding(ui, InputButton::Start, &mapping);
            self.render_button_binding(ui, InputButton::Select, &mapping);
        });

        ui.add_space(10.0);

        // Left Stick
        ui.group(|ui| {
            ui.heading("Left Stick");
            ui.add_space(5.0);

            self.render_axis_binding(ui, InputAxis::LeftStickUp, &mapping);
            self.render_axis_binding(ui, InputAxis::LeftStickDown, &mapping);
            self.render_axis_binding(ui, InputAxis::LeftStickLeft, &mapping);
            self.render_axis_binding(ui, InputAxis::LeftStickRight, &mapping);
        });

        ui.add_space(10.0);

        // Right Stick
        ui.group(|ui| {
            ui.heading("Right Stick");
            ui.add_space(5.0);

            self.render_axis_binding(ui, InputAxis::RightStickUp, &mapping);
            self.render_axis_binding(ui, InputAxis::RightStickDown, &mapping);
            self.render_axis_binding(ui, InputAxis::RightStickLeft, &mapping);
            self.render_axis_binding(ui, InputAxis::RightStickRight, &mapping);
        });

        ui.add_space(10.0);

        // Triggers
        ui.group(|ui| {
            ui.heading("Triggers");
            ui.add_space(5.0);

            self.render_axis_binding(ui, InputAxis::LeftTrigger, &mapping);
            self.render_axis_binding(ui, InputAxis::RightTrigger, &mapping);
        });

        ui.add_space(15.0);

        // Deadzone settings
        ui.heading("Analog Settings (Gamepad)");
        ui.add_space(5.0);

        let input = &mut self.temp_config.input;
        ui.add(
            Slider::new(&mut input.stick_deadzone, 0.0..=0.5)
                .text("Stick Deadzone")
                .suffix("%")
                .custom_formatter(|n, _| format!("{:.0}", n * 100.0)),
        );
        ui.label("   Minimum stick movement to register");

        ui.add_space(5.0);

        ui.add(
            Slider::new(&mut input.trigger_deadzone, 0.0..=0.5)
                .text("Trigger Deadzone")
                .suffix("%")
                .custom_formatter(|n, _| format!("{:.0}", n * 100.0)),
        );
        ui.label("   Minimum trigger press to register");
    }

    fn render_hotkeys_tab(&self, ui: &mut Ui) {
        ui.heading("System Hotkeys");
        ui.add_space(5.0);
        ui.label("These keyboard shortcuts work anywhere in the application:");
        ui.add_space(10.0);

        // Helper to show a hotkey row
        let show_hotkey = |ui: &mut Ui, key: &str, description: &str| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(key).monospace().strong());
                ui.label("-");
                ui.label(description);
            });
        };

        ui.group(|ui| {
            ui.heading("Display");
            ui.add_space(5.0);
            show_hotkey(ui, "F11", "Toggle borderless fullscreen");
            show_hotkey(ui, "F2", "Toggle settings panel");
            show_hotkey(ui, "F3", "Toggle Runtime Stats Panel");
            show_hotkey(ui, "F4", "Toggle Debug Inspector");
        });

        ui.add_space(10.0);

        ui.group(|ui| {
            ui.heading("Playback");
            ui.add_space(5.0);
            show_hotkey(ui, "F5", "Pause/Resume game");
            show_hotkey(ui, "F6", "Step one frame (when paused)");
        });

        ui.add_space(10.0);

        ui.group(|ui| {
            ui.heading("Capture");
            ui.add_space(5.0);
            show_hotkey(ui, "F9", "Take screenshot");
            show_hotkey(ui, "F10", "Toggle GIF recording");
        });

        ui.add_space(10.0);

        ui.group(|ui| {
            ui.heading("Navigation");
            ui.add_space(5.0);
            show_hotkey(ui, "ESC", "Return to library / Exit game");
        });

        ui.add_space(15.0);

        ui.heading("Tips");
        ui.add_space(5.0);
        ui.label("Use Settings > Video to configure scaling mode");
        ui.label("Borderless fullscreen (F11) gives the best pixel-perfect scaling");
        ui.label("Game controls are configured in the Controls tab");
    }

    fn render_button_binding(
        &mut self,
        ui: &mut Ui,
        button: InputButton,
        mapping: &KeyboardMapping,
    ) {
        ui.horizontal(|ui| {
            ui.label(format!("{:16}", button.name()));

            let key = button.get_key(mapping);
            let key_name = keycode_to_display_string(key);

            let is_waiting = self.waiting_for_key == Some(WaitingFor::Button(button));
            let button_text = if is_waiting { "..." } else { &key_name };

            if ui.button(button_text).clicked() {
                self.waiting_for_key = Some(WaitingFor::Button(button));
            }
        });
    }

    fn render_axis_binding(&mut self, ui: &mut Ui, axis: InputAxis, mapping: &KeyboardMapping) {
        ui.horizontal(|ui| {
            ui.label(format!("{:16}", axis.name()));

            let key = axis.get_key(mapping);
            let key_name = keycode_to_display_string(key);

            let is_waiting = self.waiting_for_key == Some(WaitingFor::Axis(axis));
            let button_text = if is_waiting { "..." } else { &key_name };

            if ui.button(button_text).clicked() {
                self.waiting_for_key = Some(WaitingFor::Axis(axis));
            }
        });
    }
}
