//! Multiplayer connection dialog
//!
//! Provides UI for hosting or joining online games via direct IP connection.

use super::UiAction;

/// Tab selection for multiplayer mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MultiplayerTab {
    #[default]
    Host,
    Join,
}

/// Multiplayer connection dialog state
pub struct MultiplayerDialog {
    /// The game ID this dialog is for
    pub game_id: String,
    /// Current tab (Host or Join)
    pub tab: MultiplayerTab,
    /// Port number for hosting/joining
    pub port: String,
    /// Host IP address (for join mode)
    pub host_ip: String,
    /// Number of players (for host mode)
    pub players: usize,
    /// Error message to display
    pub error: Option<String>,
    /// Cached local IPs
    local_ips: Vec<String>,
}

impl MultiplayerDialog {
    /// Create a new multiplayer dialog for the given game
    pub fn new(game_id: String) -> Self {
        // Get local IPs for display
        let local_ips = Self::get_local_ips();

        Self {
            game_id,
            tab: MultiplayerTab::Host,
            port: "7777".to_string(),
            host_ip: String::new(),
            players: 2,
            error: None,
            local_ips,
        }
    }

    /// Get local IP addresses for display to the user
    fn get_local_ips() -> Vec<String> {
        // Try to get local IPs from network interfaces
        let mut ips = Vec::new();

        #[cfg(not(target_os = "windows"))]
        {
            // On Unix-like systems, use ifaddrs
            if let Ok(output) = std::process::Command::new("hostname")
                .arg("-I")
                .output()
            {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    for ip in stdout.split_whitespace() {
                        if !ip.starts_with("127.") && !ip.contains(':') {
                            ips.push(ip.to_string());
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            // On Windows, parse ipconfig output
            if let Ok(output) = std::process::Command::new("ipconfig").output() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    for line in stdout.lines() {
                        if line.contains("IPv4") {
                            if let Some(ip) = line.split(':').nth(1) {
                                let ip = ip.trim();
                                if !ip.starts_with("127.") {
                                    ips.push(ip.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback if no IPs found
        if ips.is_empty() {
            ips.push("Unable to detect".to_string());
        }

        ips
    }

    /// Render the dialog and return any action
    pub fn show(&mut self, ctx: &egui::Context) -> Option<UiAction> {
        let mut action = None;

        egui::Window::new("Online Play")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(350.0);

                // Tab selection
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.tab, MultiplayerTab::Host, "Host Game");
                    ui.selectable_value(&mut self.tab, MultiplayerTab::Join, "Join Game");
                });

                ui.separator();
                ui.add_space(10.0);

                match self.tab {
                    MultiplayerTab::Host => {
                        self.show_host_tab(ui, &mut action);
                    }
                    MultiplayerTab::Join => {
                        self.show_join_tab(ui, &mut action);
                    }
                }

                // Error display
                if let Some(ref error) = self.error {
                    ui.add_space(10.0);
                    ui.colored_label(egui::Color32::RED, error);
                }

                ui.add_space(10.0);
                ui.separator();

                // Cancel button
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        action = Some(UiAction::CancelMultiplayer);
                    }
                });
            });

        action
    }

    /// Show the host tab UI
    fn show_host_tab(&mut self, ui: &mut egui::Ui, action: &mut Option<UiAction>) {
        ui.label("Host a game and wait for friends to connect.");
        ui.add_space(10.0);

        // Player count selection
        ui.horizontal(|ui| {
            ui.label("Players:");
            for n in 2..=4 {
                ui.selectable_value(&mut self.players, n, format!("{}", n));
            }
        });

        ui.add_space(5.0);

        // Port input
        ui.horizontal(|ui| {
            ui.label("Port:");
            ui.add(egui::TextEdit::singleline(&mut self.port).desired_width(80.0));
        });

        ui.add_space(10.0);

        // Display local IPs
        ui.group(|ui| {
            ui.label("Share your IP with your friend:");
            for ip in &self.local_ips {
                ui.horizontal(|ui| {
                    let addr = format!("{}:{}", ip, self.port);
                    ui.monospace(&addr);
                    if ui.small_button("Copy").clicked() {
                        ui.ctx().copy_text(addr);
                    }
                });
            }
        });

        ui.add_space(10.0);

        // Note about port forwarding
        ui.small("Note: For internet play, you may need to forward this port on your router.");

        ui.add_space(10.0);

        // Start button
        if ui.button("Start Hosting").clicked() {
            if let Ok(port) = self.port.parse::<u16>() {
                *action = Some(UiAction::HostGame {
                    game_id: self.game_id.clone(),
                    port,
                    players: self.players,
                });
            } else {
                self.error = Some("Invalid port number".to_string());
            }
        }
    }

    /// Show the join tab UI
    fn show_join_tab(&mut self, ui: &mut egui::Ui, action: &mut Option<UiAction>) {
        ui.label("Enter your friend's IP address to connect.");
        ui.add_space(10.0);

        // IP input
        ui.horizontal(|ui| {
            ui.label("Friend's IP:");
            ui.add(
                egui::TextEdit::singleline(&mut self.host_ip)
                    .desired_width(150.0)
                    .hint_text("192.168.1.100"),
            );
        });

        // Port input
        ui.horizontal(|ui| {
            ui.label("Port:");
            ui.add(egui::TextEdit::singleline(&mut self.port).desired_width(80.0));
        });

        ui.add_space(10.0);

        // Connection tips
        ui.group(|ui| {
            ui.label("Connection tips:");
            ui.small("- For LAN: Use your friend's local IP (192.168.x.x)");
            ui.small("- For internet: Use your friend's public IP");
            ui.small("- Make sure your friend is hosting first");
        });

        ui.add_space(10.0);

        // Connect button
        if ui.button("Connect").clicked() {
            if self.host_ip.is_empty() {
                self.error = Some("Please enter an IP address".to_string());
            } else if let Ok(port) = self.port.parse::<u16>() {
                *action = Some(UiAction::JoinGame {
                    game_id: self.game_id.clone(),
                    host_ip: self.host_ip.clone(),
                    port,
                });
            } else {
                self.error = Some("Invalid port number".to_string());
            }
        }
    }
}
