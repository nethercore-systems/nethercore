# Combined Refactor: Console-Agnostic Architecture + Per-Frame Audio

**Status:** Ready for Implementation
**Scope:** Major architectural refactor (~4,000 lines changed/moved)
**Dependencies:** None (self-contained)
**Branch:** `refactor/console-agnostic-audio` (merge all phases at once)

---

## Overview

This plan combines two specs into a single cohesive refactor:
1. **Per-frame audio** - Fixes rollback audio bugs, introduces ConsoleRollbackState
2. **Console-agnostic architecture** - DRY, separation of concerns, future consoles

**Why combined?** The per-frame audio spec introduces `ConsoleRollbackState` and `WasmGameContext` patterns that the console-agnostic spec assumes. Doing them together avoids rewriting code twice.

---

## Phase 1: Core Trait Foundation

**Goal:** Add ConsoleRollbackState trait, update GameStateWithConsole → WasmGameContext, update snapshot system.

### 1.1 Add ConsoleRollbackState Trait

**File:** `core/src/console.rs`

```rust
use bytemuck::{Pod, Zeroable};

/// Console-specific rollback state (host-side, POD for zero-copy serialization)
/// Examples: audio playhead positions, channel volumes
pub trait ConsoleRollbackState: Pod + Zeroable + Default + Send + 'static {}

// Unit type implementation for consoles with no rollback state
impl ConsoleRollbackState for () {}
```

**Update Console trait:**
```rust
pub trait Console: Send + 'static {
    type Graphics: Graphics;
    type Audio: Audio;
    type Input: ConsoleInput;
    type State: Default + Send + 'static;  // Per-frame FFI state
    type RollbackState: ConsoleRollbackState;  // NEW
    type ResourceManager: ConsoleResourceManager<...>;
    // ... existing methods unchanged
}
```

### 1.2 Rename GameStateWithConsole → WasmGameContext

**File:** `core/src/wasm/state.rs`

```rust
/// Context for WASM game execution
pub struct WasmGameContext<I: ConsoleInput, S, R: ConsoleRollbackState = ()> {
    /// Core WASM game state (memory snapshots from this)
    pub game: GameState<I>,

    /// Console-specific per-frame FFI state (NOT rolled back)
    pub ffi: S,

    /// Console-specific rollback state (IS rolled back via bytemuck)
    pub rollback: R,

    pub ram_limit: usize,
    pub debug_registry: DebugRegistry,
}
```

**Migration:** Find/replace `GameStateWithConsole` → `WasmGameContext`, rename field `console` → `ffi`.

### 1.3 Update GameStateSnapshot

**File:** `core/src/rollback/state.rs`

```rust
pub struct GameStateSnapshot {
    pub data: Vec<u8>,           // WASM memory
    pub console_data: Vec<u8>,   // NEW: Console rollback state (POD bytes)
    pub checksum: u64,
    pub frame: i32,
}
```

**Update save/load:**
```rust
impl RollbackStateManager {
    pub fn save_state<I, S, R: ConsoleRollbackState>(
        &mut self,
        game: &mut GameInstance<I, S, R>,
        frame: i32,
    ) -> Result<GameStateSnapshot> {
        let wasm_data = game.save_state()?;
        let console_data = bytemuck::bytes_of(&game.context().rollback).to_vec();
        // ... checksum both, create snapshot
    }

    pub fn load_state<I, S, R: ConsoleRollbackState>(
        &mut self,
        game: &mut GameInstance<I, S, R>,
        snapshot: &GameStateSnapshot,
    ) -> Result<()> {
        game.load_state(&snapshot.data)?;
        game.context_mut().rollback = *bytemuck::from_bytes(&snapshot.console_data);
        Ok(())
    }
}
```

### 1.4 Files to Modify in Phase 1

| File | Changes |
|------|---------|
| `core/src/console.rs` | Add ConsoleRollbackState trait, update Console trait |
| `core/src/wasm/state.rs` | Rename struct, add rollback field |
| `core/src/wasm/mod.rs` | Update GameInstance generic params |
| `core/src/rollback/state.rs` | Update snapshot to include console_data |
| `core/src/rollback/session.rs` | Update generic params |
| `core/src/rollback/config.rs` | May need generic param updates |
| `core/src/runtime.rs` | Update generic params |
| `core/Cargo.toml` | Add bytemuck dependency |
| `emberware-z/src/console.rs` | Add `type RollbackState = ()` temporarily |
| All FFI files | Update `caller.data_mut().console` → `caller.data_mut().ffi` |

