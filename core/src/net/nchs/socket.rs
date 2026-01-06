//! NCHS Socket Layer
//!
//! Provides UDP socket wrapper with NCHS framing for handshake messages.
//! After handshake completes, the socket can be converted to a [`LocalSocket`]
//! for use with GGRS.

use std::collections::VecDeque;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use super::messages::{NchsDecodeError, NchsMessage};
use crate::rollback::LocalSocket;

/// Buffer size for incoming NCHS packets
const RECV_BUFFER_SIZE: usize = 8192;

/// Default NCHS port
pub const DEFAULT_NCHS_PORT: u16 = 7770;

/// NCHS socket error types
#[derive(Debug, Clone)]
pub enum NchsSocketError {
    /// Failed to bind to address
    Bind(String),
    /// Failed to set socket options
    SocketOption(String),
    /// Failed to send message
    Send(String),
    /// Failed to receive message
    Receive(String),
    /// Message decode error
    Decode(NchsDecodeError),
    /// Address parse error
    AddressParse(String),
}

impl std::fmt::Display for NchsSocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bind(e) => write!(f, "Failed to bind: {}", e),
            Self::SocketOption(e) => write!(f, "Socket option error: {}", e),
            Self::Send(e) => write!(f, "Send error: {}", e),
            Self::Receive(e) => write!(f, "Receive error: {}", e),
            Self::Decode(e) => write!(f, "Decode error: {}", e),
            Self::AddressParse(e) => write!(f, "Address parse error: {}", e),
        }
    }
}

impl std::error::Error for NchsSocketError {}

impl From<NchsDecodeError> for NchsSocketError {
    fn from(e: NchsDecodeError) -> Self {
        Self::Decode(e)
    }
}

/// UDP socket wrapper for NCHS protocol messages
///
/// Handles NCHS framing (magic, version, length prefix) and provides
/// message-level send/receive operations.
///
/// # Example
///
/// ```rust,ignore
/// use nethercore_core::net::nchs::NchsSocket;
///
/// // Host binds to port
/// let mut host = NchsSocket::bind("0.0.0.0:7770")?;
///
/// // Guest connects to host
/// let mut guest = NchsSocket::bind("0.0.0.0:0")?;
/// guest.send("192.168.1.50:7770", &NchsMessage::JoinRequest(...))?;
///
/// // Host receives messages
/// while let Some((addr, msg)) = host.poll() {
///     match msg {
///         NchsMessage::JoinRequest(req) => { /* handle join */ }
///         _ => {}
///     }
/// }
/// ```
pub struct NchsSocket {
    /// Underlying UDP socket
    socket: UdpSocket,
    /// Local address
    local_addr: SocketAddr,
    /// Receive buffer
    recv_buf: Vec<u8>,
    /// Queue of received messages (address, message)
    recv_queue: VecDeque<(SocketAddr, NchsMessage)>,
}

impl NchsSocket {
    /// Bind to the specified address
    ///
    /// Creates a non-blocking UDP socket bound to the given address.
    ///
    /// # Arguments
    ///
    /// * `addr` - Address to bind to (e.g., "0.0.0.0:7770" or "127.0.0.1:0")
    pub fn bind(addr: &str) -> Result<Self, NchsSocketError> {
        let socket_addr: SocketAddr = addr
            .parse()
            .map_err(|e| NchsSocketError::AddressParse(format!("Invalid address '{}': {}", addr, e)))?;

        let socket = UdpSocket::bind(socket_addr)
            .map_err(|e| NchsSocketError::Bind(e.to_string()))?;

        socket
            .set_nonblocking(true)
            .map_err(|e| NchsSocketError::SocketOption(format!("Failed to set non-blocking: {}", e)))?;

        let local_addr = socket
            .local_addr()
            .map_err(|e| NchsSocketError::Bind(format!("Failed to get local addr: {}", e)))?;

        tracing::debug!(port = local_addr.port(), "NchsSocket bound");

        Ok(Self {
            socket,
            local_addr,
            recv_buf: vec![0u8; RECV_BUFFER_SIZE],
            recv_queue: VecDeque::new(),
        })
    }

    /// Bind to the default NCHS port
    pub fn bind_default() -> Result<Self, NchsSocketError> {
        Self::bind(&format!("0.0.0.0:{}", DEFAULT_NCHS_PORT))
    }

    /// Bind to any available port
    pub fn bind_any() -> Result<Self, NchsSocketError> {
        Self::bind("0.0.0.0:0")
    }

