# Nethercore Handshake (NCHS) Protocol Specification

> Status: Implemented (core protocol) — integration details may vary by frontend
> Last reviewed: 2026-01-06

## Problem Statement

GGRS (the rollback netcode library) only allows sending GGRS-specific data (inputs and sync messages). This creates several problems:

1. **No pre-game metadata exchange** - Can't share usernames, avatars, or player info
2. **No shared random seed** - Critical for deterministic rollback; without a shared seed, games desync immediately
3. **No game validation** - Players might be running different game versions, causing silent desyncs
4. **No connection orchestration** - For 4-player games, who connects to whom?
5. **Complex setup** - Players shouldn't need to know about ports or IP addresses

This spec defines the **Nethercore Handshake (NCHS)** protocol - a handshake layer that runs *before* GGRS, establishing all shared state needed for deterministic multiplayer with minimal player friction.

## Design Goals

1. **Zero port configuration for guests** - Only host needs to think about networking
2. **Game codes** - Simple 6-character codes instead of IP addresses
3. **Automatic peer discovery** - Host tells everyone about everyone else
4. **NAT-friendly** - Relay fallback when direct connection fails
5. **Fast validation** - Catch game mismatches before wasting time
6. **Instant start** - Game preloaded during lobby, zero delay when host starts

---

## Game Configuration in Manifest

### The Simpler Approach

Instead of extracting tick rate and render mode from `init()` at runtime, declare them in `nether.toml`:

```toml
# nether.toml
[game]
id = "my-game"
title = "My Game"
author = "Developer"
version = "0.1.0"

# Netplay-critical config (baked into the ROM)
tick_rate = 60      # Updates per second (30, 60, or 120)
max_players = 4     # Maximum players supported (1-4)

# ZX-only (0-3): 0=Lambert, 1=Matcap, 2=MR-Blinn-Phong, 3=Specular-Shininess
# Default: 0
render_mode = 0

[netplay]
enabled = true        # Can this game be played online?
```

### Why Manifest is Better

1. **Known before loading** - No need to run `init()` to discover config
2. **Simpler validation** - Just compare manifest values, no runtime extraction
3. **No chicken-and-egg** - Config available before any code runs
4. **Build-time validation** - `nether build` can verify settings
5. **Clear contract** - Game config is explicit, not hidden in code

### Trade-off

This requires the `nether.toml` workflow. Fully procedural games that want to dynamically choose tick rate at runtime cannot be played online. This is acceptable because:

- Variable tick rate games **can't do rollback netplay anyway**
- Tick rate is a fundamental design decision, not a runtime choice
- Most games use standard rates (60fps for action, 30fps for strategy)

### Tick Rate Options

```toml
# Standard rates supported
tick_rate = 30   # Strategy games, turn-based
tick_rate = 60   # Action games (default)
tick_rate = 120  # Fighting games, precision required
```

Tick rate and render mode are **manifest-declared** and baked into the ROM. For online play, tick rate is enforced by the NCHS session config (GGRS fps) and must match across peers; games should treat it as fixed for the session.

---

## Texture Compression

### Overview

Textures can be stored in two formats:

| Format | Use Case | Compression |
|--------|----------|-------------|
| **RGBA8** | Pixel art, sprites, fonts, UI | None (default) |
| **BC7** | 3D textures, environment, characters | ~4:1 compression |

### When to Compress

**Compress (BC7)** - Most 3D game textures:
- Environment textures (terrain, skybox)
- Character diffuse/material maps
- Props and objects
- Matcap textures
- MRE/SSE material textures

**Don't Compress (RGBA8)** - Precision-critical assets:
- Pixel art (compression artifacts destroy crisp edges)
- Sprite sheets with precise transparency
- Bitmap fonts
- UI elements
- Textures with sharp color transitions

### Manifest Configuration

#### Global Setting

```toml
[game]
id = "my-game"
title = "My Game"
author = "Developer"
version = "1.0.0"
compress_textures = true   # Compress ALL textures by default
```

Default is `false` (RGBA8) for backward compatibility and pixel-art friendliness.

> Note: As of the current `nether-cli` packer, texture compression is a **single global choice**
> (`compress_textures`) applied to all textures. Per-texture overrides are not implemented yet.

### Compression and Render Modes

| Render Mode | Recommended Default | Reason |
|-------------|---------------------|--------|
| Mode 0 (Lambert) | `compress_textures = false` | 2D/sprite games, RGBA8 preferred |
| Mode 1 (Matcap) | `compress_textures = true` | 3D stylized, BC7 saves space |
| Mode 2 (MR-Blinn-Phong) | `compress_textures = true` | 3D PBR, BC7 required for materials |
| Mode 3 (Specular-Shininess) | `compress_textures = true` | 3D retro, BC7 saves space |

The `nether build` command will warn if compression settings don't match render mode:

```
WARNING: Detected render_mode 1+ (Matcap/MR-Blinn-Phong/Specular-Shininess) but compress_textures=false.
      Consider enabling texture compression for better performance:
      Add 'compress_textures = true' to [game] section in nether.toml
```

