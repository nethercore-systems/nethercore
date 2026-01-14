//! Error types for local socket operations

/// Error type for local socket operations
#[derive(Debug, Clone)]
pub enum LocalSocketError {
    /// Failed to bind to the specified address
    Bind(String),
    /// Failed to set socket to non-blocking mode
    NonBlocking(String),
    /// Failed to connect to peer
    Connect(String),
    /// No peer connected yet
    NoPeer,
    /// Operation timed out
    Timeout,
}

impl std::fmt::Display for LocalSocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bind(e) => write!(f, "Failed to bind socket: {}", e),
            Self::NonBlocking(e) => write!(f, "Failed to set non-blocking: {}", e),
            Self::Connect(e) => write!(f, "Failed to connect: {}", e),
            Self::NoPeer => write!(f, "No peer connected"),
            Self::Timeout => write!(f, "Operation timed out"),
        }
    }
}

impl std::error::Error for LocalSocketError {}
