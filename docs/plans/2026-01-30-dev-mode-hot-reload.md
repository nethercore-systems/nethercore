# Dev Mode: Hot Reload & Hot Relaunch

**Date:** 2026-01-30
**Status:** Phase 1 Complete (Hot Relaunch)

## Summary

Add a `--watch` flag to `nether run` that enables automatic rebuilding and reloading during development. WASM code changes trigger a full relaunch; asset-only changes trigger hot reload without restarting the game.

## Motivation

Currently, the dev workflow is:
1. Edit code/assets
2. Switch to terminal
3. Run `nether run` (or `nether build` + manual launch)
4. Wait for build
5. Test
6. Repeat

This is slow and breaks flow. Developers expect watch-mode tooling from modern dev environments.

## Design

### User Experience

```bash
# Current behavior (unchanged)
nether run                  # build → launch → exit when player closes

# New: dev mode with watch
nether run --watch          # build → launch → watch → rebuild/reload on changes
```

When `--watch` is active:
- CLI stays running after player launches
- CLI watches all files referenced in `nether.toml` (source + assets)
- On file change: rebuild ROM, diff against previous, signal player
- Continues until user presses Ctrl+C or player window is closed

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         nether run --watch                       │
├─────────────────────────────────────────────────────────────────┤
│  1. Parse nether.toml                                           │
│  2. Collect watch paths:                                        │
│     - Source files (Cargo.toml dir, or build.script location)   │
│     - Asset files (all paths in assets.*)                       │
│  3. Initial build → ROM₀                                        │
│  4. Launch player with --dev flag                               │
│  5. Enter watch loop:                                           │
│     ┌─────────────────────────────────────────────────────┐     │
│     │  Wait for file change (debounced)                   │     │
│     │  Rebuild → ROM₁                                     │     │
│     │  Diff ROM₀ vs ROM₁                                  │     │
│     │  If WASM changed:                                   │     │
│     │    → Send RESTART to player                         │     │
│     │    → Player exits, CLI relaunches with new ROM      │     │
│     │  Else (assets only):                                │     │
│     │    → Send RELOAD <asset_ids> to player              │     │
│     │    → Player reloads assets in-place                 │     │
│     │  ROM₀ = ROM₁                                        │     │
│     └─────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                    nethercore-zx --dev game.nczx                 │
├─────────────────────────────────────────────────────────────────┤
│  Normal player initialization                                    │
│  + IPC listener (named pipe / Unix socket)                      │
│  + Optional "DEV" badge in corner                               │
│                                                                  │
│  On RELOAD <asset_ids>:                                         │
│    - Re-read ROM file from disk                                 │
│    - For each asset_id: reload into GPU/audio system            │
│    - Game continues running (same WASM instance)                │
│                                                                  │
│  On RESTART:                                                    │
│    - Exit cleanly (exit code 75 = "restart requested")          │
│    - CLI detects exit code, relaunches player                   │
└─────────────────────────────────────────────────────────────────┘
```

### Why Hot Reload Works for Assets

Assets in Nethercore have a clean separation:

| Component | Location | What game holds |
|-----------|----------|-----------------|
| Textures | GPU (player-side) | Handle (u32 ID) |
| Meshes | GPU (player-side) | Handle (u32 ID) |
| Sounds | Audio system (player-side) | Handle (u32 ID) |
| Skeletons | Runtime (player-side) | Handle (u32 ID) |
| Keyframes | Runtime (player-side) | Handle (u32 ID) |
| Trackers | Runtime (player-side) | Handle (u32 ID) |

The WASM game only holds opaque handles. When the player reloads an asset:
1. New data is uploaded to GPU/audio under the **same handle ID**
2. Game keeps using the same handle
3. Next render/audio call automatically uses new data

No game cooperation required.

**Exception: Data blobs** - If the game reads a data blob once at `init()` and parses it into WASM memory, hot-reload won't affect it. This is acceptable; such cases fall back to relaunch.

### Why Hot Reload Doesn't Work for WASM

WASM code changes require relaunch because:
- WASM instance has internal state (globals, linear memory)
- Game state lives in WASM memory
- No way to "swap" code while preserving arbitrary state
- Even serialization wouldn't work reliably (memory layout may change)

### ROM Diffing

To determine what changed, compare ROM sections:

```
ROM Structure:
┌──────────────────────┐
│ Header               │
│   - wasm_hash: u64   │  ← Compare this
│   - asset_count: u32 │
├──────────────────────┤
│ WASM blob            │
├──────────────────────┤
│ Asset Table          │
│   - id: string       │
│   - hash: u64        │  ← Compare each
│   - offset: u32      │
│   - size: u32        │
├──────────────────────┤
│ Asset Data...        │
└──────────────────────┘
```

Diff algorithm:
1. Compare `wasm_hash` - if different, return `Relaunch`
2. Compare asset hashes by ID - collect changed IDs
3. If any assets changed, return `Reload(changed_ids)`
4. Otherwise, return `NoChange`

### IPC Protocol

Simple line-based protocol over named pipe (Windows) or Unix socket:

```
CLI → Player:
  RELOAD texture:player,sound:jump    # Hot reload specific assets
  RESTART                              # Request clean exit for relaunch
  PING                                 # Health check

