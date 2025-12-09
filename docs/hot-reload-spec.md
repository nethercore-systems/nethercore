# WASM Hot Reload Specification

## Overview

WASM Hot Reload enables game developers to recompile their game code and see changes reflected immediately in the running emulator without restarting. This dramatically reduces iteration time from "rebuild + restart + navigate back to problem area" to just "rebuild".

## Use Cases

1. **Rapid Iteration**: Developer tweaks a function, rebuilds, and sees the change in under a second
2. **Live Tuning**: Adjust gameplay values (speeds, timings, physics) while the game is running
3. **Bug Investigation**: Modify code to add debug output while reproducing a bug
4. **Art Integration**: Test visual changes in context without losing game state

## Architecture

### File Watching

The emulator watches the game's WASM file for changes:

```
~/.emberware/games/{game_id}/rom.wasm
```

On file change detection:
1. Wait for write to complete (debounce ~100ms)
2. Validate new WASM module
3. Perform hot swap

### Hot Swap Strategy

**Option A: Full Module Replacement (Recommended)**
- Compile new WASM module
- Serialize current game state via `save_state`
- Instantiate new module
- Call `load_state` with serialized state
- Continue execution

**Option B: Function-Level Patching**
- Parse both old and new modules
- Identify changed functions
- Patch only changed functions in-place
- Requires complex diffing and memory layout compatibility

**Recommendation**: Option A is simpler, more reliable, and leverages existing `save_state`/`load_state` infrastructure.

## FFI API

### Host-to-Game Notifications

```rust
// Called before hot reload to let game prepare
// Return value: game-specific version/signature for compatibility check
extern "C" fn __hot_reload_prepare() -> u32;

// Called after hot reload completes
// prev_version: value returned by __hot_reload_prepare() from old module
extern "C" fn __hot_reload_complete(prev_version: u32);
```

### Game-to-Host Queries

```rust
// Check if hot reload is supported in current session
extern "C" fn hot_reload_enabled() -> i32;

// Get number of hot reloads this session (useful for debugging)
extern "C" fn hot_reload_count() -> u32;
```

## Workflow

