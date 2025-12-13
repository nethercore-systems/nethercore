//! Frame control for debug inspection
//!
//! Provides pause, step, and time scale controls for debugging games.

/// Preset time scale options for the UI
pub const TIME_SCALE_OPTIONS: [f32; 6] = [0.1, 0.25, 0.5, 1.0, 2.0, 4.0];

/// Frame controller for debug mode
///
/// Controls game execution timing: pause, step, and time scale.
/// This state lives in the GameSession, not in GameState (which is rollback-serialized).
#[derive(Debug, Clone)]
pub struct FrameController {
    /// Whether the game is paused
    paused: bool,
    /// Whether a single frame step was requested (consumed after one frame)
    step_requested: bool,
    /// Time scale multiplier (1.0 = normal, 0.5 = half speed, etc.)
    time_scale: f32,
    /// Index into TIME_SCALE_OPTIONS for the current time scale
    time_scale_index: usize,
    /// Whether debug features are disabled (e.g., during netplay)
    disabled: bool,
}

impl Default for FrameController {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameController {
    /// Create a new frame controller with default settings
    pub fn new() -> Self {
        Self {
            paused: false,
            step_requested: false,
            time_scale: 1.0,
            time_scale_index: 3, // Index of 1.0 in TIME_SCALE_OPTIONS
            disabled: false,
        }
    }

    /// Disable debug features (e.g., during netplay)
    ///
    /// When disabled, pause/step/time scale have no effect.
    pub fn disable(&mut self) {
        self.disabled = true;
        self.paused = false;
        self.step_requested = false;
        self.time_scale = 1.0;
        self.time_scale_index = 3;
    }

    /// Enable debug features
    pub fn enable(&mut self) {
        self.disabled = false;
    }

    /// Check if debug features are disabled
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Check if the game is paused
    pub fn is_paused(&self) -> bool {
        !self.disabled && self.paused
    }

    /// Get the current time scale
    pub fn time_scale(&self) -> f32 {
        if self.disabled { 1.0 } else { self.time_scale }
    }

    /// Toggle pause state
    pub fn toggle_pause(&mut self) {
        if self.disabled {
            return;
        }
        self.paused = !self.paused;
        if !self.paused {
            self.step_requested = false;
        }
    }

    /// Set pause state directly
    pub fn set_paused(&mut self, paused: bool) {
        if self.disabled {
            return;
        }
        self.paused = paused;
        if !paused {
            self.step_requested = false;
        }
    }

    /// Request a single frame step (only works when paused)
    pub fn request_step(&mut self) {
        if self.disabled {
            return;
        }
        if self.paused {
            self.step_requested = true;
        }
    }

    /// Decrease time scale to the previous preset
    pub fn decrease_time_scale(&mut self) {
        if self.disabled {
            return;
        }
        if self.time_scale_index > 0 {
            self.time_scale_index -= 1;
            self.time_scale = TIME_SCALE_OPTIONS[self.time_scale_index];
        }
    }

    /// Increase time scale to the next preset
    pub fn increase_time_scale(&mut self) {
        if self.disabled {
            return;
        }
        if self.time_scale_index < TIME_SCALE_OPTIONS.len() - 1 {
            self.time_scale_index += 1;
            self.time_scale = TIME_SCALE_OPTIONS[self.time_scale_index];
        }
    }

    /// Set time scale directly (snaps to nearest preset)
    pub fn set_time_scale(&mut self, scale: f32) {
        if self.disabled {
            return;
        }
        // Find nearest preset
        let mut best_idx = 0;
        let mut best_diff = f32::MAX;
        for (i, &preset) in TIME_SCALE_OPTIONS.iter().enumerate() {
            let diff = (preset - scale).abs();
            if diff < best_diff {
                best_diff = diff;
                best_idx = i;
            }
        }
        self.time_scale_index = best_idx;
        self.time_scale = TIME_SCALE_OPTIONS[best_idx];
    }

    /// Get the time scale index (for UI)
    pub fn time_scale_index(&self) -> usize {
        self.time_scale_index
    }

    /// Check if a game tick should run this frame
    ///
    /// Returns true if the game should update. Consumes the step request if one was pending.
    pub fn should_run_tick(&mut self) -> bool {
        if self.disabled {
            return true;
        }

        if self.paused {
            if self.step_requested {
                self.step_requested = false;
                return true;
            }
            return false;
        }

        true
    }