    /// Get the local address this socket is bound to
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Get the local address as a string
    pub fn local_addr_string(&self) -> String {
        self.local_addr.to_string()
    }

    /// Get the port this socket is bound to
    pub fn port(&self) -> u16 {
        self.local_addr.port()
    }

    /// Send an NCHS message to the specified address
    ///
    /// The message is serialized with NCHS framing before sending.
    pub fn send(&self, addr: &str, msg: &NchsMessage) -> Result<(), NchsSocketError> {
        let target: SocketAddr = addr
            .parse()
            .map_err(|e| NchsSocketError::AddressParse(format!("Invalid address '{}': {}", addr, e)))?;

        self.send_to(target, msg)
    }

    /// Send an NCHS message to a SocketAddr
    pub fn send_to(&self, target: SocketAddr, msg: &NchsMessage) -> Result<(), NchsSocketError> {
        let bytes = msg.to_bytes();

        self.socket
            .send_to(&bytes, target)
            .map_err(|e| NchsSocketError::Send(e.to_string()))?;

        tracing::trace!(?msg, "Sent NCHS message");
        Ok(())
    }

    /// Poll for incoming messages (non-blocking)
    ///
    /// Returns the next received message and sender address, or None if no
    /// messages are available.
    pub fn poll(&mut self) -> Option<(SocketAddr, NchsMessage)> {
        // First, try to receive more data
        self.recv_all();

        // Return the next queued message
        self.recv_queue.pop_front()
    }

    /// Poll for incoming messages with filtering by sender
    ///
    /// Returns the next message from the specified address, or None.
    /// Messages from other addresses remain in the queue.
    pub fn poll_from(&mut self, expected_addr: &SocketAddr) -> Option<NchsMessage> {
        self.recv_all();

        // Find and remove the first message from the expected address
        for i in 0..self.recv_queue.len() {
            if &self.recv_queue[i].0 == expected_addr {
                return self.recv_queue.remove(i).map(|(_, msg)| msg);
            }
        }
        None
    }

    /// Receive all available messages from the socket
    fn recv_all(&mut self) {
        loop {
            match self.socket.recv_from(&mut self.recv_buf) {
                Ok((len, from)) => {
                    let data = &self.recv_buf[..len];

                    match NchsMessage::from_bytes(data) {
                        Ok(msg) => {
                            tracing::trace!(?msg, "Received NCHS message");
                            self.recv_queue.push_back((from, msg));
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "Failed to decode NCHS message");
                        }
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No more data available
                    break;
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Receive error");
                    break;
                }
            }
        }
    }

    /// Wait for a message from any address (blocking with timeout)
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait
    ///
    /// # Returns
    ///
    /// The first received message and sender address, or None on timeout.
    pub fn wait_for_message(&mut self, timeout: Duration) -> Option<(SocketAddr, NchsMessage)> {
        let start = Instant::now();

        // Check queue first
        if let Some(msg) = self.recv_queue.pop_front() {
            return Some(msg);
        }

        // Poll with short sleeps until timeout
        while start.elapsed() < timeout {
            self.recv_all();
            if let Some(msg) = self.recv_queue.pop_front() {
                return Some(msg);
            }
            std::thread::sleep(Duration::from_millis(1));
        }

        None
    }

    /// Wait for a specific message type (blocking with timeout)
    ///
    /// Other messages received during the wait are queued.
    pub fn wait_for<F>(&mut self, timeout: Duration, predicate: F) -> Option<(SocketAddr, NchsMessage)>
    where
        F: Fn(&NchsMessage) -> bool,
    {
        let start = Instant::now();

        while start.elapsed() < timeout {
            self.recv_all();

            // Check queue for matching message
            for i in 0..self.recv_queue.len() {
                if predicate(&self.recv_queue[i].1) {
                    return self.recv_queue.remove(i);
                }
            }

            std::thread::sleep(Duration::from_millis(1));
        }

        None
    }

    /// Convert to a LocalSocket for GGRS after handshake completes
    ///
    /// This consumes the NCHS socket and creates a LocalSocket
    /// suitable for GGRS P2P sessions.
    ///
    /// Note: This creates a new LocalSocket bound to a different port.
    /// The NCHS socket continues to exist for the protocol,
    /// while GGRS uses a separate port for game traffic.
    pub fn into_local_socket(self, ggrs_port: u16) -> Result<LocalSocket, crate::rollback::LocalSocketError> {
        // Drop the NCHS socket (releases the port)
        drop(self.socket);

        // Create a new LocalSocket for GGRS
        LocalSocket::bind(&format!("0.0.0.0:{}", ggrs_port))
    }

