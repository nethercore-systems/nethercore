# State Snapshots Specification

## Overview

State Snapshots provide quick save/load functionality during development, allowing developers to save the exact game state at any moment and restore it instantly. Unlike the Replay System (which records inputs), snapshots capture complete state and allow jumping directly to that state without replaying.

This enables:
- **Instant iteration**: Save before a tricky section, retry instantly
- **Bug reproduction**: Snapshot right before a bug, reload to reproduce
- **A/B testing**: Compare behavior from the same starting state
- **State inspection**: Freeze game, examine state, restore and continue

## Architecture

### Snapshot Data

```rust
/// Complete game state snapshot
pub struct StateSnapshot {
    /// Unique identifier
    pub id: String,

    /// User-provided name
    pub name: String,

    /// When the snapshot was taken
    pub timestamp: std::time::SystemTime,

    /// Game frame when taken
    pub frame: u64,

    /// Serialized game state (from save_state)
    pub state: Vec<u8>,

    /// Thumbnail image (optional, for UI)
    pub thumbnail: Option<Vec<u8>>,

    /// User notes
    pub notes: String,

    /// Game ID for compatibility check
    pub game_id: String,

    /// ROM hash for compatibility check
    pub rom_hash: [u8; 32],
}

/// Snapshot storage manager
pub struct SnapshotManager {
    /// In-memory quick slots (1-9)
    pub quick_slots: [Option<StateSnapshot>; 9],

    /// Named snapshots (unlimited, persisted)
    pub named_snapshots: Vec<StateSnapshot>,

    /// Auto-save slot (periodically updated)
    pub auto_save: Option<StateSnapshot>,

    /// Configuration
    pub config: SnapshotConfig,
}

pub struct SnapshotConfig {
    /// Enable auto-save
    pub auto_save_enabled: bool,

    /// Auto-save interval (seconds)
    pub auto_save_interval_secs: u32,

    /// Capture thumbnails
    pub capture_thumbnails: bool,

    /// Thumbnail resolution
    pub thumbnail_size: (u32, u32),

    /// Maximum named snapshots to keep
    pub max_named_snapshots: usize,
}
```

### Quick Slots vs Named Snapshots

**Quick Slots (1-9)**:
- Instant save/load via hotkeys
- Session-only (not persisted to disk)
- No names or metadata
- Overwrite without confirmation

**Named Snapshots**:
- Persisted to disk
- User-provided names and notes
- Thumbnail preview
- Confirmation before overwrite
- Unlimited count (configurable max)

## Workflow

### Quick Save/Load

```
┌─────────────────────────────────────────────────────────────┐
│                    Quick Save/Load                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Press F5 to quick save to slot 1                          │
│           │                                                 │
│           ▼                                                 │
│  1. Pause game (optional, configurable)                    │
│           │                                                 │
│           ▼                                                 │
│  2. Call game's save_state export                          │
│           │                                                 │
│           ▼                                                 │
│  3. Store state in quick_slots[0]                          │
│           │                                                 │
│           ▼                                                 │
│  4. Show toast: "Saved to Slot 1"                          │
│           │                                                 │
│           ▼                                                 │
│  5. Resume game                                            │
│                                                             │
│  ─────────────────────────────────────────────────────────  │
│                                                             │
│  Press F9 to quick load from slot 1                        │
│           │                                                 │
│           ▼                                                 │
│  1. Retrieve state from quick_slots[0]                     │
│           │                                                 │
│           ├──── Slot empty ──► Show toast: "Slot 1 empty"  │
│           │                                                 │
│           ▼                                                 │
│  2. Call game's load_state export                          │
│           │                                                 │
│           ▼                                                 │
│  3. Show toast: "Loaded Slot 1 (frame 1234)"              │
│           │                                                 │
│           ▼                                                 │
│  4. Resume game from restored state                        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Named Snapshot Save

```
┌─────────────────────────────────────────────────────────────┐
│                    Named Snapshot Save                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Press Ctrl+S or use menu                                  │
│           │                                                 │
│           ▼                                                 │
│  1. Pause game                                             │
│           │                                                 │
│           ▼                                                 │
│  2. Capture current frame as thumbnail                     │
│           │                                                 │
│           ▼                                                 │
│  3. Show save dialog:                                      │
│     ┌─────────────────────────────────────────────────┐    │
│     │ Save Snapshot                              [×] │    │
│     ├─────────────────────────────────────────────────┤    │
│     │ Name: [boss_fight_attempt_3              ]    │    │
│     │ Notes: [Before final phase, 1HP left     ]    │    │
│     │ ┌─────────────┐                                │    │
│     │ │  thumbnail  │   Frame: 45,230              │    │
│     │ │             │   Time: 12:34 PM             │    │
│     │ └─────────────┘                                │    │
│     │           [Cancel]  [Save]                     │    │
│     └─────────────────────────────────────────────────┘    │
│           │                                                 │
│           ▼                                                 │
│  4. Serialize state                                        │
│           │                                                 │
│           ▼                                                 │
│  5. Save to ~/.emberware/games/{game}/snapshots/           │
│           │                                                 │
│           ▼                                                 │
│  6. Resume game                                            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Hotkeys

