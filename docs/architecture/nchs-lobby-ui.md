# NCHS Lobby UI Specification

> Status: Implemented (or in-progress) — verify against `nethercore/core/src/net/nchs/` and `nethercore/library/src/ui/`
> Last reviewed: 2026-01-06

## Overview

This document specifies the multiplayer lobby UI for NCHS (Nethercore Handshake) protocol integration. The lobby provides a visual interface for hosting/joining games, managing ready state, and transitioning to gameplay.

## User Flows

### Host Flow
```
Main Menu → [Host Game] → Lobby Screen (Hosting)
                              ↓
                         Wait for players
                              ↓
                         All Ready?
                              ↓
                         [Start Game] → Loading → Gameplay
```

### Guest Flow
```
Main Menu → [Join Game] → Enter IP:Port → Connecting...
                              ↓
                         Lobby Screen (Guest)
                              ↓
                         [Ready] toggle
                              ↓
                         Host starts → Loading → Gameplay
```

## Lobby States

### Host States

| State | Description | UI Elements |
|-------|-------------|-------------|
| `Listening` | Bound to port, waiting for players | Port display, "Waiting for players..." |
| `Lobby` | Players connected, waiting for ready | Player list, Start button (disabled until all ready) |
| `Starting` | SessionStart sent, punching | "Starting session..." |
| `Ready` | All connected, transitioning to GGRS | "Loading game..." |

### Guest States

| State | Description | UI Elements |
|-------|-------------|-------------|
| `Connecting` | Sent JoinRequest, waiting | "Connecting to host..." |
| `Lobby` | Accepted, in lobby | Player list, Ready checkbox |
| `Punching` | Received SessionStart, hole punching | "Connecting to peers..." |
| `Ready` | All connected, transitioning to GGRS | "Loading game..." |
| `Failed` | Rejected or timeout | Error message, Back button |

## UI Components

### 1. Lobby Header

```
┌─────────────────────────────────────────────────┐
│  MULTIPLAYER LOBBY                              │
│  Game: Paddle Arena                             │
│  Host: 192.168.1.50:7770    [Copy]              │
└─────────────────────────────────────────────────┘
```

**Elements:**
- Game title (from ROM metadata)
- Host address (for sharing with friends)
- Copy button for address

### 2. Player List

```
┌─────────────────────────────────────────────────┐
│  PLAYERS (2/4)                                  │
├─────────────────────────────────────────────────┤
│  [●] Player 1 (Host)              ✓ READY       │
│  [●] Player 2                     ✓ READY       │
│  [ ] Waiting for player...                      │
│  [ ] Waiting for player...                      │
└─────────────────────────────────────────────────┘
```

**Elements:**
- Player count indicator (current/max)
- Player slots with:
  - Color indicator (from PlayerInfo.color)
  - Player name (from PlayerInfo.name)
  - Host badge for player 0
  - Ready status indicator
- Empty slots show "Waiting for player..."

### 3. Ready Controls

**For Host:**
```
┌─────────────────────────────────────────────────┐
│  [  START GAME  ]                               │
│  (Waiting for all players to be ready)          │
└─────────────────────────────────────────────────┘
```
- Button disabled until `all_ready() && player_count() >= 2`
- Tooltip shows why disabled

**For Guest:**
```
┌─────────────────────────────────────────────────┐
│  [ ] I'm Ready                                  │
│  (Waiting for host to start)                    │
└─────────────────────────────────────────────────┘
```
- Checkbox toggles ready state
- Status text updates based on lobby state

### 4. Connection Status

```
┌─────────────────────────────────────────────────┐
│  ● Connected                    Ping: 24ms      │
└─────────────────────────────────────────────────┘
```

**States:**
- `● Connected` (green) - In lobby
- `● Connecting...` (yellow) - JoinRequest sent
- `● Connecting to peers...` (yellow) - Hole punching
- `● Disconnected` (red) - Failed

### 5. Error Display

```
┌─────────────────────────────────────────────────┐
│  ⚠ Connection Failed                            │
│                                                 │
│  ROM version mismatch                           │
│  Host: v1.2.0 (hash: abcd1234)                 │
│  You:  v1.1.0 (hash: efgh5678)                 │
│                                                 │
│  [  Back to Menu  ]                             │
└─────────────────────────────────────────────────┘
```

**Error Types:**
| Reason | User-Friendly Message |
|--------|----------------------|
| `RomHashMismatch` | "ROM version mismatch" |
| `ConsoleTypeMismatch` | "Console type mismatch" |
| `TickRateMismatch` | "Game speed mismatch" |
| `LobbyFull` | "Lobby is full" |
| `GameInProgress` | "Game already in progress" |
| `Timeout` | "Connection timed out" |

## Screen Layouts

### Host Lobby Screen

```
┌─────────────────────────────────────────────────────────────┐
│                    MULTIPLAYER LOBBY                        │
│                                                             │
│  Game: Paddle Arena v1.0                                    │
│  Your Address: 192.168.1.50:7770  [Copy]                    │
│  Share this with friends to join!                           │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  PLAYERS (2/4)                                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ [■] HostPlayer (You)                    ✓ Ready     │   │
│  │ [■] GuestPlayer1                        ✓ Ready     │   │
│  │ [ ] Waiting for player...                           │   │
│  │ [ ] Waiting for player...                           │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│                    [  START GAME  ]                         │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  ● Connected                                   [  Back  ]   │
└─────────────────────────────────────────────────────────────┘
```

### Guest Lobby Screen

