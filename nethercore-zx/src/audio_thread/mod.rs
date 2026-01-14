//! Threaded audio generation
//!
//! Decouples CPU-intensive sample generation from the main game loop.
//! This prevents audio pops/crackles during system load or rollback replays.
//!
//! # Architecture
//!
//! ```text
//! Main Thread                    Audio Gen Thread              cpal Thread
//!     │                                │                           │
//! [Game Tick]                          │                           │
//!     │                                │                           │
//! [Create Snapshot]────(channel)────►[Receive]                     │
//!     │                              [Generate Samples]            │
//!     │                              [Push]─────────(ring)──────►[Consume]
//! ```
//!
//! # Usage
//!
//! ```ignore
//! // Create audio output with threaded generation
//! let audio = ThreadedAudioOutput::new()?;
//!
//! // Each frame, send a snapshot
//! let snapshot = AudioGenSnapshot::new(...);
//! audio.send_snapshot(snapshot);
//! ```

mod handle;
mod metrics;
mod output;
mod snapshot;
mod thread;

// Re-export public API
pub use output::ThreadedAudioOutput;
pub use snapshot::AudioGenSnapshot;

// Re-export for internal use by tests
#[cfg(test)]
use thread::AudioGenThread;

#[cfg(test)]
mod tests;