```
┌─────────────────────────────────────────────────────────────────┐
│                     Hot Reload Sequence                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. File watcher detects rom.wasm change                       │
│                    │                                            │
│                    ▼                                            │
│  2. Debounce (100ms) - wait for write complete                 │
│                    │                                            │
│                    ▼                                            │
│  3. Compile new WASM module (async, in background)             │
│                    │                                            │
│                    ├──── Compilation error ──► Show error UI   │
│                    │                           Continue running │
│                    ▼                                            │
│  4. Pause game loop                                            │
│                    │                                            │
│                    ▼                                            │
│  5. Call __hot_reload_prepare() on old module                  │
│                    │                                            │
│                    ▼                                            │
│  6. Serialize game state via save_state                        │
│                    │                                            │
│                    ▼                                            │
│  7. Instantiate new module                                     │
│                    │                                            │
│                    ├──── Instantiation error ──► Show error UI │
│                    │                             Revert to old  │
│                    ▼                                            │
│  8. Call init() on new module                                  │
│                    │                                            │
│                    ▼                                            │
│  9. Restore state via load_state                               │
│                    │                                            │
│                    ├──── State incompatible ──► Show warning   │
│                    │                            Reset to init   │
│                    ▼                                            │
│  10. Call __hot_reload_complete(prev_version)                  │
│                    │                                            │
│                    ▼                                            │
│  11. Resume game loop                                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## State Compatibility

### The Challenge

Game state includes memory layout. If a developer changes struct layouts, the serialized state from the old module may not be compatible with the new module.

### Solutions

1. **Version Tag**: Games export a `__state_version() -> u32` function. If version changes, state is incompatible.

2. **Schema Hash**: Automatically hash the memory layout of registered debug variables. If hash changes, warn but attempt reload.

3. **Graceful Degradation**:
   - Try to load state
   - If `load_state` returns error, call `init()` instead
   - Show notification: "State incompatible, game reset"

4. **Selective State**: Games can implement `__hot_reload_preserve() -> *const u8` to return only stable state (e.g., player position, level ID) that survives layout changes.

## Configuration

```toml
# config.toml
[hot_reload]
enabled = true
watch_path = "~/.emberware/games/{game_id}/rom.wasm"
debounce_ms = 100
compile_timeout_ms = 5000
show_notifications = true
auto_retry_on_error = false
```

## UI Integration

### Notifications

- **Success**: Brief toast "Hot reload complete (23ms)"
- **Compile Error**: Persistent panel showing compiler output
- **State Warning**: Toast "State incompatible, game reset"

### Debug Panel Integration

Add to debug panel:
- Hot reload status (enabled/disabled)
- Reload count this session
- Last reload timestamp
- Manual "Reload Now" button
- "Watch File" toggle

### Hotkey

- **Ctrl+R**: Force hot reload (even if file unchanged)
- **Ctrl+Shift+R**: Force hot reload with state reset

## Edge Cases

### During Netplay
Hot reload is disabled during P2P netplay sessions. Both players would need synchronized reloads which is impractical.

### During Rollback
If hot reload triggers during a GGRS rollback:
- Queue the reload
- Complete rollback first
- Apply reload after rollback settles

### Multiple Rapid Changes
Debounce handles this - only the final stable file is loaded.

### Corrupt WASM
Validation catches this before any state modification.

## Pending Questions

### Q1: Should `init()` be called on reload?
**Options:**
- A) Always call `init()` after reload, then `load_state` (current proposal)
- B) Skip `init()` if state loads successfully
- C) Add `__hot_reload_init()` separate from `init()`

**Consideration**: `init()` typically registers resources (textures, etc). These need re-registration with new module. Recommend Option A.

### Q2: How to handle resource handles?
When the new module calls `init()`, it will re-register textures, meshes, etc. The resource manager needs to handle this gracefully:
- Clear old registrations
- Allow re-registration with same handles
- Or: persist resource manager state across reload

### Q3: Watch file or directory?
**Options:**
- A) Watch specific `rom.wasm` file
- B) Watch entire game directory for any `.wasm` file
- C) Watch source directory and trigger builds

**Recommendation**: Option A for simplicity. Build tool integration (Option C) is out of scope.

### Q4: Compilation threading?
Should WASM compilation happen:
- A) On main thread (simple, but causes frame stutter)
- B) On background thread (smooth, but more complex)

**Recommendation**: Option B - compilation can take 100ms+, which would cause noticeable stutter.

### Q5: State format versioning?
How strict should state compatibility checking be?
- A) Strict: any change = incompatible
- B) Lenient: try to load, fail gracefully
- C) Smart: semantic versioning with migration support

**Recommendation**: Option B with clear warnings.

## Pros

1. **Massive iteration speedup**: Changes visible in <1 second vs 10+ seconds
2. **Preserves game state**: No need to replay to problem area
3. **Leverages existing infrastructure**: Uses `save_state`/`load_state` already built for GGRS
4. **Simple mental model**: "Save, swap module, restore"
5. **Graceful failure**: Compilation errors don't crash the game
6. **Console-agnostic**: Lives entirely in core crate

## Cons

1. **State compatibility challenges**: Struct layout changes can cause issues
2. **Resource handle complexity**: Need to handle texture/mesh re-registration
3. **init() side effects**: If `init()` does things beyond registration, they'll re-run
4. **Memory overhead**: Need to hold both old and new modules briefly during swap
5. **Not available in netplay**: Disabled for multiplayer sessions
6. **File system dependency**: Requires file watching, not pure WASM

## Implementation Complexity

**Estimated effort**: Medium

**Key components:**
1. File watcher (use `notify` crate) - 1 day
2. Background compilation - 1 day
3. State save/restore integration - 1 day
4. Resource manager reload handling - 2 days
5. UI notifications - 1 day
6. Testing & edge cases - 2 days

**Total**: ~8 days

## Integration Points

### Core Changes
- `GameInstance`: Add `hot_reload()` method
- `Runtime`: Add file watcher, reload state machine
- `GameSession`: Add UI notifications, hotkey handling

### Console Changes
Minimal - just forward hotkey and show notifications via existing egui integration.

## Future Enhancements

1. **Incremental Compilation**: Watch source files, trigger cargo build
2. **Live Shader Reload**: Similar system for shader hot-swapping
3. **Asset Hot Reload**: Reload textures, audio without code changes
4. **Network Sync**: Coordinate hot reload across netplay (very complex)