---

## Phase 2: Per-Frame Audio Implementation

**Goal:** Replace rodio streaming with cpal per-frame generation, audio state becomes rollback state.

### 2.1 Add Dependencies

**File:** `emberware-z/Cargo.toml`

```toml
# Remove
# rodio = "0.21"

# Add
cpal = "0.15"
ringbuf = "0.4"
```

### 2.2 Create ZRollbackState

**File:** `emberware-z/src/state/rollback_state.rs` (NEW)

```rust
use bytemuck::{Pod, Zeroable};

/// State for a single audio channel (20 bytes, POD)
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Pod, Zeroable)]
pub struct ChannelState {
    pub sound: u32,      // Sound handle (0 = silent)
    pub position: u32,   // Playhead in samples
    pub looping: u32,    // 0 = no, 1 = yes
    pub volume: f32,     // 0.0 to 1.0
    pub pan: f32,        // -1.0 to 1.0
}

/// Audio playback state (340 bytes total)
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Pod, Zeroable)]
pub struct AudioPlaybackState {
    pub channels: [ChannelState; 16],
    pub music: ChannelState,
}

/// Emberware Z rollback state
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Pod, Zeroable)]
pub struct ZRollbackState {
    pub audio: AudioPlaybackState,
}

impl ConsoleRollbackState for ZRollbackState {}
```

### 2.3 Create AudioOutput (cpal + ringbuf)

**File:** `emberware-z/src/audio.rs` (REWRITE)

```rust
use ringbuf::{HeapRb, traits::{Consumer, Producer, Split}};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct AudioOutput {
    producer: ringbuf::HeapProd<f32>,
    _stream: cpal::Stream,
    sample_rate: u32,
}

impl AudioOutput {
    pub fn new() -> Result<Self, String> { /* see spec */ }
    pub fn push_frame(&mut self, samples: &[f32]) { /* see spec */ }
    pub fn sample_rate(&self) -> u32 { self.sample_rate }
}

/// Generate one frame of audio samples
pub fn generate_audio_frame(
    playback_state: &mut AudioPlaybackState,
    sounds: &[Option<Sound>],
    tick_rate: u32,
    sample_rate: u32,
    output: &mut Vec<f32>,
) { /* see spec - mix all channels with equal-power panning */ }
```

### 2.4 Update Audio FFI Functions

**File:** `emberware-z/src/ffi/audio.rs`

**Before (command buffering):**
```rust
fn play_sound(mut caller, sound: u32, volume: f32, pan: f32) {
    let state = &mut caller.data_mut().ffi;
    state.audio_commands.push(AudioCommand::PlaySound { sound, volume, pan });
}
```

**After (direct state modification):**
```rust
fn channel_play(
    mut caller: Caller<'_, WasmGameContext<ZInput, ZFFIState, ZRollbackState>>,
    channel: u32, sound: u32, volume: f32, pan: f32, looping: u32,
) {
    let ctx = caller.data_mut();
    let ch = &mut ctx.rollback.audio.channels[channel as usize];
    ch.sound = sound;
    ch.position = 0;
    ch.volume = volume;
    ch.pan = pan;
    ch.looping = looping;
}
```

### 2.5 Update Game Loop

**File:** `emberware-z/src/app/game_session.rs`

```rust
// After render(), generate audio (only on confirmed frames)
if !session.is_rolling_back() {
    let tick_rate = /* from specs */;
    let sample_rate = audio_output.sample_rate();

    let ctx = runtime.context_mut();
    let sounds = resource_manager.sounds();

    let mut audio_buffer = Vec::new();
    generate_audio_frame(
        &mut ctx.rollback.audio,
        sounds,
        tick_rate,
        sample_rate,
        &mut audio_buffer,
    );

    audio_output.push_frame(&audio_buffer);
}
```

### 2.6 Update Console Implementation

**File:** `emberware-z/src/console.rs`

```rust
impl Console for EmberwareZ {
    type RollbackState = ZRollbackState;  // Was () in Phase 1
    // ... rest unchanged
}
```

