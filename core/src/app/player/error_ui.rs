//! Error screen UI and network waiting screen

use std::time::{Duration, Instant};

use super::super::GameError;
use crate::rollback::LocalSocket;

/// Action from error screen UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorAction {
    None,
    Restart,
    Quit,
}

/// State for waiting for a peer to connect in Host mode.
pub struct WaitingForPeer {
    /// The socket bound and waiting for connections
    pub socket: LocalSocket,
    /// The port we're hosting on
    pub port: u16,
    /// Local IP addresses to display
    pub local_ips: Vec<String>,
    /// Game ID for generating shareable join URLs
    pub game_id: String,
}

impl WaitingForPeer {
    pub fn new(socket: LocalSocket, port: u16, game_id: String) -> Self {
        let local_ips = LocalSocket::get_local_ips();
        Self {
            socket,
            port,
            local_ips,
            game_id,
        }
    }

    /// Generate a shareable join URL for an IP address
    pub fn join_url(&self, ip: &str) -> String {
        format!("nethercore://join/{}:{}/{}", ip, self.port, self.game_id)
    }
}

/// State for connecting to a host in Join mode.
pub struct JoiningPeer {
    /// The socket used for connection
    pub socket: LocalSocket,
    /// Address we're connecting to
    pub address: String,
    /// When the connection attempt started
    pub started_at: Instant,
    /// Connection timeout duration
    pub timeout: Duration,
    /// Number of connection attempts made
    pub attempt_count: u32,
    /// Current connection state
    pub state: JoinConnectionState,
    /// Error message if connection failed
    pub error: Option<String>,
}

/// State of a join connection attempt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinConnectionState {
    /// Attempting to connect to host
    Connecting,
    /// Connection established, waiting for initial response
    WaitingForResponse,
    /// Connected and ready
    Connected,
    /// Connection timed out
    TimedOut,
    /// Connection failed with error
    Failed,
}

impl JoiningPeer {
    /// Default connection timeout (10 seconds)
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

    /// Interval between connection probe packets
    pub const PROBE_INTERVAL: Duration = Duration::from_millis(250);

    /// Magic bytes for connection probe
    pub const PROBE_MAGIC: &'static [u8] = b"NC_JOIN";

    /// Magic bytes for connection acknowledgment
    pub const ACK_MAGIC: &'static [u8] = b"NC_ACK";

    pub fn new(socket: LocalSocket, address: String) -> Self {
        Self {
            socket,
            address,
            started_at: Instant::now(),
            timeout: Self::DEFAULT_TIMEOUT,
            attempt_count: 0,
            state: JoinConnectionState::Connecting,
            error: None,
        }
    }

    /// Check if the connection attempt has timed out
    pub fn is_timed_out(&self) -> bool {
        self.started_at.elapsed() > self.timeout
    }

    /// Get elapsed time since connection started
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Get remaining time before timeout
    pub fn remaining(&self) -> Duration {
        self.timeout.saturating_sub(self.started_at.elapsed())
    }

    /// Reset the connection attempt for retry
    pub fn reset_for_retry(&mut self) {
        self.started_at = Instant::now();
        self.attempt_count = 0;
        self.state = JoinConnectionState::Connecting;
        self.error = None;
    }

    /// Mark the connection as failed with an error
    pub fn fail(&mut self, error: impl Into<String>) {
        self.state = JoinConnectionState::Failed;
        self.error = Some(error.into());
    }

    /// Mark the connection as timed out
    pub fn mark_timed_out(&mut self) {
        self.state = JoinConnectionState::TimedOut;
        self.error = Some(format!(
            "Connection timed out after {} seconds",
            self.timeout.as_secs()
        ));
    }

    /// Mark the connection as connected
    pub fn mark_connected(&mut self) {
        self.state = JoinConnectionState::Connected;
        self.error = None;
    }

    /// Check if we should send another probe packet
    pub fn should_send_probe(&self) -> bool {
        matches!(self.state, JoinConnectionState::Connecting)
    }
}

/// Render the error screen overlay.
pub fn render_error_screen(ctx: &egui::Context, error: &GameError) -> ErrorAction {
    let mut action = ErrorAction::None;

    egui::Area::new(egui::Id::new("error_overlay_bg"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .order(egui::Order::Background)
        .show(ctx, |ui| {
            let screen = ctx
                .input(|i| i.raw.viewport().inner_rect)
                .unwrap_or_else(|| egui::Rect::from_min_size(egui::Pos2::ZERO, ctx.used_size()));
            ui.painter().rect_filled(
                screen,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200),
            );
        });

    egui::Window::new("Game Error")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(450.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("⚠")
                        .size(24.0)
                        .color(egui::Color32::YELLOW),
                );
                ui.heading(&error.summary);
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label(format!("Phase: {}", error.phase));
                if let Some(tick) = error.tick {
                    ui.separator();
                    ui.label(format!("Tick: {}", tick));
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            if !error.suggestions.is_empty() {
                ui.label(egui::RichText::new("Possible causes:").strong());
                for suggestion in &error.suggestions {
                    ui.horizontal(|ui| {
                        ui.label("  •");
                        ui.label(suggestion);
                    });
                }
                ui.add_space(10.0);
            }

            egui::CollapsingHeader::new("Error Details")
                .default_open(false)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut error.details.as_str())
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY),
                            );
                        });
                });

            if let Some(ref trace) = error.stack_trace {
                egui::CollapsingHeader::new("Stack Trace")
                    .default_open(false)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(150.0)
                            .show(ui, |ui| {
                                for frame in trace {
                                    ui.monospace(frame);
                                }
                            });
                    });
            }

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("Restart Game").clicked() {
                    action = ErrorAction::Restart;
                }
                ui.add_space(20.0);
                if ui.button("Quit").clicked() {
                    action = ErrorAction::Quit;
                }
            });

            ui.add_space(5.0);
            ui.label(egui::RichText::new("Press Escape to quit").weak().small());
        });

    action
}