| Key | Action |
|-----|--------|
| F5 | Quick save to slot 1 |
| Shift+F5 | Quick save dialog (choose slot 1-9) |
| F9 | Quick load from slot 1 |
| Shift+F9 | Quick load dialog (choose slot 1-9) |
| 1-9 (while Shift+F5/F9 open) | Select slot |
| Ctrl+S | Save named snapshot (with dialog) |
| Ctrl+Shift+S | Save named snapshot (quick, auto-name) |
| Ctrl+L | Load snapshot browser |

## UI Design

### Snapshot Browser

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Snapshots                                                              [×] │
├─────────────────────────────────────────────────────────────────────────────┤
│ Quick Slots                                                                 │
│ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐   │
│ │ [1]  │ │ [2]  │ │ [3]  │ │ [4]  │ │ [5]  │ │ [6]  │ │ [7]  │ │ [8]  │   │
│ │thumb │ │ ---- │ │thumb │ │ ---- │ │ ---- │ │ ---- │ │ ---- │ │ ---- │   │
│ │F:1234│ │empty │ │F:5678│ │empty │ │empty │ │empty │ │empty │ │empty │   │
│ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘   │
├─────────────────────────────────────────────────────────────────────────────┤
│ Named Snapshots                                               [New] [Import]│
│ ┌───────────────────────────────────────────────────────────────────────┐  │
│ │ ┌────────┐                                                            │  │
│ │ │        │  boss_fight_attempt_3                                      │  │
│ │ │ thumb  │  Frame: 45,230  │  2024-01-15 12:34 PM                     │  │
│ │ │        │  Notes: Before final phase, 1HP left                       │  │
│ │ └────────┘                               [Load] [Delete] [Export]     │  │
│ │ ─────────────────────────────────────────────────────────────────────│  │
│ │ ┌────────┐                                                            │  │
│ │ │        │  level_start                                               │  │
│ │ │ thumb  │  Frame: 10,500  │  2024-01-15 10:15 AM                     │  │
│ │ │        │  Notes: Clean start of level 3                             │  │
│ │ └────────┘                               [Load] [Delete] [Export]     │  │
│ └───────────────────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────────────────┤
│ Auto-save: 2 minutes ago (Frame: 44,100)                      [Load Auto]  │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Quick Slot Overlay

When pressing Shift+F5 or Shift+F9:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              SAVE TO SLOT                                  │
│                                                                            │
│   [1]        [2]        [3]        [4]        [5]                         │
│  ┌────┐    ┌────┐    ┌────┐    ┌────┐    ┌────┐                           │
│  │████│    │    │    │████│    │    │    │    │                           │
│  │████│    │    │    │████│    │    │    │    │                           │
│  └────┘    └────┘    └────┘    └────┘    └────┘                           │
│   used     empty      used     empty     empty                             │
│                                                                            │
│   [6]        [7]        [8]        [9]                                    │
│  ┌────┐    ┌────┐    ┌────┐    ┌────┐                                     │
│  │    │    │    │    │    │    │    │                                     │
│  │    │    │    │    │    │    │    │                                     │
│  └────┘    └────┘    └────┘    └────┘                                     │
│  empty     empty     empty     empty                                       │
│                                                                            │
│                    Press 1-9 to select, ESC to cancel                      │
└─────────────────────────────────────────────────────────────────────────────┘
```

## FFI API

State snapshots primarily use the existing `save_state`/`load_state` exports that games already implement for GGRS rollback. No new FFI is strictly required.

### Optional Enhancements

```rust
// Get estimated state size (for UI feedback)
extern "C" fn __state_size_hint() -> u32;