### 2.7 Remove Old Audio Code

- Delete `AudioCommand` enum
- Delete `AudioServer` thread
- Delete `SharedPan`, `PannedSource`, `RepeatingPannedSource`
- Delete `process_commands()` method
- Remove `audio_commands` from `ZFFIState`
- Remove `set_rollback_mode()` from Audio trait and runtime

### 2.8 Files to Modify in Phase 2

| File | Changes |
|------|---------|
| `emberware-z/Cargo.toml` | Remove rodio, add cpal + ringbuf |
| `emberware-z/src/audio.rs` | Complete rewrite (~400 lines) |
| `emberware-z/src/state/rollback_state.rs` | NEW file (~60 lines) |
| `emberware-z/src/state/mod.rs` | Export rollback_state |
| `emberware-z/src/state/ffi_state.rs` | Remove audio_commands, sounds stays for loading |
| `emberware-z/src/ffi/audio.rs` | Update all functions (~200 lines) |
| `emberware-z/src/console.rs` | Update RollbackState type |
| `emberware-z/src/app/game_session.rs` | Add generate_audio_frame call |
| `core/src/console.rs` | Update Audio trait (remove set_rollback_mode) |
| `core/src/runtime.rs` | Remove audio rollback mode call |

---

## Phase 3: Console-Agnostic Structure

**Goal:** Create library/ crate, move generic code, remove Z from core.

### 3.1 Create ConsoleRunner and ActiveGame

**File:** `core/src/runner.rs` (NEW)

```rust
/// Runs a game for any console
pub struct ConsoleRunner<C: Console> {
    console: C,
    graphics: C::Graphics,
    audio_output: AudioOutput,  // Per-frame audio output
    runtime: Runtime<C>,
    resource_manager: C::ResourceManager,
}

impl<C: Console> ConsoleRunner<C> {
    pub fn new(console: C, window: Arc<Window>) -> Result<Self>;
    pub fn load_game(&mut self, rom_path: &Path) -> Result<()>;
    pub fn update(&mut self, input: &RawInput);
    pub fn render(&mut self);
    pub fn resize(&mut self, width: u32, height: u32);
    pub fn debug_stats(&self) -> Vec<DebugStat>;
    pub fn quit_requested(&self) -> bool;
}
```

**File:** `library/src/registry.rs` (NEW)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleType { Z }

/// Pure static dispatch - no vtables
pub enum ActiveGame {
    Z(ConsoleRunner<EmberwareZ>),
}

impl ActiveGame {
    pub fn update(&mut self, input: &RawInput) {
        match self { Self::Z(r) => r.update(input) }
    }
    pub fn render(&mut self) {
        match self { Self::Z(r) => r.render() }
    }
    // ... other methods
}

pub fn create_game(
    console_type: ConsoleType,
    window: Arc<Window>,
    rom_path: &Path,
) -> Result<ActiveGame> {
    match console_type {
        ConsoleType::Z => {
            let console = EmberwareZ::new();
            let mut runner = ConsoleRunner::new(console, window)?;
            runner.load_game(rom_path)?;
            Ok(ActiveGame::Z(runner))
        }
    }
}
```

### 3.2 Create library/ Crate

**Structure:**
```
library/
├── Cargo.toml
└── src/
    ├── main.rs           # Entry point
    ├── lib.rs
    ├── registry.rs       # ConsoleType, ActiveGame, create_game
    ├── app/
    │   ├── mod.rs        # App struct
    │   └── event_loop.rs # Window events
    └── ui/
        ├── mod.rs
        ├── library.rs    # Game browser
        ├── settings.rs   # Settings panel
        └── debug.rs      # Debug panel
```

**File:** `library/Cargo.toml`

```toml
[package]
name = "emberware"
version = "0.1.0"

[dependencies]
emberware-core = { path = "../core" }
emberware-z = { path = "../emberware-z" }
winit = "0.30"
wgpu = "24"
egui = "0.30"
egui-winit = "0.30"
egui-wgpu = "0.30"
```

### 3.3 Remove Z from Core

**File:** `core/src/library/rom.rs` (NEW)

```rust
/// Trait for loading console-specific ROM formats
pub trait RomLoader: Send + Sync {
    fn extension(&self) -> &'static str;
    fn magic_bytes(&self) -> &'static [u8];
    fn load(&self, path: &Path) -> Result<LoadedRom>;
    fn install(&self, path: &Path, library_path: &Path) -> Result<LocalGame>;
}