### Size Impact

Example ROM sizes for a typical 3D game:

| Compression | Texture Size | ROM Size |
|-------------|--------------|----------|
| RGBA8 (none) | 4 MB | ~5 MB |
| BC7 (compressed) | 1 MB | ~2 MB |

Savings: **~75% reduction** in texture size.

### Pre-Load Flow (Simplified)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     Optimized Connection Flow                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌────────────┐   ┌────────────┐   ┌─────────────┐   ┌──────────────────┐  │
│  │  Pre-Load  │ → │   NCHS     │ → │  Peer       │ → │  GGRS Session    │  │
│  │  Game      │   │  Handshake │   │  Connection │   │  (Gameplay)      │  │
│  │            │   │            │   │             │   │                  │  │
│  │ - Read     │   │ - Validate │   │ - Connect   │   │ - Input sync     │  │
│  │   manifest │   │   manifest │   │   to peers  │   │ - Rollback       │  │
│  │ - Load ROM │   │ - Get seed │   │ - UDP punch │   │ - Snapshots      │  │
│  │ - Call init()│ │ - Get peers│   │             │   │                  │  │
│  └────────────┘   └────────────┘   └─────────────┘   └──────────────────┘  │
│        │                                                      │             │
│        └──────────────── Game already loaded ─────────────────┘             │
│                         INSTANT START!                                       │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Manifest in ROM

The manifest is embedded in the ROM header, so it's available without parsing WASM:

```rust
pub struct RomHeader {
    /// Magic bytes "NCRT" (Nethercore ROM)
    pub magic: [u8; 4],
    /// ROM format version
    pub version: u16,
    /// Console type
    pub console_type: ConsoleType,
    /// ROM hash (xxHash3 of WASM section)
    pub rom_hash: u64,

    // === From nether.toml ===
    pub tick_rate: TickRate,
    pub render_mode: RenderMode,
    pub max_players: u8,
    pub netplay_enabled: bool,
}

#[repr(u8)]
pub enum TickRate {
    /// 30 updates per second
    Fixed30 = 30,
    /// 60 updates per second (default)
    Fixed60 = 60,
    /// 120 updates per second
    Fixed120 = 120,
}
```

### Validation During Join

```rust
fn validate_join_request(host: &RomHeader, guest: &JoinRequest) -> Result<(), JoinRejectReason> {
    if host.rom_hash != guest.rom_hash {
        return Err(JoinRejectReason::RomMismatch);
    }
    if host.tick_rate != guest.tick_rate {
        return Err(JoinRejectReason::TickRateMismatch);
    }
    if !host.netplay_enabled {
        return Err(JoinRejectReason::NetplayDisabled);
    }
    Ok(())
}
```

---

## Save Slot Ordering Problem

### The Chicken-and-Egg

There's a conflict between pre-loading and save slots:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  PROBLEM: What happens when?                                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  init() may need:                 │  NCHS provides:                         │
│  ───────────────                  │  ──────────────                         │
│  • Player handle (which player?)  │  • Player handle (via JoinAccept)       │
│  • Save slot data                 │  • Save slot index (via SessionStart)   │
│                                   │                                          │
│  BUT init() runs BEFORE NCHS      │  BUT NCHS runs AFTER init()             │
│  (to set up game state)           │  (to assign player handles)             │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Solution: Two-Phase Initialization

Split initialization into two phases:

```rust
/// Phase 1: Called BEFORE network connection
/// Sets up game state defaults, does NOT access save slots or player handles
/// Note: tick_rate and render_mode come from nether.toml, not code
#[no_mangle]
pub extern "C" fn init() {
    // Initialize game state to defaults
    // DO NOT access save slots or player_handle() here!

    // Set up static data structures
    init_entity_pool();
    init_audio_banks();
}

/// Phase 2: Called AFTER network connection established
/// Now we know our player handle and can access save slots
#[no_mangle]
pub extern "C" fn post_connect() {
    // Now player_handle() returns our assigned handle
    let handle = player_handle();

    // Now we can load save data
    if let Some(save) = load_save_slot(handle) {
        restore_progress(&save);
    }
}
```

### Updated Connection Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     Full Connection Flow with Save Slots                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌────────────┐   ┌────────────┐   ┌─────────────┐   ┌────────────────────┐ │
│  │  Phase 1   │ → │   NCHS     │ → │  Phase 2    │ → │  GGRS Session      │ │
│  │  Pre-Load  │   │  Handshake │   │  Post-Connect│  │  (Gameplay)        │ │
│  │            │   │            │   │             │   │                    │ │
│  │ - Read hdr │   │ - Validate │   │ - Apply seed│   │ - Input sync       │ │
│  │ - Load ROM │   │   manifest │   │ - Load saves│   │ - Rollback         │ │
│  │ - Call init()│ │ - Get peers│   │ - post_connect()│ - Snapshots       │ │
│  │ - NO saves │   │ - Assign   │   │             │   │                    │ │
│  │            │   │   handles  │   │             │   │                    │ │
│  └────────────┘   └────────────┘   └─────────────┘   └────────────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### What init() CAN Do

