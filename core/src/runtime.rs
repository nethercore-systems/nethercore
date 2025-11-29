//! Game loop orchestration
//!
//! Manages the main game loop with fixed timestep updates
//! and variable render rate.

use std::time::{Duration, Instant};

use anyhow::Result;

use crate::console::Console;
use crate::wasm::GameInstance;

/// Runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Target tick rate in Hz
    pub tick_rate: u32,
    /// Maximum delta time clamp (prevents spiral of death)
    pub max_delta: Duration,
    /// CPU budget warning threshold per tick
    pub cpu_budget: Duration,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            tick_rate: 60,
            max_delta: Duration::from_millis(100),
            cpu_budget: Duration::from_micros(4000), // 4ms at 60fps
        }
    }
}

/// Main runtime managing game execution
///
/// Generic over the console type to support different fantasy consoles
/// while sharing the core game loop and rollback infrastructure.
pub struct Runtime<C: Console> {
    #[allow(dead_code)]
    console: C,
    config: RuntimeConfig,
    game: Option<GameInstance>,
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

    /// Load a game instance
    pub fn load_game(&mut self, game: GameInstance) {
        self.game = Some(game);
        self.accumulator = Duration::ZERO;
        self.last_update = None;
    }

    /// Initialize the loaded game
    pub fn init_game(&mut self) -> Result<()> {
        if let Some(game) = &mut self.game {
            game.init()?;
        }
        Ok(())
    }

    /// Run a single frame (may include multiple ticks)
    ///
    /// Returns the number of ticks that were executed and the interpolation factor
    /// for rendering between the last two states.
    pub fn frame(&mut self) -> Result<(u32, f32)> {
        let now = Instant::now();

        // Calculate delta time
        let delta = if let Some(last) = self.last_update {
            let d = now - last;
            if d > self.config.max_delta {
                self.config.max_delta
            } else {
                d
            }
        } else {
            self.tick_duration
        };
        self.last_update = Some(now);

        self.accumulator += delta;

        // Run fixed timestep updates
        let mut ticks = 0u32;
        while self.accumulator >= self.tick_duration {
            let tick_start = Instant::now();

            if let Some(game) = &mut self.game {
                game.update(self.tick_duration.as_secs_f32())?;
            }

            self.accumulator -= self.tick_duration;
            ticks += 1;

            // Check CPU budget
            let tick_time = tick_start.elapsed();
            if tick_time > self.config.cpu_budget {
                log::warn!(
                    "Tick took {:?}, exceeds budget of {:?}",
                    tick_time,
                    self.config.cpu_budget
                );
            }
        }

        // Calculate interpolation factor for rendering
        let alpha = self.accumulator.as_secs_f32() / self.tick_duration.as_secs_f32();

        Ok((ticks, alpha))
    }

    /// Render the current frame
    pub fn render(&mut self) -> Result<()> {
        if let Some(game) = &mut self.game {
            game.render()?;
        }
        Ok(())
    }

    /// Get a reference to the loaded game
    pub fn game(&self) -> Option<&GameInstance> {
        self.game.as_ref()
    }

    /// Get a mutable reference to the loaded game
    pub fn game_mut(&mut self) -> Option<&mut GameInstance> {
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
}