pub struct LoadedRom {
    pub wasm: Vec<u8>,
    pub data_pack: Option<Vec<u8>>,
    pub manifest: LocalGameManifest,
}
```

**File:** `z-common/src/loader.rs` (NEW)

```rust
pub struct ZRomLoader;

impl RomLoader for ZRomLoader {
    fn extension(&self) -> &'static str { "ewz" }
    fn magic_bytes(&self) -> &'static [u8] { b"EWZ\x00" }
    fn load(&self, path: &Path) -> Result<LoadedRom> { /* move from core */ }
    fn install(&self, path: &Path, library_path: &Path) -> Result<LocalGame> { /* move from core */ }
}
```

**Update `core/Cargo.toml`:** Remove `z-common` dependency.

### 3.4 Add Debug Stats to Console Trait

**File:** `core/src/debug/stats.rs` (NEW)

```rust
pub struct DebugStat {
    pub category: &'static str,
    pub name: &'static str,
    pub value: DebugValue,
}

pub enum DebugValue {
    Bytes(usize),
    Count(usize),
    Percent(f32),
    Text(String),
}
```

**Update Console trait:**
```rust
pub trait Console: Send + 'static {
    // ... existing
    fn debug_stats(&self, state: &Self::State) -> Vec<DebugStat>;
}
```

### 3.5 Files for Phase 3

**New Files:**
| File | Lines (est) |
|------|-------------|
| `core/src/runner.rs` | ~200 |
| `core/src/library/rom.rs` | ~50 |
| `core/src/debug/stats.rs` | ~30 |
| `library/Cargo.toml` | ~25 |
| `library/src/main.rs` | ~50 |
| `library/src/lib.rs` | ~10 |
| `library/src/registry.rs` | ~80 |
| `library/src/app/mod.rs` | ~400 |
| `library/src/app/event_loop.rs` | ~150 |
| `library/src/ui/mod.rs` | ~20 |
| `library/src/ui/library.rs` | ~200 |
| `library/src/ui/settings.rs` | ~300 |
| `library/src/ui/debug.rs` | ~100 |
| `z-common/src/loader.rs` | ~100 |

**Modified Files:**
| File | Changes |
|------|---------|
| `core/Cargo.toml` | Remove z-common dependency |
| `core/src/lib.rs` | Export runner, debug/stats |
| `core/src/library/mod.rs` | Export rom module |
| `core/src/library/game.rs` | Use RomLoader trait |
| `core/src/library/cart.rs` | Use RomLoader trait |
| `core/src/console.rs` | Add debug_stats method |
| `Cargo.toml` (workspace) | Add library member |

---

## Phase 4: Cleanup

**Goal:** Remove old code, update workspace, verify everything works.

### 4.1 Remove Old emberware-z App Code

**Delete:**
- `emberware-z/src/main.rs`
- `emberware-z/src/app/` (entire directory)
- `emberware-z/src/ui.rs` (moved to library)
- `emberware-z/src/settings_ui.rs` (moved to library)

**Update `emberware-z/Cargo.toml`:** Remove `[[bin]]` section, keep only `[lib]`.

**Update `emberware-z/src/lib.rs`:** Remove `pub mod app`, `pub mod ui`, etc.

### 4.2 Update Root Workspace

**File:** `Cargo.toml`

```toml
[workspace]
members = [
    "core",
    "shared",
    "z-common",
    "emberware-z",
    "library",    # NEW
    "tools/ember-cli",
    "tools/ember-export",
]
default-members = ["library"]  # NEW: library is default binary