- Initialize default game state
- Set up static data structures
- Configure audio settings (volume, etc.)
- Initialize entity pools, asset caches
- Set up input mappings

### What init() CANNOT Do (Must Wait for post_connect)

- Access `player_handle()` - not assigned yet
- Load save slots - need player handle first
- Access network state - not connected yet
- Read player count - still in lobby

### Save Slot Data in SessionStart

Add save slot synchronization to `SessionStart`:

```rust
pub struct SessionStart {
    // ... existing fields ...

    /// Save slot configuration for this session
    pub save_config: SaveConfig,
}

pub struct SaveConfig {
    /// Which save slot index to use (0-3 typically)
    pub slot_index: u8,

    /// Save slot mode
    pub mode: SaveMode,

    /// For synchronized saves: the save data to use
    /// All players start with identical save state
    pub synchronized_save: Option<Vec<u8>>,
}

#[repr(u8)]
pub enum SaveMode {
    /// Each player uses their own local save
    PerPlayer = 0x00,
    /// All players share host's save (synchronized)
    Synchronized = 0x01,
    /// Fresh game, no save data
    NewGame = 0x02,
}
```

### post_connect() Implementation

The runtime calls `post_connect()` after NCHS completes:

```rust
impl GameSession {
    fn after_nchs_complete(&mut self, session: &SessionStart) {
        // 1. Apply shared random seed
        self.game_state.rng_state = session.random_seed;

        // 2. Set player handles
        for (i, player) in session.players.iter().enumerate() {
            if player.active {
                self.player_handles[i] = player.handle;
            }
        }

        // 3. Load/sync save data
        match session.save_config.mode {
            SaveMode::PerPlayer => {
                // Load local save for this player
                let my_handle = self.local_player_handle();
                self.load_local_save(my_handle, session.save_config.slot_index);
            }
            SaveMode::Synchronized => {
                // Apply host's save data
                if let Some(save_data) = &session.save_config.synchronized_save {
                    self.apply_synchronized_save(save_data);
                }
            }
            SaveMode::NewGame => {
                // No save data, start fresh
            }
        }

        // 4. Call game's post_connect hook
        self.game.call_export("post_connect");
    }
}
```

### Backward Compatibility

Games that don't need save slots during init don't need to change:

```rust
// Old games: post_connect is optional
if game.has_export("post_connect") {
    game.call_export("post_connect");
}
```

### Alternative: Lazy Player Handle

For simpler games, `player_handle()` could return a sentinel value before connection:

```rust
/// Returns player handle, or 0xFF if not yet connected
pub fn player_handle() -> u8;

/// Returns true if we're connected and handle is valid
pub fn is_connected() -> bool;
```

Games can then defer handle-dependent logic:

```rust
fn init() {
    // Initialize game state
    // Don't access player_handle() here - not assigned yet
}

fn update() {
    // First frame after connection, load save
    static mut SAVE_LOADED: bool = false;
    if !SAVE_LOADED && is_connected() {
        load_my_save();
        SAVE_LOADED = true;
    }
    // ... rest of update
}
```

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Connection Lifecycle                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌────────────┐   ┌─────────────┐   ┌─────────────┐   ┌──────────────────┐  │
│  │  Join via  │ → │   NCHS      │ → │  Peer       │ → │  GGRS Session    │  │
│  │  Game Code │   │  Handshake  │   │  Connection │   │  (Gameplay)      │  │
│  │            │   │             │   │             │   │                  │  │
│  │ "ABC123"   │   │ - Validate  │   │ - Connect   │   │ - Input sync     │  │
│  │            │   │ - Get seed  │   │   to peers  │   │ - Rollback       │  │
│  │            │   │ - Get peers │   │ - UDP punch │   │ - Snapshots      │  │
│  └────────────┘   └─────────────┘   └─────────────┘   └──────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Player Experience

### Hosting a Game

```
1. Player selects game from library
2. (Behind scenes: ROM loads, init() called, game paused at frame 0)
3. Player selects "Host Game"
4. Game displays: "Your game code: ABC123"
5. (Behind scenes: NCHS listener starts, registers with relay)
6. Players join...
7. Host sees lobby fill up
8. Host presses "Start"
9. Game begins INSTANTLY (already loaded!)
```

### Joining a Game

```
1. Player selects game from library
2. (Behind scenes: ROM loads, init() called, game paused at frame 0)
3. Player selects "Join Game"
4. Player enters code: "ABC123"
5. (Behind scenes: NCHS connects, validates ROM + tick rate, gets peer list)
6. Player sees lobby with other players
7. Host starts game
8. Game begins INSTANTLY (already loaded!)
```

**No ports. No IPs. Just a code. Instant start.**

---

## Game Codes

### Format

```
6 alphanumeric characters (uppercase + digits, no ambiguous chars)
Alphabet: ABCDEFGHJKLMNPQRSTUVWXYZ23456789  (no 0/O, 1/I/L)
Examples: "ABC123", "XY7KM2", "H3NQ9P"
```

### Resolution