// Get human-readable state version
extern "C" fn __state_version_string(out_ptr: u32, out_len: u32) -> u32;

// Callback after state load (for game to re-sync derived state)
extern "C" fn __on_state_loaded();
```

## Storage

### File Structure

```
~/.emberware/games/{game_id}/snapshots/
├── quickslots.bin          # Quick slots (session persistence optional)
├── boss_fight_attempt_3.snapshot
├── level_start.snapshot
├── auto_save.snapshot
└── thumbnails/
    ├── boss_fight_attempt_3.png
    ├── level_start.png
    └── auto_save.png
```

### Snapshot File Format

```
┌────────────────────────────────────────┐
│ Header                                 │
│ ├─ magic: "EMBS" (4 bytes)            │
│ ├─ version: u32                       │
│ ├─ name_len: u16                      │
│ ├─ name: [u8; name_len]               │
│ ├─ notes_len: u16                     │
│ ├─ notes: [u8; notes_len]             │
│ ├─ timestamp: u64 (unix epoch)        │
│ ├─ frame: u64                         │
│ ├─ game_id_len: u16                   │
│ ├─ game_id: [u8; game_id_len]         │
│ ├─ rom_hash: [u8; 32]                 │
│ ├─ state_len: u32                     │
│ └─ thumbnail_len: u32                 │
├────────────────────────────────────────┤
│ State Data                             │
│ └─ [u8; state_len] (compressed)       │
├────────────────────────────────────────┤
│ Thumbnail                              │
│ └─ [u8; thumbnail_len] (PNG)          │
└────────────────────────────────────────┘
```

## Integration with Other Debug Tools

### Memory Viewer Integration

After loading a snapshot, the Memory Viewer can:
- Show "Snapshot loaded" indicator
- Compare current state to snapshot state
- Highlight memory differences

### Replay System Integration

- Save snapshot during replay playback
- Load snapshot, then record new inputs from that point
- "Branch" replay: Load snapshot, play differently

### Debug Inspection Integration

- All registered debug variables update after load
- Graphs show discontinuity marker at load point

## Auto-Save

### Configuration

```toml
[snapshots]
auto_save_enabled = true
auto_save_interval_secs = 120  # Every 2 minutes
auto_save_keep_count = 3       # Keep last 3 auto-saves
auto_save_on_crash = true      # Save on unhandled error
```

### Behavior

1. Every `auto_save_interval_secs`, silently save state
2. Rotate auto-saves: auto_save_1 → auto_save_2 → auto_save_3 → delete
3. On crash/error, save crash_recovery snapshot
4. Show "Auto-saved" toast briefly

## Edge Cases

### During Netplay
Snapshots are disabled during P2P netplay (would desync players).

### State Incompatibility
If loading a snapshot from a different ROM version:
- Warn user
- Attempt load anyway
- If `load_state` fails, show error

### Snapshot While Paused
Allowed - captures paused state exactly.

### Rapid Save/Load
Debounce saves (100ms minimum between saves).

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Cross-session quick slots | **Configurable** (default: session-only) | Flexibility for different workflows |
| Undo slot on load | **Yes, automatic** | Non-intrusive safety net - always save current state before loading |
| Snapshot sharing | **Yes with ROM hash verification** | Allow sharing, but verify compatibility |
| Thumbnail timing | **Last rendered frame** | No extra rendering overhead |
| State compression | **LZ4** | Fast enough for real-time, significant size reduction |

## Pros

1. **Instant**: Load state directly, no replay needed
2. **Simple**: Uses existing save_state/load_state
3. **Versatile**: Quick slots for iteration, named for persistence
4. **Visual**: Thumbnails make browsing easy
5. **Auto-save**: Safety net for crashes

## Cons

1. **File size**: Snapshots can be large (full state)
2. **No inputs**: Can't "continue" from snapshot with recorded inputs
3. **ROM coupling**: Only works with matching ROM version
4. **Memory cost**: Quick slots held in RAM
5. **No netplay**: Disabled in multiplayer

## Implementation Complexity

**Estimated effort:** Low-Medium

**Key components:**
1. Snapshot data structures - 0.5 days
2. Quick slot system - 1 day
3. Named snapshot management - 1 day
4. Snapshot browser UI - 2 days
5. File format & persistence - 1 day
6. Thumbnail capture - 0.5 days
7. Auto-save system - 1 day
8. Testing - 1 day

**Total:** ~8 days

## Console-Agnostic Design

State snapshots are fully console-agnostic:
- Uses game's `save_state`/`load_state` exports
- SnapshotManager lives in core's GameSession
- UI is egui (shared across consoles)
- Thumbnail capture uses existing render target

## Way Forward: Implementation Guide

This section provides concrete implementation steps based on the current Emberware codebase architecture.

### Step 1: Add Snapshot Types

**File: `core/src/snapshot/mod.rs` (new file)**

```rust
//! State snapshot system (quick save/load)

