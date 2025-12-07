//! UI action handling and input processing

use emberware_core::app::config;
use emberware_core::app::AppMode;
use emberware_core::console::Graphics;
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::Fullscreen,
};

use crate::input::InputManager;
use crate::library;
use crate::ui::UiAction;

use super::App;

impl App {
    /// Handle UI actions
    pub(super) fn handle_ui_action(&mut self, action: UiAction) {
        match action {
            UiAction::PlayGame(game_id) => {
                tracing::info!("Playing game: {}", game_id);
                self.last_error = None; // Clear any previous error

                // Try to start the game
                match self.start_game(&game_id) {
                    Ok(()) => {
                        self.mode = AppMode::Playing { game_id };
                    }
                    Err(e) => {
                        self.handle_runtime_error(e);
                    }
                }
            }
            UiAction::DeleteGame(game_id) => {
                tracing::info!("Deleting game: {}", game_id);
                if let Err(e) = library::delete_game(&library::ZDataDirProvider, &game_id) {
                    tracing::error!("Failed to delete game: {}", e);
                }
                self.local_games = library::get_local_games(&library::ZDataDirProvider);
                self.library_ui.selected_game = None;
            }
            UiAction::OpenBrowser => {
                const PLATFORM_URL: &str = "https://emberware.io";
                tracing::info!("Opening browser to {}", PLATFORM_URL);
                if let Err(e) = open::that(PLATFORM_URL) {
                    tracing::error!("Failed to open browser: {}", e);
                }
            }
            UiAction::OpenSettings => {
                // Toggle between Library and Settings
                self.mode = match self.mode {
                    AppMode::Settings => {
                        tracing::info!("Returning to library");
                        AppMode::Library
                    }
                    _ => {
                        tracing::info!("Opening settings");
                        // Update settings UI with current config
                        self.settings_ui.update_temp_config(&self.config);
                        AppMode::Settings
                    }
                };
            }
            UiAction::DismissError => {
                self.last_error = None;
            }
            UiAction::RefreshLibrary => {
                tracing::info!("Refreshing game library");
                self.local_games = library::get_local_games(&library::ZDataDirProvider);
                self.library_ui.selected_game = None;
            }
            UiAction::SaveSettings(new_config) => {
                tracing::info!("Saving settings...");
                self.config = new_config.clone();

                // Save to disk
                if let Err(e) = config::save(&self.config) {
                    tracing::error!("Failed to save config: {}", e);
                } else {
                    tracing::info!("Settings saved successfully");
                }

                // Apply changes to input manager (recreate with new config)
                self.input_manager = Some(InputManager::new(self.config.input.clone()));

                if let Some(graphics) = &mut self.graphics {
                    graphics.set_scale_mode(self.config.video.scale_mode);
                }

                // Apply fullscreen setting if changed
                if let Some(window) = &self.window {
                    let is_fullscreen = window.fullscreen().is_some();
                    if is_fullscreen != self.config.video.fullscreen {
                        let new_fullscreen = if self.config.video.fullscreen {
                            Some(Fullscreen::Borderless(None))
                        } else {
                            None
                        };
                        window.set_fullscreen(new_fullscreen);
                    }
                }

                // Update settings UI temp config
                self.settings_ui.update_temp_config(&self.config);

                // Return to library
                self.mode = AppMode::Library;
            }
            UiAction::SetScaleMode(scale_mode) => {
                // Preview scale mode change
                if let Some(graphics) = &mut self.graphics {
                    graphics.set_scale_mode(scale_mode);
                }
            }
        }
    }

    /// Handle window resize
    pub(super) fn handle_resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            if let Some(graphics) = &mut self.graphics {
                graphics.resize(new_size.width, new_size.height);
            }
        }
    }

    /// Toggle fullscreen mode
    pub(super) fn toggle_fullscreen(&mut self) {
        if let Some(window) = &self.window {
            let is_fullscreen = window.fullscreen().is_some();
            let new_fullscreen = if is_fullscreen {
                None
            } else {
                Some(Fullscreen::Borderless(None))
            };

            window.set_fullscreen(new_fullscreen);
            self.config.video.fullscreen = !is_fullscreen;

            // Save config
            if let Err(e) = config::save(&self.config) {
                tracing::warn!("Failed to save config: {}", e);
            }
        }
    }

    /// Handle keyboard input
    pub(super) fn handle_key_input(&mut self, key_event: KeyEvent) {
        let pressed = key_event.state == ElementState::Pressed;

        // Update input manager with key state
        if let PhysicalKey::Code(key_code) = key_event.physical_key {
            // Handle key remapping in Settings mode first
            if pressed && matches!(self.mode, AppMode::Settings) {
                // Let settings UI handle the key press for remapping
                self.settings_ui.handle_key_press(key_code);
                // Don't process other key logic when remapping
                return;
            }

            if let Some(input_manager) = &mut self.input_manager {
                input_manager.update_keyboard(key_code, pressed);
            }

            // Handle special keys
            if pressed {
                match key_code {
                    KeyCode::F3 => {
                        self.debug_overlay = !self.debug_overlay;
                    }
                    KeyCode::F11 => {
                        self.toggle_fullscreen();
                    }
                    KeyCode::Enter => {
                        // Alt+Enter for fullscreen toggle
                        // Note: Alt modifier check would go here
                        // For now, we use F11 as the primary method
                    }
                    KeyCode::Escape => {
                        // Return to library when in game or settings
                        match self.mode {
                            AppMode::Playing { .. } => {
                                tracing::info!("Exiting game via ESC");
                                self.game_session = None; // Clean up game session
                                self.mode = AppMode::Library;
                                self.local_games =
                                    library::get_local_games(&library::ZDataDirProvider);
                            }
                            AppMode::Settings => {
                                // If waiting for key binding, cancel it; otherwise return to library
                                if !self.settings_ui.handle_key_press(key_code) {
                                    self.mode = AppMode::Library;
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Update input state (call this each frame)
    pub(super) fn update_input(&mut self) {
        if let Some(input_manager) = &mut self.input_manager {
            input_manager.update();
        }
    }
}