[workspace.dependencies]
# ... existing
```

### 4.3 Delete Old Root Binary

**Delete:**
- `src/main.rs`
- `src/registry.rs`
- `src/` directory

### 4.4 Update Documentation

- Update `CLAUDE.md` architecture diagram
- Update `TASKS.md` - mark refactor complete
- Move spec files to `docs/archive/` or mark as implemented

---

## Verification Checklist

After each phase, verify:

**Phase 1:**
- [ ] `cargo build -p emberware-core` succeeds
- [ ] `cargo build -p emberware-z` succeeds
- [ ] Games still run (rollback state is () so no behavior change)

**Phase 2:**
- [ ] Audio plays correctly
- [ ] No audio during rollback replay (automatic - state rolls back)
- [ ] No audio glitches or stuttering
- [ ] Music loops correctly
- [ ] Pan/volume work correctly

**Phase 3:**
- [ ] `cargo run -p library` launches library UI
- [ ] `cargo run -p library -- cube` launches game directly
- [ ] `cargo build -p emberware-core` has no z-common imports

**Phase 4:**
- [ ] `cargo run` launches library (default member)
- [ ] All examples work: cube, platformer, lighting, billboard
- [ ] `cargo build --workspace` succeeds
- [ ] No dead code warnings

---

## Implementation Order Summary

```
Phase 1: Core Traits (~8 files, ~200 lines changed)
    ↓
Phase 2: Per-Frame Audio (~10 files, ~800 lines changed)
    ↓
Phase 3: Console-Agnostic (~17 files, ~1700 lines new/moved)
    ↓
Phase 4: Cleanup (~10 files deleted, ~100 lines changed)
```

**Total estimated changes:** ~2,800 lines new/changed, ~2,500 lines deleted/moved

---

## Out of Scope (Separate Tasks)

- **QOA Audio Compression** - Implement as standalone codec module after this refactor
- **Standalone Player** - Minimal `player/` crate without egui (deferred)
- **Additional Consoles** - Emberware Classic, Y, X (architecture supports, implementation later)

---

## Critical File Reference

### Must Read Before Implementing:
- `core/src/console.rs` - Console trait definition
- `core/src/wasm/state.rs` - GameStateWithConsole struct
- `core/src/rollback/state.rs` - Snapshot save/load
- `core/src/runtime.rs` - Game loop integration
- `emberware-z/src/audio.rs` - Current audio implementation
- `emberware-z/src/app/mod.rs` - App structure to move
- `emberware-z/src/ffi/audio.rs` - Audio FFI functions

### Spec Documents:
- `docs/ready/per-frame-audio-spec.md` - Full audio spec
- `docs/pending/console-agnostic-architecture.md` - Full architecture spec

---

## Notes for Claude Code Sessions

1. **Phase 1 is self-contained** - Can be done in one session
2. **Phase 2 is complex** - May need 2 sessions (audio rewrite + FFI updates)
3. **Phase 3 is large** - Expect 2-3 sessions (create crates, move code, wire up)
4. **Phase 4 is cleanup** - Quick, one session

**If context runs low:** Each phase is independently verifiable. Complete current phase, commit, then continue in new session.

**Commit strategy:** Commit after each phase with descriptive message like "Phase 1: Add ConsoleRollbackState trait foundation"

---

## Quick Reference: Key Type Changes

| Before | After |
|--------|-------|
| `GameStateWithConsole<I, S>` | `WasmGameContext<I, S, R>` |
| `.console` field | `.ffi` field |
| (none) | `.rollback` field |
| `GameStateSnapshot { data, checksum, frame }` | `GameStateSnapshot { data, console_data, checksum, frame }` |
| `Console::State` | `Console::State` (FFI) + `Console::RollbackState` (rollback) |
| `AudioCommand` buffering | Direct `ZRollbackState.audio` modification |
| `rodio` streaming | `cpal` + `ringbuf` per-frame |
| `set_rollback_mode()` | (removed - automatic via state rollback) |

---

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Audio glitches during transition | Test with simple sine wave before complex audio |
| Ring buffer underruns | Size buffer for ~50ms (~3 frames at 60fps) |
| Generic param explosion | Use type aliases where helpful |
| Large merge conflicts | Work on feature branch, merge phases incrementally |
| Examples break | Run all examples after each phase |

---

## Success Criteria

After full implementation:
- [ ] `cargo run` launches library UI (console-agnostic)
- [ ] `cargo run -- cube` launches game directly
- [ ] Audio plays correctly without rollback artifacts
- [ ] No audio during rollback replay (state-based, not muting)
- [ ] `core/` has zero z-common imports
- [ ] `emberware-z/` has no `main.rs` or `app/` directory
- [ ] Adding future console = add enum variant + match arms only
- [ ] All examples work: cube, platformer, lighting, billboard
