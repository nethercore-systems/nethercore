//! Library application state and main loop
//!
//! The library is a simple launcher UI that:
//! - Shows installed games
//! - Launches games as separate player processes
//! - Does NOT run games in-process

mod init;
pub mod lobby;

pub use init::AppError;
pub use lobby::{LobbyPhase, LobbySession};

use eframe::egui;

use crate::registry::{ConnectionMode, PlayerOptions};
use crate::ui::{LibraryUi, LobbyUi, MultiplayerDialog, UiAction};
use nethercore_core::app::config::Config;
use nethercore_core::library::{LocalGame, RomLoaderRegistry};
use nethercore_core::net::nchs::{NchsConfig, NchsSession, NetworkConfig, PlayerInfo};
use nethercore_shared::{MAX_ROM_BYTES, read_file_with_limit};
use zx_common::ZXRom;

/// Library application state
pub struct App {
    /// User configuration
    config: Config,
    /// Library UI state
    library_ui: LibraryUi,
    /// Settings UI state
    settings_ui: crate::ui::SettingsUi,
    /// Multiplayer dialog state
    multiplayer_dialog: Option<MultiplayerDialog>,
    /// Active lobby session (replaces multiplayer_dialog during lobby phase)
    lobby: Option<LobbySession>,
    /// Cached local games list
    local_games: Vec<LocalGame>,
    /// ROM loader registry
    rom_loader_registry: RomLoaderRegistry,
    /// Last error message (for displaying in UI)
    last_error: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Create a new library application
    pub fn new() -> Self {
        let config = nethercore_core::app::config::load();
        let rom_loader_registry = crate::registry::create_rom_loader_registry();
        let local_games = nethercore_core::library::get_local_games_with_loaders(
            &nethercore_core::library::DefaultDataDirProvider,
            &rom_loader_registry,
        );

        Self {
            settings_ui: crate::ui::SettingsUi::new(&config),
            config,
            library_ui: LibraryUi::new(),
            multiplayer_dialog: None,
            lobby: None,
            local_games,
            rom_loader_registry,
            last_error: None,
        }
    }

    /// Refresh the local games list
    fn refresh_games(&mut self) {
        self.local_games = nethercore_core::library::get_local_games_with_loaders(
            &nethercore_core::library::DefaultDataDirProvider,
            &self.rom_loader_registry,
        );
    }

