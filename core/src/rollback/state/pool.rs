//! Pre-allocated buffer pool for rollback state saves

use super::STATE_POOL_SIZE;
use crate::rollback::config::MAX_STATE_SIZE;

/// Pre-allocated buffer pool for rollback state saves
///
/// Avoids allocations in the hot path during rollback. GGRS may need to
/// save/load state multiple times per frame during rollback, so this is
/// critical for performance.
pub struct StatePool {
    /// Pool of reusable buffers
    buffers: Vec<Vec<u8>>,
    /// Size each buffer was pre-allocated to
    buffer_size: usize,
}

impl StatePool {
    /// Create a new state pool with pre-allocated buffers
    pub fn new(buffer_size: usize, pool_size: usize) -> Self {
        let buffers = (0..pool_size)
            .map(|_| Vec::with_capacity(buffer_size))
            .collect();
        Self {
            buffers,
            buffer_size,
        }
    }

    /// Create a pool with default settings
    pub fn with_defaults() -> Self {
        Self::new(MAX_STATE_SIZE, STATE_POOL_SIZE)
    }

    /// Acquire a buffer from the pool
    ///
    /// Returns a buffer with capacity >= buffer_size.
    /// If pool is empty, allocates a new buffer (should be rare in steady state).
    pub fn acquire(&mut self) -> Vec<u8> {
        self.buffers.pop().unwrap_or_else(|| {
            tracing::warn!("StatePool exhausted, allocating new buffer");
            Vec::with_capacity(self.buffer_size)
        })
    }

    /// Return a buffer to the pool
    ///
    /// The buffer is cleared but retains its capacity for reuse.
    pub fn release(&mut self, mut buffer: Vec<u8>) {
        buffer.clear();
        // Only keep buffers that haven't grown too large
        if buffer.capacity() <= self.buffer_size * 2 {
            self.buffers.push(buffer);
        }
    }

    /// Number of available buffers in the pool
    pub fn available(&self) -> usize {
        self.buffers.len()
    }
}

impl Default for StatePool {
    fn default() -> Self {
        Self::with_defaults()
    }
}