Game codes resolve to connection info via:

1. **Relay server lookup** (primary) - `relay.nethercore.systems/code/ABC123`
2. **Local network broadcast** (LAN fallback) - mDNS/UDP broadcast

```rust
pub struct GameCodeRegistration {
    /// The 6-character game code
    pub code: [u8; 6],
    /// Host's public address (from relay's perspective)
    pub host_public_addr: SocketAddr,
    /// Host's local address (for LAN play)
    pub host_local_addr: SocketAddr,
    /// Host's relay channel ID (for NAT traversal)
    pub relay_channel: u64,
    /// ROM hash for pre-validation
    pub rom_hash: u64,
    /// Current player count / max players
    pub players: u8,
    pub max_players: u8,
    /// Timestamp for expiry
    pub created_at: u64,
}
```

---

## Connection Topology

### Host-Centric Model

For simplicity, **the host orchestrates all connections**:

```
                    ┌──────────┐
         ┌──────────│   Host   │──────────┐
         │          │ (P0)     │          │
         │          └────┬─────┘          │
         │               │                │
         ▼               ▼                ▼
    ┌────────┐      ┌────────┐      ┌────────┐
    │ Guest1 │      │ Guest2 │      │ Guest3 │
    │  (P1)  │      │  (P2)  │      │  (P3)  │
    └────────┘      └────────┘      └────────┘
         │               │                │
         └───────────────┼────────────────┘
                    (mesh after handshake)
```

1. All guests connect to host first (via game code)
2. Host validates each guest (game hash, version)
3. Once all players joined, host broadcasts peer list
4. Guests establish direct connections to each other
5. GGRS session starts with full mesh

### Why Host-Centric?

- **Simple for players** - Only host needs open port (or relay)
- **Centralized validation** - Host is authority on game settings
- **Ordered joining** - Player handles assigned in join order
- **Clean lobby UX** - Host controls when game starts

---

## Port Handling

### Host Port Selection

Host needs ONE port. Selection priority:

```rust
const DEFAULT_NCHS_PORT: u16 = 7770;
const GGRS_PORT_OFFSET: u16 = 1;  // GGRS uses port + 1

pub fn select_host_port() -> io::Result<(u16, u16)> {
    // Try default first
    if let Ok(socket) = UdpSocket::bind(("0.0.0.0", DEFAULT_NCHS_PORT)) {
        drop(socket);
        return Ok((DEFAULT_NCHS_PORT, DEFAULT_NCHS_PORT + GGRS_PORT_OFFSET));
    }

    // Try a few alternatives
    for port in [7780, 7790, 27770, 27780] {
        if let Ok(socket) = UdpSocket::bind(("0.0.0.0", port)) {
            drop(socket);
            return Ok((port, port + GGRS_PORT_OFFSET));
        }
    }

    // Let OS pick
    let socket = UdpSocket::bind(("0.0.0.0", 0))?;
    let port = socket.local_addr()?.port();
    Ok((port, port + GGRS_PORT_OFFSET))
}
```

### Guest Port Selection

Guests don't care about ports - they use ephemeral ports:

```rust
pub fn select_guest_port() -> io::Result<u16> {
    // Always let OS pick - guests never need port forwarding
    let socket = UdpSocket::bind(("0.0.0.0", 0))?;
    Ok(socket.local_addr()?.port())
}
```

### 4-Player Port Assignment

After handshake, each player needs to know everyone's GGRS address:

```
Player 0 (Host):  public_ip:7771  (or relay)
Player 1 (Guest): public_ip:49152 (ephemeral, via relay/punch)
Player 2 (Guest): public_ip:52341 (ephemeral, via relay/punch)
Player 3 (Guest): public_ip:51888 (ephemeral, via relay/punch)
```

The host distributes this in the `SessionStart` message (see below).

---

## Protocol Messages

### Magic & Versioning

```rust
/// Magic bytes identifying NCHS messages
const NCHS_MAGIC: [u8; 4] = *b"NCHS";

/// Protocol version
const NCHS_VERSION: u16 = 1;
```

### Message Format

```
┌────────────────────────────────────────────────────────────────┐
│  0   1   2   3   4   5   6   7   8   ...                       │
├────────────────────────────────────────────────────────────────┤
│  N   C   H   S │ Ver │ Type │ Len │ Payload...                 │
│  (magic)       │(u16)│ (u8) │(u16)│                            │
└────────────────────────────────────────────────────────────────┘
```

### Message Types

```rust
#[repr(u8)]
pub enum NchsMessageType {
    // === Guest → Host ===
    /// Request to join session
    JoinRequest = 0x01,
    /// Confirm ready to start
    GuestReady = 0x02,

    // === Host → Guest ===
    /// Accept join, send player handle
    JoinAccept = 0x10,
    /// Reject join (game full, hash mismatch, etc.)
    JoinReject = 0x11,
    /// Lobby update (player joined/left)
    LobbyUpdate = 0x12,
    /// Session starting, here's everything you need
    SessionStart = 0x20,

    // === Peer ↔ Peer ===
    /// UDP hole punch packet
    PunchHello = 0x30,
    /// Punch acknowledged
    PunchAck = 0x31,

    // === Any ===
    /// Heartbeat/keepalive
    Ping = 0xF0,
    Pong = 0xF1,
    /// Error
    Error = 0xFF,
}
```

