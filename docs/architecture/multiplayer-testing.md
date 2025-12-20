# Multiplayer Testing Plan

> **Status:** Proposal
> **Created:** 2024-12-20

## Executive Summary

Emberware has a **complete GGRS-based rollback netcode implementation** but currently lacks an accessible way for developers and end users to test multiplayer functionality. This document outlines the current state and proposes a phased implementation plan.

---

## Current State

### What Exists

| Component | Status | Location |
|-----------|--------|----------|
| GGRS Rollback Session | ✅ Complete | `core/src/rollback/session.rs` |
| Local Session (no rollback) | ✅ Complete | `RollbackSession::new_local()` |
| Sync Test Session | ✅ Complete | `RollbackSession::new_sync_test()` |
| P2P Session | ✅ Complete | `RollbackSession::new_p2p()` |
| LocalSocket (UDP for testing) | ✅ Complete | `core/src/rollback/local_socket.rs` |
| Network Stats & Quality | ✅ Complete | `core/src/rollback/events.rs` |
| State Snapshots | ✅ Complete | `core/src/rollback/state.rs` |
| Example Games | ✅ Complete | `paddle` (2P), `platformer` (4P) |

### What's Missing

| Feature | Status | Issue |
|---------|--------|-------|
| CLI flags for session types | ❌ Missing | ConsoleRunner always creates Local session |
| Sync test mode exposure | ❌ Missing | Can't validate determinism without code changes |
| Local P2P testing workflow | ❌ Missing | Requires custom harness |
| UI multiplayer configuration | ❌ Missing | No player count selection |
| Developer documentation | ❌ Missing | No guide for testing multiplayer |

### The Problem

In `core/src/runner.rs:168`, games always start with a local session:

```rust
// Current behavior - always local
let rollback_session = RollbackSession::new_local(num_players, self.specs.ram_limit);
```

There's no mechanism to:
1. Run sync test sessions to validate determinism
2. Start P2P sessions for local network testing
3. Configure player count or input delay

---

## Three Session Types Explained

### 1. Local Session (Default)
- **Purpose:** Couch co-op, single machine
- **Rollback:** None (immediate execution)
- **Use case:** 1-4 players on same machine with multiple controllers
- **Current:** ✅ This is what games run by default

### 2. Sync Test Session
- **Purpose:** Validate game determinism
- **Rollback:** Simulates rollback every frame
- **Use case:** Developer testing before online release
- **Current:** ❌ Not exposed to users

**Why it matters:** Catches non-determinism bugs early:
- Floating point inconsistencies
- Uninitialized memory reads
- Order-dependent operations
- External randomness

### 3. P2P Session
- **Purpose:** Full rollback netcode
- **Rollback:** GGRS handles rollback on input mismatch
- **Use case:** Online multiplayer, LAN testing
- **Current:** ❌ Not exposed (needs LocalSocket integration)

---

## Implementation Plan

### Phase 1: CLI Flags for Session Types

**Goal:** Enable developers to test different session types from command line.

#### 1.1 Add Session Configuration to StandaloneConfig

```rust
// In library/src/config.rs or similar
pub struct SessionFlags {
    /// Number of local players (1-4)
    pub num_players: usize,
    /// Enable sync test mode (simulates rollback every frame)
    pub sync_test: bool,
    /// Input delay frames (0-10)
    pub input_delay: usize,
}
```

#### 1.2 CLI Argument Parsing

```bash
# Basic usage - current behavior
cargo run -- platformer

# Sync test mode - validate determinism
cargo run -- platformer --sync-test

# Multiple local players with explicit count
cargo run -- platformer --players 2

# With input delay (for testing feel at different latencies)
cargo run -- platformer --input-delay 3
```

#### 1.3 Modify ConsoleRunner::load_game

Accept session configuration:

```rust
pub fn load_game(
    &mut self,
    console: C,
    wasm_bytes: &[u8],
    session_config: SessionFlags,
) -> Result<()> {
    // ...existing code...

    let rollback_session = if session_config.sync_test {
        let config = SessionConfig::sync_test_with_delay(
            session_config.num_players,
            session_config.input_delay,
        );
        RollbackSession::new_sync_test(config, self.specs.ram_limit)?
    } else {
        RollbackSession::new_local(session_config.num_players, self.specs.ram_limit)
    };

    runtime.set_session(rollback_session);
}
```

