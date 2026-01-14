//! GGRS session management
//!
//! Provides the RollbackSession wrapper for GGRS local, sync-test, and P2P sessions.

mod builder;
mod session;
mod types;

#[cfg(test)]
mod tests;

// Re-export public types
pub use session::RollbackSession;
pub use types::{NetworkInput, SessionType};