---

## Handshake Flow

### Phase 1: Join Request

Guest connects to host via game code:

```
   Guest                         Host
     │                             │
     │──── JoinRequest ───────────►│
     │                             │  (validate game hash)
     │◄─── JoinAccept/Reject ──────│
     │                             │
```

#### JoinRequest (Guest → Host)

```rust
pub struct JoinRequest {
    /// Game ROM hash (xxHash3)
    pub rom_hash: u64,
    /// ROM size in bytes
    pub rom_size: u32,
    /// Console type
    pub console_type: ConsoleType,
    /// Runtime version
    pub runtime_version: RuntimeVersion,

    // === From ROM Header (nether.toml baked in) ===

    /// Tick rate from manifest
    pub tick_rate: TickRate,
    /// Max players from manifest
    pub max_players: u8,

    /// Guest's player info
    pub player_info: PlayerInfo,
    /// Guest's observed public address (if known from STUN)
    pub public_addr: Option<SocketAddr>,
    /// Guest's local address (for LAN detection)
    pub local_addr: SocketAddr,
}

pub struct PlayerInfo {
    /// Display name (UTF-8, null-terminated, max 32 bytes)
    pub name: [u8; 32],
    /// Avatar ID (0 = default)
    pub avatar_id: u16,
    /// Preferred color (RGB)
    pub color: [u8; 3],
}

pub struct RuntimeVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

#[repr(u8)]
pub enum ConsoleType {
    ZX = 0x01,
    Chroma = 0x02,
    One = 0x03,
}
```

#### JoinAccept (Host → Guest)

```rust
pub struct JoinAccept {
    /// Assigned player handle (1, 2, or 3; host is always 0)
    pub player_handle: u8,
    /// Current lobby state
    pub lobby: LobbyState,
}

pub struct LobbyState {
    /// All players currently in lobby
    pub players: [Option<LobbyPlayer>; 4],
    /// Max players for this session
    pub max_players: u8,
    /// Host's player info
    pub host_info: PlayerInfo,
}

pub struct LobbyPlayer {
    pub handle: u8,
    pub info: PlayerInfo,
    pub ready: bool,
}
```

#### JoinReject (Host → Guest)

```rust
pub struct JoinReject {
    pub reason: JoinRejectReason,
}

#[repr(u8)]
pub enum JoinRejectReason {
    /// Lobby is full
    LobbyFull = 0x01,
    /// ROM hash doesn't match
    RomMismatch = 0x02,
    /// Console type doesn't match
    ConsoleMismatch = 0x03,
    /// Runtime version incompatible
    RuntimeIncompatible = 0x04,
    /// Tick rate doesn't match (from manifest)
    TickRateMismatch = 0x05,
    /// Game doesn't support netplay (netplay.enabled = false)
    NetplayDisabled = 0x06,
    /// Game already in progress
    GameInProgress = 0x07,
    /// Banned/blocked
    Blocked = 0x08,
}
```

### Phase 2: Lobby

Players wait in lobby, can see each other:

```
   Guest1        Host         Guest2
     │            │             │
     │◄── LobbyUpdate ─────────►│  (Guest2 joined)
     │            │             │
     │── GuestReady ──►│        │
     │            │◄── GuestReady ─│
     │            │             │
```

#### LobbyUpdate (Host → All Guests)

Sent whenever lobby state changes:

```rust
pub struct LobbyUpdate {
    pub lobby: LobbyState,
    /// What changed
    pub event: LobbyEvent,
}

#[repr(u8)]
pub enum LobbyEvent {
    PlayerJoined = 0x01,
    PlayerLeft = 0x02,
    PlayerReady = 0x03,
    PlayerUnready = 0x04,
    SettingsChanged = 0x05,
}
```

#### GuestReady (Guest → Host)

```rust
pub struct GuestReady {
    pub ready: bool,
}
```

### Phase 3: Session Start

Host has all players, starts the game:

```
   Guest1        Host         Guest2        Guest3
     │            │             │             │
     │◄────────── SessionStart ──────────────►│
     │            │             │             │
     │            │             │             │
     │◄─────── PunchHello ──────┼─────────────│
     │──────── PunchAck ────────┼────────────►│
     │            │             │             │
     │◄───────────┼── PunchHello ─────────────│
     │────────────┼── PunchAck ──────────────►│
     │            │             │             │
     ╔════════════════════════════════════════╗
     ║         GGRS SESSION BEGINS            ║
     ╚════════════════════════════════════════╝
```

#### SessionStart (Host → All)

**The critical message** - contains everything needed for deterministic play:

