//! Minimal egui library UI
//!
//! Provides the game library interface using egui for rendering.
//! The UI displays locally cached games and allows users to play,
//! delete, or browse for more games online.

use crate::library::LocalGame;

/// The game library UI state and rendering.
///
/// Displays a list of locally cached games with options to play or delete them.
/// Handles game selection and returns user actions for the application to process.
pub struct LibraryUi {
    /// Currently selected game ID, if any
    pub selected_game: Option<String>,
}

impl LibraryUi {
    /// Creates a new library UI with no game selected.
    pub fn new() -> Self {
        Self {
            selected_game: None,
        }
    }

    /// Renders the library UI and returns any user action.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The egui context for rendering
    /// * `games` - List of locally cached games to display
    ///
    /// # Returns
    ///
    /// An optional [`UiAction`] if the user triggered an action this frame.
    pub fn show(&mut self, ctx: &egui::Context, games: &[LocalGame]) -> Option<UiAction> {
        let mut action = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("EMBERWARE Z");
            ui.separator();

            if games.is_empty() {
                ui.label("No games downloaded yet.");
                if ui.button("Browse Games Online").clicked() {
                    action = Some(UiAction::OpenBrowser);
                }
            } else {
                ui.heading(format!("Your Games ({})", games.len()));
                ui.add_space(10.0);

                for game in games {
                    let selected = self.selected_game.as_ref() == Some(&game.id);
                    if ui.selectable_label(selected, &game.title).clicked() {
                        self.selected_game = Some(game.id.clone());
                    }
                }

                ui.separator();

                if let Some(ref game_id) = self.selected_game {
                    if let Some(game) = games.iter().find(|g| &g.id == game_id) {
                        ui.label(format!("By: {}", game.author));
                        ui.add_space(5.0);

                        if ui.button("▶ Play").clicked() {
                            action = Some(UiAction::PlayGame(game_id.clone()));
                        }
                        if ui.button("Delete").clicked() {
                            action = Some(UiAction::DeleteGame(game_id.clone()));
                        }
                    }
                }

                ui.separator();
                if ui.button("🌐 Browse Games Online").clicked() {
                    action = Some(UiAction::OpenBrowser);
                }
            }

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("🔄 Refresh").clicked() {
                    action = Some(UiAction::RefreshLibrary);
                }
                if ui.button("Settings").clicked() {
                    action = Some(UiAction::OpenSettings);
                }
            });
        });

        action
    }
}

/// Actions the user can trigger from the library UI.
///
/// Returned by [`LibraryUi::show`] when the user interacts with the interface.
/// The application handles these actions to transition between states.
#[derive(Debug, Clone, PartialEq)]
pub enum UiAction {
    /// Start playing a game with the given ID
    PlayGame(String),
    /// Delete a cached game with the given ID
    DeleteGame(String),
    /// Open the game browser in a web browser
    OpenBrowser,
    /// Open the settings screen
    OpenSettings,
    /// Dismiss the current error message and return to library
    DismissError,
    /// Refresh the game library
    RefreshLibrary,
    /// Save settings and apply changes
    SaveSettings(emberware_core::app::config::Config),
    /// Set scale mode immediately (for preview)
    SetScaleMode(emberware_core::app::config::ScaleMode),
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // LibraryUi Tests
    // ========================================================================

    #[test]
    fn test_library_ui_new() {
        let ui = LibraryUi::new();
        assert!(ui.selected_game.is_none());
    }

    #[test]
    fn test_library_ui_select_game() {
        let mut ui = LibraryUi::new();
        ui.selected_game = Some("test-game-id".to_string());
        assert_eq!(ui.selected_game, Some("test-game-id".to_string()));
    }

    #[test]
    fn test_library_ui_deselect_game() {
        let mut ui = LibraryUi::new();
        ui.selected_game = Some("test-game-id".to_string());
        ui.selected_game = None;
        assert!(ui.selected_game.is_none());
    }

    #[test]
    fn test_library_ui_change_selection() {
        let mut ui = LibraryUi::new();
        ui.selected_game = Some("game-1".to_string());
        assert_eq!(ui.selected_game, Some("game-1".to_string()));
        ui.selected_game = Some("game-2".to_string());
        assert_eq!(ui.selected_game, Some("game-2".to_string()));
    }

    // ========================================================================
    // UiAction Tests
    // ========================================================================

    #[test]
    fn test_ui_action_play_game() {
        let action = UiAction::PlayGame("my-game".to_string());
        match action {
            UiAction::PlayGame(id) => assert_eq!(id, "my-game"),
            _ => panic!("Expected PlayGame action"),
        }
    }

    #[test]
    fn test_ui_action_delete_game() {
        let action = UiAction::DeleteGame("old-game".to_string());
        match action {
            UiAction::DeleteGame(id) => assert_eq!(id, "old-game"),
            _ => panic!("Expected DeleteGame action"),
        }
    }

    #[test]
    fn test_ui_action_open_browser() {
        let action = UiAction::OpenBrowser;
        assert!(matches!(action, UiAction::OpenBrowser));
    }

    #[test]
    fn test_ui_action_open_settings() {
        let action = UiAction::OpenSettings;
        assert!(matches!(action, UiAction::OpenSettings));
    }

    #[test]
    fn test_ui_action_dismiss_error() {
        let action = UiAction::DismissError;
        assert!(matches!(action, UiAction::DismissError));
    }

    #[test]
    fn test_ui_action_debug() {
        let action = UiAction::PlayGame("test".to_string());
        let debug_str = format!("{:?}", action);
        assert!(debug_str.contains("PlayGame"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_ui_action_clone() {
        let action = UiAction::PlayGame("clone-test".to_string());
        let cloned = action.clone();
        assert_eq!(action, cloned);
    }

    #[test]
    fn test_ui_action_equality() {
        let action1 = UiAction::PlayGame("same".to_string());
        let action2 = UiAction::PlayGame("same".to_string());
        let action3 = UiAction::PlayGame("different".to_string());

        assert_eq!(action1, action2);
        assert_ne!(action1, action3);
    }

    #[test]
    fn test_ui_action_variant_inequality() {
        let play = UiAction::PlayGame("game".to_string());
        let delete = UiAction::DeleteGame("game".to_string());
        let browser = UiAction::OpenBrowser;
        let settings = UiAction::OpenSettings;
        let dismiss = UiAction::DismissError;

        // Different variants should never be equal
        assert_ne!(play, delete);
        assert_ne!(browser, settings);
        assert_ne!(settings, dismiss);
    }

    #[test]
    fn test_ui_action_empty_string() {
        let action = UiAction::PlayGame(String::new());
        match action {
            UiAction::PlayGame(id) => assert!(id.is_empty()),
            _ => panic!("Expected PlayGame action"),
        }
    }

    #[test]
    fn test_ui_action_unicode_game_id() {
        let action = UiAction::PlayGame("游戏-🎮-test".to_string());
        match action {
            UiAction::PlayGame(id) => assert_eq!(id, "游戏-🎮-test"),
            _ => panic!("Expected PlayGame action"),
        }
    }

    #[test]
    fn test_ui_action_long_game_id() {
        let long_id = "a".repeat(1000);
        let action = UiAction::DeleteGame(long_id.clone());
        match action {
            UiAction::DeleteGame(id) => assert_eq!(id.len(), 1000),
            _ => panic!("Expected DeleteGame action"),
        }
    }
}
