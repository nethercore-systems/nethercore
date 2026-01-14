//! Peer <-> Peer NCHS protocol messages (UDP hole punching)

use bitcode::{Decode, Encode};

/// UDP hole punch initiation
///
/// Sent by guests to each other to establish peer connections
/// after receiving SessionStart from host.
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct PunchHello {
    /// Sender's player handle
    pub sender_handle: u8,
    /// Nonce for matching hello/ack pairs
    pub nonce: u64,
}

/// UDP hole punch acknowledgement
///
/// Response to PunchHello confirming connection established.
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct PunchAck {
    /// Sender's player handle
    pub sender_handle: u8,
    /// Nonce from corresponding PunchHello
    pub nonce: u64,
}