```
┌─────────────────────────────────────────────────────────────┐
│                    MULTIPLAYER LOBBY                        │
│                                                             │
│  Game: Paddle Arena v1.0                                    │
│  Host: 192.168.1.50:7770                                    │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  PLAYERS (2/4)                                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ [■] HostPlayer                          ✓ Ready     │   │
│  │ [■] You                                 ○ Not Ready │   │
│  │ [ ] Waiting for player...                           │   │
│  │ [ ] Waiting for player...                           │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│              [✓] I'm Ready                                  │
│              Waiting for host to start...                   │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  ● Connected                                   [  Leave  ]  │
└─────────────────────────────────────────────────────────────┘
```

### Connecting Screen (Guest)

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│                    CONNECTING...                            │
│                                                             │
│                    ◐ ◓ ◑ ◒                                  │
│                                                             │
│              Connecting to 192.168.1.50:7770                │
│                                                             │
│                                                             │
│                                              [  Cancel  ]   │
└─────────────────────────────────────────────────────────────┘
```

### Join Dialog

```
┌─────────────────────────────────────────────────────────────┐
│                    JOIN GAME                                │
│                                                             │
│  Enter host address:                                        │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 192.168.1.50:7770                                   │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Recent:                                                    │
│  • 192.168.1.50:7770 (Paddle Arena)                        │
│  • 10.0.0.5:7770 (Neon Racer)                              │
│                                                             │
│              [  Cancel  ]     [  Connect  ]                 │
└─────────────────────────────────────────────────────────────┘
```

## Implementation Phases

### Phase 1: Basic Lobby
- [ ] Host/Join buttons in main menu
- [ ] Simple text-based lobby display
- [ ] Ready checkbox for guests
- [ ] Start button for host
- [ ] Basic error messages

### Phase 2: Polish
- [ ] Player color indicators
- [ ] Animated connecting states
- [ ] Copy address to clipboard
- [ ] Recent connections history
- [ ] Ping display

### Phase 3: Advanced
- [ ] Player avatars
- [ ] Chat/emotes
- [ ] Spectator support
- [ ] Game settings (input delay, etc.)

## Integration Points

### PlayerApp Changes

```rust
enum AppState {
    // Existing...
    MainMenu,
    Playing,

    // New lobby states
    HostLobby { session: NchsSession },
    JoinDialog { address: String },
    Connecting { session: NchsSession },
    GuestLobby { session: NchsSession },
    SessionStarting { session: NchsSession },
}
```

### Event Handling

```rust
// In update loop
match &mut self.state {
    AppState::HostLobby { session } => {
        match session.poll() {
            NchsEvent::PlayerJoined { handle, info } => {
                // Update player list UI
            }
            NchsEvent::LobbyUpdated(lobby) => {
                // Refresh player list
            }
            NchsEvent::Ready(session_start) => {
                // Transition to game loading
                self.state = AppState::SessionStarting { ... };
            }
            _ => {}
        }
    }
    // Similar for GuestLobby...
}
```

### UI Rendering

```rust
fn render_lobby(&self, ctx: &egui::Context, session: &NchsSession) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("MULTIPLAYER LOBBY");

        // Game info
        ui.label(format!("Game: {}", self.game_name));

        if session.role() == NchsRole::Host {
            ui.horizontal(|ui| {
                ui.label(format!("Your Address: 127.0.0.1:{}", session.port()));
                if ui.button("Copy").clicked() {
                    // Copy to clipboard
                }
            });
        }

        // Player list
        ui.separator();
        if let Some(lobby) = session.lobby() {
            ui.label(format!("PLAYERS ({}/{})",
                lobby.players.iter().filter(|p| p.active).count(),
                lobby.max_players));

            for slot in &lobby.players {
                ui.horizontal(|ui| {
                    if slot.active {
                        if let Some(info) = &slot.info {
                            let color = egui::Color32::from_rgb(
                                info.color[0], info.color[1], info.color[2]);
                            ui.colored_label(color, "●");
                            ui.label(&info.name);

                            if slot.handle == 0 {
                                ui.label("(Host)");
                            }

                            if slot.ready {
                                ui.label("✓ Ready");
                            }
                        }
                    } else {
                        ui.label("[ ] Waiting for player...");
                    }
                });
            }
        }

        // Controls
        ui.separator();
        match session.role() {
            NchsRole::Host => {
                let can_start = session.all_ready() && session.player_count() >= 2;
                if ui.add_enabled(can_start, egui::Button::new("START GAME")).clicked() {
                    // Call session.start()
                }
            }
            NchsRole::Guest => {
                // Ready checkbox
                let mut ready = session.is_ready();
                if ui.checkbox(&mut ready, "I'm Ready").changed() {
                    session.set_ready(ready);
                }
            }
        }
    });
}
```

## ROM Metadata Requirements

For the lobby to display game information, ROMs should include:
- `game_name` - Display name
- `game_version` - Version string (optional, for mismatch messages)
- `netplay.enabled` - Must be true
- `netplay.max_players` - 2-4

## Accessibility Considerations

- High contrast text for readability
- Keyboard navigation for all controls
- Screen reader labels for status indicators
- Clear visual distinction between states
- Colorblind-friendly status indicators (not just red/green)

## Future Considerations

### Game Codes (Phase 2)
When relay server is added:
- Replace IP:port with 6-character game codes
- "Create Game" generates code
- "Join Game" accepts code

### LAN Discovery (Phase 2)
- "Find Local Games" button
- mDNS broadcast/discovery
- Auto-populate lobby list
