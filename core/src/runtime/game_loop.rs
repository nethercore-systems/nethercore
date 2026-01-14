//! Game loop execution with fixed timestep

use std::time::{Duration, Instant};

use anyhow::Result;
use ggrs::SessionState;

use crate::console::Console;
use crate::rollback::RollbackSession;
use crate::wasm::GameInstance;

use super::RuntimeConfig;

/// Execute a single frame with optional time scaling
///
/// This implements a fixed timestep game loop with variable render rate.
/// Returns the number of ticks executed and interpolation factor for rendering.
pub fn execute_frame<C: Console>(
    config: &RuntimeConfig,
    tick_duration: Duration,
    accumulator: &mut Duration,
    last_update: &mut Option<Instant>,
    game: &mut Option<GameInstance<C::Input, C::State, C::RollbackState>>,
    session: &mut Option<RollbackSession<C::Input, C::State, C::RollbackState>>,
    time_scale: f32,
) -> Result<(u32, f32)> {
    let now = Instant::now();

    // Calculate delta time
    let delta = if let Some(last) = *last_update {
        let d = now - last;
        if d > config.max_delta {
            config.max_delta
        } else {
            d
        }
    } else {
        tick_duration
    };
    *last_update = Some(now);

    // Apply time scale to delta before accumulating
    let scaled_delta = delta.mul_f32(time_scale.max(0.0));
    *accumulator += scaled_delta;

    // Run fixed timestep updates
    let mut ticks = 0u32;

    // If we have a rollback session, use GGRS
    if let Some(session) = session {
        // Check if P2P session is still synchronizing
        if let Some(state) = session.session_state()
            && state != SessionState::Running
        {
            // Session is still synchronizing - poll it but don't advance frames
            session.poll_remote_clients();
            // Reset accumulator to prevent catchup burst when session starts
            *accumulator = Duration::ZERO;
            // Return 0 ticks and 0.0 interpolation (no game state to interpolate)
            return Ok((0, 0.0));
        }

        // For P2P sessions, only advance once per render frame to match input cadence
        // (we only add input once per run_game_frame call)
        let is_p2p = session.session_state().is_some();

        while *accumulator >= tick_duration {
            let tick_start = Instant::now();

            // Advance GGRS frame and get requests
            let requests = session
                .advance_frame()
                .map_err(|e| anyhow::anyhow!("GGRS advance_frame failed: {}", e))?;

            // Handle all requests (SaveGameState, LoadGameState, AdvanceFrame)
            if let Some(game) = game {
                let advance_inputs = session
                    .handle_requests(game, requests)
                    .map_err(|e| anyhow::anyhow!("GGRS handle_requests failed: {}", e))?;

                // Note: Audio rollback is automatic via ConsoleRollbackState
                // Audio state is part of snapshot, no explicit mode tracking needed

                // Execute each AdvanceFrame with its inputs
                for inputs in advance_inputs {
                    // Set inputs in GameState for FFI access
                    // Each entry is (input, status) for one player
                    for (player_idx, (input, _status)) in inputs.iter().enumerate() {
                        game.set_input(player_idx, *input);
                    }
                    game.update(tick_duration.as_secs_f32())?;
                    ticks += 1;
                }
            }

            *accumulator -= tick_duration;

            // Check CPU budget
            let tick_time = tick_start.elapsed();
            if tick_time > config.cpu_budget {
                tracing::warn!(
                    "Tick took {:?}, exceeds budget of {:?}",
                    tick_time,
                    config.cpu_budget
                );
            }

            // For P2P sessions, only advance once per render frame
            // We only receive one input per run_game_frame call, so we can't
            // advance multiple times without new input
            if is_p2p {
                // Clamp remaining accumulator to prevent runaway catchup
                if *accumulator > tick_duration {
                    *accumulator = tick_duration;
                }
                break;
            }
        }
    } else {
        // No rollback session, run normally
        while *accumulator >= tick_duration {
            let tick_start = Instant::now();

            if let Some(game) = game {
                game.update(tick_duration.as_secs_f32())?;
            }

            *accumulator -= tick_duration;
            ticks += 1;

            // Check CPU budget
            let tick_time = tick_start.elapsed();
            if tick_time > config.cpu_budget {
                tracing::warn!(
                    "Tick took {:?}, exceeds budget of {:?}",
                    tick_time,
                    config.cpu_budget
                );
            }
        }
    }

    // Calculate interpolation factor for rendering
    let alpha = accumulator.as_secs_f32() / tick_duration.as_secs_f32();

    Ok((ticks, alpha))
}
