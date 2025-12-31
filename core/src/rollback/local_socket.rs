//! Local UDP socket for P2P testing without signaling server
//!
//! This module provides a simple UDP socket wrapper that implements GGRS's
//! `NonBlockingSocket` trait, allowing local network testing without the
//! complexity of WebRTC signaling.
//!
//! # Usage
//!
//! For 2-player local testing, run two instances of the application:
//!
//! ```ignore
//! // Instance 1 (host):
//! let socket = LocalSocket::bind("127.0.0.1:7777")?;
//! socket.connect("127.0.0.1:7778")?;
//!
//! // Instance 2 (client):
//! let socket = LocalSocket::bind("127.0.0.1:7778")?;
//! socket.connect("127.0.0.1:7777")?;
//!
//! // Create P2P session (same for both):
//! let players = vec![
//!     (0, PlayerType::Local),
//!     (1, PlayerType::Remote(peer_addr)),
//! ];
//! let session = RollbackSession::<ZInput>::new_p2p(config, socket, players)?;
//! ```
//!
//! # Limitations
//!
//! - No NAT traversal (localhost only)
//! - No ICE/STUN/TURN
//! - Simple point-to-point (no mesh networking for >2 players without manual port assignment)

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use ggrs::NonBlockingSocket;

/// Default port for local testing
pub const DEFAULT_LOCAL_PORT: u16 = 7777;

/// Buffer size for incoming packets (GGRS packets are small)
const RECV_BUFFER_SIZE: usize = 4096;

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

/// Simple UDP socket for local P2P testing
///
/// Implements `NonBlockingSocket<String>` for GGRS, allowing P2P sessions
/// to be created without a signaling server.
///
/// The address type is `String` to match GGRS's expected type, with the
/// format `"ip:port"` (e.g., `"127.0.0.1:7777"`).
pub struct LocalSocket {
    socket: UdpSocket,
    local_addr: SocketAddr,
    /// Connected peer address (for point-to-point)
    peer_addr: Option<SocketAddr>,
    /// Receive buffer
    recv_buf: Vec<u8>,
}

impl LocalSocket {
    /// Bind to the specified address
    ///
    /// # Arguments
    ///
    /// * `addr` - Address to bind to (e.g., "127.0.0.1:7777" or "0.0.0.0:7777")
    ///
    /// # Example
    ///
    /// ```ignore
    /// let socket = LocalSocket::bind("127.0.0.1:7777")?;
    /// ```
    pub fn bind(addr: &str) -> Result<Self, LocalSocketError> {
        let socket_addr: SocketAddr = addr
            .parse()
            .map_err(|e| LocalSocketError::Bind(format!("Invalid address '{}': {}", addr, e)))?;

        let socket =
            UdpSocket::bind(socket_addr).map_err(|e| LocalSocketError::Bind(e.to_string()))?;

        socket
            .set_nonblocking(true)
            .map_err(|e| LocalSocketError::NonBlocking(e.to_string()))?;

        // Set a short read timeout for non-blocking behavior
        socket.set_read_timeout(Some(Duration::from_millis(1))).ok(); // Ignore error, nonblocking mode takes precedence

        let local_addr = socket
            .local_addr()
            .map_err(|e| LocalSocketError::Bind(format!("Failed to get local addr: {}", e)))?;

        log::info!("LocalSocket bound to {}", local_addr);

        Ok(Self {
            socket,
            local_addr,
            peer_addr: None,
            recv_buf: vec![0u8; RECV_BUFFER_SIZE],
        })
    }

    /// Bind to the default local testing port
    ///
    /// Equivalent to `LocalSocket::bind("127.0.0.1:7777")`.
    pub fn bind_default() -> Result<Self, LocalSocketError> {
        Self::bind(&format!("127.0.0.1:{}", DEFAULT_LOCAL_PORT))
    }

    /// Bind to any available port on localhost
    ///
    /// Useful when you need multiple instances and don't want to manage port assignments.
    pub fn bind_any() -> Result<Self, LocalSocketError> {
        Self::bind("127.0.0.1:0")
    }

    /// Connect to a peer
    ///
    /// This sets the default peer for sending messages. For GGRS P2P sessions,
    /// call this with the remote player's address.
    ///
    /// # Arguments
    ///
    /// * `peer` - Peer address as string (e.g., "127.0.0.1:7778")
    ///
    /// # Example
    ///
    /// ```ignore
    /// socket.connect("127.0.0.1:7778")?;
    /// ```
    pub fn connect(&mut self, peer: &str) -> Result<(), LocalSocketError> {
        let peer_addr: SocketAddr = peer
            .parse()
            .map_err(|e| LocalSocketError::Connect(format!("Invalid peer '{}': {}", peer, e)))?;

        log::info!("LocalSocket connecting to {}", peer_addr);
        self.peer_addr = Some(peer_addr);
        Ok(())
    }