    /// Handle UI actions
    fn handle_ui_action(&mut self, action: UiAction, ctx: &egui::Context) {
        match action {
            UiAction::PlayGame(game_id) => {
                tracing::info!("Launching game: {}", game_id);

                // Find the game and launch it
                if let Some(game) = self.local_games.iter().find(|g| g.id == game_id) {
                    match crate::registry::launch_game_by_id(game) {
                        Ok(()) => {
                            tracing::info!("Player process spawned for: {}", game_id);
                            self.last_error = None;
                        }
                        Err(e) => {
                            tracing::error!("Failed to launch game: {}", e);
                            self.last_error = Some(format!("Failed to launch: {}", e));
                        }
                    }
                } else {
                    self.last_error = Some(format!("Game not found: {}", game_id));
                }
            }
            UiAction::DeleteGame(game_id) => {
                tracing::info!("Deleting game: {}", game_id);
                if let Err(e) = nethercore_core::library::delete_game(
                    &nethercore_core::library::DefaultDataDirProvider,
                    &game_id,
                ) {
                    tracing::error!("Failed to delete game: {}", e);
                }
                self.refresh_games();
                self.library_ui.selected_game = None;
            }
            UiAction::OpenBrowser => {
                const PLATFORM_URL: &str = "https://nethercore.systems";
                tracing::info!("Opening browser to {}", PLATFORM_URL);
                if let Err(e) = open::that(PLATFORM_URL) {
                    tracing::error!("Failed to open browser: {}", e);
                }
            }
            UiAction::OpenSettings => {
                // Toggle settings panel in library UI
                self.library_ui.show_settings = !self.library_ui.show_settings;
                if self.library_ui.show_settings {
                    self.settings_ui.update_temp_config(&self.config);
                }
            }
            UiAction::DismissError => {
                self.last_error = None;
            }
            UiAction::RefreshLibrary => {
                tracing::info!("Refreshing game library");
                self.refresh_games();
                self.library_ui.selected_game = None;
            }
            UiAction::OpenGame => {
                tracing::info!("Opening file picker to run game directly");

                let file_handle = rfd::FileDialog::new()
                    .add_filter("Game Files", &["nczx", "wasm"])
                    .add_filter("Nethercore ROM", &["nczx"])
                    .add_filter("WebAssembly", &["wasm"])
                    .set_title("Open Game File")
                    .pick_file();

                if let Some(path) = file_handle {
                    tracing::info!("Launching game from: {}", path.display());
                    match crate::registry::launch_game_from_path(&path) {
                        Ok(()) => {
                            tracing::info!("Player process spawned for: {}", path.display());
                            self.last_error = None;
                        }
                        Err(e) => {
                            tracing::error!("Failed to launch game: {}", e);
                            self.last_error = Some(format!("Failed to launch: {}", e));
                        }
                    }
                }
            }
            UiAction::ImportRom => {
                tracing::info!("Opening file picker for ROM import");

                let file_handle = rfd::FileDialog::new()
                    .add_filter("Nethercore ROM", &["nczx"])
                    .set_title("Import ROM File")
                    .pick_file();

                if let Some(source_path) = file_handle {
                    tracing::info!("Selected ROM file: {}", source_path.display());

                    if let Some(data_dir) = nethercore_core::app::config::data_dir() {
                        let games_dir = data_dir.join("games");

                        if let Err(e) = std::fs::create_dir_all(&games_dir) {
                            tracing::error!("Failed to create games directory: {}", e);
                            self.last_error =
                                Some(format!("Failed to create games directory: {}", e));
                            return;
                        }

                        if let Some(filename) = source_path.file_name() {
                            let dest_path = games_dir.join(filename);

                            match std::fs::copy(&source_path, &dest_path) {
                                Ok(_) => {
                                    tracing::info!(
                                        "ROM imported successfully to: {}",
                                        dest_path.display()
                                    );
                                    self.refresh_games();
                                }
                                Err(e) => {
                                    tracing::error!("Failed to copy ROM file: {}", e);
                                    self.last_error = Some(format!("Failed to import ROM: {}", e));
                                }
                            }
                        } else {
                            self.last_error = Some("Invalid file path".to_string());
                        }
                    } else {
                        self.last_error = Some("Could not determine data directory".to_string());
                    }
                }
            }
            UiAction::SaveSettings(new_config) => {
                tracing::info!("Saving settings...");
                self.config = new_config.clone();

                if let Err(e) = nethercore_core::app::config::save(&self.config) {
                    tracing::error!("Failed to save config: {}", e);
                } else {
                    tracing::info!("Settings saved successfully");
                }

                // Apply fullscreen setting
                let is_fullscreen = ctx.input(|i| i.viewport().fullscreen).unwrap_or(false);
                if is_fullscreen != self.config.video.fullscreen {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                        self.config.video.fullscreen,
                    ));
                }

                self.settings_ui.update_temp_config(&self.config);
                self.library_ui.show_settings = false;
            }
            UiAction::SetScaleMode(_scale_mode) => {
                // Scale mode only affects game rendering, which happens in player process
                // This is a no-op in the library
            }
            UiAction::ShowMultiplayerDialog(game_id) => {
                tracing::info!("Opening multiplayer dialog for: {}", game_id);
                self.multiplayer_dialog = Some(MultiplayerDialog::new(game_id));
            }
            UiAction::HostGame {
                game_id,
                port,
                players,
            } => {
                tracing::info!(
                    "Hosting game {} on port {} with {} players",
                    game_id,
                    port,
                    players
                );

                if let Some(game) = self.local_games.iter().find(|g| g.id == game_id) {
                    let options = PlayerOptions {
                        players: Some(players),
                        connection: Some(ConnectionMode::Host { port }),
                        ..Default::default()
                    };

                    match crate::registry::launch_game_by_id_with_options(game, &options) {
                        Ok(()) => {
                            tracing::info!("Player process spawned in host mode for: {}", game_id);
                            self.last_error = None;
                            self.multiplayer_dialog = None;
                        }
                        Err(e) => {
                            tracing::error!("Failed to launch game: {}", e);
                            self.last_error = Some(format!("Failed to launch: {}", e));
                        }
                    }
                } else {
                    self.last_error = Some(format!("Game not found: {}", game_id));
                }
            }
            UiAction::JoinGame {
                game_id,
                host_ip,
                port,
            } => {
                tracing::info!("Joining game {} at {}:{}", game_id, host_ip, port);

                if let Some(game) = self.local_games.iter().find(|g| g.id == game_id) {
                    let options = PlayerOptions {
                        connection: Some(ConnectionMode::Join { host_ip, port }),
                        ..Default::default()
                    };

                    match crate::registry::launch_game_by_id_with_options(game, &options) {
                        Ok(()) => {
                            tracing::info!("Player process spawned in join mode for: {}", game_id);
                            self.last_error = None;
                            self.multiplayer_dialog = None;
                        }
                        Err(e) => {
                            tracing::error!("Failed to launch game: {}", e);
                            self.last_error = Some(format!("Failed to launch: {}", e));
                        }
                    }
                } else {
                    self.last_error = Some(format!("Game not found: {}", game_id));
                }
            }
            UiAction::CancelMultiplayer => {
                tracing::info!("Cancelling multiplayer dialog");
                self.multiplayer_dialog = None;
            }
            UiAction::StartHostLobby {
                game_id,
                port,
                max_players,
            } => {
                tracing::info!(
                    "Starting host lobby for {} on port {} with max {} players",
                    game_id,
                    port,
                    max_players
                );

                if let Some(game) = self.local_games.iter().find(|g| g.id == game_id) {
                    match self.create_host_session(game.clone(), port, max_players) {
                        Ok(session) => {
                            self.lobby = Some(session);
                            self.multiplayer_dialog = None;
                        }
                        Err(e) => {
                            tracing::error!("Failed to create host session: {}", e);
                            self.last_error = Some(format!("Failed to host: {}", e));
                        }
                    }
                } else {
                    self.last_error = Some(format!("Game not found: {}", game_id));
                }
            }
            UiAction::StartJoinLobby { game_id, host_addr } => {
                tracing::info!("Starting join lobby for {} at {}", game_id, host_addr);

                if let Some(game) = self.local_games.iter().find(|g| g.id == game_id) {
                    match self.create_guest_session(game.clone(), &host_addr) {
                        Ok(session) => {
                            self.lobby = Some(session);
                            self.multiplayer_dialog = None;
                        }
                        Err(e) => {
                            tracing::error!("Failed to create guest session: {}", e);
                            self.last_error = Some(format!("Failed to join: {}", e));
                        }
                    }
                } else {
                    self.last_error = Some(format!("Game not found: {}", game_id));
                }
            }
            UiAction::ToggleReady => {
                if let Some(ref mut lobby) = self.lobby {
                    let new_ready = !lobby.local_ready;
                    tracing::info!("Toggling ready state to {}", new_ready);
                    if let Err(e) = lobby.set_ready(new_ready) {
                        self.last_error = Some(format!("Failed to set ready: {}", e));
                    }
                }
            }
            UiAction::StartGame => {
                if let Some(ref mut lobby) = self.lobby {
                    tracing::info!("Host starting game");
                    if let Err(e) = lobby.start() {
                        self.last_error = Some(format!("Failed to start game: {}", e));
                    }
                }
            }
            UiAction::LeaveLobby => {
                tracing::info!("Leaving lobby");
                self.lobby = None;
            }
            UiAction::CopyAddress(addr) => {
                tracing::info!("Copying address to clipboard: {}", addr);
                ctx.copy_text(addr);
            }
        }
    }

    /// Create a host session for the given game
    fn create_host_session(
        &self,
        game: LocalGame,
        port: u16,
        max_players: u8,
    ) -> anyhow::Result<LobbySession> {
        // Load ROM to get netplay metadata
        let rom_bytes = read_file_with_limit(&game.rom_path, MAX_ROM_BYTES)?;
        let rom = ZXRom::from_bytes(&rom_bytes)?;
        let mut netplay = rom.metadata.netplay;

        // Override max_players with UI selection (capped at ROM max)
        netplay.max_players = max_players.min(netplay.max_players);

        // Create NCHS config
        let config = NchsConfig {
            netplay,
            player_info: PlayerInfo {
                name: "Host".to_string(),
                color: [100, 149, 237], // Cornflower blue
                avatar_id: 0,
            },
            network_config: NetworkConfig::default(),
            save_config: None,
        };

        // Create host session
        let session = NchsSession::host(port, config)?;

        Ok(LobbySession::new_host(session, game))
    }

    /// Create a guest session to join the given host
    fn create_guest_session(
        &self,
        game: LocalGame,
        host_addr: &str,
    ) -> anyhow::Result<LobbySession> {
        // Load ROM to get netplay metadata
        let rom_bytes = read_file_with_limit(&game.rom_path, MAX_ROM_BYTES)?;
        let rom = ZXRom::from_bytes(&rom_bytes)?;
        let netplay = rom.metadata.netplay;

        // Create NCHS config
        let config = NchsConfig {
            netplay,
            player_info: PlayerInfo {
                name: "Guest".to_string(),
                color: [255, 165, 0], // Orange
                avatar_id: 0,
            },
            network_config: NetworkConfig::default(),
            save_config: None,
        };

        // Create guest session
        let session = NchsSession::join(host_addr, config)?;

        Ok(LobbySession::new_guest(session, game))
    }

    /// Spawn the player process when lobby is ready
    fn spawn_player_for_lobby(&mut self) {
        if let Some(lobby) = self.lobby.take() {
            if lobby.phase != LobbyPhase::Ready {
                tracing::warn!("Attempted to spawn player but lobby not ready");
                self.lobby = Some(lobby);
                return;
            }

            // Get session config and local player handle
            let mut session_config = match lobby.session.session_config() {
                Some(config) => config.clone(),
                None => {
                    tracing::error!("Lobby ready but no session config available");
                    self.last_error = Some("Session config not available".to_string());
                    return;
                }
            };

            // Set the local player handle for this process
            let local_handle = lobby.session.local_handle().unwrap_or(0);
            session_config.local_player_handle = local_handle;

            // Serialize session config to temp file
            let session_file =
                std::env::temp_dir().join(format!("nchs-session-{}.bin", std::process::id()));

            let encoded = bitcode::encode(&session_config);
            if let Err(e) = std::fs::write(&session_file, &encoded) {
                tracing::error!("Failed to write session file: {}", e);
                self.last_error = Some(format!("Failed to write session: {}", e));
                return;
            }

            // Create player options with session file
            let options = PlayerOptions {
                connection: Some(ConnectionMode::Session {
                    file: session_file.clone(),
                }),
                ..Default::default()
            };

            // Launch player
            match crate::registry::launch_game_by_id_with_options(&lobby.game, &options) {
                Ok(()) => {
                    tracing::info!("Player process spawned with session for: {}", lobby.game.id);
                    self.last_error = None;
                }
                Err(e) => {
                    tracing::error!("Failed to launch game: {}", e);
                    self.last_error = Some(format!("Failed to launch: {}", e));
                    // Clean up session file
                    let _ = std::fs::remove_file(&session_file);
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle F11 for fullscreen toggle
        if ctx.input(|i| i.key_pressed(egui::Key::F11)) {
            let is_fullscreen = ctx.input(|i| i.viewport().fullscreen).unwrap_or(false);
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!is_fullscreen));
            self.config.video.fullscreen = !is_fullscreen;
            let _ = nethercore_core::app::config::save(&self.config);
        }

        let mut ui_action = None;

        // Poll lobby session if active
        if let Some(ref mut lobby) = self.lobby {
            ctx.request_repaint(); // Keep polling
            lobby.poll();

            // Check if lobby is ready to spawn player
            if lobby.phase == LobbyPhase::Ready {
                // Will spawn player after UI rendering
            }
        }

        // Show error panel if there's an error
        if let Some(ref error) = self.last_error {
            egui::TopBottomPanel::top("error_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                    if ui.button("Dismiss").clicked() {
                        ui_action = Some(UiAction::DismissError);
                    }
                });
            });
        }

        // Show lobby UI if active, otherwise show library/settings
        if let Some(ref mut lobby) = self.lobby {
            if let Some(action) = LobbyUi::show(lobby, ctx) {
                ui_action = Some(action);
            }
        } else if self.library_ui.show_settings {
            if let Some(action) = self.settings_ui.show(ctx) {
                ui_action = Some(action);
            }
        } else if let Some(action) = self.library_ui.show(ctx, &self.local_games) {
            ui_action = Some(action);
        }

        // Show multiplayer dialog if open (only when not in lobby)
        if self.lobby.is_none()
            && let Some(ref mut dialog) = self.multiplayer_dialog
            && let Some(action) = dialog.show(ctx)
        {
            ui_action = Some(action);
        }

        // Handle UI action
        if let Some(action) = ui_action {
            self.handle_ui_action(action, ctx);
        }

        // Spawn player when lobby is ready (after UI rendering)
        if self
            .lobby
            .as_ref()
            .is_some_and(|l| l.phase == LobbyPhase::Ready)
        {
            self.spawn_player_for_lobby();
        }
    }
}

/// Run the library application
pub fn run() -> Result<(), AppError> {
    tracing::info!("Starting Nethercore Library");

    let config = nethercore_core::app::config::load();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Nethercore Library")
            .with_inner_size([960.0, 540.0])
            .with_fullscreen(config.video.fullscreen),
        ..Default::default()
    };

    eframe::run_native(
        "Nethercore Library",
        native_options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
    .map_err(|e| AppError::EventLoop(format!("eframe error: {}", e)))?;

    Ok(())
}
