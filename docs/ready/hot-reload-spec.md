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

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Call `init()` on reload | **Yes, always** | `init()` registers resources (textures, meshes). Must re-register with new module. |
| Resource handle strategy | **Clear all, re-init** | Call `resource_manager.clear()` before `init()`. Clean slate approach is simpler and more reliable. |
| Watch file or directory | **Watch specific `rom.wasm`** | Simple and predictable. Build tool integration out of scope. |
| Compilation threading | **Background thread** | Compilation can take 100ms+, would cause frame stutter on main thread. |
| State compatibility | **Lenient with warnings** | Try to load state, fail gracefully if incompatible. Show clear warning to user. |

### Resource Handle Implementation

The hot reload sequence handles resources as follows:

```rust
// In hot_reload() method:
// 1. Save current state
let state_data = self.save_state()?;

// 2. Clear ALL resources before instantiating new module
self.resource_manager.clear();

// 3. Create new instance
let mut new_instance = GameInstance::new(engine, new_module, linker)?;

// 4. Call init() - game re-registers all resources fresh
new_instance.call_init()?;

// 5. Restore state (resources now have fresh handles)
new_instance.load_state(&state_data)?;
```

This "clear all, re-init" approach ensures:
- No stale resource handles from old module
- No handle collision between old and new registrations
- Clean separation between module lifetimes

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

## Way Forward: Implementation Guide

This section provides concrete implementation steps based on the current Emberware codebase architecture.

### Step 1: Add Hot Reload State to Core Types

**File: `core/src/wasm/state.rs`**

Add hot reload tracking to `GameState`:

```rust
// Add to GameState<I>
pub struct GameState<I: ConsoleInput> {
    // ... existing fields ...

    /// Hot reload enabled for this session
    pub hot_reload_enabled: bool,
    /// Number of hot reloads this session
    pub hot_reload_count: u32,
}
```

### Step 2: Add File Watcher to Runtime

**File: `core/src/runtime.rs`**

Add file watching capability using the `notify` crate:

```rust
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event};
use std::sync::mpsc::{channel, Receiver};
use std::path::PathBuf;

pub struct Runtime<C: Console> {
    // ... existing fields ...

    /// File watcher for hot reload
    file_watcher: Option<RecommendedWatcher>,
    /// Channel to receive file change events
    file_change_rx: Option<Receiver<Result<Event, notify::Error>>>,
    /// Path being watched
    watched_path: Option<PathBuf>,
    /// Pending reload (debounced)
    pending_reload: Option<std::time::Instant>,
}

impl<C: Console> Runtime<C> {
    /// Start watching a ROM file for changes
    pub fn watch_rom(&mut self, path: PathBuf) -> anyhow::Result<()> {
        let (tx, rx) = channel();
        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })?;
        watcher.watch(&path, RecursiveMode::NonRecursive)?;

        self.file_watcher = Some(watcher);
        self.file_change_rx = Some(rx);
        self.watched_path = Some(path);
        Ok(())
    }

    /// Check for pending hot reload and execute if ready
    pub fn check_hot_reload(&mut self) -> Option<HotReloadResult> {
        // Check for file change events
        if let Some(rx) = &self.file_change_rx {
            while let Ok(event) = rx.try_recv() {
                if event.is_ok() {
                    // Debounce: schedule reload 100ms from now
                    self.pending_reload = Some(
                        std::time::Instant::now() + std::time::Duration::from_millis(100)
                    );
                }
            }
        }

        // Check if debounce period has passed
        if let Some(reload_at) = self.pending_reload {
            if std::time::Instant::now() >= reload_at {
                self.pending_reload = None;
                return Some(self.execute_hot_reload());
            }
        }
        None
    }
}
```

### Step 3: Implement Hot Reload Logic in GameInstance

**File: `core/src/wasm/mod.rs`**

Add the core hot reload method to `GameInstance`:

```rust
impl<I: ConsoleInput, S: Send + Default + 'static> GameInstance<I, S> {
    /// Perform hot reload with a new WASM module
    pub fn hot_reload(
        &mut self,
        engine: &WasmEngine,
        new_module: &wasmtime::Module,
        linker: &wasmtime::Linker<GameStateWithConsole<I, S>>,
    ) -> anyhow::Result<HotReloadResult> {
        // 1. Save current state
        let state_data = self.save_state()?;
        let prev_version = self.call_hot_reload_prepare().unwrap_or(0);

        // 2. Create new instance
        let mut new_instance = GameInstance::new(engine, new_module, linker)?;

        // 3. Preserve timing/session state from old instance
        new_instance.store.data_mut().game.tick_count = self.store.data().game.tick_count;
        new_instance.store.data_mut().game.elapsed_time = self.store.data().game.elapsed_time;
        new_instance.store.data_mut().game.hot_reload_count =
            self.store.data().game.hot_reload_count + 1;

        // 4. Call init() on new module (re-registers resources)
        new_instance.call_init()?;

        // 5. Try to restore state
        let state_restored = match new_instance.load_state(&state_data) {
            Ok(()) => true,
            Err(e) => {
                tracing::warn!("Hot reload: state incompatible, starting fresh: {}", e);
                false
            }
        };

        // 6. Notify game of reload completion
        let _ = new_instance.call_hot_reload_complete(prev_version);

        // 7. Replace self with new instance
        *self = new_instance;

        Ok(HotReloadResult {
            success: true,
            state_restored,
            reload_count: self.store.data().game.hot_reload_count,
        })
    }
}
```

### Step 4: Register FFI Functions

**File: `core/src/ffi.rs`**

Add hot reload FFI functions to `register_common_ffi`:

