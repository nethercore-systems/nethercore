//! Lobby UI rendering
//!
//! Provides the multiplayer lobby interface for host and guest players.

use super::UiAction;
use crate::app::lobby::{LobbyPhase, LobbySession};
use eframe::egui;
use nethercore_core::net::nchs::NchsRole;

/// Lobby UI component
pub struct LobbyUi;

impl LobbyUi {
    /// Render the lobby UI based on current phase
    pub fn show(lobby: &mut LobbySession, ctx: &egui::Context) -> Option<UiAction> {
        match lobby.phase {
            LobbyPhase::Connecting => Self::show_connecting(lobby, ctx),
            LobbyPhase::Listening => Self::show_lobby(lobby, ctx),
            LobbyPhase::InLobby => Self::show_lobby(lobby, ctx),
            LobbyPhase::Starting => Self::show_starting(lobby, ctx),
            LobbyPhase::Failed => Self::show_error(lobby, ctx),
            LobbyPhase::Ready => None, // Handled in App::update to spawn player
        }
    }

    /// Connecting screen (guest)
    fn show_connecting(_lobby: &LobbySession, ctx: &egui::Context) -> Option<UiAction> {
        let mut action = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);

                ui.heading("CONNECTING...");
                ui.add_space(20.0);

                // Simple animated dots
                let dots = match (ctx.input(|i| i.time) * 2.0) as i32 % 4 {
                    0 => ".",
                    1 => "..",
                    2 => "...",
                    _ => "",
                };
                ui.label(format!("Connecting to host{}", dots));

                ui.add_space(40.0);

                if ui.button("Cancel").clicked() {
                    action = Some(UiAction::LeaveLobby);
                }
            });
        });

        // Keep requesting repaints for animation
        ctx.request_repaint();

        action
    }

    /// Main lobby screen (both host and guest)
    fn show_lobby(lobby: &mut LobbySession, ctx: &egui::Context) -> Option<UiAction> {
        let mut action = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            // Header
            ui.heading("MULTIPLAYER LOBBY");
            ui.add_space(10.0);

            ui.label(format!("Game: {}", lobby.game.title));

            // Host address display (host only)
            if lobby.is_host() {
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    let port = lobby.port();
                    if let Some(ip) = lobby.local_ips.first() {
                        let addr = format!("{}:{}", ip, port);
                        ui.label("Your Address:");
                        ui.monospace(&addr);
                        if ui.small_button("Copy").clicked() {
                            action = Some(UiAction::CopyAddress(addr));
                        }
                    } else {
                        ui.label(format!("Port: {}", port));
                    }
                });
                ui.small("Share this with friends to join!");
            }

            ui.separator();
            ui.add_space(10.0);

            // Player list
            if let Some(lobby_state) = lobby.lobby() {
                let active_count = lobby_state.players.iter().filter(|p| p.active).count();
                ui.heading(format!(
                    "PLAYERS ({}/{})",
                    active_count, lobby_state.max_players
                ));
                ui.add_space(10.0);

                ui.group(|ui| {
                    for slot in &lobby_state.players {
                        ui.horizontal(|ui| {
                            if slot.active {
                                if let Some(ref info) = slot.info {
                                    // Color indicator
                                    let color = egui::Color32::from_rgb(
                                        info.color[0],
                                        info.color[1],
                                        info.color[2],
                                    );
                                    ui.colored_label(color, "●");

                                    // Player name
                                    ui.label(&info.name);

                                    // Host badge
                                    if slot.handle == 0 {
                                        ui.label("(Host)");
                                    }

                                    // Ready status
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if slot.ready {
                                                ui.colored_label(egui::Color32::GREEN, "✓ Ready");
                                            } else {
                                                ui.colored_label(
                                                    egui::Color32::GRAY,
                                                    "○ Not Ready",
                                                );
                                            }
                                        },
                                    );
                                }
                            } else {
                                ui.colored_label(egui::Color32::DARK_GRAY, "[ ]");
                                ui.colored_label(egui::Color32::GRAY, "Waiting for player...");
                            }
                        });
                    }
                });
            } else {
                ui.label("Waiting for lobby update...");
            }

            ui.add_space(20.0);

            // Controls
            match lobby.role() {
                NchsRole::Host => {
                    // Host: Start button
                    let can_start = lobby.can_start();
                    let button = egui::Button::new("START GAME");

                    if ui.add_enabled(can_start, button).clicked() {
                        action = Some(UiAction::StartGame);
                    }

                    if !can_start {
                        if lobby.player_count() < 2 {
                            ui.small("Waiting for more players to join...");
                        } else if !lobby.all_ready() {
                            ui.small("Waiting for all players to be ready...");
                        }
                    }
                }
                NchsRole::Guest => {
                    // Guest: Ready checkbox
                    let mut ready = lobby.local_ready;
                    if ui.checkbox(&mut ready, "I'm Ready").changed() {
                        action = Some(UiAction::ToggleReady);
                    }
                    ui.small("Waiting for host to start...");
                }
            }

            ui.separator();

            // Status bar
            ui.horizontal(|ui| {
                // Connection status
                ui.colored_label(egui::Color32::GREEN, "●");
                ui.label("Connected");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Leave").clicked() {
                        action = Some(UiAction::LeaveLobby);
                    }
                });
            });
        });

        action
    }

    /// Starting screen (both)
    fn show_starting(_lobby: &LobbySession, ctx: &egui::Context) -> Option<UiAction> {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);

                ui.heading("STARTING SESSION...");
                ui.add_space(20.0);

                // Simple animated dots
                let dots = match (ctx.input(|i| i.time) * 2.0) as i32 % 4 {
                    0 => ".",
                    1 => "..",
                    2 => "...",
                    _ => "",
                };
                ui.label(format!("Connecting to other players{}", dots));
            });
        });

        // Keep requesting repaints for animation
        ctx.request_repaint();

        None
    }

    /// Error screen
    fn show_error(lobby: &LobbySession, ctx: &egui::Context) -> Option<UiAction> {
        let mut action = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);

                ui.colored_label(egui::Color32::RED, "CONNECTION FAILED");
                ui.add_space(20.0);

                if let Some(ref error) = lobby.error {
                    ui.label(error);
                } else {
                    ui.label("An unknown error occurred.");
                }

                ui.add_space(40.0);

                if ui.button("Back").clicked() {
                    action = Some(UiAction::LeaveLobby);
                }
            });
        });

        action
    }
}