    /// Create a separate LocalSocket for GGRS (keeping NCHS socket alive)
    ///
    /// This creates a new LocalSocket on a different port while keeping
    /// the NCHS socket operational for late-join or session management.
    pub fn create_ggrs_socket(&self, ggrs_port: u16) -> Result<LocalSocket, crate::rollback::LocalSocketError> {
        LocalSocket::bind(&format!("0.0.0.0:{}", ggrs_port))
    }

    /// Get local IP addresses that can be shared with peers
    ///
    /// Returns a list of non-loopback IPv4 addresses that can be used
    /// for peer-to-peer connections. Falls back to localhost if no
    /// external interface is found.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let ips = NchsSocket::get_local_ips();
    /// for ip in ips {
    ///     println!("Share this address: {}:{}", ip, port);
    /// }
    /// ```
    pub fn get_local_ips() -> Vec<String> {
        let mut ips = Vec::new();

        // Try to get local IP by connecting to a public address
        // This doesn't actually send data, just determines the route
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            if socket.connect("8.8.8.8:80").is_ok() {
                if let Ok(addr) = socket.local_addr() {
                    if let IpAddr::V4(ipv4) = addr.ip() {
                        if !ipv4.is_loopback() {
                            ips.push(ipv4.to_string());
                        }
                    }
                }
            }
        }

        // Also include localhost for local testing
        ips.push(Ipv4Addr::LOCALHOST.to_string());

        ips
    }

    /// Get the underlying socket reference (for advanced use)
    pub fn socket(&self) -> &UdpSocket {
        &self.socket
    }

    /// Drain all pending messages from the queue
    pub fn drain(&mut self) -> Vec<(SocketAddr, NchsMessage)> {
        self.recv_all();
        self.recv_queue.drain(..).collect()
    }
}

