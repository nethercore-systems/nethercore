//! GGRS rollback integration
//!
//! Provides the configuration and state management for GGRS rollback netcode.
//!
//! # Architecture
//!
//! GGRS (Good Game Rollback SDK) handles deterministic rollback netcode. This module provides:
//!
//! - `EmberwareConfig<I>`: GGRS configuration parameterized by console input type
//! - `GameStateSnapshot`: Serialized game state with checksum for desync detection
//! - `RollbackSession<I>`: Session manager for local, sync-test, and P2P modes
//! - `StatePool`: Pre-allocated buffer pool to avoid allocations during rollback
//! - `LocalSocket`: UDP socket for local P2P testing without signaling server
//!
//! # Input Flow
//!
//! 1. Physical input is mapped to console-specific `ConsoleInput` (e.g., `ZInput`)
//! 2. Input is added to GGRS via `session.add_local_input()`
//! 3. GGRS handles prediction, confirmation, and rollback
//! 4. Confirmed inputs are passed to `GameInstance::update()` during advance
//!
//! # Local Network Testing
//!
//! For testing P2P sessions without a signaling server, use `LocalSocket`:
//!
//! ```ignore
//! // Instance 1 (host on port 7777):
//! let mut socket = LocalSocket::bind("127.0.0.1:7777")?;
//! socket.connect("127.0.0.1:7778")?;
//! let session = RollbackSession::new_p2p(config, socket, players)?;
//!
//! // Instance 2 (client on port 7778):
//! let mut socket = LocalSocket::bind("127.0.0.1:7778")?;
//! socket.connect("127.0.0.1:7777")?;
//! let session = RollbackSession::new_p2p(config, socket, players)?;
//! ```
//!
//! # Module Structure
//!
//! - `config`: GGRS configuration types and constants
//! - `player`: Player session configuration (local vs remote)
//! - `state`: State snapshot and buffer pool management
//! - `session`: GGRS session wrapper and event handling
//! - `local_socket`: UDP socket for local network testing

mod config;
mod events;
pub mod local_socket;
mod player;
mod session;
mod state;

// Re-export public types from config
pub use config::{
    DEFAULT_INPUT_DELAY, DEFAULT_ONLINE_INPUT_DELAY, EmberwareConfig, MAX_INPUT_DELAY,
    MAX_ROLLBACK_FRAMES, MAX_STATE_SIZE, SessionConfig,
};

// Re-export public types from player
pub use player::{MAX_PLAYERS, PlayerSessionConfig};

// Re-export public types from state
pub use state::{
    GameStateSnapshot, LoadStateError, RollbackStateManager, STATE_POOL_SIZE, SaveStateError,
    StatePool,
};

// Re-export public types from session
pub use session::{NetworkInput, RollbackSession, SessionType};

// Re-export public types from events
pub use events::{ConnectionQuality, PlayerNetworkStats, SessionError, SessionEvent};

// Re-export public types from local_socket
pub use local_socket::{DEFAULT_LOCAL_PORT, LocalSocket, LocalSocketError};
