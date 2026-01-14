//! GGRS NonBlockingSocket trait implementation

use std::io;
use std::net::SocketAddr;

use ggrs::NonBlockingSocket;

use super::socket::LocalSocket;

impl NonBlockingSocket<String> for LocalSocket {
    fn send_to(&mut self, msg: &ggrs::Message, addr: &String) {
        // Parse the target address
        let target: SocketAddr = match addr.parse() {
            Ok(a) => a,
            Err(e) => {
                tracing::warn!(error = %e, "Invalid send address");
                return;
            }
        };

        // Serialize the GGRS message
        let data = match bincode::serialize(msg) {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to serialize message");
                return;
            }
        };

        // Send immediately
        if let Err(e) = self.socket.send_to(&data, target) {
            // WouldBlock is expected for non-blocking sockets when buffer is full
            if e.kind() != io::ErrorKind::WouldBlock {
                tracing::warn!(error = %e, "Failed to send message");
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
                            tracing::warn!(error = %e, "Failed to deserialize message");
                        }
                    }
                }
                Err(e) => {
                    // WouldBlock means no more data available
                    if e.kind() == io::ErrorKind::WouldBlock {
                        break;
                    }
                    // Other errors are unexpected
                    tracing::warn!(error = %e, "Receive error");
                    break;
                }
            }
        }

        messages
    }
}
