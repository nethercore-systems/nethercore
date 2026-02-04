//! Input handling for keyboard and gamepad events

use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Fullscreen;

use crate::capture::CaptureSupport;
use crate::console::Console;

use super::StandaloneApp;
use super::types::{RomLoader, StandaloneGraphicsSupport};

impl<C, L> StandaloneApp<C, L>
where
    C: Console + Clone,
    C::Graphics: StandaloneGraphicsSupport,
    L: RomLoader<Console = C>,
{
    /// Handles keyboard input events, including hotkeys and game input
    pub(super) fn handle_key_input(&mut self, event: KeyEvent) {
        // First, let settings UI consume key if waiting for rebind
        if event.state == ElementState::Pressed
            && let PhysicalKey::Code(key_code) = event.physical_key
            && self.settings_ui.is_waiting_for_key()
            && self.settings_ui.handle_key_press(key_code)
        {
            self.needs_redraw = true;
            return; // Key was consumed by settings UI
        }

        if event.state == ElementState::Pressed {
            match event.physical_key {
                PhysicalKey::Code(KeyCode::Escape) => {
                    self.should_exit = true;
                }
                PhysicalKey::Code(KeyCode::F2) => {
                    self.settings_ui.toggle();
                    self.needs_redraw = true;
                }
                PhysicalKey::Code(KeyCode::F3) => {
                    self.debug_overlay = !self.debug_overlay;
                    self.needs_redraw = true;
                }
                PhysicalKey::Code(KeyCode::F4) => {
                    self.debug_panel.toggle();
                    self.needs_redraw = true;
                }
                PhysicalKey::Code(KeyCode::F5) => {
                    self.frame_controller.toggle_pause();
                    self.needs_redraw = true;
                }
                PhysicalKey::Code(KeyCode::F6) => {
                    self.frame_controller.request_step();
                    self.needs_redraw = true;
                }
                PhysicalKey::Code(KeyCode::Backquote) => {
                    self.console_debug_panel_visible = !self.console_debug_panel_visible;
                    self.needs_redraw = true;
                }
                PhysicalKey::Code(KeyCode::F11) => {
                    if let Some(window) = &self.window {
                        let is_fullscreen = window.fullscreen().is_some();
                        if is_fullscreen {
                            window.set_fullscreen(None);
                        } else {
                            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                        }
                    }
                }
                PhysicalKey::Code(KeyCode::F12) => {
                    self.network_overlay_visible = !self.network_overlay_visible;
                    self.needs_redraw = true;
                }
                PhysicalKey::Code(key) if key == self.screenshot_key => {
                    self.capture.request_screenshot();
                    tracing::info!("Screenshot requested");
                    self.needs_redraw = true;
                }
                PhysicalKey::Code(key) if key == self.gif_toggle_key => {
                    if let Some(runner) = &self.runner {
                        let (w, h) = runner.graphics().render_target_dimensions();
                        self.capture.toggle_recording(w, h);
                        if self.capture.is_recording() {
                            tracing::info!("GIF recording started");
                        } else {
                            tracing::info!("GIF recording stopped, saving...");
                        }
                        self.needs_redraw = true;
                    }
                }
                _ => {}
            }
        }

        let pressed = event.state == ElementState::Pressed;
        if let PhysicalKey::Code(key_code) = event.physical_key {
            self.input_manager.update_keyboard(key_code, pressed);
        }
    }
}
