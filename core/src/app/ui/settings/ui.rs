//! Main settings UI implementation with rendering logic

use egui::{ComboBox, Context, Slider, Ui};
use winit::keyboard::KeyCode;

use crate::app::config::{Config, ScaleMode};
use crate::app::input::KeyboardMapping;

use super::input_mapping::{InputAxis, InputButton, WaitingFor};
use super::types::{SettingsAction, SettingsTab};
use crate::app::ui::keycode_to_display_string;

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
    /// Currently selected player for keyboard configuration (0-3)
    selected_player: usize,
}

impl SharedSettingsUi {
    /// Create a new settings UI with the given config
    pub fn new(config: &Config) -> Self {
        Self {
            visible: false,
            selected_tab: SettingsTab::Video,
            waiting_for_key: None,
            temp_config: config.clone(),
            selected_player: 0,
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

            // Set the new key binding for the specified player
            match waiting {
                WaitingFor::Button(player, button) => {
                    if let Some(mapping) = self.temp_config.input.keyboards.get_mut(player) {
                        button.set_key(mapping, key);
                    }
                }
                WaitingFor::Axis(player, axis) => {
                    if let Some(mapping) = self.temp_config.input.keyboards.get_mut(player) {
                        axis.set_key(mapping, key);
                    }
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
                    action = SettingsAction::Save(Box::new(self.temp_config.clone()));
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

        // Player selector tabs
        ui.horizontal(|ui| {
            ui.label("Player:");
            for i in 0..4 {
                let label = format!("P{}", i + 1);
                let is_enabled = self.temp_config.input.keyboards.is_enabled(i);
                let style = if is_enabled {
                    egui::RichText::new(&label).strong()
                } else {
                    egui::RichText::new(&label).weak()
                };
                if ui
                    .selectable_label(self.selected_player == i, style)
                    .clicked()
                {
                    self.selected_player = i;
                    // Clear any pending rebind when switching players
                    self.waiting_for_key = None;
                }
            }
        });

        ui.add_space(10.0);

        // Enable/disable checkbox for selected player
        let player = self.selected_player;
        let is_enabled = self.temp_config.input.keyboards.is_enabled(player);

        let mut enabled = is_enabled;
        if ui
            .checkbox(&mut enabled, "Enable keyboard for this player")
            .changed()
        {
            if enabled && !is_enabled {
                // Enable with default mapping
                self.temp_config
                    .input
                    .keyboards
                    .set(player, Some(KeyboardMapping::default()));
            } else if !enabled && is_enabled {
                // Disable
                self.temp_config.input.keyboards.set(player, None);
                // Clear any pending rebind
                self.waiting_for_key = None;
            }
        }

        ui.add_space(5.0);

        // Show warning about key conflicts
        let conflicts = self.find_key_conflicts();
        if !conflicts.is_empty() {
            ui.colored_label(
                egui::Color32::YELLOW,
                format!(
                    "Warning: {} key conflict(s) between players",
                    conflicts.len()
                ),
            );
            ui.add_space(5.0);
        }

        if !is_enabled {
            ui.colored_label(
                egui::Color32::GRAY,
                "Keyboard input is disabled for this player.\nCheck the box above to enable.",
            );
            return;
        }

        if self.waiting_for_key.is_some() {
            ui.colored_label(egui::Color32::YELLOW, "Press any key to rebind...");
            ui.label("Press ESC to cancel");
            ui.add_space(10.0);
        } else {
            ui.label("Click a button to rebind it");
            ui.add_space(10.0);
        }

        // Get the mapping for the selected player
        let mapping = self
            .temp_config
            .input
            .keyboards
            .get(player)
            .cloned()
            .unwrap_or_default();

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
        let player = self.selected_player;
        ui.horizontal(|ui| {
            ui.label(format!("{:16}", button.name()));

            let key = button.get_key(mapping);
            let key_name = keycode_to_display_string(key);

            let is_waiting = self.waiting_for_key == Some(WaitingFor::Button(player, button));
            let button_text = if is_waiting { "..." } else { &key_name };

            if ui.button(button_text).clicked() {
                self.waiting_for_key = Some(WaitingFor::Button(player, button));
            }
        });
    }

    fn render_axis_binding(&mut self, ui: &mut Ui, axis: InputAxis, mapping: &KeyboardMapping) {
        let player = self.selected_player;
        ui.horizontal(|ui| {
            ui.label(format!("{:16}", axis.name()));

            let key = axis.get_key(mapping);
            let key_name = keycode_to_display_string(key);

            let is_waiting = self.waiting_for_key == Some(WaitingFor::Axis(player, axis));
            let button_text = if is_waiting { "..." } else { &key_name };

            if ui.button(button_text).clicked() {
                self.waiting_for_key = Some(WaitingFor::Axis(player, axis));
            }
        });
    }

    /// Find key conflicts between players (same key bound to different players)
    fn find_key_conflicts(&self) -> Vec<(usize, usize, KeyCode)> {
        use hashbrown::HashMap;
        let mut conflicts = Vec::new();
        let mut key_to_player: HashMap<KeyCode, usize> = HashMap::new();

        for (player, mapping) in self.temp_config.input.keyboards.iter_enabled() {
            for key in mapping.all_keys() {
                if let Some(&other_player) = key_to_player.get(&key) {
                    conflicts.push((other_player, player, key));
                } else {
                    key_to_player.insert(key, player);
                }
            }
        }

        conflicts
    }
}
