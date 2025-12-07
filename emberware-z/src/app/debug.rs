//! Debug statistics and FPS calculation

use super::App;

impl App {
    /// Calculate render FPS from frame times
    pub(super) fn calculate_fps(&self) -> f32 {
        if self.frame_times.len() < 2 {
            return 0.0;
        }
        let elapsed = self
            .frame_times
            .last()
            .unwrap()
            .duration_since(*self.frame_times.first().unwrap())
            .as_secs_f32();
        if elapsed > 0.0 {
            self.frame_times.len() as f32 / elapsed
        } else {
            0.0
        }
    }

    /// Calculate game tick FPS (actual update rate)
    pub(super) fn calculate_game_tick_fps(&self) -> f32 {
        if self.game_tick_times.len() < 2 {
            return 0.0;
        }
        let elapsed = self
            .game_tick_times
            .last()
            .unwrap()
            .duration_since(*self.game_tick_times.first().unwrap())
            .as_secs_f32();
        if elapsed > 0.0 {
            self.game_tick_times.len() as f32 / elapsed
        } else {
            0.0
        }
    }
}
