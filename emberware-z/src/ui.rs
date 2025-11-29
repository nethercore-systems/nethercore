//! Minimal egui library UI

use crate::library::LocalGame;

pub struct LibraryUi {
    pub selected_game: Option<String>,
}

impl LibraryUi {
    pub fn new() -> Self {
        Self {
            selected_game: None,
        }
    }

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

pub enum UiAction {
    PlayGame(String),
    DeleteGame(String),
    OpenBrowser,
    OpenSettings,
    DismissError,
}
