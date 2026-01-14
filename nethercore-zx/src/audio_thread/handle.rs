//! Audio generation thread handle
//!
//! Provides a handle for sending snapshots to the audio thread and managing its lifecycle.

use std::sync::mpsc::{SyncSender, TrySendError};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

use tracing::{debug, warn};

use super::snapshot::AudioGenSnapshot;

/// Handle to the audio generation thread
///
/// Returned from `AudioGenThread::spawn()`. Use this to send snapshots
/// to the audio thread and to shut down cleanly.
pub struct AudioGenHandle {
    /// Sender for snapshots to audio thread (Option to allow explicit drop before join)
    pub(super) tx: Option<SyncSender<AudioGenSnapshot>>,

    /// Thread join handle
    pub(super) handle: Option<JoinHandle<()>>,

    /// Condition variable for signaling from cpal callback
    /// Shared with audio thread - cpal notifies when buffer space available
    pub(crate) condvar: Arc<(Mutex<bool>, Condvar)>,
}

impl AudioGenHandle {
    /// Send a snapshot to the audio generation thread
    ///
    /// Non-blocking - if the channel is full, the snapshot is dropped
    /// and a warning is logged. This prevents the main thread from
    /// blocking on audio generation.
    pub fn send_snapshot(&self, snapshot: AudioGenSnapshot) -> bool {
        let Some(ref tx) = self.tx else {
            warn!("Audio thread sender already dropped");
            return false;
        };
        match tx.try_send(snapshot) {
            Ok(()) => true,
            Err(TrySendError::Full(_)) => {
                debug!("Audio snapshot channel full, dropping frame");
                false
            }
            Err(TrySendError::Disconnected(_)) => {
                warn!("Audio thread disconnected");
                false
            }
        }
    }

    /// Check if the audio thread is still running
    pub fn is_alive(&self) -> bool {
        self.handle
            .as_ref()
            .map(|h| !h.is_finished())
            .unwrap_or(false)
    }
}

impl Drop for AudioGenHandle {
    fn drop(&mut self) {
        // IMPORTANT: Drop the sender FIRST to signal the thread to exit.
        // The thread's recv_timeout() will return Disconnected and break the loop.
        // If we join() before dropping the sender, we deadlock!
        drop(self.tx.take());

        if let Some(handle) = self.handle.take() {
            // Now wait for the thread to finish
            let _ = handle.join();
        }
    }
}
