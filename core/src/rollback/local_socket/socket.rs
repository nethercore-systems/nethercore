//! Core UDP socket implementation for local P2P testing

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use super::error::LocalSocketError;

/// Default port for local testing
pub const DEFAULT_LOCAL_PORT: u16 = 7777;

/// Buffer size for incoming packets (GGRS packets are small)
const RECV_BUFFER_SIZE: usize = 4096;

/// Simple UDP socket for local P2P testing
///
/// Implements `NonBlockingSocket<String>` for GGRS, allowing P2P sessions
/// to be created without a signaling server.
///
/// The address type is `String` to match GGRS's expected type, with the
/// format `"ip:port"` (e.g., `"127.0.0.1:7777"`).
pub struct LocalSocket {
    pub(super) socket: UdpSocket,
    pub(super) local_addr: SocketAddr,
    /// Connected peer address (for point-to-point)
    pub(super) peer_addr: Option<SocketAddr>,
    /// Receive buffer
    pub(super) recv_buf: Vec<u8>,
    content_hash: Option<u64>,
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

        tracing::info!(port = local_addr.port(), "LocalSocket bound");

        Ok(Self {
            socket,
            local_addr,
            peer_addr: None,
            recv_buf: vec![0u8; RECV_BUFFER_SIZE],
            content_hash: None,
        })
    }

    /// Bounded, versioned compatibility exchange. This is not authentication.
    pub fn verify_content(
        &mut self,
        hash: u64,
        peers: &[SocketAddr],
        timeout: Duration,
    ) -> anyhow::Result<()> {
        self.content_hash = Some(hash);
        let mut probe = *b"NCID1?00000000";
        probe[6..].copy_from_slice(&hash.to_le_bytes());
        let mut pending = peers.to_vec();
        let deadline = Instant::now() + timeout;
        while !pending.is_empty() {
            anyhow::ensure!(
                Instant::now() < deadline,
                "Cartridge compatibility handshake timed out; use matching player versions"
            );
            for peer in &pending {
                self.socket.send_to(&probe, peer)?;
            }
            let next_probe = (Instant::now() + Duration::from_millis(100)).min(deadline);
            while Instant::now() < next_probe && !pending.is_empty() {
                let mut bytes = [0; 128];
                match self.socket.recv_from(&mut bytes) {
                    Ok((len, from)) if peers.contains(&from) => {
                        let packet = &bytes[..len];
                        if packet.len() != probe.len()
                            || !packet.starts_with(b"NCID1")
                            || !matches!(packet[5], b'?' | b'!')
                        {
                            continue;
                        }
                        self.handle_content_probe(packet, from);
                        anyhow::ensure!(
                            packet[6..] == hash.to_le_bytes(),
                            "Cartridge content mismatch with {from}"
                        );
                        if packet[5] == b'!' {
                            pending.retain(|peer| *peer != from);
                        }
                    }
                    Ok(_) => {}
                    Err(error)
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::WouldBlock
                                | std::io::ErrorKind::TimedOut
                                | std::io::ErrorKind::ConnectionReset
                        ) =>
                    {
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    Err(error) => return Err(error.into()),
                }
            }
        }
        tracing::info!(
            "Cartridge compatibility verified for {} peer(s)",
            peers.len()
        );
        Ok(())
    }

    // Continue acknowledging retries after GGRS starts, so a lost final ACK
    // cannot leave one side waiting. ACKs never provoke another ACK.
    pub(super) fn handle_content_probe(&self, packet: &[u8], from: SocketAddr) -> bool {
        let Some(hash) = self.content_hash else {
            return false;
        };
        if packet.len() != 14 || !packet.starts_with(b"NCID1") {
            return false;
        }
        if packet[5] == b'?' {
            let mut ack = *b"NCID1!00000000";
            ack[6..].copy_from_slice(&hash.to_le_bytes());
            let _ = self.socket.send_to(&ack, from);
        }
        true
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

        tracing::info!(port = peer_addr.port(), "LocalSocket connecting");
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

    /// Get a reference to the underlying UDP socket
    ///
    /// This is useful for performing custom operations like handshakes
    /// before the GGRS session takes over.
    pub fn socket(&self) -> &UdpSocket {
        &self.socket
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
                    tracing::info!("Peer connected");
                    self.peer_addr = Some(from);

                    // Restore non-blocking mode
                    self.socket
                        .set_read_timeout(Some(Duration::from_millis(1)))
                        .ok();

                    return Ok(from.to_string());
                }
                Err(e) => {
                    // Timeout or WouldBlock is expected, keep waiting
                    if e.kind() != std::io::ErrorKind::WouldBlock
                        && e.kind() != std::io::ErrorKind::TimedOut
                    {
                        tracing::warn!(error = %e, "Unexpected error while waiting for peer");
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
                tracing::info!("Peer connected");
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

impl std::fmt::Debug for LocalSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalSocket")
            .field("local_addr", &self.local_addr)
            .field("peer_addr", &self.peer_addr)
            .finish()
    }
}