#### Files to Modify
- `library/src/main.rs` - CLI argument parsing
- `library/src/config.rs` - Session configuration types
- `core/src/runner.rs` - Accept session config in load_game

---

### Phase 2: Local P2P Testing (Two Instances)

**Goal:** Enable testing P2P netcode on localhost with two game instances.

#### 2.1 P2P CLI Flags

```bash
# Instance 1 (Host - Player 1 local)
cargo run -- platformer --p2p --bind 7777 --peer 7778 --local-player 0

# Instance 2 (Client - Player 2 local)
cargo run -- platformer --p2p --bind 7778 --peer 7777 --local-player 1
```

#### 2.2 Session Creation with LocalSocket

```rust
// When --p2p is specified
let mut socket = LocalSocket::bind(&format!("127.0.0.1:{}", bind_port))?;
socket.connect(&format!("127.0.0.1:{}", peer_port))?;

let players = vec![
    (0, if local_player == 0 { PlayerType::Local } else { PlayerType::Remote(peer_addr) }),
    (1, if local_player == 1 { PlayerType::Local } else { PlayerType::Remote(peer_addr) }),
];

let config = SessionConfig::p2p(2, input_delay);
let rollback_session = RollbackSession::new_p2p(config, socket, players, ram_limit)?;
```

#### 2.3 Helper Script for Testing

Create `scripts/test-p2p.sh`:

```bash
#!/bin/bash
# Launches two game instances for P2P testing
GAME="${1:-platformer}"

# Start host in background
cargo run -- "$GAME" --p2p --bind 7777 --peer 7778 --local-player 0 &
HOST_PID=$!

# Wait for host to bind
sleep 1

# Start client
cargo run -- "$GAME" --p2p --bind 7778 --peer 7777 --local-player 1 &
CLIENT_PID=$!

# Wait for both
wait $HOST_PID $CLIENT_PID
```

#### Files to Modify
- `library/src/main.rs` - P2P CLI flags
- `core/src/runner.rs` - P2P session creation path
- `scripts/test-p2p.sh` - Helper script (new file)

---

### Phase 3: ember-cli Integration

**Goal:** Provide developer-friendly testing commands in ember-cli.

```bash
# Run game with sync test enabled
ember run --sync-test

# Run two-player P2P test (launches both instances)
ember run --p2p-test

# Run with simulated latency
ember run --sync-test --input-delay 3
```

#### Implementation in ember-cli

```rust
// tools/ember-cli/src/commands/run.rs

#[derive(Parser)]
pub struct RunCommand {
    /// Enable sync test mode for determinism validation
    #[arg(long)]
    sync_test: bool,

    /// Launch two instances for P2P testing
    #[arg(long)]
    p2p_test: bool,

    /// Simulated input delay frames (0-10)
    #[arg(long, default_value = "0")]
    input_delay: usize,

    /// Number of local players (1-4)
    #[arg(long, short, default_value = "1")]
    players: usize,
}
```

---

### Phase 4: Library UI Integration

**Goal:** Allow end users to configure multiplayer from the library UI.

#### 4.1 Game Launch Dialog

When launching a game that supports multiplayer:

```
┌─────────────────────────────────────────┐
│  Launch: Platformer                     │
├─────────────────────────────────────────┤
│  Players: ○ 1  ● 2  ○ 3  ○ 4           │
│                                         │
│  Mode:    ● Local (same machine)        │
│           ○ Host online game            │
│           ○ Join online game            │
│                                         │
│  [Advanced ▼]                           │
│                                         │
│        [Cancel]  [Launch]               │
└─────────────────────────────────────────┘
```

#### 4.2 Developer Options (Hidden Menu)

Hold Shift while launching to show developer options:

```
┌─────────────────────────────────────────┐
│  Developer Options                      │
├─────────────────────────────────────────┤
│  ☑ Enable sync test mode               │
│  Input delay: [0] frames               │
│  ☐ Show network stats overlay          │
│  ☐ Log rollback events                 │
└─────────────────────────────────────────┘
```