```rust
pub struct SessionStart {
    // === Determinism-Critical ===

    /// **THE RANDOM SEED** - All clients MUST use this
    pub random_seed: u64,

    /// Starting frame number (always 0 for new games)
    pub start_frame: u32,

    // === Network Topology ===

    /// All player connection info
    pub players: [PlayerConnectionInfo; 4],

    /// Number of active players
    pub player_count: u8,

    // === Session Settings ===

    /// Network configuration
    pub network_config: NetworkConfig,

    /// Game settings
    pub game_settings: GameSettings,
}

pub struct PlayerConnectionInfo {
    /// Player handle (0-3)
    pub handle: u8,

    /// Is this slot active?
    pub active: bool,

    /// Player display info
    pub info: PlayerInfo,

    /// Public address for direct connection
    pub public_addr: SocketAddr,

    /// Local address (for LAN optimization)
    pub local_addr: SocketAddr,

    /// Relay channel ID (fallback if direct fails)
    pub relay_channel: u64,

    /// This player's GGRS port
    pub ggrs_port: u16,
}

pub struct NetworkConfig {
    /// Input delay in frames (0-10)
    pub input_delay: u8,

    /// Max rollback frames (typically 8)
    pub max_rollback: u8,

    /// Disconnect timeout in ms
    pub disconnect_timeout_ms: u32,

    /// Enable desync detection
    pub desync_detection: bool,
}

pub struct GameSettings {
    /// Fixed timestep in microseconds (16667 = 60fps)
    pub timestep_us: u32,

    /// Allow spectators
    pub spectators_allowed: bool,

    /// Max spectators
    pub max_spectators: u8,
}
```

### Phase 4: Peer Connection

After receiving `SessionStart`, guests connect to each other:

#### UDP Hole Punching

For NAT traversal, peers send simultaneous packets to punch through:

```rust
pub struct PunchHello {
    /// Sender's player handle
    pub from_handle: u8,
    /// Target's player handle
    pub to_handle: u8,
    /// Random nonce for this punch attempt
    pub nonce: u32,
}

pub struct PunchAck {
    /// Echo the nonce
    pub nonce: u32,
    /// Confirmed player handle
    pub handle: u8,
}
```

#### Punch Sequence

```
1. All guests receive SessionStart with peer addresses
2. Each guest sends PunchHello to all other peers (not host)
3. When PunchHello received, respond with PunchAck
4. When PunchAck received, mark peer as connected
5. After all peers connected (or timeout), start GGRS
```

For 4 players, punch connections needed:

```
P1 ←→ P2  (Guest1 ↔ Guest2)
P1 ←→ P3  (Guest1 ↔ Guest3)
P2 ←→ P3  (Guest2 ↔ Guest3)

Host (P0) already has direct connection to all guests.
Total: 3 peer pairs for 4 players
```

#### Relay Fallback

If direct punch fails after 3 attempts (1.5s total):

```rust
pub struct RelayRequest {
    /// Source player
    pub from_handle: u8,
    /// Destination player
    pub to_handle: u8,
    /// Relay channel from SessionStart
    pub channel: u64,
}
```

The relay server forwards packets between peers who can't connect directly.

---

## Random Seed Distribution

### Why This Is Critical

The `random()` FFI function depends on deterministic RNG:

```rust
// In HostRollbackState (rolled back with game state)
pub struct HostRollbackState {
    pub rng_state: u64,  // ← This MUST be identical on all clients at frame 0
    pub tick_count: u64,
    pub elapsed_time_bits: u32,
}
```

Without a shared initial seed, the first `random()` call will return different values on different clients → **immediate desync**.

### Seed Generation (Host Only)

```rust
use rand::RngCore;

impl Host {
    fn generate_session(&self) -> SessionStart {
        let mut rng = rand::thread_rng();

        SessionStart {
            random_seed: rng.next_u64(),  // Cryptographically random
            // ...
        }
    }
}
```

### Seed Application (All Clients)

```rust
impl GameSession {
    fn apply_session_start(&mut self, msg: &SessionStart) {
        // CRITICAL: Apply the shared seed before first frame
        self.game_state.rng_state = msg.random_seed;

        // Apply other settings
        self.input_delay = msg.network_config.input_delay;
        self.max_rollback = msg.network_config.max_rollback;
    }
}
```

---

## NAT Traversal Strategy

### Connection Attempt Order

For each peer pair:

```
1. Try LAN address first (same subnet?)     [0-100ms]
2. Try public address (direct)              [100-500ms]
3. Try UDP hole punch (simultaneous send)   [500-1500ms]
4. Fall back to relay                       [1500ms+]
```

### STUN for Public Address Discovery

Before hosting, query public IP:

```rust
const STUN_SERVERS: &[&str] = &[
    "stun.nethercore.systems:3478",
    "stun.l.google.com:19302",  // Fallback
];

pub async fn discover_public_addr(local_socket: &UdpSocket) -> Option<SocketAddr> {
    for server in STUN_SERVERS {
        if let Ok(addr) = stun_request(local_socket, server).await {
            return Some(addr);
        }
    }
    None
}
```

### Relay Server

For players behind symmetric NAT or strict firewalls:

```
┌─────────┐         ┌─────────────┐         ┌─────────┐
│ Player1 │ ──────► │   Relay     │ ◄────── │ Player2 │
│ (NAT)   │ ◄────── │   Server    │ ──────► │ (NAT)   │
└─────────┘         └─────────────┘         └─────────┘
```

Relay adds latency but ensures connectivity.

---

## Error Handling

### Error Message

```rust
pub struct ErrorMessage {
    pub code: NchsError,
    pub message: [u8; 64],  // Human-readable, null-terminated
}

#[repr(u8)]
pub enum NchsError {
    /// Protocol version mismatch
    VersionMismatch = 0x01,
    /// Unexpected message in current state
    InvalidState = 0x02,
    /// Timeout waiting for response
    Timeout = 0x03,
    /// Peer disconnected
    Disconnected = 0x04,
    /// Handshake aborted
    Aborted = 0x05,
    /// Relay unavailable
    RelayUnavailable = 0x06,
    /// All connection methods failed
    ConnectionFailed = 0x07,
}
```

### Timeout Policy

| Phase | Timeout | Retries | Total |
|-------|---------|---------|-------|
| JoinRequest | 2s | 3 | 6s |
| LobbyUpdate | 5s | ∞ | (heartbeat) |
| SessionStart | 3s | 3 | 9s |
| PunchHello | 500ms | 3 | 1.5s |
| Total handshake | - | - | <15s |

---

## State Machine

### Host States

```
┌──────────────────────────────────────────────────────────────────────────┐
│                         Host State Machine                                │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│    ┌─────────┐                                                            │
│    │  Idle   │                                                            │
│    └────┬────┘                                                            │
│         │ host_game()                                                     │
│         ▼                                                                 │
│    ┌─────────────┐                                                        │
│    │  Listening  │◄──────────────────────────────────────┐               │
│    │             │                                        │               │
│    │ - Register game code                                │               │
│    │ - Wait for guests                                   │               │
│    └──────┬──────┘                                        │               │
│           │ recv JoinRequest                              │               │
│           ▼                                               │               │
│    ┌─────────────┐   reject    ┌─────────────┐           │               │
│    │  Validating │────────────►│  (continue) │───────────┘               │
│    │             │             └─────────────┘                            │
│    └──────┬──────┘                                                        │
│           │ valid → send JoinAccept                                       │
│           ▼                                                               │
│    ┌─────────────┐                                                        │
│    │   Lobby     │◄─────────────────────────┐                            │
│    │             │                           │                            │
│    │ - Show players                         │ player joins/leaves        │
│    │ - Wait for ready                       │                            │
│    └──────┬──────┘───────────────────────────┘                            │
│           │ all ready + host starts                                       │
│           ▼                                                               │
│    ┌─────────────┐                                                        │
│    │  Starting   │                                                        │
│    │             │                                                        │
│    │ - Generate seed                                                      │
│    │ - Send SessionStart                                                  │
│    │ - Wait for punches                                                   │
│    └──────┬──────┘                                                        │
│           │ all peers connected                                           │
│           ▼                                                               │
│    ╔═════════════╗                                                        │
│    ║    GGRS     ║                                                        │
│    ╚═════════════╝                                                        │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

### Guest States

```
┌──────────────────────────────────────────────────────────────────────────┐
│                         Guest State Machine                               │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│    ┌─────────┐                                                            │
│    │  Idle   │                                                            │
│    └────┬────┘                                                            │
│         │ join_game("ABC123")                                             │
│         ▼                                                                 │
│    ┌─────────────┐                                                        │
│    │  Resolving  │                                                        │
│    │             │                                                        │
│    │ - Lookup game code                                                   │
│    │ - Get host address                                                   │
│    └──────┬──────┘                                                        │
│           │ resolved                                                      │
│           ▼                                                               │
│    ┌─────────────┐                                                        │
│    │  Joining    │                                                        │
│    │             │                                                        │
│    │ - Send JoinRequest                                                   │
│    │ - Wait for Accept                                                    │
│    └──────┬──────┘                                                        │
│           │ recv JoinAccept                                               │
│           ▼                                                               │
│    ┌─────────────┐                                                        │
│    │   Lobby     │                                                        │
│    │             │                                                        │
│    │ - See other players                                                  │
│    │ - Mark ready                                                         │
│    │ - Wait for start                                                     │
│    └──────┬──────┘                                                        │
│           │ recv SessionStart                                             │
│           ▼                                                               │
│    ┌─────────────┐                                                        │
│    │  Punching   │                                                        │
│    │             │                                                        │
│    │ - Apply seed (!!)                                                    │
│    │ - Connect to peers                                                   │
│    │ - Hole punch                                                         │
│    └──────┬──────┘                                                        │
│           │ all peers connected                                           │
│           ▼                                                               │
│    ╔═════════════╗                                                        │
│    ║    GGRS     ║                                                        │
│    ╚═════════════╝                                                        │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Guide

### File Structure