    /// Get the local address this socket is bound to
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Get the local address as a string (for use as GGRS address)
    pub fn local_addr_string(&self) -> String {
        self.local_addr.to_string()
    }

    /// Get the peer address if connected
    pub fn peer_addr(&self) -> Option<SocketAddr> {
        self.peer_addr
    }

    /// Get the peer address as a string (for use as GGRS address)
    pub fn peer_addr_string(&self) -> Option<String> {
        self.peer_addr.map(|a| a.to_string())
    }

    /// Check if a peer is connected
    pub fn is_connected(&self) -> bool {
        self.peer_addr.is_some()
    }

    /// Wait for a peer to connect (blocking with timeout)
    ///
    /// Blocks until a packet is received from any peer, then sets that
    /// peer as the connected peer and returns their address.
    ///
    /// This is useful for host mode where we wait for a client to connect.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait for a peer
    ///
    /// # Returns
    ///
    /// The peer's address as a string (e.g., "192.168.1.100:7778")
    ///
    /// # Errors
    ///
    /// Returns `LocalSocketError::Timeout` if no peer connects within the timeout.
    pub fn wait_for_peer(&mut self, timeout: Duration) -> Result<String, LocalSocketError> {
        let start = Instant::now();

        // Temporarily set socket to blocking with timeout
        self.socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(|e| LocalSocketError::Bind(format!("Failed to set read timeout: {}", e)))?;

        while start.elapsed() < timeout {
            let mut buf = [0u8; 128];
            match self.socket.recv_from(&mut buf) {
                Ok((_len, from)) => {
                    log::info!("Peer connected from {}", from);
                    self.peer_addr = Some(from);

                    // Restore non-blocking mode
                    self.socket
                        .set_read_timeout(Some(Duration::from_millis(1)))
                        .ok();

                    return Ok(from.to_string());
                }
                Err(e) => {
                    // Timeout or WouldBlock is expected, keep waiting
                    if e.kind() != io::ErrorKind::WouldBlock && e.kind() != io::ErrorKind::TimedOut
                    {
                        log::warn!("Unexpected error while waiting for peer: {}", e);
                    }
                }
            }
        }

        // Restore non-blocking mode
        self.socket
            .set_read_timeout(Some(Duration::from_millis(1)))
            .ok();

        Err(LocalSocketError::Timeout)
    }

    /// Poll for a peer connection (non-blocking)
    ///
    /// Checks if any packets have been received and sets the sender
    /// as the connected peer.
    ///
    /// # Returns
    ///
    /// The peer's address if a connection was detected, or `None`.
    pub fn poll_for_peer(&mut self) -> Option<String> {
        let mut buf = [0u8; 128];
        match self.socket.recv_from(&mut buf) {
            Ok((_len, from)) => {
                log::info!("Peer connected from {}", from);
                self.peer_addr = Some(from);
                Some(from.to_string())
            }
            Err(_) => None,
        }
    }

    /// Get all local IP addresses for display in host mode
    ///
    /// Returns a list of non-loopback IPv4 addresses that can be shared
    /// with friends to connect.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let ips = LocalSocket::get_local_ips();
    /// for ip in ips {
    ///     println!("Share this address: {}:{}", ip, port);
    /// }
    /// ```
    pub fn get_local_ips() -> Vec<String> {
        let mut ips = Vec::new();

        // Try to get local IP by connecting to a public address
        // This doesn't actually send data, just determines the route
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0")
            && socket.connect("8.8.8.8:80").is_ok()
            && let Ok(addr) = socket.local_addr()
            && let IpAddr::V4(ipv4) = addr.ip()
            && !ipv4.is_loopback()
        {
            ips.push(ipv4.to_string());
        }

        // Also include localhost for local testing
        ips.push(Ipv4Addr::LOCALHOST.to_string());

        ips
    }
}

impl NonBlockingSocket<String> for LocalSocket {
    fn send_to(&mut self, msg: &ggrs::Message, addr: &String) {
        // Parse the target address
        let target: SocketAddr = match addr.parse() {
            Ok(a) => a,
            Err(e) => {
                log::warn!("Invalid send address '{}': {}", addr, e);
                return;
            }
        };

        // Serialize the GGRS message
        let data = match bincode::serialize(msg) {
            Ok(d) => d,
            Err(e) => {
                log::warn!("Failed to serialize message: {}", e);
                return;
            }
        };

        // Send immediately
        if let Err(e) = self.socket.send_to(&data, target) {
            // WouldBlock is expected for non-blocking sockets when buffer is full
            if e.kind() != io::ErrorKind::WouldBlock {
                log::warn!("Failed to send to {}: {}", target, e);
            }
        }
    }