use serde::{Deserialize, Serialize};

/// A complete state snapshot
#[derive(Serialize, Deserialize)]
pub struct StateSnapshot {
    pub name: String,
    pub timestamp: u64,
    pub frame: u64,
    pub state: Vec<u8>,
    pub thumbnail: Option<Vec<u8>>,
    pub game_id: String,
    pub rom_hash: [u8; 32],
}

impl StateSnapshot {
    pub const MAGIC: [u8; 4] = *b"EMBS";
    pub const VERSION: u32 = 1;
}

/// Manages quick slots and named snapshots
pub struct SnapshotManager {
    /// Quick slots (F5/F9 style, session-only)
    pub quick_slots: [Option<StateSnapshot>; 9],
    /// Undo slot (auto-saved before load)
    pub undo_slot: Option<StateSnapshot>,
    /// Auto-save interval tracking
    pub last_auto_save: std::time::Instant,
    pub auto_save_interval_secs: u32,
}

impl SnapshotManager {
    pub fn new() -> Self {
        Self {
            quick_slots: Default::default(),
            undo_slot: None,
            last_auto_save: std::time::Instant::now(),
            auto_save_interval_secs: 120,
        }
    }

    pub fn quick_save(&mut self, slot: usize, snapshot: StateSnapshot) {
        if slot < 9 {
            self.quick_slots[slot] = Some(snapshot);
        }
    }

    pub fn quick_load(&mut self, slot: usize) -> Option<&StateSnapshot> {
        if slot < 9 {
            self.quick_slots[slot].as_ref()
        } else {
            None
        }
    }

    pub fn save_undo(&mut self, current_state: StateSnapshot) {
        self.undo_slot = Some(current_state);
    }

    pub fn should_auto_save(&self) -> bool {
        self.last_auto_save.elapsed().as_secs() >= self.auto_save_interval_secs as u64
    }

    pub fn mark_auto_saved(&mut self) {
        self.last_auto_save = std::time::Instant::now();
    }
}
```

### Step 2: Integrate into GameSession

**File: `core/src/app/session.rs`**

```rust
use crate::snapshot::{SnapshotManager, StateSnapshot};

pub struct GameSession<C: Console> {
    pub runtime: Runtime<C>,
    pub resource_manager: C::ResourceManager,
    pub snapshot_manager: SnapshotManager,  // NEW
    pub game_id: String,
    pub rom_hash: [u8; 32],
}

impl<C: Console> GameSession<C> {
    /// Create a snapshot from current game state
    pub fn create_snapshot(&self, name: &str) -> anyhow::Result<StateSnapshot> {
        let game = self.runtime.game()
            .ok_or_else(|| anyhow::anyhow!("No game loaded"))?;

        let state = game.save_state()?;
        let frame = game.store().data().game.tick_count;

        Ok(StateSnapshot {
            name: name.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            frame,
            state,
            thumbnail: None,  // Captured separately
            game_id: self.game_id.clone(),
            rom_hash: self.rom_hash,
        })
    }

