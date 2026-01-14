//! NCHS Host State Machine
//!
//! Manages the host side of NCHS handshake, including:
//! - Listening for incoming connections
//! - Validating join requests
//! - Managing lobby state
//! - Initiating game start and distributing session info

mod messages;
mod session;
mod state;

#[cfg(test)]
mod tests;

pub use state::{ConnectedPlayer, HostEvent, HostState, HostStateMachine};
