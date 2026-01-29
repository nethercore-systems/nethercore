//! Window and game initialization logic

use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use ggrs::PlayerType;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Fullscreen, Window};

use crate::capture::CaptureSupport;
use crate::console::{Audio, Console};
use crate::rollback::{ConnectionMode, LocalSocket, RollbackSession, SessionConfig};
use crate::runner::ConsoleRunner;

use super::StandaloneApp;
use super::error_ui::{WaitingForPeer, sanitize_game_id};
use super::types::{RomLoader, StandaloneGraphicsSupport};

impl<C, L> StandaloneApp<C, L>
where
    C: Console + Clone,
    C::Graphics: StandaloneGraphicsSupport,
    L: RomLoader<Console = C>,
{
    /// Called when the window is created, initializes graphics and loads the game
    pub(super) fn on_window_created_impl(
        &mut self,
        window: Arc<Window>,
        _event_loop: &ActiveEventLoop,
    ) -> Result<()> {
        let startup_started = Instant::now();

        if self.config.fullscreen {
            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
        }

        let started = Instant::now();
        let rom = L::load_rom(&self.config.rom_path)?;
        tracing::info!("ROM load took {:?}", started.elapsed());
        self.loaded_rom = Some(rom.clone());

        window.set_title(&format!("{} - {}", C::specs().name, rom.game_name));
        self.capture.set_game_name(rom.game_name.clone());

        let console = rom.console.clone();
        let specs = C::specs();

        let (render_width, render_height) = specs.resolution;
        window.set_min_inner_size(Some(winit::dpi::PhysicalSize::new(
            render_width,
            render_height,
        )));

        let started = Instant::now();
        let mut runner = ConsoleRunner::new(console.clone(), window.clone())?;
        tracing::info!("Graphics init took {:?}", started.elapsed());
        runner.graphics_mut().set_scale_mode(self.scale_mode);

        // Create session based on connection mode
        match &self.config.connection_mode {
            ConnectionMode::Local => {
                // Standard local session (no rollback)
                let started = Instant::now();
                runner
                    .load_game(
                        rom.console,
                        &rom.code,
                        self.config.num_players,
                        &rom.game_id,
                    )
                    .context("Failed to load game")?;
                tracing::info!("Game load (WASM compile/init) took {:?}", started.elapsed());
            }
            ConnectionMode::SyncTest { check_distance } => {
                // Sync test session for determinism testing
                let session_config = SessionConfig::sync_test_with_params(
                    self.config.num_players,
                    self.config.input_delay,
                );
                let session = RollbackSession::new_sync_test(session_config, specs.ram_limit)
                    .context("Failed to create sync test session")?;
                let started = Instant::now();
                runner
                    .load_game_with_session(rom.console, &rom.code, session, None, &rom.game_id)
                    .context("Failed to load game with sync test session")?;
                tracing::info!("Game load (WASM compile/init) took {:?}", started.elapsed());
                tracing::info!(
                    "Sync test mode enabled (check_distance: {})",
                    check_distance
                );
            }
            ConnectionMode::P2P {
                bind_port,
                peer_port,
                local_player,
            } => {
                // Local P2P testing mode
                let mut socket = LocalSocket::bind(&format!("127.0.0.1:{}", bind_port))
                    .context("Failed to bind local socket")?;
                socket
                    .connect(&format!("127.0.0.1:{}", peer_port))
                    .context("Failed to connect to peer")?;

                let peer_addr = format!("127.0.0.1:{}", peer_port);
                let session_config =
                    SessionConfig::online(2).with_input_delay(self.config.input_delay);

                let players = vec![
                    (
                        0,
                        if *local_player == 0 {
                            PlayerType::Local
                        } else {
                            PlayerType::Remote(peer_addr.clone())
                        },
                    ),
                    (
                        1,
                        if *local_player == 1 {
                            PlayerType::Local
                        } else {
                            PlayerType::Remote(peer_addr)
                        },
                    ),
                ];

                let session =
                    RollbackSession::new_p2p(session_config, socket, players, specs.ram_limit)
                        .context("Failed to create P2P session")?;
                let started = Instant::now();
                runner
                    .load_game_with_session(rom.console, &rom.code, session, None, &rom.game_id)
                    .context("Failed to load game with P2P session")?;
                tracing::info!("Game load (WASM compile/init) took {:?}", started.elapsed());
                tracing::info!(
                    "P2P mode: bind={}, peer={}, local_player={}",
                    bind_port,
                    peer_port,
                    local_player
                );
            }
            ConnectionMode::Host { port } => {
                // Host mode - bind and wait for connection
                let socket = LocalSocket::bind(&format!("0.0.0.0:{}", port))
                    .context("Failed to bind host socket")?;
                tracing::info!("Hosting on port {}, waiting for connection...", port);

                // Enter waiting state - game will be loaded when peer connects
                // Use a sanitized game ID for URLs (lowercase, no spaces)
                let game_id = sanitize_game_id(&rom.game_name);
                self.waiting_for_peer = Some(WaitingForPeer::new(socket, *port, game_id));

                // Don't load game yet - will be loaded when peer connects
            }
            ConnectionMode::Join { address } => {
                // Join mode - connect to host
                // TODO: Implement proper connection UI
                let mut socket =
                    LocalSocket::bind("0.0.0.0:0").context("Failed to bind client socket")?;
                socket
                    .connect(address)
                    .context("Failed to connect to host")?;
                tracing::info!("Joining game at {}", address);

                // For MVP, create P2P session immediately
                // This will be improved in Phase 0 with proper connection flow
                let session_config =
                    SessionConfig::online(2).with_input_delay(self.config.input_delay);

                let players = vec![
                    (0, PlayerType::Remote(address.clone())),
                    (1, PlayerType::Local),
                ];
                tracing::info!("Join mode: creating session with players {:?}", players);

                let session =
                    RollbackSession::new_p2p(session_config, socket, players, specs.ram_limit)
                        .context("Failed to create P2P session")?;
                tracing::info!(
                    "Join mode: session created, local_players = {:?}",
                    session.local_players()
                );
                let started = Instant::now();
                runner
                    .load_game_with_session(rom.console, &rom.code, session, None, &rom.game_id)
                    .context("Failed to load game with P2P session")?;
                tracing::info!("Game load (WASM compile/init) took {:?}", started.elapsed());
            }
            ConnectionMode::Session { session_file } => {
                // Session mode - pre-negotiated session from library lobby (NCHS protocol)
                let session_file_result = super::connection::create_session_from_file::<C>(
                    session_file,
                    &self.config,
                    specs,
                )?;

                let session = session_file_result.session;
                let save_config = session_file_result.save_config;

                let started = Instant::now();
                runner
                    .load_game_with_session(
                        rom.console,
                        &rom.code,
                        session,
                        save_config,
                        &rom.game_id,
                    )
                    .context("Failed to load game with NCHS session")?;
                tracing::info!("Game load (WASM compile/init) took {:?}", started.elapsed());

                // Clean up the session file after reading
                let _ = std::fs::remove_file(session_file);
            }
        }

        if let Some(session) = runner.session_mut()
            && let Some(audio) = session.runtime.audio_mut()
        {
            let config = super::super::config::load();
            audio.set_master_volume(config.audio.master_volume);
        }
        if let Some(session) = runner.session() {
            self.capture.set_source_fps(session.runtime.tick_rate());
        }

        // Load replay script if specified
        if let Some(ref script_path) = self.config.replay_script {
            match crate::replay::script::ReplayScript::from_file(script_path) {
                Ok(script) => {
                    if let Some(session) = runner.session() {
                        if let Some(layout) = session.runtime.console().replay_input_layout() {
                            match crate::replay::script::Compiler::new(layout.as_ref())
                                .compile(&script)
                            {
                                Ok(compiled) => {
                                    tracing::info!(
                                        "Replay script loaded: {} frames, {} screenshots",
                                        compiled.frame_count,
                                        compiled.screenshot_frames.len()
                                    );
                                    self.replay_executor =
                                        Some(crate::replay::ScriptExecutor::new(compiled));
                                }
                                Err(e) => {
                                    tracing::error!("Failed to compile replay script: {}", e)
                                }
                            }
                        } else {
                            tracing::error!("Console does not support replay scripts");
                        }
                    }
                }
                Err(e) => tracing::error!("Failed to load replay script: {}", e),
            }
        }

        let egui_state = egui_winit::State::new(
            self.egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            runner.graphics().device(),
            runner.graphics().surface_format(),
            egui_wgpu::RendererOptions::default(),
        );
        self.egui_state = Some(egui_state);
        self.egui_renderer = Some(egui_renderer);

        self.window = Some(window);
        self.runner = Some(runner);
        self.next_tick = Instant::now();

        tracing::info!("Game loaded: {}", self.config.rom_path.display());
        tracing::info!("Player startup total {:?}", startup_started.elapsed());
        Ok(())
    }
}