```rust
pub fn register_common_ffi<I: ConsoleInput, S: Send + Default + 'static>(
    linker: &mut Linker<GameStateWithConsole<I, S>>,
) -> Result<()> {
    // ... existing registrations ...

    // Hot reload functions
    linker.func_wrap("env", "hot_reload_enabled", hot_reload_enabled)?;
    linker.func_wrap("env", "hot_reload_count", hot_reload_count)?;

    Ok(())
}

fn hot_reload_enabled<I: ConsoleInput, S>(
    caller: Caller<'_, GameStateWithConsole<I, S>>
) -> i32 {
    if caller.data().game.hot_reload_enabled { 1 } else { 0 }
}

fn hot_reload_count<I: ConsoleInput, S>(
    caller: Caller<'_, GameStateWithConsole<I, S>>
) -> u32 {
    caller.data().game.hot_reload_count
}
```

### Step 5: Integrate into GameSession

**File: `core/src/app/session.rs`**

Add hot reload handling to `GameSession`:

```rust
pub struct GameSession<C: Console> {
    pub runtime: Runtime<C>,
    pub resource_manager: C::ResourceManager,

    /// Hot reload UI state
    hot_reload_status: HotReloadStatus,
    /// Toast message to display
    toast_message: Option<(String, std::time::Instant)>,
}

impl<C: Console> GameSession<C> {
    /// Call this each frame to check for hot reload
    pub fn update_hot_reload(&mut self, wasm_engine: &WasmEngine) {
        if let Some(result) = self.runtime.check_hot_reload() {
            match result {
                Ok(hr) => {
                    // Clear resource manager for re-registration
                    self.resource_manager.clear();

                    let msg = if hr.state_restored {
                        format!("Hot reload #{} complete", hr.reload_count)
                    } else {
                        format!("Hot reload #{} (state reset)", hr.reload_count)
                    };
                    self.show_toast(msg);
                }
                Err(e) => {
                    self.show_toast(format!("Hot reload failed: {}", e));
                }
            }
        }
    }

    /// Handle Ctrl+R hotkey
    pub fn handle_hot_reload_hotkey(&mut self, force_reset: bool) -> bool {
        // Trigger immediate reload
        if let Some(path) = self.runtime.watched_path.clone() {
            // Touch the file or force reload
            self.runtime.pending_reload = Some(std::time::Instant::now());
            true
        } else {
            false
        }
    }
}
```

### Step 6: Wire into Console App (Emberware Z)

**File: `emberware-z/src/app/mod.rs`**

Add hot reload integration to the app event loop:

```rust
impl App {
    fn run_game_frame(&mut self) -> Result<(bool, bool), RuntimeError> {
        // Check for hot reload before processing frame
        if let Some(session) = &mut self.game_session {
            if let Some(engine) = &self.wasm_engine {
                session.update_hot_reload(engine);
            }
        }

        // ... existing frame code ...
    }

    fn handle_key_input(&mut self, event: KeyEvent) {
        // ... existing key handling ...

        // Hot reload hotkeys
        if event.state.is_pressed() {
            match event.logical_key {
                Key::Character(c) if c == "r" => {
                    let ctrl = event.modifiers.contains(Modifiers::CONTROL);
                    let shift = event.modifiers.contains(Modifiers::SHIFT);
                    if ctrl {
                        if let Some(session) = &mut self.game_session {
                            session.handle_hot_reload_hotkey(shift);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
```

### Step 7: Handle Resource Manager Reset

**File: `emberware-z/src/resource_manager.rs`**

Add clear method for hot reload. This clears ALL game-registered resources so `init()` can re-register them fresh:

```rust
impl ZResourceManager {
    /// Clear all resources for hot reload
    ///
    /// Called before instantiating new WASM module. The new module's init()
    /// will re-register all resources. Built-in resources (font, default textures)
    /// are re-created automatically during game start.
    pub fn clear(&mut self) {
        // Clear all game-registered resources
        self.texture_map.clear();
        self.mesh_map.clear();
        self.palette_map.clear();

        // GPU resources will be dropped when maps clear
        // Built-in resources (font, white texture) are re-added in start_game()

        tracing::debug!("Resource manager cleared for hot reload");
    }
}
```

### File Checklist

| File | Changes |
|------|---------|
| `core/src/wasm/state.rs` | Add `hot_reload_enabled`, `hot_reload_count` to `GameState` |
| `core/src/wasm/mod.rs` | Add `hot_reload()` method to `GameInstance` |
| `core/src/runtime.rs` | Add file watcher, `check_hot_reload()` |
| `core/src/ffi.rs` | Add `hot_reload_enabled`, `hot_reload_count` FFI functions |
| `core/src/app/session.rs` | Add `update_hot_reload()`, toast notifications |
| `emberware-z/src/app/mod.rs` | Wire hot reload into game loop and hotkeys |
| `emberware-z/src/resource_manager.rs` | Add `clear()` method |
| `core/Cargo.toml` | Add `notify = "6.0"` dependency |

### Test Cases

1. **Basic hot reload**: Modify WASM, verify game continues with state
2. **State incompatibility**: Change struct layout, verify graceful fallback
3. **Compilation error**: Introduce syntax error, verify game continues
4. **Rapid changes**: Multiple quick saves, verify debounce works
5. **Resource re-registration**: Add new texture in init(), verify it loads
6. **Netplay disabled**: Start P2P session, verify hot reload is disabled
7. **Hotkey**: Press Ctrl+R, verify manual reload triggers

## Future Enhancements

1. **Incremental Compilation**: Watch source files, trigger cargo build
2. **Live Shader Reload**: Similar system for shader hot-swapping
3. **Asset Hot Reload**: Reload textures, audio without code changes
4. **Network Sync**: Coordinate hot reload across netplay (very complex)