    fn receive_all_messages(&mut self) -> Vec<(String, ggrs::Message)> {
        let mut messages = Vec::new();

        // Read all available messages
        loop {
            match self.socket.recv_from(&mut self.recv_buf) {
                Ok((len, from)) => {
                    // Deserialize the GGRS message
                    match bincode::deserialize::<ggrs::Message>(&self.recv_buf[..len]) {
                        Ok(msg) => {
                            messages.push((from.to_string(), msg));
                        }
                        Err(e) => {
                            log::warn!("Failed to deserialize message from {}: {}", from, e);
                        }
                    }
                }
                Err(e) => {
                    // WouldBlock means no more data available
                    if e.kind() == io::ErrorKind::WouldBlock {
                        break;
                    }
                    // Other errors are unexpected
                    log::warn!("Receive error: {}", e);
                    break;
                }
            }
        }

        messages
    }
}

impl std::fmt::Debug for LocalSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalSocket")
            .field("local_addr", &self.local_addr)
            .field("peer_addr", &self.peer_addr)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_socket_bind() {
        let socket = LocalSocket::bind("127.0.0.1:0").unwrap();
        assert!(socket.local_addr().port() > 0);
        assert!(!socket.is_connected());
    }

    #[test]
    fn test_local_socket_bind_any() {
        let socket = LocalSocket::bind_any().unwrap();
        assert!(socket.local_addr().port() > 0);
    }

    #[test]
    fn test_local_socket_connect() {
        let mut socket = LocalSocket::bind("127.0.0.1:0").unwrap();
        socket.connect("127.0.0.1:9999").unwrap();
        assert!(socket.is_connected());
        assert_eq!(socket.peer_addr().unwrap().port(), 9999);
    }

    #[test]
    fn test_local_socket_local_addr_string() {
        let socket = LocalSocket::bind("127.0.0.1:0").unwrap();
        let addr_str = socket.local_addr_string();
        assert!(addr_str.starts_with("127.0.0.1:"));
    }

    #[test]
    fn test_local_socket_peer_addr_string() {
        let mut socket = LocalSocket::bind("127.0.0.1:0").unwrap();
        assert!(socket.peer_addr_string().is_none());

        socket.connect("127.0.0.1:8888").unwrap();
        assert_eq!(
            socket.peer_addr_string(),
            Some("127.0.0.1:8888".to_string())
        );
    }

    #[test]
    fn test_local_socket_error_display() {
        let bind_err = LocalSocketError::Bind("address in use".to_string());
        assert!(bind_err.to_string().contains("address in use"));

        let no_peer_err = LocalSocketError::NoPeer;
        assert!(no_peer_err.to_string().contains("No peer"));
    }

    #[test]
    fn test_local_socket_debug() {
        let socket = LocalSocket::bind("127.0.0.1:0").unwrap();
        let debug = format!("{:?}", socket);
        assert!(debug.contains("LocalSocket"));
        assert!(debug.contains("local_addr"));
    }

    #[test]
    fn test_local_socket_receive_empty() {
        let mut socket = LocalSocket::bind("127.0.0.1:0").unwrap();
        // Should return empty vec when no messages available
        let messages = socket.receive_all_messages();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_local_socket_raw_udp_send_receive() {
        // Test raw UDP communication (without GGRS Message serialization)
        // This validates the underlying socket works correctly
        let socket1 = LocalSocket::bind("127.0.0.1:0").unwrap();
        let socket2 = LocalSocket::bind("127.0.0.1:0").unwrap();

        let addr1 = socket1.local_addr();
        let addr2 = socket2.local_addr();

        // Use the inner sockets directly to test raw UDP
        let test_data = b"hello";
        socket1.socket.send_to(test_data, addr2).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));

        let mut buf = [0u8; 64];
        let (len, from) = socket2.socket.recv_from(&mut buf).unwrap();
        assert_eq!(&buf[..len], test_data);
        assert_eq!(from, addr1);
    }

    #[test]
    fn test_local_socket_bidirectional_raw() {
        // Test bidirectional raw UDP communication
        let socket1 = LocalSocket::bind("127.0.0.1:0").unwrap();
        let socket2 = LocalSocket::bind("127.0.0.1:0").unwrap();

        let addr1 = socket1.local_addr();
        let addr2 = socket2.local_addr();

        // Send data both ways
        socket1.socket.send_to(b"from1", addr2).unwrap();
        socket2.socket.send_to(b"from2", addr1).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));

        let mut buf1 = [0u8; 64];
        let mut buf2 = [0u8; 64];

        let (len1, from1) = socket1.socket.recv_from(&mut buf1).unwrap();
        let (len2, from2) = socket2.socket.recv_from(&mut buf2).unwrap();

        assert_eq!(&buf1[..len1], b"from2");
        assert_eq!(from1, addr2);
        assert_eq!(&buf2[..len2], b"from1");
        assert_eq!(from2, addr1);
    }

    #[test]
    fn test_local_socket_nonblocking() {
        // Verify socket is non-blocking
        let mut socket = LocalSocket::bind("127.0.0.1:0").unwrap();

        // receive_all_messages should not block when no data available
        let start = std::time::Instant::now();
        let messages = socket.receive_all_messages();
        let elapsed = start.elapsed();

        assert!(messages.is_empty());
        // Should return almost immediately (< 100ms)
        assert!(
            elapsed.as_millis() < 100,
            "Socket blocked for {}ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_local_socket_invalid_bind_address() {
        let result = LocalSocket::bind("not-an-address");
        assert!(result.is_err());
        match result {
            Err(LocalSocketError::Bind(_)) => {}
            _ => panic!("Expected Bind error"),
        }
    }

    #[test]
    fn test_local_socket_invalid_connect_address() {
        let mut socket = LocalSocket::bind("127.0.0.1:0").unwrap();
        let result = socket.connect("not-an-address");
        assert!(result.is_err());
        match result {
            Err(LocalSocketError::Connect(_)) => {}
            _ => panic!("Expected Connect error"),
        }
    }

    #[test]
    fn test_get_local_ips_includes_localhost() {
        let ips = LocalSocket::get_local_ips();
        // Should always include localhost for local testing
        assert!(
            ips.contains(&"127.0.0.1".to_string()),
            "get_local_ips should include localhost: {:?}",
            ips
        );
    }

    #[test]
    fn test_get_local_ips_not_empty() {
        let ips = LocalSocket::get_local_ips();
        assert!(
            !ips.is_empty(),
            "get_local_ips should return at least one IP"
        );
    }

    #[test]
    fn test_poll_for_peer_returns_none_when_no_data() {
        let mut socket = LocalSocket::bind("127.0.0.1:0").unwrap();
        // Should return None immediately when no peer has sent data
        let result = socket.poll_for_peer();
        assert!(result.is_none());
        assert!(!socket.is_connected());
    }

    #[test]
    fn test_poll_for_peer_detects_connection() {
        // Create two sockets
        let mut host = LocalSocket::bind("127.0.0.1:0").unwrap();
        let client = LocalSocket::bind("127.0.0.1:0").unwrap();

        let host_addr = host.local_addr();

        // Client sends a packet to host
        client.socket.send_to(b"hello", host_addr).unwrap();

        // Give the packet time to arrive
        std::thread::sleep(std::time::Duration::from_millis(10));

        // poll_for_peer should detect the connection
        let result = host.poll_for_peer();
        assert!(
            result.is_some(),
            "poll_for_peer should detect incoming packet"
        );
        assert!(
            host.is_connected(),
            "Socket should be connected after peer detected"
        );
    }

    #[test]
    fn test_wait_for_peer_timeout() {
        let mut socket = LocalSocket::bind("127.0.0.1:0").unwrap();

        // Wait for a very short timeout (no peer will connect)
        let start = std::time::Instant::now();
        let result = socket.wait_for_peer(Duration::from_millis(100));
        let elapsed = start.elapsed();

        // Should return Timeout error
        assert!(matches!(result, Err(LocalSocketError::Timeout)));

        // Should have waited at least the timeout duration
        assert!(
            elapsed >= Duration::from_millis(90), // Allow 10ms tolerance
            "Should have waited at least 90ms, but waited {:?}",
            elapsed
        );

        // Should not have waited too much longer than the timeout
        assert!(
            elapsed < Duration::from_millis(500),
            "Should not wait much longer than timeout, but waited {:?}",
            elapsed
        );
    }

    #[test]
    fn test_wait_for_peer_success() {
        let mut host = LocalSocket::bind("127.0.0.1:0").unwrap();
        let client = LocalSocket::bind("127.0.0.1:0").unwrap();
        let host_addr = host.local_addr();

        // Spawn a thread to send a packet after a short delay
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            client.socket.send_to(b"hello", host_addr).unwrap();
        });

        // Wait for peer with sufficient timeout
        let result = host.wait_for_peer(Duration::from_secs(1));

        handle.join().unwrap();

        assert!(result.is_ok(), "wait_for_peer should succeed: {:?}", result);
        assert!(
            host.is_connected(),
            "Socket should be connected after peer detected"
        );
    }

    #[test]
    fn test_timeout_error_display() {
        let err = LocalSocketError::Timeout;
        let display = err.to_string();
        assert!(
            display.contains("timed out"),
            "Timeout error display should contain 'timed out': {}",
            display
        );
    }
}