/// Convert a game name to a URL-safe game ID.
///
/// - Lowercases the string
/// - Replaces spaces and underscores with hyphens
/// - Removes non-alphanumeric characters (except hyphens)
/// - Collapses multiple hyphens into one
pub fn sanitize_game_id(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    let mut last_was_hyphen = false;

    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            result.push(c.to_ascii_lowercase());
            last_was_hyphen = false;
        } else if (c == ' ' || c == '_' || c == '-') && !last_was_hyphen && !result.is_empty() {
            result.push('-');
            last_was_hyphen = true;
        }
        // Skip other characters
    }

    // Remove trailing hyphen
    if result.ends_with('-') {
        result.pop();
    }

    if result.is_empty() {
        "game".to_string()
    } else {
        result
    }
}

/// Parse a key string to a winit KeyCode.
pub fn parse_key_code(s: &str) -> Option<winit::keyboard::KeyCode> {
    use winit::keyboard::KeyCode;
    match s.to_uppercase().as_str() {
        "F1" => Some(KeyCode::F1),
        "F2" => Some(KeyCode::F2),
        "F3" => Some(KeyCode::F3),
        "F4" => Some(KeyCode::F4),
        "F5" => Some(KeyCode::F5),
        "F6" => Some(KeyCode::F6),
        "F7" => Some(KeyCode::F7),
        "F8" => Some(KeyCode::F8),
        "F9" => Some(KeyCode::F9),
        "F10" => Some(KeyCode::F10),
        "F11" => Some(KeyCode::F11),
        "F12" => Some(KeyCode::F12),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{JoinConnectionState, JoiningPeer, WaitingForPeer, parse_key_code, sanitize_game_id};
    use crate::rollback::LocalSocket;
    use winit::keyboard::KeyCode;

    #[test]
    fn sanitize_game_id_basic_transforms() {
        assert_eq!(sanitize_game_id("My Game"), "my-game");
        assert_eq!(sanitize_game_id("My__Game"), "my-game");
        assert_eq!(sanitize_game_id("Game---Name"), "game-name");
        assert_eq!(sanitize_game_id("Game-"), "game");
    }

    #[test]
    fn sanitize_game_id_falls_back_for_non_ascii_or_empty() {
        assert_eq!(sanitize_game_id(""), "game");
        assert_eq!(sanitize_game_id("日本語ゲーム"), "game");
        assert_eq!(sanitize_game_id("___"), "game");
        assert_eq!(sanitize_game_id("!!!"), "game");
    }

    #[test]
    fn parse_key_code_is_case_insensitive_for_function_keys() {
        assert_eq!(parse_key_code("f1"), Some(KeyCode::F1));
        assert_eq!(parse_key_code("F12"), Some(KeyCode::F12));
        assert_eq!(parse_key_code("F13"), None);
    }

    #[test]
    fn join_url_includes_ip_port_and_game_id() {
        let socket = LocalSocket::bind("0.0.0.0:0").expect("bind LocalSocket");
        let waiting = WaitingForPeer::new(socket, 1234, "my-game".to_string());
        assert_eq!(
            waiting.join_url("127.0.0.1"),
            "nethercore://join/127.0.0.1:1234/my-game"
        );
    }

    #[test]
    fn joining_peer_initial_state() {
        let socket = LocalSocket::bind("0.0.0.0:0").expect("bind LocalSocket");
        let joining = JoiningPeer::new(socket, "192.168.1.100:7777".to_string());

        assert_eq!(joining.address, "192.168.1.100:7777");
        assert_eq!(joining.state, JoinConnectionState::Connecting);
        assert_eq!(joining.attempt_count, 0);
        assert!(joining.error.is_none());
        assert!(!joining.is_timed_out());
    }

    #[test]
    fn joining_peer_fail_sets_error() {
        let socket = LocalSocket::bind("0.0.0.0:0").expect("bind LocalSocket");
        let mut joining = JoiningPeer::new(socket, "192.168.1.100:7777".to_string());

        joining.fail("Connection refused");

        assert_eq!(joining.state, JoinConnectionState::Failed);
        assert_eq!(joining.error, Some("Connection refused".to_string()));
    }

    #[test]
    fn joining_peer_mark_connected_clears_error() {
        let socket = LocalSocket::bind("0.0.0.0:0").expect("bind LocalSocket");
        let mut joining = JoiningPeer::new(socket, "192.168.1.100:7777".to_string());

        joining.fail("Some error");
        joining.mark_connected();

        assert_eq!(joining.state, JoinConnectionState::Connected);
        assert!(joining.error.is_none());
    }

    #[test]
    fn joining_peer_reset_for_retry() {
        let socket = LocalSocket::bind("0.0.0.0:0").expect("bind LocalSocket");
        let mut joining = JoiningPeer::new(socket, "192.168.1.100:7777".to_string());

        joining.attempt_count = 5;
        joining.fail("Timed out");

        joining.reset_for_retry();

        assert_eq!(joining.state, JoinConnectionState::Connecting);
        assert_eq!(joining.attempt_count, 0);
        assert!(joining.error.is_none());
    }

    #[test]
    fn joining_peer_should_send_probe_only_when_connecting() {
        let socket = LocalSocket::bind("0.0.0.0:0").expect("bind LocalSocket");
        let mut joining = JoiningPeer::new(socket, "192.168.1.100:7777".to_string());

        assert!(joining.should_send_probe());

        joining.mark_connected();
        assert!(!joining.should_send_probe());
    }
}