    /// Load a snapshot into the game
    pub fn load_snapshot(&mut self, snapshot: &StateSnapshot) -> anyhow::Result<()> {
        // Save undo state before loading
        if let Ok(undo) = self.create_snapshot("undo") {
            self.snapshot_manager.save_undo(undo);
        }

        // Load the snapshot
        let game = self.runtime.game_mut()
            .ok_or_else(|| anyhow::anyhow!("No game loaded"))?;

        game.load_state(&snapshot.state)?;

        Ok(())
    }

    /// Quick save to slot (0-8)
    pub fn quick_save(&mut self, slot: usize) -> anyhow::Result<()> {
        let snapshot = self.create_snapshot(&format!("Quick Slot {}", slot + 1))?;
        self.snapshot_manager.quick_save(slot, snapshot);
        Ok(())
    }

    /// Quick load from slot (0-8)
    pub fn quick_load(&mut self, slot: usize) -> anyhow::Result<()> {
        let snapshot = self.snapshot_manager.quick_load(slot)
            .ok_or_else(|| anyhow::anyhow!("Slot {} is empty", slot + 1))?
            .clone();
        self.load_snapshot(&snapshot)
    }

    /// Handle snapshot hotkeys
    pub fn handle_snapshot_hotkey(&mut self, key: &winit::keyboard::Key, shift: bool) -> bool {
        use winit::keyboard::{Key, NamedKey};
        match key {
            Key::Named(NamedKey::F5) if !shift => {
                // Quick save to slot 1
                let _ = self.quick_save(0);
                true
            }
            Key::Named(NamedKey::F9) if !shift => {
                // Quick load from slot 1
                let _ = self.quick_load(0);
                true
            }
            Key::Character(c) if shift && c.len() == 1 => {
                // Shift+1-9 for specific slots (save)
                if let Some(digit) = c.chars().next().and_then(|c| c.to_digit(10)) {
                    if digit >= 1 && digit <= 9 {
                        let _ = self.quick_save((digit - 1) as usize);
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Check and perform auto-save if needed
    pub fn update_auto_save(&mut self) {
        if self.snapshot_manager.should_auto_save() {
            if let Ok(snapshot) = self.create_snapshot("auto_save") {
                // Save to disk
                let _ = self.save_snapshot_to_disk(&snapshot, "auto_save");
                self.snapshot_manager.mark_auto_saved();
            }
        }
    }

    /// Save snapshot to disk
    pub fn save_snapshot_to_disk(&self, snapshot: &StateSnapshot, filename: &str) -> anyhow::Result<()> {
        let path = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("No home directory"))?
            .join(".emberware")
            .join("games")
            .join(&self.game_id)
            .join("snapshots")
            .join(format!("{}.snapshot", filename));

        std::fs::create_dir_all(path.parent().unwrap())?;

        let data = bincode::serialize(snapshot)?;
        std::fs::write(path, data)?;

        Ok(())
    }

    /// Load snapshot from disk
    pub fn load_snapshot_from_disk(&self, filename: &str) -> anyhow::Result<StateSnapshot> {
        let path = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("No home directory"))?
            .join(".emberware")
            .join("games")
            .join(&self.game_id)
            .join("snapshots")
            .join(format!("{}.snapshot", filename));

        let data = std::fs::read(path)?;
        let snapshot: StateSnapshot = bincode::deserialize(&data)?;

        Ok(snapshot)
    }
}
```

### Step 3: Add Snapshot Browser UI

**File: `core/src/app/snapshot_ui.rs` (new file)**

```rust
use crate::snapshot::{SnapshotManager, StateSnapshot};
use egui::{Ui, Vec2};

pub struct SnapshotBrowserUI {
    visible: bool,
    new_name: String,
}

impl SnapshotBrowserUI {
    pub fn new() -> Self {
        Self {
            visible: false,
            new_name: String::new(),
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        manager: &SnapshotManager,
    ) -> SnapshotAction {
        if !self.visible {
            return SnapshotAction::None;
        }

        let mut action = SnapshotAction::None;

        egui::Window::new("Snapshots")
            .default_size([400.0, 300.0])
            .show(ctx, |ui| {
                // Quick slots
                ui.heading("Quick Slots");
                ui.horizontal(|ui| {
                    for i in 0..9 {
                        let label = if manager.quick_slots[i].is_some() {
                            format!("[{}]", i + 1)
                        } else {
                            format!(" {} ", i + 1)
                        };

                        if ui.button(label).clicked() {
                            action = SnapshotAction::LoadQuickSlot(i);
                        }
                    }
                });

                ui.separator();

                // Named snapshots would be loaded from disk here
                ui.heading("Named Snapshots");
                ui.label("(Load from disk)");

                ui.separator();

                // Save new snapshot
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.new_name);
                    if ui.button("Save").clicked() && !self.new_name.is_empty() {
                        action = SnapshotAction::SaveNamed(std::mem::take(&mut self.new_name));
                    }
                });
            });

        action
    }
}

pub enum SnapshotAction {
    None,
    LoadQuickSlot(usize),
    SaveQuickSlot(usize),
    SaveNamed(String),
    LoadNamed(String),
}
```

### Step 4: Wire into Console App

**File: `emberware-z/src/app/mod.rs`**

```rust
impl App {
    fn handle_key_input(&mut self, event: KeyEvent) {
        // ... existing key handling ...

        let shift = event.modifiers.contains(Modifiers::SHIFT);

        if event.state.is_pressed() {
            // Snapshot hotkeys
            if let Some(session) = &mut self.game_session {
                if session.handle_snapshot_hotkey(&event.logical_key, shift) {
                    return;
                }
            }

            // Snapshot browser toggle
            match event.logical_key {
                Key::Named(NamedKey::F4) => {
                    self.snapshot_browser.toggle();
                }
                _ => {}
            }
        }
    }

