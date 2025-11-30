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
            if ui.button("Settings").clicked() {
                action = Some(UiAction::OpenSettings);
            }
        });

        action
    }
}

/// Actions the user can trigger from the library UI.
///
/// Returned by [`LibraryUi::show`] when the user interacts with the interface.
/// The application handles these actions to transition between states.
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
}
