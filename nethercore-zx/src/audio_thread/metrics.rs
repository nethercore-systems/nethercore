//! Audio thread health monitoring and diagnostics

use tracing::debug;

/// Ring buffer capacity (must match RING_BUFFER_SIZE in output.rs)
pub(super) const RING_BUFFER_CAPACITY: usize = 13230;
/// Generate more when buffer drops below this (~35%)
pub(super) const LOW_BUFFER_THRESHOLD: usize = 4600;

/// Metrics for audio thread health monitoring and diagnostics
#[derive(Debug, Clone)]
pub(super) struct AudioMetrics {
    /// Total frames generated
    pub frames_generated: u64,
    /// Total audio samples generated (stereo pairs counted as 2)
    pub samples_generated: u64,
    /// Total snapshots received from main thread
    pub snapshots_received: u64,
    /// Rollback snapshots processed
    pub rollbacks_processed: u64,
    /// Current buffer fill level (samples)
    pub buffer_fill: usize,
    /// Minimum buffer fill level seen
    pub buffer_fill_min: usize,
    /// Maximum buffer fill level seen
    pub buffer_fill_max: usize,
    /// Number of times buffer dropped below LOW threshold
    pub buffer_underruns: u64,
    /// Number of times buffer filled above TARGET (dropped samples)
    pub buffer_overruns: u64,
    /// Average time to generate one frame (microseconds)
    pub avg_generation_time_us: f64,
    /// Number of sample discontinuities detected (>0.3 amplitude jump)
    pub discontinuities: u64,
    /// Timestamp of last metrics log
    pub last_log_time: std::time::Instant,
}

impl AudioMetrics {
    pub fn new() -> Self {
        Self {
            frames_generated: 0,
            samples_generated: 0,
            snapshots_received: 0,
            rollbacks_processed: 0,
            buffer_fill: 0,
            buffer_fill_min: RING_BUFFER_CAPACITY,
            buffer_fill_max: 0,
            buffer_underruns: 0,
            buffer_overruns: 0,
            avg_generation_time_us: 0.0,
            discontinuities: 0,
            last_log_time: std::time::Instant::now(),
        }
    }

    /// Log metrics if enough time has passed (every 1 second)
    pub fn maybe_log(&mut self) {
        let elapsed = self.last_log_time.elapsed();
        if elapsed.as_secs() >= 1 {
            let buffer_pct = (self.buffer_fill as f64 / RING_BUFFER_CAPACITY as f64) * 100.0;
            let buffer_min_pct =
                (self.buffer_fill_min as f64 / RING_BUFFER_CAPACITY as f64) * 100.0;
            let buffer_max_pct =
                (self.buffer_fill_max as f64 / RING_BUFFER_CAPACITY as f64) * 100.0;
            let buffer_range = self.buffer_fill_max.saturating_sub(self.buffer_fill_min);

            debug!(
                "🎵 AUDIO METRICS [tid={:?}]: buf={:.1}% (min={:.1}%, max={:.1}%, range={}), \
                 frames={}, samples={}, underruns={}, overruns={}, \
                 discontinuities={}, avg_gen={:.2}μs",
                std::thread::current().id(),
                buffer_pct,
                buffer_min_pct,
                buffer_max_pct,
                buffer_range,
                self.frames_generated,
                self.samples_generated,
                self.buffer_underruns,
                self.buffer_overruns,
                self.discontinuities,
                self.avg_generation_time_us
            );

            // Reset counters for next interval (show per-second rates)
            self.frames_generated = 0;
            self.samples_generated = 0;
            self.buffer_underruns = 0;
            self.buffer_overruns = 0;
            self.discontinuities = 0;
            self.buffer_fill_min = self.buffer_fill;
            self.buffer_fill_max = self.buffer_fill;
            self.last_log_time = std::time::Instant::now();
        }
    }

    /// Update buffer fill metrics
    pub fn update_buffer_fill(&mut self, fill: usize) {
        self.buffer_fill = fill;
        self.buffer_fill_min = self.buffer_fill_min.min(fill);
        self.buffer_fill_max = self.buffer_fill_max.max(fill);
    }
}