    fn update(&mut self) {
        // ... existing update code ...

        // Auto-save check (once per second is fine)
        if let Some(session) = &mut self.game_session {
            session.update_auto_save();
        }
    }

    fn render(&mut self) {
        // ... in egui context ...

        // Snapshot browser
        if let Some(session) = &mut self.game_session {
            let action = self.snapshot_browser.show(&self.egui_ctx, &session.snapshot_manager);
            match action {
                SnapshotAction::LoadQuickSlot(i) => { let _ = session.quick_load(i); }
                SnapshotAction::SaveNamed(name) => {
                    if let Ok(snap) = session.create_snapshot(&name) {
                        let _ = session.save_snapshot_to_disk(&snap, &name);
                    }
                }
                _ => {}
            }
        }
    }
}
```

### File Checklist

| File | Changes |
|------|---------|
| `core/src/snapshot/mod.rs` | New file: StateSnapshot, SnapshotManager |
| `core/src/app/session.rs` | Add SnapshotManager, quick_save/load methods |
| `core/src/app/snapshot_ui.rs` | New file: SnapshotBrowserUI |
| `core/src/lib.rs` | Export snapshot module |
| `emberware-z/src/app/mod.rs` | Wire hotkeys and browser UI |
| `core/Cargo.toml` | Add `bincode` for serialization, `dirs` for paths |

### Test Cases

1. **Quick save/load**: Press F5, modify game, press F9, verify state restored
2. **Multiple slots**: Save to slots 1-3, load each, verify correct state
3. **Undo**: Load snapshot, verify undo slot populated
4. **Named save**: Save with name, restart emulator, load, verify works
5. **Auto-save**: Wait 2 minutes, verify auto-save created
6. **Empty slot**: Try to load empty slot, verify error message
7. **ROM mismatch**: Load snapshot from different ROM, verify warning

## Future Enhancements

1. **Snapshot timeline**: Visual timeline of all snapshots
2. **Diff view**: Compare two snapshots
3. **Cloud sync**: Sync snapshots across machines
4. **Snapshot annotations**: Mark specific moments
5. **Checkpoint system**: Auto-save at game events
