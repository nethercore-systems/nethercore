//! Core types for rollback sessions

use bytemuck::{Pod, Zeroable};
use ggrs::SyncTestSession;

use crate::console::ConsoleInput;

use super::super::config::NethercoreConfig;

// ============================================================================
// Session Types
// ============================================================================

/// Session type for GGRS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    /// Local session (no rollback, single machine)
    Local,
    /// Sync test session (local with rollback for testing determinism)
    SyncTest,
    /// P2P session with rollback netcode
    P2P,
}

// ============================================================================
// Network Input Wrapper
// ============================================================================

/// Wrapper type to implement Pod + Zeroable for generic inputs
///
/// GGRS requires inputs to be POD (Plain Old Data) for network serialization.
/// This wrapper ensures the generic input type satisfies those requirements.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct NetworkInput<I: ConsoleInput> {
    /// The console-specific input data
    pub input: I,
}

impl<I: ConsoleInput> NetworkInput<I> {
    /// Create a new network input wrapper
    pub fn new(input: I) -> Self {
        Self { input }
    }
}

// SAFETY: I is required to be Pod + Zeroable by ConsoleInput trait bounds.
// NetworkInput is a #[repr(transparent)] wrapper, so it has the same layout as I.
unsafe impl<I: ConsoleInput> Pod for NetworkInput<I> {}
unsafe impl<I: ConsoleInput> Zeroable for NetworkInput<I> {}

// ============================================================================
// GGRS Session Wrapper
// ============================================================================

/// Inner session types for different modes
///
/// Note: SyncTest and P2P variants are boxed to reduce overall enum size,
/// as their GGRS sessions are significantly larger than the Local variant.
pub(super) enum SessionInner<I: ConsoleInput> {
    /// Local session - no GGRS, just direct execution
    Local {
        current_frame: i32,
        /// Stored inputs for each player (set via add_local_input)
        stored_inputs: Vec<I>,
    },
    /// Sync test session for determinism testing (boxed to reduce enum size)
    SyncTest {
        session: Box<SyncTestSession<NethercoreConfig<I>>>,
        current_frame: i32,
    },
    /// P2P session with rollback (boxed to reduce enum size)
    P2P(Box<ggrs::P2PSession<NethercoreConfig<I>>>),
}
