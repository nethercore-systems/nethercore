//! Minimal egui library UI
//!
//! Provides the game library interface using egui for rendering.
//! The UI displays locally cached games and allows users to play,
//! delete, or browse for more games online.

mod settings;

pub use settings::SettingsUi;

use emberware_core::library::LocalGame;

/// The game library UI state and rendering.
///
/// Displays a list of locally cached games with options to play or delete them.
/// Handles game selection and returns user actions for the application to process.
pub struct LibraryUi {
    /// Currently selected game ID, if any
    pub selected_game: Option<String>,
    /// Whether to show the settings panel
    pub show_settings: bool,
}

impl LibraryUi {
    /// Creates a new library UI with no game selected.
    pub fn new() -> Self {
        Self {
            selected_game: None,
            show_settings: false,
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

                // Calculate available height for the scroll area
                // Reserve space for selection details and bottom buttons
                let available_height = ui.available_height() - 150.0;

                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .show(ui, |ui| {
                        // Use grid layout to make use of horizontal space
                        let item_width = 200.0;
                        let spacing = 10.0;
                        let available_width = ui.available_width();
                        let columns = ((available_width + spacing) / (item_width + spacing))
                            .floor()
                            .max(1.0) as usize;

                        egui::Grid::new("games_grid")
                            .num_columns(columns)
                            .spacing([spacing, spacing])
                            .show(ui, |ui| {
                                for (i, game) in games.iter().enumerate() {
                                    let selected = self.selected_game.as_ref() == Some(&game.id);
                                    if ui.selectable_label(selected, &game.title).clicked() {
                                        self.selected_game = Some(game.id.clone());
                                    }

                                    // End row after reaching column count
                                    if (i + 1) % columns == 0 {
                                        ui.end_row();
                                    }
                                }
                            });
                    });

                ui.separator();

                if let Some(ref game_id) = self.selected_game
                    && let Some(game) = games.iter().find(|g| &g.id == game_id)
                {
                    ui.label(format!("By: {}", game.author));
                    ui.add_space(5.0);

                    if ui.button("Play").clicked() {
                        action = Some(UiAction::PlayGame(game_id.clone()));
                    }
                    if ui.button("Delete").clicked() {
                        action = Some(UiAction::DeleteGame(game_id.clone()));
                    }
                }

                ui.separator();
                if ui.button("Browse Games Online").clicked() {
                    action = Some(UiAction::OpenBrowser);
                }
            }

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Import ROM").clicked() {
                    action = Some(UiAction::ImportRom);
                }
                if ui.button("Open Game").clicked() {
                    action = Some(UiAction::OpenGame);
                }
                if ui.button("Refresh").clicked() {
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
    /// Import a ROM file from disk
    ImportRom,
    /// Open and run a game file directly (without importing to library)
    OpenGame,
}
