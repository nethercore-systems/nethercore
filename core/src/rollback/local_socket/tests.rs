//! Tests for local socket implementation

use std::time::Duration;

use ggrs::NonBlockingSocket;

use super::error::LocalSocketError;
use super::socket::LocalSocket;

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
