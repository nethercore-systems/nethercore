//! Generic standalone player for any console
//!
//! Provides a console-agnostic player application that can run ROM files
//! for any console that implements the Console trait and required support traits.

mod error_ui;
mod types;
#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Fullscreen, Window};

use ggrs::PlayerType;

use crate::capture::{CaptureSupport, ScreenCapture, read_render_target_pixels};
use crate::console::{Audio, AudioGenerator, Console, ConsoleResourceManager};
use crate::debug::registry::RegisteredValue;
use crate::debug::types::DebugValue;
use crate::debug::{ActionRequest, FrameController};
use crate::rollback::{
    ConnectionMode, ConnectionQuality, LocalSocket, RollbackSession, SessionConfig, SessionType,
};
use crate::runner::ConsoleRunner;

use super::config::ScaleMode;
use super::event_loop::ConsoleApp;
use super::ui::{SettingsAction, SharedSettingsUi};
use super::{
    DebugStats, FRAME_TIME_HISTORY_SIZE, GameError, GameErrorPhase, RuntimeError, parse_wasm_error,
};

// Re-export types from submodules
pub use error_ui::{
    ErrorAction, WaitingForPeer, parse_key_code, render_error_screen, sanitize_game_id,
};
pub use types::{LoadedRom, RomLoader, StandaloneConfig, StandaloneGraphicsSupport};

fn format_ggrs_addr(addr: &str, port: u16) -> String {
    if let Ok(socket_addr) = addr.parse::<SocketAddr>() {
        return SocketAddr::new(socket_addr.ip(), port).to_string();
    }

    if let Ok(ip) = addr.parse::<std::net::IpAddr>() {
        return SocketAddr::new(ip, port).to_string();
    }

    if let Some(stripped) = addr
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        && let Ok(ip) = stripped.parse::<std::net::IpAddr>()
    {
        return SocketAddr::new(ip, port).to_string();
    }

    // Fallback: split on the last ':' to preserve IPv6 host parts without brackets.
    let mut parts = addr.rsplitn(2, ':');
    let _port_part = parts.next();
    if let Some(host) = parts.next() {
        if host.contains(':') && !(host.starts_with('[') && host.ends_with(']')) {
            return format!("[{}]:{}", host, port);
        }
        return format!("{}:{}", host, port);
    }

    format!("{}:{}", addr, port)
}

/// Generic standalone player application.
///
/// This provides a complete player implementation for any console that:
/// - Implements the `Console` trait
/// - Has a `Graphics` type that implements `StandaloneGraphicsSupport`
/// - Has a ROM loader that implements `RomLoader`
pub struct StandaloneApp<C, L>
where
    C: Console + Clone,
    C::Graphics: StandaloneGraphicsSupport,
    L: RomLoader<Console = C>,
{
    config: StandaloneConfig,
    window: Option<Arc<Window>>,
    runner: Option<ConsoleRunner<C>>,
    input_manager: super::InputManager,
    scale_mode: ScaleMode,
    settings_ui: SharedSettingsUi,
    debug_overlay: bool,
    debug_panel: crate::debug::DebugPanel,
    frame_controller: FrameController,
    next_tick: Instant,
    last_sim_rendered: bool,
    needs_redraw: bool,
    should_exit: bool,
    debug_stats: DebugStats,
    game_tick_times: Vec<Instant>,
    last_game_tick: Instant,
    egui_ctx: egui::Context,
    egui_state: Option<egui_winit::State>,
    egui_renderer: Option<egui_wgpu::Renderer>,
    loaded_rom: Option<LoadedRom<C>>,
    error_state: Option<GameError>,
    capture: ScreenCapture,
    screenshot_key: KeyCode,
    gif_toggle_key: KeyCode,
    /// Network statistics overlay visibility (F12)
    network_overlay_visible: bool,
    /// State for waiting for a peer to connect (Host mode)
    waiting_for_peer: Option<WaitingForPeer>,
    _vram_limit: usize,
    _loader_marker: std::marker::PhantomData<L>,
}

