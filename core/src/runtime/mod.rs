//! Game loop orchestration
//!
//! Manages the main game loop with fixed timestep updates
//! and variable render rate.

use std::time::{Duration, Instant};

use anyhow::Result;
use ggrs::GgrsError;

use crate::console::Console;
use crate::rollback::{RollbackSession, SessionEvent};
use crate::wasm::GameInstance;

mod config;
mod game_loop;

#[cfg(test)]
mod tests;

pub use config::RuntimeConfig;

/// Tuple of mutable references to game instance and audio, where either can be None
type GameAndAudioMut<'a, C> = (
    Option<
        &'a mut GameInstance<
            <C as Console>::Input,
            <C as Console>::State,
            <C as Console>::RollbackState,
        >,
    >,
    Option<&'a mut <C as Console>::Audio>,
);

/// Main runtime managing game execution
///
/// Generic over the console type to support different fantasy consoles
/// while sharing the core game loop and rollback infrastructure.
pub struct Runtime<C: Console> {
    /// The console implementation.
    /// Used for console-specific FFI state initialization and ownership of the console
    /// instance for the runtime's lifetime.
    console: C,
    config: RuntimeConfig,
    game: Option<GameInstance<C::Input, C::State, C::RollbackState>>,
    session: Option<RollbackSession<C::Input, C::State, C::RollbackState>>,
    audio: Option<C::Audio>,
    accumulator: Duration,
    last_update: Option<Instant>,
    tick_duration: Duration,
}

impl<C: Console> Runtime<C> {
    /// Create a new runtime for the given console
    pub fn new(console: C) -> Self {
        let config = RuntimeConfig::default();
        let tick_duration = Duration::from_secs_f64(1.0 / config.tick_rate as f64);

        Self {
            console,
            config,
            game: None,
            session: None,
            audio: None,
            accumulator: Duration::ZERO,
            last_update: None,
            tick_duration,
        }
    }

    /// Set the tick rate
    pub fn set_tick_rate(&mut self, tick_rate: u32) {
        self.config.tick_rate = tick_rate;
        self.tick_duration = Duration::from_secs_f64(1.0 / tick_rate as f64);
    }

    /// Get the tick duration (time per tick, inverse of tick rate)
    pub fn tick_duration(&self) -> Duration {
        self.tick_duration
    }

    /// Load a game instance
    pub fn load_game(&mut self, game: GameInstance<C::Input, C::State, C::RollbackState>) {
        self.game = Some(game);
        self.accumulator = Duration::ZERO;
        self.last_update = None;
    }

    /// Set the rollback session
    pub fn set_session(&mut self, session: RollbackSession<C::Input, C::State, C::RollbackState>) {
        self.session = Some(session);
    }

    /// Set the audio backend
    pub fn set_audio(&mut self, audio: C::Audio) {
        self.audio = Some(audio);
    }

    /// Initialize console-specific FFI state before calling game init()
    ///
    /// This allows the console to set up state that the game needs during
    /// initialization (e.g., datapack for rom_* functions).
    pub fn initialize_console_state(&mut self) {
        if let Some(game) = &mut self.game {
            self.console.initialize_ffi_state(game.console_state_mut());
        }
    }

    /// Initialize the loaded game
    pub fn init_game(&mut self) -> Result<()> {
        if let Some(game) = &mut self.game {
            game.init()?;
        }
        Ok(())
    }

    /// Add local input for a player
    ///
    /// Input should be added before calling `frame()` each render loop.
    pub fn add_local_input(
        &mut self,
        player_handle: usize,
        input: C::Input,
    ) -> Result<(), GgrsError> {
        if let Some(session) = &mut self.session {
            session.add_local_input(player_handle, input)?;
        }
        Ok(())
    }

    /// Poll remote clients (for P2P sessions)
    ///
    /// Should be called regularly, typically at the start of each frame.
    pub fn poll_remote_clients(&mut self) {
        if let Some(session) = &mut self.session {
            session.poll_remote_clients();
        }
    }

    /// Handle session events and return them for the application to process
    ///
    /// Should be called once per frame to get network events, desync warnings, etc.
    pub fn handle_session_events(&mut self) -> Vec<SessionEvent> {
        if let Some(session) = &mut self.session {
            session.handle_events()
        } else {
            Vec::new()
        }
    }

    /// Run a single frame (may include multiple ticks)
    ///
    /// Returns the number of ticks that were executed and the interpolation factor
    /// for rendering between the last two states.
    pub fn frame(&mut self) -> Result<(u32, f32)> {
        self.frame_with_time_scale(1.0)
    }

    /// Run a frame with a time scale modifier.
    ///
    /// Time scale affects how fast game time passes:
    /// - 1.0 = normal speed
    /// - 0.5 = half speed (slow motion)
    /// - 2.0 = double speed (fast forward)
    ///
    /// Returns the number of ticks that were executed and the interpolation factor
    /// for rendering between the last two states.
    pub fn frame_with_time_scale(&mut self, time_scale: f32) -> Result<(u32, f32)> {
        game_loop::execute_frame::<C>(
            &self.config,
            self.tick_duration,
            &mut self.accumulator,
            &mut self.last_update,
            &mut self.game,
            &mut self.session,
            time_scale,
        )
    }

    /// Render the current frame
    pub fn render(&mut self) -> Result<()> {
        if let Some(game) = &mut self.game {
            game.render()?;
        }
        Ok(())
    }

    /// Get a reference to the loaded game
    pub fn game(&self) -> Option<&GameInstance<C::Input, C::State, C::RollbackState>> {
        self.game.as_ref()
    }

    /// Get a mutable reference to the loaded game
    pub fn game_mut(&mut self) -> Option<&mut GameInstance<C::Input, C::State, C::RollbackState>> {
        self.game.as_mut()
    }

    /// Get the current tick rate
    pub fn tick_rate(&self) -> u32 {
        self.config.tick_rate
    }

    /// Get the console
    pub fn console(&self) -> &C {
        &self.console
    }

    /// Get mutable reference to the console
    pub fn console_mut(&mut self) -> &mut C {
        &mut self.console
    }

    /// Get a reference to the rollback session
    pub fn session(&self) -> Option<&RollbackSession<C::Input, C::State, C::RollbackState>> {
        self.session.as_ref()
    }

    /// Get a mutable reference to the rollback session
    pub fn session_mut(
        &mut self,
    ) -> Option<&mut RollbackSession<C::Input, C::State, C::RollbackState>> {
        self.session.as_mut()
    }

    /// Get a reference to the audio backend
    pub fn audio(&self) -> Option<&C::Audio> {
        self.audio.as_ref()
    }

    /// Get a mutable reference to the audio backend
    pub fn audio_mut(&mut self) -> Option<&mut C::Audio> {
        self.audio.as_mut()
    }

    /// Get mutable references to both game and audio for audio processing
    ///
    /// Returns (game, audio) tuple where either can be None.
    /// This allows the caller to access both without borrowing issues.
    pub fn game_and_audio_mut(&mut self) -> GameAndAudioMut<'_, C> {
        (self.game.as_mut(), self.audio.as_mut())
    }

    /// Get mutable references to console and game state for debug UI syncing.
    ///
    /// Returns (console, Option<state>) where state is the game's console state if loaded.
    /// This allows debug UI to sync state between the console and game.
    pub fn console_and_state_mut(&mut self) -> (&mut C, Option<&mut C::State>) {
        let state = self
            .game
            .as_mut()
            .map(|game| game.console_state_mut());
        (&mut self.console, state)
    }
}