```
core/src/net/
├── mod.rs
├── nchs/
│   ├── mod.rs           # Module root, NchsHandler
│   ├── messages.rs      # All message types
│   ├── host.rs          # Host state machine
│   ├── guest.rs         # Guest state machine
│   ├── lobby.rs         # Lobby management
│   ├── punch.rs         # UDP hole punching
│   ├── relay.rs         # Relay client
│   └── codes.rs         # Game code generation/resolution
├── stun.rs              # STUN client for public IP
└── simulator.rs         # Network condition simulator
```

### Core API

```rust
/// Nethercore Handshake handler
pub struct NchsHandler {
    state: NchsState,
    socket: UdpSocket,
    config: NchsConfig,
}

impl NchsHandler {
    /// Host a new game session
    pub async fn host(config: NchsConfig) -> Result<Self, NchsError> {
        let socket = UdpSocket::bind(("0.0.0.0", 0))?;
        let code = register_game_code(&socket).await?;
        println!("Game code: {}", code);
        Ok(Self { state: NchsState::Listening, socket, config })
    }

    /// Join an existing game
    pub async fn join(code: &str, player_info: PlayerInfo) -> Result<Self, NchsError> {
        let host_addr = resolve_game_code(code).await?;
        let socket = UdpSocket::bind(("0.0.0.0", 0))?;
        // Send JoinRequest...
        Ok(Self { state: NchsState::Joining, socket, config: Default::default() })
    }

    /// Poll for events (non-blocking)
    pub fn poll(&mut self) -> NchsEvent {
        // Process incoming packets, advance state machine
    }

    /// Get session config (only valid after SessionStart received)
    pub fn session_config(&self) -> Option<&SessionStart> {
        match &self.state {
            NchsState::Ready(config) => Some(config),
            _ => None,
        }
    }

    /// Transition to GGRS (consumes handler)
    pub fn into_ggrs(self) -> Result<GgrsSessionBuilder, NchsError> {
        let config = self.session_config().ok_or(NchsError::InvalidState)?;
        // Build GGRS session with all peer connections...
    }
}

pub enum NchsEvent {
    /// Still working
    Pending,
    /// Game code ready (host only)
    CodeReady(String),
    /// Lobby updated
    LobbyUpdated(LobbyState),
    /// Player joined (host only)
    PlayerJoined(PlayerInfo),
    /// Ready to start GGRS
    Ready(SessionStart),
    /// Error occurred
    Error(NchsError),
}
```

### Integration Example

```rust
// Hosting a game
async fn host_multiplayer(game: &GameData) -> Result<GgrsSession, Error> {
    let mut nchs = NchsHandler::host(NchsConfig {
        max_players: 4,
        rom_hash: game.rom_hash(),
        player_info: local_player_info(),
        ..Default::default()
    }).await?;

    // Wait for players and start
    loop {
        match nchs.poll() {
            NchsEvent::CodeReady(code) => {
                ui.show_game_code(&code);
            }
            NchsEvent::PlayerJoined(info) => {
                ui.add_player(&info);
            }
            NchsEvent::Ready(session) => {
                break;
            }
            NchsEvent::Error(e) => return Err(e.into()),
            _ => {}
        }

        if ui.start_pressed() && nchs.all_ready() {
            nchs.start_session()?;
        }

        tokio::time::sleep(Duration::from_millis(16)).await;
    }

    nchs.into_ggrs()
}

// Joining a game
async fn join_multiplayer(code: &str, game: &GameData) -> Result<GgrsSession, Error> {
    let mut nchs = NchsHandler::join(code, local_player_info()).await?;

    loop {
        match nchs.poll() {
            NchsEvent::LobbyUpdated(lobby) => {
                ui.show_lobby(&lobby);
            }
            NchsEvent::Ready(session) => {
                break;
            }
            NchsEvent::Error(e) => return Err(e.into()),
            _ => {}
        }

        tokio::time::sleep(Duration::from_millis(16)).await;
    }

    nchs.into_ggrs()
}
```

---

## Summary

| Problem | Solution |
|---------|----------|
| Players need to share ports/IPs | **Game codes** - 6-char code resolves to host |
| No shared random seed | **SessionStart.random_seed** - host generates, all clients apply |
| No game validation | **JoinRequest.rom_hash** - validated before joining lobby |
| Tick rate mismatch | **nether.toml** - baked into ROM header, validated before join |
| 4-player connection complexity | **Host-centric** - host distributes peer list, guests punch through |
| NAT traversal | **STUN + hole punch + relay fallback** |
| No player names/avatars | **PlayerInfo** in JoinRequest/JoinAccept |
| Save slots need player handle | **Two-phase init** - init() for config, post_connect() for saves |
| Slow game start after lobby | **Pre-load** - game loaded before handshake, instant start |

NCHS ensures:
1. **Zero configuration for guests** - Just enter a code
2. **All clients start with identical state** - Shared seed prevents desync
3. **Validated game versions** - ROM hash + tick rate from manifest checked before lobby
4. **Robust connectivity** - Multiple fallback strategies
5. **Instant start** - Games pre-loaded during lobby wait
6. **Save slot compatibility** - post_connect() hook for save-dependent init
7. **Simple manifest** - Tick rate, max players declared in nether.toml