Player → CLI:
  OK                                   # Command acknowledged
  ERROR <message>                      # Command failed
  PONG                                 # Response to PING
```

IPC path:
- Windows: `\\.\pipe\nethercore-dev-{pid}`
- Unix: `/tmp/nethercore-dev-{pid}.sock`

### File Watching

Watch targets (derived from nether.toml):

1. **Source files**: Watch the project directory for common source extensions
   - `.rs`, `.zig`, `.c`, `.cpp`, `.h`, `.toml`
   - Excludes `target/`, `zig-out/`, etc.

2. **Asset files**: Watch exact paths from `assets.*` sections
   - `assets/player.png`, `models/level.gltf`, etc.

3. **Manifest**: Watch `nether.toml` itself (triggers full rebuild + rewatch)

Debouncing: 100ms delay to batch rapid saves (e.g., editor auto-save + format).

### Edge Cases

| Scenario | Behavior |
|----------|----------|
| WASM + assets both change | Relaunch (conservative) |
| Build fails | Print error, keep watching, don't signal player |
| Player crashes | Detect exit, prompt user: relaunch or quit watch? |
| Player closed normally | Exit watch mode |
| nether.toml changes | Re-parse, update watch list, full rebuild |
| New asset added | Requires relaunch (new handle allocation) |
| Asset removed | Requires relaunch (handle cleanup) |

### CLI Changes

**New flag on `nether run`:**

```rust
#[derive(Args)]
pub struct RunArgs {
    // ... existing fields ...

    /// Watch for changes and hot reload/relaunch
    #[arg(long)]
    pub watch: bool,
}
```

**New module:** `tools/nether-cli/src/watch.rs`
- File watcher setup (using `notify` crate)
- ROM diffing logic
- IPC client to communicate with player

### Player Changes

**New flag:**

```rust
#[derive(Parser)]
pub struct Args {
    // ... existing fields ...

    /// Enable dev mode (IPC listener for hot reload)
    #[arg(long)]
    pub dev: bool,
}
```

**New module:** `library/src/dev_mode.rs` or `nethercore-zx/src/dev_mode.rs`
- IPC server (named pipe / Unix socket)
- Asset reload logic (re-read ROM, re-upload to GPU/audio)
- Clean exit on RESTART command

### Dependencies

New crates for CLI:
- `notify` - Cross-platform file watching

New crates for player:
- Platform-specific IPC (or use existing async runtime)

### Future Enhancements (Out of Scope)

- **State preservation**: Serialize game state before relaunch, restore after (complex, game-specific)
- **Partial WASM reload**: Only reload changed functions (requires toolchain support)
- **Network sync**: Hot reload across networked players in sync-test mode
- **Browser support**: WebSocket-based reload for future web player

## Implementation Plan

### Phase 1: Hot Relaunch Only (MVP) ✅ COMPLETE

1. ✅ Add `--watch` flag to `nether run`
2. ✅ Implement file watching with `notify` crate
3. ✅ On any change: rebuild ROM, kill player, relaunch
4. ✅ No IPC, no diffing - always full relaunch

This gives immediate value with minimal complexity.

**Implementation Notes:**
- `tools/nether-cli/src/watch.rs` - File watcher module
- `tools/nether-cli/src/run.rs` - Watch loop integration
- Watches source files (.rs, .zig, .c, .cpp, .h, .toml) and asset files from manifest
- Excludes build directories (target/, zig-out/, .git/, etc.)
- 100ms debounce to batch rapid saves

### Phase 2: ROM Diffing + Asset Hot Reload

1. Add hash fields to ROM format (if not already present)
2. Implement ROM diffing in CLI
3. Add `--dev` flag to player
4. Implement IPC server in player
5. Implement asset reload logic in player
6. CLI sends RELOAD/RESTART based on diff

### Phase 3: Polish

1. Add "DEV" badge to player window
2. Improve error messages and recovery
3. Add `nether dev` as alias for `nether run --watch`
4. Documentation and examples

## Design Decisions

1. **`--watch` is orthogonal to build mode.** Works with both `--debug` and default release builds. Developers choose speed vs optimization independently.

2. **No cargo watch integration.** Our file watcher handles everything; adding cargo watch detection overcomplicates the design for minimal benefit.

3. **Preserve frame count across asset hot reloads.** When reloading assets (not relaunching), the player maintains the current frame count. This enables debugging animations at specific frames without restarting.

## References

- Similar features: Bevy hot reload, Unity hot reload, Unreal Live Coding
- `notify` crate: https://docs.rs/notify
