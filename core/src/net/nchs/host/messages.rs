//! Message handling and validation logic for host

use std::net::SocketAddr;
use std::time::Instant;

use crate::net::nchs::messages::{
    JoinAccept, JoinReject, JoinRejectReason, JoinRequest, NchsMessage,
};
use crate::net::nchs::NchsError;

use super::state::{ConnectedPlayer, HostEvent, HostState, HostStateMachine};

impl HostStateMachine {
    /// Handle an incoming message
    pub(super) fn handle_message(&mut self, from: SocketAddr, msg: NchsMessage) -> Option<HostEvent> {
        match msg {
            NchsMessage::JoinRequest(req) => self.handle_join_request(from, req),
            NchsMessage::GuestReady(ready) => self.handle_guest_ready(from, ready.ready),
            NchsMessage::Ping => {
                // Respond with Pong
                if let Err(e) = self.socket.send_to(from, &NchsMessage::Pong) {
                    tracing::warn!(error = %e, "Failed to send Pong to {}", from);
                }
                // Update last_seen if this is a known player
                if let Some(handle) = self.addr_to_handle.get(&from)
                    && let Some(player) = self.players.get_mut(handle)
                {
                    player.last_seen = Instant::now();
                }
                None
            }
            NchsMessage::PunchAck(_) => {
                // Ignore punch acks from guests to host
                None
            }
            _ => {
                tracing::warn!(?msg, "Unexpected message from peer");
                None
            }
        }
    }

    /// Handle a join request
    fn handle_join_request(&mut self, from: SocketAddr, req: JoinRequest) -> Option<HostEvent> {
        // Check if already connected
        if self.addr_to_handle.contains_key(&from) {
            // Resend accept with existing handle
            if let Some(&handle) = self.addr_to_handle.get(&from) {
                let accept = JoinAccept {
                    player_handle: handle,
                    lobby: self.lobby_state(),
                };
                if let Err(e) = self.socket.send_to(from, &NchsMessage::JoinAccept(accept)) {
                    tracing::warn!(error = %e, "Failed to send JoinAccept to {}", from);
                }
            }
            return None;
        }

        // Validate request
        if let Some(reject) = self.validate_join_request(&req) {
            if let Err(e) = self
                .socket
                .send_to(from, &NchsMessage::JoinReject(reject.clone()))
            {
                tracing::warn!(error = %e, "Failed to send JoinReject to {}", from);
            }
            return Some(HostEvent::Error(NchsError::ValidationFailed(format!(
                "{:?}",
                reject.reason
            ))));
        }

        // Check if lobby is full
        if self.is_full() {
            let reject = JoinReject {
                reason: JoinRejectReason::LobbyFull,
                message: None,
            };
            if let Err(e) = self.socket.send_to(from, &NchsMessage::JoinReject(reject)) {
                tracing::warn!(error = %e, "Failed to send JoinReject to {}", from);
            }
            return None;
        }

        // Assign handle and add player
        let handle = self.next_handle;
        self.next_handle += 1;

        let player = ConnectedPlayer {
            handle,
            info: req.player_info.clone(),
            addr: from,
            ready: false,
            last_seen: Instant::now(),
        };

        self.players.insert(handle, player);
        self.addr_to_handle.insert(from, handle);

        tracing::info!(player = handle, "Player joined");

        // Send accept
        let accept = JoinAccept {
            player_handle: handle,
            lobby: self.lobby_state(),
        };
        if let Err(e) = self.socket.send_to(from, &NchsMessage::JoinAccept(accept)) {
            tracing::warn!(error = %e, "Failed to send JoinAccept to {}", from);
        }

        // Broadcast lobby update to all other players
        self.broadcast_lobby_update();

        // Update state if we were just listening
        if self.state == HostState::Listening {
            self.state = HostState::Lobby;
        }

        Some(HostEvent::PlayerJoined {
            handle,
            info: req.player_info,
        })
    }

    /// Validate a join request
    fn validate_join_request(&self, req: &JoinRequest) -> Option<JoinReject> {
        // Check console type
        if req.console_type != self.netplay.console_type {
            return Some(JoinReject {
                reason: JoinRejectReason::ConsoleTypeMismatch,
                message: Some(format!(
                    "Expected {:?}, got {:?}",
                    self.netplay.console_type, req.console_type
                )),
            });
        }

        // Check ROM hash
        if req.rom_hash != self.netplay.rom_hash {
            return Some(JoinReject {
                reason: JoinRejectReason::RomHashMismatch,
                message: Some("Different game version".to_string()),
            });
        }

        // Check tick rate
        if req.tick_rate != self.netplay.tick_rate {
            return Some(JoinReject {
                reason: JoinRejectReason::TickRateMismatch,
                message: Some(format!(
                    "Expected {}Hz, got {}Hz",
                    self.netplay.tick_rate.as_hz(),
                    req.tick_rate.as_hz()
                )),
            });
        }

        // Check if game already started
        if self.state == HostState::Starting || self.state == HostState::Ready {
            return Some(JoinReject {
                reason: JoinRejectReason::GameInProgress,
                message: None,
            });
        }

        None
    }

    /// Handle guest ready state change
    fn handle_guest_ready(&mut self, from: SocketAddr, ready: bool) -> Option<HostEvent> {
        let handle = *self.addr_to_handle.get(&from)?;
        let player = self.players.get_mut(&handle)?;

        if player.ready != ready {
            player.ready = ready;
            player.last_seen = Instant::now();

            tracing::info!(player = handle, ready, "Player ready");

            // Broadcast lobby update
            self.broadcast_lobby_update();

            // Check if all ready
            if self.all_ready() && self.player_count() > 1 {
                return Some(HostEvent::AllReady);
            }

            return Some(HostEvent::PlayerReadyChanged { handle, ready });
        }

        None
    }
}