---

### Phase 5: Network Statistics Overlay

**Goal:** Visual feedback during P2P sessions.

Display in-game overlay (toggleable with F4):

```
┌──────────────────────────┐
│ P1: 45ms ████████ Good   │
│ P2: 78ms █████── Fair    │
│ Rollback: 3 frames       │
│ Frame: 1847              │
└──────────────────────────┘
```

Data sources (already implemented):
- `PlayerNetworkStats.ping_ms`
- `PlayerNetworkStats.quality` (Excellent/Good/Fair/Poor)
- `RollbackSession.total_rollback_frames()`
- `RollbackSession.current_frame()`

---

## Testing Workflow for Developers

### Daily Development (Phase 1)

```bash
# 1. Build your game
ember build

# 2. Run with sync test to catch determinism bugs
ember run --sync-test

# 3. If sync test passes, game is rollback-safe
```

### Pre-Release Testing (Phase 2)

```bash
# 1. Test on localhost with two instances
./scripts/test-p2p.sh my-game

# 2. Watch for desyncs in console output
# [DESYNC] Frame 1234: local=0xDEAD, remote=0xBEEF

# 3. If no desyncs, game is ready for online play
```

### User Experience (Phase 4)

1. Launch library
2. Click game
3. Select player count
4. Choose Local or Online
5. Play!

---

## Technical Considerations

### Session Type Selection Priority

```
CLI flags > Config file > UI selection > Default (Local)
```

### Graceful Degradation

If P2P connection fails:
1. Show clear error message
2. Offer to retry or fall back to local
3. Never leave player stuck

### Desync Handling

When desync detected:
1. Log detailed state comparison
2. Show user-friendly message
3. Offer to reconnect or return to menu

---

## File Reference

| File | Purpose |
|------|---------|
| `core/src/rollback/session.rs` | Session creation (modify for new entry points) |
| `core/src/rollback/local_socket.rs` | LocalSocket (use for P2P testing) |
| `core/src/runner.rs` | ConsoleRunner (modify load_game signature) |
| `library/src/main.rs` | CLI parsing (add flags) |
| `tools/ember-cli/src/commands/run.rs` | ember run command (add flags) |

---

## Success Criteria

### Phase 1 (Minimum Viable)
- [ ] `--sync-test` flag works
- [ ] `--players N` flag works
- [ ] `--input-delay N` flag works
- [ ] Sync test catches intentionally non-deterministic code

### Phase 2 (P2P Testing)
- [ ] Two instances can connect via LocalSocket
- [ ] Game plays normally with rollback
- [ ] Desyncs are detected and logged
- [ ] Helper script simplifies testing

### Phase 3 (Developer Experience)
- [ ] `ember run --sync-test` works
- [ ] `ember run --p2p-test` launches both instances

### Phase 4 (User Experience)
- [ ] Library shows player count selector
- [ ] Games launch with correct session type
- [ ] Error messages are user-friendly

### Phase 5 (Polish)
- [ ] Network stats overlay works
- [ ] Overlay is toggleable
- [ ] Stats are accurate

---

## Appendix: LocalSocket Usage

The `LocalSocket` type (`core/src/rollback/local_socket.rs`) implements GGRS's `NonBlockingSocket` trait for UDP communication:

```rust
// Instance 1 (port 7777)
let mut socket1 = LocalSocket::bind("127.0.0.1:7777")?;
socket1.connect("127.0.0.1:7778")?;

// Instance 2 (port 7778)
let mut socket2 = LocalSocket::bind("127.0.0.1:7778")?;
socket2.connect("127.0.0.1:7777")?;

// Create P2P session
let players = vec![
    (0, PlayerType::Local),
    (1, PlayerType::Remote("127.0.0.1:7778".to_string())),
];
let session = RollbackSession::new_p2p(config, socket1, players, ram_limit)?;
```

### Limitations
- Localhost only (no NAT traversal)
- No STUN/TURN (no public internet)
- Point-to-point (for >2 players, needs manual port assignment)

For production online play, a proper signaling server with WebRTC would be needed (out of scope for this testing plan).
