//! Runtime configuration

use std::time::Duration;

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
