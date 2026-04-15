//! Generic scripted sync-test runner.

use anyhow::{Result, bail};

use crate::console::Console;
use crate::rollback::{RollbackSession, SessionConfig};

use super::Runtime;

/// Configuration for a deterministic scripted sync-test run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScriptedSyncTestConfig {
    /// Number of input samples to submit.
    pub frames: u32,
    /// Number of players in the sync-test session.
    pub players: usize,
    /// GGRS input delay in frames.
    pub input_delay: usize,
    /// GGRS sync-test checksum distance in frames.
    pub check_distance: usize,
}

impl ScriptedSyncTestConfig {
    /// Create a sync-test config using the default checksum distance.
    pub fn new(frames: u32, players: usize) -> Self {
        Self {
            frames,
            players,
            input_delay: 0,
            check_distance: SessionConfig::default().check_distance,
        }
    }
}

/// Summary of a scripted sync-test run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScriptedSyncTestReport {
    /// Number of input samples submitted by the harness.
    pub input_frames: u32,
    /// Number of simulation advances executed, including rollback replays.
    pub simulated_frames: u32,
    /// Number of simulation advances executed while rolling back.
    pub rollback_frames: u64,
    /// Session frame counter after the run.
    pub final_session_frame: i32,
}

impl<C: Console> Runtime<C> {
    /// Run a deterministic sync-test sequence against the currently loaded game.
    ///
    /// This is console-generic: callers provide a native input value for each
    /// `(frame, player)` pair, and the core runtime handles GGRS input
    /// submission, rollback request ordering, and fixed-step advancement. The
    /// method does not call `render()`.
    pub fn run_scripted_sync_test<F>(
        &mut self,
        config: ScriptedSyncTestConfig,
        mut input_for_frame: F,
    ) -> Result<ScriptedSyncTestReport>
    where
        F: FnMut(u32, usize) -> C::Input,
    {
        if self.game.is_none() {
            bail!("scripted sync-test requires a loaded game");
        }
        if !(1..=4).contains(&config.players) {
            bail!("scripted sync-test player count must be between 1 and 4");
        }

        let session_config =
            SessionConfig::sync_test_with_params(config.players, config.input_delay)
                .with_check_distance(config.check_distance);
        let session = RollbackSession::new_sync_test(session_config, C::specs().ram_limit)?;
        self.set_session(session);

        let mut simulated_frames = 0u32;

        for frame in 0..config.frames {
            for player in 0..config.players {
                let input = input_for_frame(frame, player);

                if let Some(game) = self.game.as_mut() {
                    game.set_input(player, input);
                }

                self.add_local_input(player, input)?;
            }

            let (ticks, _alpha) = self.replay_step()?;
            if ticks == 0 {
                bail!("scripted sync-test advanced no frames at input frame {frame}");
            }
            simulated_frames = simulated_frames.saturating_add(ticks);
        }

        let session = self
            .session
            .as_ref()
            .expect("scripted sync-test session should be installed");

        Ok(ScriptedSyncTestReport {
            input_frames: config.frames,
            simulated_frames,
            rollback_frames: session.total_rollback_frames(),
            final_session_frame: session.current_frame(),
        })
    }
}
