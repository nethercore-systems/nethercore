//! GGRS rollback integration
//!
//! Provides the configuration and state management for GGRS rollback netcode.

use bytemuck::{Pod, Zeroable};
use ggrs::Config;

use crate::console::ConsoleInput;

/// Maximum rollback frames
pub const MAX_ROLLBACK_FRAMES: usize = 8;

/// Maximum input delay
pub const MAX_INPUT_DELAY: usize = 10;

/// GGRS configuration for Emberware
pub struct EmberwareConfig<I: ConsoleInput> {
    _phantom: std::marker::PhantomData<I>,
}

impl<I: ConsoleInput> Config for EmberwareConfig<I> {
    type Input = I;
    type State = GameStateSnapshot;
    type Address = String; // WebRTC peer address
}

/// Snapshot of game state for rollback
#[derive(Clone)]
pub struct GameStateSnapshot {
    /// Serialized WASM game state
    pub data: Vec<u8>,
    /// Checksum for desync detection
    pub checksum: u64,
}

impl GameStateSnapshot {
    /// Create a new empty snapshot
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            checksum: 0,
        }
    }

    /// Create a snapshot from serialized data
    pub fn from_data(data: Vec<u8>) -> Self {
        let checksum = Self::compute_checksum(&data);
        Self { data, checksum }
    }

    /// Compute FNV-1a checksum
    fn compute_checksum(data: &[u8]) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in data {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }
}

impl Default for GameStateSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

/// Session type for GGRS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    /// Local session (no rollback, single machine)
    Local,
    /// Sync test session (local with rollback for testing)
    SyncTest,
    /// P2P session with rollback netcode
    P2P,
}

/// Rollback session manager
pub struct RollbackSession<I: ConsoleInput> {
    session_type: SessionType,
    _phantom: std::marker::PhantomData<I>,
}

impl<I: ConsoleInput> RollbackSession<I> {
    /// Create a new local session (no rollback)
    pub fn new_local() -> Self {
        Self {
            session_type: SessionType::Local,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get the session type
    pub fn session_type(&self) -> SessionType {
        self.session_type
    }
}

/// Wrapper type to implement Pod + Zeroable for generic inputs
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct NetworkInput<I: ConsoleInput> {
    pub input: I,
}

// Safety: I is required to be Pod + Zeroable by ConsoleInput
unsafe impl<I: ConsoleInput> Pod for NetworkInput<I> {}
unsafe impl<I: ConsoleInput> Zeroable for NetworkInput<I> {}