    /// Get the effective delta time, accounting for time scale
    pub fn get_effective_delta(&self, base_delta: f32) -> f32 {
        if self.disabled {
            base_delta
        } else {
            base_delta * self.time_scale
        }
    }

    /// Reset controller to default state
    pub fn reset(&mut self) {
        self.paused = false;
        self.step_requested = false;
        self.time_scale = 1.0;
        self.time_scale_index = 3;
        // Note: `disabled` is not reset - that's controlled externally
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let fc = FrameController::new();
        assert!(!fc.is_paused());
        assert_eq!(fc.time_scale(), 1.0);
        assert!(!fc.is_disabled());
    }

    #[test]
    fn test_pause_toggle() {
        let mut fc = FrameController::new();
        assert!(!fc.is_paused());

        fc.toggle_pause();
        assert!(fc.is_paused());

        fc.toggle_pause();
        assert!(!fc.is_paused());
    }

    #[test]
    fn test_step_when_paused() {
        let mut fc = FrameController::new();
        fc.set_paused(true);

        // When paused, should_run_tick returns false
        assert!(!fc.should_run_tick());

        // Request a step
        fc.request_step();
        assert!(fc.should_run_tick()); // Consumes the step
        assert!(!fc.should_run_tick()); // No more steps pending
    }

    #[test]
    fn test_step_when_not_paused() {
        let mut fc = FrameController::new();

        // Step request does nothing when not paused
        fc.request_step();
        assert!(fc.should_run_tick()); // Always runs when not paused
    }

    #[test]
    fn test_time_scale() {
        let mut fc = FrameController::new();
        assert_eq!(fc.time_scale(), 1.0);

        fc.decrease_time_scale();
        assert_eq!(fc.time_scale(), 0.5);

        fc.decrease_time_scale();
        assert_eq!(fc.time_scale(), 0.25);

        fc.increase_time_scale();
        assert_eq!(fc.time_scale(), 0.5);

        // Go to max
        fc.set_time_scale(4.0);
        assert_eq!(fc.time_scale(), 4.0);

        // Can't go higher
        fc.increase_time_scale();
        assert_eq!(fc.time_scale(), 4.0);
    }

    #[test]
    fn test_effective_delta() {
        let mut fc = FrameController::new();
        let base_delta = 1.0 / 60.0;

        assert_eq!(fc.get_effective_delta(base_delta), base_delta);

        fc.set_time_scale(0.5);
        assert_eq!(fc.get_effective_delta(base_delta), base_delta * 0.5);

        fc.set_time_scale(2.0);
        assert_eq!(fc.get_effective_delta(base_delta), base_delta * 2.0);
    }

    #[test]
    fn test_disabled_mode() {
        let mut fc = FrameController::new();
        fc.set_paused(true);
        fc.set_time_scale(0.5);

        // Disable debug features
        fc.disable();

        // All controls should be ignored
        assert!(!fc.is_paused());
        assert_eq!(fc.time_scale(), 1.0);
        assert!(fc.should_run_tick());

        // Pause/step/scale changes should be ignored
        fc.toggle_pause();
        assert!(!fc.is_paused());

        fc.request_step();
        assert!(fc.should_run_tick()); // Still just returns true

        fc.set_time_scale(0.1);
        assert_eq!(fc.time_scale(), 1.0);

        // Re-enable
        fc.enable();
        fc.set_paused(true);
        assert!(fc.is_paused());
    }

    #[test]
    fn test_set_time_scale_snaps() {
        let mut fc = FrameController::new();

        fc.set_time_scale(0.3); // Between 0.25 and 0.5, snaps to 0.25
        assert_eq!(fc.time_scale(), 0.25);

        fc.set_time_scale(0.4); // Closer to 0.5
        assert_eq!(fc.time_scale(), 0.5);

        fc.set_time_scale(3.1); // Closer to 4.0 than 2.0
        assert_eq!(fc.time_scale(), 4.0);
    }

    #[test]
    fn test_reset() {
        let mut fc = FrameController::new();
        fc.set_paused(true);
        fc.set_time_scale(0.5);
        fc.request_step();

        fc.reset();

        assert!(!fc.is_paused());
        assert_eq!(fc.time_scale(), 1.0);
    }
}