impl<C, L> StandaloneApp<C, L>
where
    C: Console + Clone,
    C::Graphics: StandaloneGraphicsSupport,
    L: RomLoader<Console = C>,
{
    /// Create a new standalone app with the given configuration.
    pub fn new(config: StandaloneConfig, vram_limit: usize) -> Self {
        let now = Instant::now();
        let app_config = super::config::load();
        let input_config = app_config.input.clone();
        let scale_mode = app_config.video.scale_mode;
        let settings_ui = SharedSettingsUi::new(&app_config);

        let warnings = super::config::validate_keybindings(&app_config);
        for warning in warnings {
            tracing::warn!("Keybinding conflict: {}", warning);
        }

        let screenshot_key = parse_key_code(&app_config.capture.screenshot).unwrap_or_else(|| {
            tracing::warn!(
                "Invalid screenshot key '{}', using F9",
                app_config.capture.screenshot
            );
            KeyCode::F9
        });
        let gif_toggle_key = parse_key_code(&app_config.capture.gif_toggle).unwrap_or_else(|| {
            tracing::warn!(
                "Invalid GIF toggle key '{}', using F10",
                app_config.capture.gif_toggle
            );
            KeyCode::F10
        });

        let initial_game_name = config
            .rom_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("game")
            .to_string();
        let capture = ScreenCapture::new(
            app_config.capture.gif_fps,
            app_config.capture.gif_max_seconds,
            initial_game_name,
            C::specs().console_type.to_string(),
        );

        Self {
            debug_overlay: config.debug,
            debug_panel: crate::debug::DebugPanel::new(),
            config,
            window: None,
            runner: None,
            input_manager: super::InputManager::new(input_config),
            scale_mode,
            settings_ui,
            frame_controller: FrameController::new(),
            next_tick: now,
            last_sim_rendered: false,
            needs_redraw: true,
            should_exit: false,
            debug_stats: DebugStats {
                frame_times: VecDeque::with_capacity(FRAME_TIME_HISTORY_SIZE),
                vram_limit,
                ..Default::default()
            },
            game_tick_times: Vec::with_capacity(120),
            last_game_tick: now,
            egui_ctx: egui::Context::default(),
            egui_state: None,
            egui_renderer: None,
            loaded_rom: None,
            error_state: None,
            capture,
            screenshot_key,
            gif_toggle_key,
            network_overlay_visible: false,
            waiting_for_peer: None,
            _vram_limit: vram_limit,
            _loader_marker: std::marker::PhantomData,
        }
    }

    fn tick_duration(&self) -> Duration {
        if let Some(runner) = &self.runner
            && let Some(session) = runner.session()
        {
            return session.runtime.tick_duration();
        }
        Duration::from_secs_f64(1.0 / 60.0)
    }

    fn restart_game(&mut self) {
        self.error_state = None;

        // Try to reload ROM if not already loaded
        if self.loaded_rom.is_none() {
            match L::load_rom(&self.config.rom_path) {
                Ok(rom) => {
                    self.loaded_rom = Some(rom);
                }
                Err(_) => {
                    tracing::error!("Failed to reload ROM for restart");
                    self.should_exit = true;
                    return;
                }
            }
        }

        let rom = match &self.loaded_rom {
            Some(rom) => rom.clone(),
            None => {
                tracing::error!("No ROM loaded for restart");
                self.should_exit = true;
                return;
            }
        };

        if let Some(runner) = &mut self.runner {
            runner.unload_game();
        }

        let console = rom.console.clone();

        if let Some(runner) = &mut self.runner {
            if let Err(e) = runner.load_game(console, &rom.code, 1) {
                tracing::error!("Failed to restart game: {}", e);
                self.error_state = Some(GameError {
                    summary: "Restart Failed".to_string(),
                    details: format!("{:#}", e),
                    stack_trace: None,
                    tick: None,
                    phase: GameErrorPhase::Init,
                    suggestions: vec![
                        "The ROM file may be corrupted".to_string(),
                        "Try closing and reopening the player".to_string(),
                    ],
                });
                return;
            }

            if let Some(session) = runner.session_mut()
                && let Some(audio) = session.runtime.audio_mut()
            {
                let config = super::config::load();
                audio.set_master_volume(config.audio.master_volume);
            }
            if let Some(session) = runner.session() {
                self.capture.set_source_fps(session.runtime.tick_rate());
            }
        }

        self.next_tick = Instant::now();
        self.needs_redraw = true;
        tracing::info!("Game restarted successfully");
    }

    fn handle_key_input(&mut self, event: KeyEvent) {
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

    fn run_game_frame(&mut self) -> Result<(bool, bool), RuntimeError> {
        let runner = self
            .runner
            .as_mut()
            .ok_or_else(|| RuntimeError("No runner".to_string()))?;

        let session = runner
            .session_mut()
            .ok_or_else(|| RuntimeError("No session".to_string()))?;

        // Get local player handles from session (e.g., [0] for host, [1] for joiner)
        let local_players: Vec<usize> = session
            .runtime
            .session()
            .map(|s| s.local_players().to_vec())
            .unwrap_or_else(|| vec![0]);

        // Log once at startup (not every frame)
        static LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !LOGGED.swap(true, std::sync::atomic::Ordering::Relaxed) {
            tracing::info!("run_game_frame: local_players = {:?}", local_players);
        }

        // Map physical input devices to player handles
        // physical_idx 0 = keyboard/first gamepad, physical_idx 1 = second gamepad, etc.
        // player_handle = the session's player slot (e.g., 0 for host, 1 for joiner)
        for (physical_idx, &player_handle) in local_players.iter().enumerate() {
            let raw_input = self.input_manager.get_player_input(physical_idx);
            let console_input = session.runtime.console().map_input(&raw_input);

            if let Some(game) = session.runtime.game_mut() {
                game.set_input(player_handle, console_input);
            }

            // Always add input - GGRS will handle synchronization
            if let Err(e) = session
                .runtime
                .add_local_input(player_handle, console_input)
            {
                tracing::error!(
                    "Failed to add local input for handle {}: {:?}",
                    player_handle,
                    e
                );
            }
        }

        let should_run = self.frame_controller.should_run_tick();
        let time_scale = self.frame_controller.time_scale();

        let tick_start = Instant::now();
        let (ticks, _alpha) = if should_run {
            session
                .runtime
                .frame_with_time_scale(time_scale)
                .map_err(|e| RuntimeError(format!("Game frame error: {}", e)))?
        } else {
            (0, 0.0)
        };
        let tick_elapsed = tick_start.elapsed();

        let did_render = if ticks > 0 {
            let tick_time_ms = tick_elapsed.as_secs_f32() * 1000.0 / ticks as f32;
            self.debug_stats.game_tick_times.push_back(tick_time_ms);
            while self.debug_stats.game_tick_times.len() > FRAME_TIME_HISTORY_SIZE {
                self.debug_stats.game_tick_times.pop_front();
            }

            if let Some(game) = session.runtime.game_mut() {
                C::clear_frame_state(game.console_state_mut());
            }

            let render_start = Instant::now();
            session
                .runtime
                .render()
                .map_err(|e| RuntimeError(format!("Render error: {}", e)))?;
            let render_time_ms = render_start.elapsed().as_secs_f32() * 1000.0;

            self.debug_stats.game_render_times.push_back(render_time_ms);
            while self.debug_stats.game_render_times.len() > FRAME_TIME_HISTORY_SIZE {
                self.debug_stats.game_render_times.pop_front();
            }

            let now = Instant::now();
            for _ in 0..ticks {
                self.game_tick_times.push(now);
                if self.game_tick_times.len() > FRAME_TIME_HISTORY_SIZE {
                    self.game_tick_times.remove(0);
                }
            }
            self.last_game_tick = now;

            true
        } else {
            false
        };

        // Process audio using the console's AudioGenerator
        // This handles both synchronous and threaded audio modes automatically
        if did_render {
            let tick_rate = session.runtime.tick_rate();
            let sample_rate = session
                .runtime
                .audio()
                .map(|a| a.sample_rate())
                .unwrap_or_else(C::AudioGenerator::default_sample_rate);

            let (game_opt, audio_opt) = session.runtime.game_and_audio_mut();
            if let (Some(game), Some(audio)) = (game_opt, audio_opt) {
                let (ffi_state, rollback_state) = game.ffi_and_rollback_mut();
                C::AudioGenerator::process_audio(
                    rollback_state,
                    ffi_state,
                    audio,
                    tick_rate,
                    sample_rate,
                );
            }
        }

        let quit_requested = session
            .runtime
            .game()
            .map(|g| g.state().quit_requested)
            .unwrap_or(false);

        Ok((!quit_requested, did_render))
    }

    fn execute_draw_commands(&mut self) {
        if let Some(runner) = &mut self.runner {
            let (graphics, session_opt) = runner.graphics_and_session_mut();
            if let Some(session) = session_opt
                && let Some(game) = session.runtime.game_mut()
            {
                let state = game.console_state_mut();
                session
                    .resource_manager
                    .execute_draw_commands(graphics, state);
            }
        }
    }

    fn get_clear_color(&self) -> [f32; 4] {
        if let Some(runner) = &self.runner
            && let Some(session) = runner.session()
            && let Some(game) = session.runtime.game()
        {
            return C::clear_color_from_state(game.console_state());
        }
        [0.1, 0.1, 0.1, 1.0]
    }
}

impl<C, L> ConsoleApp<C> for StandaloneApp<C, L>
where
    C: Console + Clone,
    C::Graphics: StandaloneGraphicsSupport,
    L: RomLoader<Console = C>,
{
    fn on_window_created(
        &mut self,
        window: Arc<Window>,
        _event_loop: &ActiveEventLoop,
    ) -> Result<()> {
        if self.config.fullscreen {
            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
        }

        let rom = L::load_rom(&self.config.rom_path)?;
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

        let mut runner = ConsoleRunner::new(console.clone(), window.clone())?;
        runner.graphics_mut().set_scale_mode(self.scale_mode);

        // Create session based on connection mode
        match &self.config.connection_mode {
            ConnectionMode::Local => {
                // Standard local session (no rollback)
                runner
                    .load_game(rom.console, &rom.code, self.config.num_players)
                    .context("Failed to load game")?;
            }
            ConnectionMode::SyncTest { check_distance } => {
                // Sync test session for determinism testing
                let session_config = SessionConfig::sync_test_with_params(
                    self.config.num_players,
                    self.config.input_delay,
                );
                let session = RollbackSession::new_sync_test(session_config, specs.ram_limit)
                    .context("Failed to create sync test session")?;
                runner
                    .load_game_with_session(rom.console, &rom.code, session)
                    .context("Failed to load game with sync test session")?;
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
                runner
                    .load_game_with_session(rom.console, &rom.code, session)
                    .context("Failed to load game with P2P session")?;
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
                runner
                    .load_game_with_session(rom.console, &rom.code, session)
                    .context("Failed to load game with P2P session")?;
            }
            ConnectionMode::Session { session_file } => {
                // Session mode - pre-negotiated session from library lobby (NCHS protocol)
                use crate::net::nchs::SessionStart;
                use std::collections::HashMap;
                use std::net::SocketAddr;
                use std::time::{Duration, Instant};

                const MAX_SESSION_FILE_BYTES: u64 = 1024 * 1024; // 1 MiB
                let session_file_len = std::fs::metadata(session_file)
                    .with_context(|| {
                        format!("Failed to stat session file: {}", session_file.display())
                    })?
                    .len();
                anyhow::ensure!(
                    session_file_len <= MAX_SESSION_FILE_BYTES,
                    "Session file is too large ({} bytes, max {} bytes): {}",
                    session_file_len,
                    MAX_SESSION_FILE_BYTES,
                    session_file.display()
                );

                let bytes = std::fs::read(session_file).context("Failed to read session file")?;
                let session_start: SessionStart = bitcode::decode(&bytes)
                    .map_err(|e| anyhow::anyhow!("Failed to decode session: {}", e))?;

                tracing::info!(
                    "Session mode: loading pre-negotiated session (local_player={}, player_count={}, seed={})",
                    session_start.local_player_handle,
                    session_start.player_count,
                    session_start.random_seed
                );

                let local_handle = session_start.local_player_handle as usize;
                let is_host = session_start.local_player_handle == 0;

                // Get our own ggrs_port
                let own_ggrs_port = session_start
                    .players
                    .iter()
                    .find(|p| p.handle == session_start.local_player_handle)
                    .map(|p| p.ggrs_port)
                    .unwrap_or(0);

                tracing::info!(
                    "Session mode: binding to ggrs_port {} (handle {}, is_host={})",
                    own_ggrs_port,
                    session_start.local_player_handle,
                    is_host
                );

                // Bind to our GGRS port
                let socket = LocalSocket::bind(&format!("0.0.0.0:{}", own_ggrs_port))
                    .context("Failed to bind GGRS socket")?;

                // Handshake magic bytes to identify our packets
                const HANDSHAKE_HELLO: &[u8] = b"NCHS_HELLO";
                const HANDSHAKE_READY: &[u8] = b"NCHS_READY";
                const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

                // Perform handshake to ensure all peers are ready before creating GGRS session
                // This prevents the race condition where one side starts sending GGRS packets
                // before the other side has bound to its port.
                let peer_addresses: HashMap<u8, SocketAddr> = if is_host {
                    // HOST: Wait for all guests to send HELLO, then send READY to each
                    let expected_guests: Vec<u8> = session_start
                        .players
                        .iter()
                        .filter(|p| p.active && p.handle != 0)
                        .map(|p| p.handle)
                        .collect();

                    tracing::info!(
                        "Session mode: host waiting for {} guest(s) to connect",
                        expected_guests.len()
                    );

                    let mut received_from: HashMap<u8, SocketAddr> = HashMap::new();
                    let start = Instant::now();

                    while received_from.len() < expected_guests.len() {
                        if start.elapsed() > HANDSHAKE_TIMEOUT {
                            anyhow::bail!("Timeout waiting for guests to connect");
                        }

                        // Try to receive HELLO from guests
                        let mut buf = [0u8; 64];
                        match socket.socket().recv_from(&mut buf) {
                            Ok((len, from)) => {
                                if len >= HANDSHAKE_HELLO.len()
                                    && &buf[..HANDSHAKE_HELLO.len()] == HANDSHAKE_HELLO
                                {
                                    // Extract handle from after HELLO
                                    if len > HANDSHAKE_HELLO.len() {
                                        let handle = buf[HANDSHAKE_HELLO.len()];
                                        if expected_guests.contains(&handle)
                                            && !received_from.contains_key(&handle)
                                        {
                                            tracing::info!(
                                                "Session mode: received HELLO from guest {} at {}",
                                                handle,
                                                from
                                            );
                                            received_from.insert(handle, from);

                                            // Send READY back immediately
                                            let mut ready_msg = HANDSHAKE_READY.to_vec();
                                            ready_msg.push(session_start.local_player_handle);
                                            let _ = socket.socket().send_to(&ready_msg, from);
                                        }
                                    }
                                }
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                std::thread::sleep(Duration::from_millis(10));
                            }
                            Err(_) => {
                                std::thread::sleep(Duration::from_millis(10));
                            }
                        }
                    }

                    tracing::info!(
                        "Session mode: all {} guest(s) connected",
                        received_from.len()
                    );
                    received_from
                } else {
                    // GUEST: Send HELLO to host, wait for READY
                    let host_player = session_start
                        .players
                        .iter()
                        .find(|p| p.handle == 0)
                        .ok_or_else(|| anyhow::anyhow!("Session has no host player"))?;

                    let host_addr: SocketAddr = format!(
                        "{}:{}",
                        host_player.addr.split(':').next().unwrap_or("127.0.0.1"),
                        host_player.ggrs_port
                    )
                    .parse()
                    .context("Invalid host address")?;

                    tracing::info!("Session mode: guest sending HELLO to host at {}", host_addr);

                    let start = Instant::now();
                    let mut received_ready = false;

                    while !received_ready {
                        if start.elapsed() > HANDSHAKE_TIMEOUT {
                            anyhow::bail!("Timeout waiting for host READY");
                        }

                        // Send HELLO
                        let mut hello_msg = HANDSHAKE_HELLO.to_vec();
                        hello_msg.push(session_start.local_player_handle);
                        let _ = socket.socket().send_to(&hello_msg, host_addr);

                        // Wait a bit for READY
                        std::thread::sleep(Duration::from_millis(50));

                        // Check for READY
                        let mut buf = [0u8; 64];
                        match socket.socket().recv_from(&mut buf) {
                            Ok((len, _from)) => {
                                if len >= HANDSHAKE_READY.len()
                                    && &buf[..HANDSHAKE_READY.len()] == HANDSHAKE_READY
                                {
                                    tracing::info!("Session mode: received READY from host");
                                    received_ready = true;
                                }
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                            Err(_) => {}
                        }
                    }

                    // Return host address
                    let mut addrs = HashMap::new();
                    addrs.insert(0, host_addr);
                    addrs
                };

                // Now build the player list using actual discovered addresses for guests (host only)
                // Guests use pre-specified host address
                let players: Vec<(usize, PlayerType<String>)> = session_start
                    .players
                    .iter()
                    .filter(|p| p.active)
                    .map(|p| {
                        let handle = p.handle as usize;
                        if handle == local_handle {
                            (handle, PlayerType::Local)
                        } else if let Some(actual_addr) = peer_addresses.get(&p.handle) {
                            // Use actual discovered address (from handshake)
                            (handle, PlayerType::Remote(actual_addr.to_string()))
                        } else {
                            // Fallback to pre-specified address
                            let ggrs_addr = format_ggrs_addr(&p.addr, p.ggrs_port);
                            (handle, PlayerType::Remote(ggrs_addr))
                        }
                    })
                    .collect();

                tracing::info!("Session mode: players (after handshake) = {:?}", players);

                let mut session_config = SessionConfig::online(session_start.player_count as usize)
                    .with_input_delay(session_start.network_config.input_delay as usize);
                session_config.fps = session_start.tick_rate.as_hz() as usize;

                let session =
                    RollbackSession::new_p2p(session_config, socket, players, specs.ram_limit)
                        .context("Failed to create session from NCHS config")?;

                tracing::info!(
                    "Session mode: session created, local_players = {:?}",
                    session.local_players()
                );

                runner
                    .load_game_with_session(rom.console, &rom.code, session)
                    .context("Failed to load game with NCHS session")?;

                // Clean up the session file after reading
                let _ = std::fs::remove_file(session_file);
            }
        }

        if let Some(session) = runner.session_mut()
            && let Some(audio) = session.runtime.audio_mut()
        {
            let config = super::config::load();
            audio.set_master_volume(config.audio.master_volume);
        }
        if let Some(session) = runner.session() {
            self.capture.set_source_fps(session.runtime.tick_rate());
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
        Ok(())
    }

    fn on_window_event(&mut self, event: &WindowEvent) -> bool {
        if let (Some(egui_state), Some(window)) = (&mut self.egui_state, &self.window) {
            let response = egui_state.on_window_event(window, event);
            if response.consumed {
                self.needs_redraw = true;
                return true;
            }
            if response.repaint {
                self.needs_redraw = true;
            }
        }

        match event {
            WindowEvent::Resized(size) => {
                if let Some(runner) = &mut self.runner {
                    runner.resize(size.width, size.height);
                }
                self.needs_redraw = true;
                false
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                self.handle_key_input(key_event.clone());
                false
            }
            _ => false,
        }
    }

    fn next_tick(&self) -> Instant {
        self.next_tick
    }

    fn advance_simulation(&mut self) {
        self.last_sim_rendered = false;

        if self.error_state.is_some() {
            return;
        }

        // Poll for peer connection in Host mode
        if let Some(ref mut waiting) = self.waiting_for_peer {
            if let Some(peer_addr) = waiting.socket.poll_for_peer() {
                tracing::info!("Peer connected from {}", peer_addr);

                // Take the waiting state to get ownership of the socket
                let waiting = self.waiting_for_peer.take().unwrap();

                // Create the P2P session now that we have a peer
                if let (Some(rom), Some(runner)) = (&self.loaded_rom, &mut self.runner) {
                    let specs = C::specs();
                    let session_config =
                        SessionConfig::online(2).with_input_delay(self.config.input_delay);

                    // Host is player 0, peer is player 1
                    let players = vec![
                        (0, PlayerType::Local),
                        (1, PlayerType::Remote(peer_addr.clone())),
                    ];
                    tracing::info!(
                        "Host mode: creating P2P session (host=local p0, peer=remote p1)"
                    );

                    match RollbackSession::new_p2p(
                        session_config,
                        waiting.socket,
                        players,
                        specs.ram_limit,
                    ) {
                        Ok(session) => {
                            tracing::info!(
                                "Host mode: session created, local_players = {:?}",
                                session.local_players()
                            );
                            if let Err(e) = runner.load_game_with_session(
                                rom.console.clone(),
                                &rom.code,
                                session,
                            ) {
                                tracing::error!("Failed to load game with P2P session: {}", e);
                                self.error_state = Some(GameError {
                                    summary: "Connection Error".to_string(),
                                    details: format!("Failed to start game: {}", e),
                                    stack_trace: None,
                                    tick: None,
                                    phase: GameErrorPhase::Update,
                                    suggestions: vec![],
                                });
                            } else {
                                tracing::info!("Host mode: game started with peer");
                                // Set audio volume
                                if let Some(session) = runner.session_mut()
                                    && let Some(audio) = session.runtime.audio_mut()
                                {
                                    let config = super::config::load();
                                    audio.set_master_volume(config.audio.master_volume);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to create P2P session: {}", e);
                            self.error_state = Some(GameError {
                                summary: "Connection Error".to_string(),
                                details: format!("Failed to create session: {}", e),
                                stack_trace: None,
                                tick: None,
                                phase: GameErrorPhase::Update,
                                suggestions: vec![],
                            });
                        }
                    }
                }

                self.needs_redraw = true;
                return;
            }
            // Still waiting for peer - don't run game simulation
            return;
        }

        self.input_manager.update();

        let tick_before = self
            .runner
            .as_ref()
            .and_then(|r| r.session())
            .and_then(|s| s.runtime.game())
            .map(|g| g.state().tick_count);

        match self.run_game_frame() {
            Ok((game_running, did_render)) => {
                self.last_sim_rendered = did_render;

                if did_render {
                    self.execute_draw_commands();
                }

                if !game_running {
                    tracing::info!("Game requested quit");
                    self.should_exit = true;
                }
            }
            Err(e) => {
                let error_msg = e.0.clone();
                let phase = if error_msg.contains("Render error") {
                    GameErrorPhase::Render
                } else {
                    GameErrorPhase::Update
                };

                let game_error =
                    parse_wasm_error(&anyhow::anyhow!("{}", error_msg), tick_before, phase);
                tracing::error!("Game error: {}", game_error);
                self.error_state = Some(game_error);
                self.needs_redraw = true;
            }
        }
    }

    fn update_next_tick(&mut self) {
        self.next_tick += self.tick_duration();
    }

    fn render(&mut self) {
        let mut restart_requested = false;

        // Get clear color before borrowing runner mutably
        let clear_color = self.get_clear_color();

        {
            let runner = match &mut self.runner {
                Some(r) => r,
                None => return,
            };

            let surface_texture = match runner.graphics_mut().get_current_texture() {
                Ok(tex) => tex,
                Err(e) => {
                    tracing::warn!("Failed to get surface texture: {}", e);
                    return;
                }
            };

            let view = surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = runner.graphics().device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("Standalone Frame Encoder"),
                },
            );

            // Render game if we have new content
            if self.last_sim_rendered {
                let (graphics, session_opt) = runner.graphics_and_session_mut();

                if let Some(session) = session_opt
                    && let Some(game) = session.runtime.game()
                {
                    let state = game.console_state();
                    session.resource_manager.render_game_to_target(
                        graphics,
                        &mut encoder,
                        state,
                        clear_color,
                    );
                }
            }

            runner.graphics().blit_to_window(&mut encoder, &view);

            // Render overlays via egui
            if self.debug_overlay
                || self.debug_panel.visible
                || self.settings_ui.visible
                || self.error_state.is_some()
                || self.network_overlay_visible
                || self.waiting_for_peer.is_some()
            {
                let pending_writes: RefCell<Vec<(RegisteredValue, DebugValue)>> =
                    RefCell::new(Vec::new());
                let pending_action: RefCell<Option<ActionRequest>> = RefCell::new(None);
                let settings_action: RefCell<SettingsAction> = RefCell::new(SettingsAction::None);
                let error_action: RefCell<ErrorAction> = RefCell::new(ErrorAction::None);

                if let (Some(egui_state), Some(egui_renderer), Some(window)) =
                    (&mut self.egui_state, &mut self.egui_renderer, &self.window)
                {
                    let raw_input = egui_state.take_egui_input(window);

                    let debug_overlay = self.debug_overlay;
                    let debug_stats = &self.debug_stats;
                    let game_tick_times = &self.game_tick_times;
                    let debug_panel = &mut self.debug_panel;
                    let frame_controller = &mut self.frame_controller;
                    let settings_ui = &mut self.settings_ui;
                    let error_state_ref = &self.error_state;
                    let waiting_for_peer_ref = &self.waiting_for_peer;
                    let network_overlay_visible = self.network_overlay_visible;

                    // Get network session info for overlay
                    let (
                        session_type,
                        network_stats,
                        local_players,
                        total_rollbacks,
                        current_frame,
                    ) = {
                        if let Some(game_session) = runner.session() {
                            if let Some(rollback) = game_session.runtime.session() {
                                (
                                    rollback.session_type(),
                                    rollback.all_player_stats().to_vec(),
                                    rollback.local_players().to_vec(),
                                    rollback.total_rollback_frames(),
                                    rollback.current_frame(),
                                )
                            } else {
                                (SessionType::Local, Vec::new(), Vec::new(), 0, 0)
                            }
                        } else {
                            (SessionType::Local, Vec::new(), Vec::new(), 0, 0)
                        }
                    };

                    let full_output = self.egui_ctx.run(raw_input, |ctx| {
                        let action = settings_ui.show_as_window(ctx);
                        if !matches!(action, SettingsAction::None) {
                            *settings_action.borrow_mut() = action;
                        }
                        if debug_overlay {
                            let frame_time_ms = debug_stats.frame_times.back().copied().unwrap_or(16.67);
                            let render_fps = super::debug::calculate_fps(game_tick_times);
                            super::debug::render_debug_overlay(
                                ctx,
                                debug_stats,
                                true,
                                frame_time_ms,
                                render_fps,
                                render_fps,
                            );
                        }

                        // Network statistics overlay (F4)
                        if network_overlay_visible && session_type != SessionType::Local {
                            egui::Window::new("Network")
                                .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
                                .collapsible(false)
                                .resizable(false)
                                .show(ctx, |ui| {
                                    ui.set_min_width(180.0);

                                    // Player stats with quality bar
                                    for (i, stats) in network_stats.iter().enumerate() {
                                        let is_local = local_players.contains(&i);

                                        // Quality color and label
                                        let (color, quality_label) = match stats.quality {
                                            ConnectionQuality::Excellent => {
                                                (egui::Color32::GREEN, "Excellent")
                                            }
                                            ConnectionQuality::Good => {
                                                (egui::Color32::from_rgb(144, 238, 144), "Good")
                                            }
                                            ConnectionQuality::Fair => {
                                                (egui::Color32::YELLOW, "Fair")
                                            }
                                            ConnectionQuality::Poor => (egui::Color32::RED, "Poor"),
                                            ConnectionQuality::Disconnected => {
                                                (egui::Color32::DARK_GRAY, "Disconnected")
                                            }
                                        };

                                        if is_local {
                                            ui.horizontal(|ui| {
                                                ui.label(format!("P{}: Local", i + 1));
                                            });
                                        } else if stats.connected {
                                            // Show: P2: 45ms ████████ Good
                                            ui.horizontal(|ui| {
                                                ui.label(format!("P{}: {}ms ", i + 1, stats.ping_ms));

                                                // Quality bar (8 blocks max)
                                                let filled = match stats.quality {
                                                    ConnectionQuality::Excellent => 8,
                                                    ConnectionQuality::Good => 6,
                                                    ConnectionQuality::Fair => 4,
                                                    ConnectionQuality::Poor => 2,
                                                    ConnectionQuality::Disconnected => 0,
                                                };
                                                let bar: String = "\u{2588}"
                                                    .repeat(filled)
                                                    .chars()
                                                    .chain("\u{2591}".repeat(8 - filled).chars())
                                                    .collect();
                                                ui.colored_label(color, bar);
                                                ui.label(quality_label);
                                            });
                                        } else {
                                            ui.horizontal(|ui| {
                                                ui.colored_label(
                                                    egui::Color32::DARK_GRAY,
                                                    format!("P{}: Disconnected", i + 1),
                                                );
                                            });
                                        }
                                    }

                                    ui.separator();
                                    ui.label(format!("Rollbacks: {} frames", total_rollbacks));
                                    ui.label(format!("Frame: {}", current_frame));
                                });
                        }

                        if debug_panel.visible
                            && let Some(session) = runner.session()
                                && let Some(game) = session.runtime.game()
                            {
                                let registry = game.store().data().debug_registry.clone();

                                let read_value = |reg_val: &RegisteredValue| -> Option<DebugValue> {
                                    let mem = game.store().data().game.memory?;
                                    let data = mem.data(game.store());

                                    let ptr = reg_val.wasm_ptr as usize;
                                    let size = reg_val.value_type.byte_size();
                                    let end = ptr.checked_add(size)?;
                                    if end > data.len() {
                                        return None;
                                    }

                                    Some(registry.read_value_from_slice(
                                        &data[ptr..end],
                                        reg_val.value_type,
                                    ))
                                };

                                let write_value =
                                    |reg_val: &RegisteredValue, new_val: &DebugValue| -> bool {
                                        pending_writes
                                            .borrow_mut()
                                            .push((reg_val.clone(), new_val.clone()));
                                        true
                                    };

                                let (_changed, action) = debug_panel.render(
                                    ctx,
                                    &registry,
                                    frame_controller,
                                    read_value,
                                    write_value,
                                );
                                if let Some(action) = action {
                                    *pending_action.borrow_mut() = Some(action);
                                }
                            }

                        // Waiting for peer connection dialog (Host mode)
                        if let Some(waiting) = waiting_for_peer_ref {
                            egui::Window::new("Hosting Game")
                                .collapsible(false)
                                .resizable(false)
                                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                                .show(ctx, |ui| {
                                    ui.set_min_width(400.0);

                                    ui.vertical_centered(|ui| {
                                        ui.add_space(10.0);
                                        ui.spinner();
                                        ui.add_space(10.0);
                                        ui.label("Waiting for player to connect...");
                                        ui.add_space(15.0);
                                    });

                                    ui.separator();
                                    ui.add_space(10.0);

                                    ui.label("Share one of these links with your friend:");
                                    ui.add_space(5.0);

                                    for ip in &waiting.local_ips {
                                        let join_url = waiting.join_url(ip);
                                        ui.horizontal(|ui| {
                                            // Show truncated URL for display
                                            let display_url = if join_url.len() > 50 {
                                                format!("{}...", &join_url[..47])
                                            } else {
                                                join_url.clone()
                                            };
                                            ui.monospace(&display_url);
                                            if ui.small_button("Copy").clicked() {
                                                ctx.copy_text(join_url.clone());
                                            }
                                        });
                                    }

                                    ui.add_space(10.0);

                                    // Collapsible section for manual connection
                                    ui.collapsing("Manual connection (IP:port)", |ui| {
                                        ui.label(
                                            egui::RichText::new("If the link doesn't work, share this address:")
                                                .weak()
                                                .small(),
                                        );
                                        ui.add_space(5.0);
                                        for ip in &waiting.local_ips {
                                            let addr = format!("{}:{}", ip, waiting.port);
                                            ui.horizontal(|ui| {
                                                ui.monospace(&addr);
                                                if ui.small_button("Copy").clicked() {
                                                    ctx.copy_text(addr.clone());
                                                }
                                            });
                                        }
                                    });

                                    ui.add_space(10.0);
                                    ui.label(
                                        egui::RichText::new("Your friend can paste the link in their browser or use 'Join Game'")
                                            .weak()
                                            .small(),
                                    );
                                    ui.add_space(10.0);
                                });
                        }

                        if let Some(error) = error_state_ref {
                            let action = render_error_screen(ctx, error);
                            if action != ErrorAction::None {
                                *error_action.borrow_mut() = action;
                            }
                        }
                    });

                    egui_state.handle_platform_output(window, full_output.platform_output);

                    let screen_descriptor = egui_wgpu::ScreenDescriptor {
                        size_in_pixels: [runner.graphics().width(), runner.graphics().height()],
                        pixels_per_point: window.scale_factor() as f32,
                    };

                    let tris = self
                        .egui_ctx
                        .tessellate(full_output.shapes, full_output.pixels_per_point);

                    for (id, delta) in &full_output.textures_delta.set {
                        egui_renderer.update_texture(
                            runner.graphics().device(),
                            runner.graphics().queue(),
                            *id,
                            delta,
                        );
                    }

                    egui_renderer.update_buffers(
                        runner.graphics().device(),
                        runner.graphics().queue(),
                        &mut encoder,
                        &tris,
                        &screen_descriptor,
                    );

                    {
                        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Egui Render Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: wgpu::StoreOp::Store,
                                },
                                depth_slice: None,
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });
                        let mut render_pass_static = render_pass.forget_lifetime();
                        egui_renderer.render(&mut render_pass_static, &tris, &screen_descriptor);
                    }

                    for id in &full_output.textures_delta.free {
                        egui_renderer.free_texture(id);
                    }
                }

                // Apply pending writes
                let writes = pending_writes.into_inner();
                if !writes.is_empty()
                    && let Some(session) = runner.session_mut()
                    && let Some(game) = session.runtime.game_mut()
                {
                    let has_debug_callback = game.has_debug_change_callback();
                    let memory = game.store().data().game.memory;
                    if let Some(mem) = memory {
                        let registry = game.store().data().debug_registry.clone();
                        let data = mem.data_mut(game.store_mut());
                        for (reg_val, new_val) in &writes {
                            let ptr = reg_val.wasm_ptr as usize;
                            let size = reg_val.value_type.byte_size();
                            if ptr + size <= data.len() {
                                registry.write_value_to_slice(&mut data[ptr..ptr + size], new_val);
                            }
                        }
                    }
                    if has_debug_callback {
                        game.call_on_debug_change();
                    }
                }

                // Apply pending action
                if let Some(action_req) = pending_action.into_inner()
                    && let Some(session) = runner.session_mut()
                    && let Some(game) = session.runtime.game_mut()
                    && let Err(e) = game.call_action(&action_req.func_name, &action_req.args)
                {
                    tracing::warn!("Debug action '{}' failed: {}", action_req.func_name, e);
                }

                // Apply settings actions
                match settings_action.into_inner() {
                    SettingsAction::None => {}
                    SettingsAction::Close => {
                        // Settings panel was closed, nothing else to do
                    }
                    SettingsAction::ToggleFullscreen(fullscreen) => {
                        if let Some(window) = &self.window {
                            if fullscreen {
                                window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                            } else {
                                window.set_fullscreen(None);
                            }
                        }
                    }
                    SettingsAction::PreviewScaleMode(scale_mode) => {
                        self.scale_mode = scale_mode;
                        runner.graphics_mut().set_scale_mode(scale_mode);
                    }
                    SettingsAction::SetVolume(volume) => {
                        if let Some(session) = runner.session_mut()
                            && let Some(audio) = session.runtime.audio_mut()
                        {
                            audio.set_master_volume(volume);
                        }
                    }
                    SettingsAction::ResetDefaults => {
                        // Defaults were applied to temp config in UI, nothing else needed
                    }
                    SettingsAction::Save(config) => {
                        // Update local state from the saved config
                        self.scale_mode = config.video.scale_mode;
                        runner
                            .graphics_mut()
                            .set_scale_mode(config.video.scale_mode);
                        if let Some(window) = &self.window {
                            if config.video.fullscreen {
                                window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                            } else {
                                window.set_fullscreen(None);
                            }
                        }
                        if let Some(session) = runner.session_mut()
                            && let Some(audio) = session.runtime.audio_mut()
                        {
                            audio.set_master_volume(config.audio.master_volume);
                        }
                        // Update input manager with new keyboard mappings
                        self.input_manager.update_config(config.input.clone());
                        // Save to disk
                        if let Err(e) = super::config::save(&config) {
                            tracing::error!("Failed to save config: {}", e);
                        } else {
                            tracing::info!("Settings saved to config");
                        }
                    }
                }

                // Apply error actions
                match error_action.into_inner() {
                    ErrorAction::None => {}
                    ErrorAction::Restart => {
                        restart_requested = true;
                    }
                    ErrorAction::Quit => {
                        self.should_exit = true;
                    }
                }
            }

            runner
                .graphics()
                .queue()
                .submit(std::iter::once(encoder.finish()));
            surface_texture.present();

            // Process screen capture
            if self.capture.needs_capture() {
                let (width, height) = runner.graphics().render_target_dimensions();
                let pixels = read_render_target_pixels(
                    runner.graphics().device(),
                    runner.graphics().queue(),
                    runner.graphics().render_target_texture(),
                    width,
                    height,
                );
                self.capture.process_frame(pixels, width, height);
            }

            // Check for capture results
            if let Some(result) = self.capture.poll_save_result() {
                match result {
                    crate::capture::SaveResult::Screenshot(Ok(path)) => {
                        tracing::info!("Screenshot saved: {}", path.display());
                    }
                    crate::capture::SaveResult::Screenshot(Err(e)) => {
                        tracing::error!("Failed to save screenshot: {}", e);
                    }
                    crate::capture::SaveResult::Gif(Ok(path)) => {
                        tracing::info!("GIF saved: {}", path.display());
                    }
                    crate::capture::SaveResult::Gif(Err(e)) => {
                        tracing::error!("Failed to save GIF: {}", e);
                    }
                }
            }
        }

        if restart_requested {
            self.restart_game();
        }
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn mark_needs_redraw(&mut self) {
        self.needs_redraw = true;
    }

    fn clear_needs_redraw(&mut self) {
        self.needs_redraw = false;
    }

    fn on_runtime_error(&mut self, error: RuntimeError) {
        let tick = self
            .runner
            .as_ref()
            .and_then(|r| r.session())
            .and_then(|s| s.runtime.game())
            .map(|g| g.state().tick_count);

        let game_error = parse_wasm_error(
            &anyhow::anyhow!("{}", error.0),
            tick,
            GameErrorPhase::Update,
        );

        tracing::error!("Runtime error: {}", game_error);
        self.error_state = Some(game_error);
        self.needs_redraw = true;
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }

    fn request_exit(&mut self) {
        self.should_exit = true;
    }

    fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

/// Run a standalone player for the given console.
pub fn run_standalone<C, L>(config: StandaloneConfig) -> Result<()>
where
    C: Console + Clone,
    C::Graphics: StandaloneGraphicsSupport,
    L: RomLoader<Console = C>,
{
    let vram_limit = C::specs().vram_limit;
    let app = StandaloneApp::<C, L>::new(config, vram_limit);
    super::run(app)
}