impl std::fmt::Debug for NchsSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NchsSocket")
            .field("local_addr", &self.local_addr)
            .field("queued_messages", &self.recv_queue.len())
            .finish()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::nchs::messages::{PlayerInfo, JoinRequest};
    use nethercore_shared::console::{ConsoleType, TickRate};

    #[test]
    fn test_socket_bind() {
        let socket = NchsSocket::bind("127.0.0.1:0").unwrap();
        assert!(socket.port() > 0);
    }

    #[test]
    fn test_socket_bind_any() {
        let socket = NchsSocket::bind_any().unwrap();
        assert!(socket.port() > 0);
    }

    #[test]
    fn test_socket_local_addr() {
        let socket = NchsSocket::bind("127.0.0.1:0").unwrap();
        let addr = socket.local_addr();
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
    }

    #[test]
    fn test_socket_send_receive_ping_pong() {
        let mut socket1 = NchsSocket::bind("127.0.0.1:0").unwrap();
        let mut socket2 = NchsSocket::bind("127.0.0.1:0").unwrap();

        let addr1 = socket1.local_addr_string();
        let addr2 = socket2.local_addr_string();

        // Send Ping from socket1 to socket2
        socket1.send(&addr2, &NchsMessage::Ping).unwrap();

        // Give it time to arrive
        std::thread::sleep(Duration::from_millis(10));

        // Socket2 should receive Ping
        let received = socket2.poll();
        assert!(received.is_some());
        let (from, msg) = received.unwrap();
        assert_eq!(from, socket1.local_addr());
        assert_eq!(msg, NchsMessage::Ping);

        // Send Pong back
        socket2.send(&addr1, &NchsMessage::Pong).unwrap();

        std::thread::sleep(Duration::from_millis(10));

        // Socket1 should receive Pong
        let received = socket1.poll();
        assert!(received.is_some());
        let (from, msg) = received.unwrap();
        assert_eq!(from, socket2.local_addr());
        assert_eq!(msg, NchsMessage::Pong);
    }

    #[test]
    fn test_socket_send_receive_join_request() {
        let mut host = NchsSocket::bind("127.0.0.1:0").unwrap();
        let guest = NchsSocket::bind("127.0.0.1:0").unwrap();

        let host_addr = host.local_addr_string();

        let join_request = NchsMessage::JoinRequest(JoinRequest {
            console_type: ConsoleType::ZX,
            rom_hash: 0x123456789ABCDEF0,
            tick_rate: TickRate::Fixed60,
            max_players: 4,
            player_info: PlayerInfo {
                name: "TestPlayer".to_string(),
                avatar_id: 1,
                color: [255, 0, 0],
            },
            local_addr: guest.local_addr_string(),
            extra_data: vec![],
        });

        guest.send(&host_addr, &join_request).unwrap();

        std::thread::sleep(Duration::from_millis(10));

        let received = host.poll();
        assert!(received.is_some());
        let (from, msg) = received.unwrap();
        assert_eq!(from, guest.local_addr());
        assert_eq!(msg, join_request);
    }

    #[test]
    fn test_socket_poll_from() {
        let mut host = NchsSocket::bind("127.0.0.1:0").unwrap();
        let guest1 = NchsSocket::bind("127.0.0.1:0").unwrap();
        let guest2 = NchsSocket::bind("127.0.0.1:0").unwrap();

        let host_addr = host.local_addr_string();

        // Both guests send messages
        guest1.send(&host_addr, &NchsMessage::Ping).unwrap();
        guest2.send(&host_addr, &NchsMessage::Pong).unwrap();

        std::thread::sleep(Duration::from_millis(10));

        // poll_from should return only the message from the specified sender
        let from_guest2 = host.poll_from(&guest2.local_addr());
        assert!(from_guest2.is_some());
        assert_eq!(from_guest2.unwrap(), NchsMessage::Pong);

        // Guest1's message should still be in queue
        let from_guest1 = host.poll_from(&guest1.local_addr());
        assert!(from_guest1.is_some());
        assert_eq!(from_guest1.unwrap(), NchsMessage::Ping);
    }

    #[test]
    fn test_socket_wait_for_message_timeout() {
        let mut socket = NchsSocket::bind("127.0.0.1:0").unwrap();

        let start = Instant::now();
        let result = socket.wait_for_message(Duration::from_millis(50));
        let elapsed = start.elapsed();

        assert!(result.is_none());
        assert!(elapsed >= Duration::from_millis(40));
        assert!(elapsed < Duration::from_millis(200));
    }

    #[test]
    fn test_socket_wait_for_message_success() {
        let mut host = NchsSocket::bind("127.0.0.1:0").unwrap();
        let guest = NchsSocket::bind("127.0.0.1:0").unwrap();
        let host_addr = host.local_addr_string();

        // Spawn thread to send message after delay
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            guest.send(&host_addr, &NchsMessage::Ping).unwrap();
        });

        let result = host.wait_for_message(Duration::from_secs(1));
        handle.join().unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().1, NchsMessage::Ping);
    }

    #[test]
    fn test_socket_drain() {
        let mut socket1 = NchsSocket::bind("127.0.0.1:0").unwrap();
        let socket2 = NchsSocket::bind("127.0.0.1:0").unwrap();

        let addr1 = socket1.local_addr_string();

        // Send multiple messages
        socket2.send(&addr1, &NchsMessage::Ping).unwrap();
        socket2.send(&addr1, &NchsMessage::Pong).unwrap();

        std::thread::sleep(Duration::from_millis(10));

        let messages = socket1.drain();
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn test_socket_debug() {
        let socket = NchsSocket::bind("127.0.0.1:0").unwrap();
        let debug = format!("{:?}", socket);
        assert!(debug.contains("NchsSocket"));
        assert!(debug.contains("local_addr"));
    }

    #[test]
    fn test_socket_error_display() {
        let err = NchsSocketError::Bind("address in use".to_string());
        assert!(err.to_string().contains("address in use"));

        let err = NchsSocketError::AddressParse("invalid".to_string());
        assert!(err.to_string().contains("invalid"));
    }

    #[test]
    fn test_socket_create_ggrs_socket() {
        let nchs_socket = NchsSocket::bind("127.0.0.1:0").unwrap();
        let ggrs_socket = nchs_socket.create_ggrs_socket(0).unwrap();

        // Both sockets should be on different ports
        assert_ne!(nchs_socket.port(), ggrs_socket.local_addr().port());
    }

    #[test]
    fn test_get_local_ips_not_empty() {
        let ips = NchsSocket::get_local_ips();
        assert!(!ips.is_empty(), "get_local_ips should return at least one IP");
        // Should always include localhost as fallback
        assert!(
            ips.contains(&"127.0.0.1".to_string()),
            "get_local_ips should include localhost"
        );
    }

    #[test]
    fn test_get_local_ips_no_zero_address() {
        let ips = NchsSocket::get_local_ips();
        assert!(
            !ips.iter().any(|ip| ip == "0.0.0.0"),
            "get_local_ips should not return 0.0.0.0"
        );
    }
}
