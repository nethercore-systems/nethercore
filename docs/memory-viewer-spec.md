# Memory Viewer Specification

## Overview

The Memory Viewer provides a hex-dump view of the WASM linear memory, allowing developers to inspect raw memory state, find memory corruption, understand data layouts, and debug low-level issues. This is the "cheat engine" / "memory scanner" equivalent for Emberware.

## Use Cases

1. **Memory corruption debugging**: Find unexpected overwrites, buffer overflows
2. **Data structure inspection**: Verify struct layouts match expectations
3. **Save state debugging**: Understand serialization format
4. **Memory usage analysis**: See how much memory is used, find leaks
5. **Reverse engineering**: Understand memory patterns in your own code
6. **Cheat development**: Find and modify game values (for testing)

## Architecture

### Memory Access

WASM linear memory is a contiguous byte array. The host has full read/write access:

```rust
// In wasmtime
let memory = instance.get_memory(&mut store, "memory").unwrap();
let data = memory.data(&store);  // &[u8]
let data_mut = memory.data_mut(&mut store);  // &mut [u8]
```

### Memory Regions

```rust
/// A named memory region for easier navigation
pub struct MemoryRegion {
    /// Region name (e.g., "heap", "stack", "static")
    pub name: String,

    /// Start address
    pub start: u32,

    /// End address (exclusive)
    pub end: u32,

    /// Description
    pub description: String,

    /// Access pattern (read-only, read-write, etc.)
    pub access: MemoryAccess,
}

pub enum MemoryAccess {
    ReadOnly,
    ReadWrite,
    Executable,  // For WASM function table area
}

/// Bookmarked address for quick navigation
pub struct Bookmark {
    pub name: String,
    pub address: u32,
    pub data_type: DataType,
    pub notes: String,
}

/// How to interpret bytes at an address
pub enum DataType {
    U8, I8,
    U16Le, U16Be, I16Le, I16Be,
    U32Le, U32Be, I32Le, I32Be,
    U64Le, U64Be, I64Le, I64Be,
    F32Le, F32Be,
    F64Le, F64Be,
    Pointer,      // u32 address
    String,       // null-terminated
    StringLen(u32), // fixed length
    Hex,          // raw bytes
    Binary,       // bit view
}
```

### Memory Scanner

```rust
/// Search parameters
pub struct MemorySearch {
    /// What to search for
    pub pattern: SearchPattern,

    /// Where to search
    pub range: Option<(u32, u32)>,

    /// Previous results (for filtering)
    pub previous_results: Option<Vec<u32>>,
}

pub enum SearchPattern {
    /// Exact byte sequence
    Bytes(Vec<u8>),

    /// Value of specific type
    Value { data_type: DataType, value: Vec<u8> },

    /// Unknown value that changed
    Changed,

    /// Unknown value that stayed the same
    Unchanged,

    /// Unknown value that increased
    Increased,

    /// Unknown value that decreased
    Decreased,

    /// Value in range
    Range { data_type: DataType, min: Vec<u8>, max: Vec<u8> },
}

/// Search result
pub struct SearchResult {
    pub address: u32,
    pub current_value: Vec<u8>,
    pub previous_value: Option<Vec<u8>>,
}
```

## UI Design

### Main Memory View

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Memory Viewer                                                          [×] │
├─────────────────────────────────────────────────────────────────────────────┤
│ Address: [0x00010000    ] [Go] │ Region: [Heap      ▼] │ [Search] [Bookmarks]│
├─────────────────────────────────────────────────────────────────────────────┤
│          00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F   ASCII           │
├─────────────────────────────────────────────────────────────────────────────┤
│ 00010000 48 65 6C 6C 6F 20 57 6F  72 6C 64 21 00 00 00 00   Hello World!....│
│ 00010010 FF FF FF FF 00 00 00 00  2A 00 00 00 96 00 00 00   ........*.......│
│ 00010020 00 00 80 3F 00 00 00 40  CD CC 4C 40 00 00 00 00   ...?...@..L@....│
│ 00010030 ▓▓▓▓▓▓▓▓ ▓▓▓▓▓▓▓▓ ▓▓▓▓  ▓▓▓▓▓▓▓▓ ▓▓▓▓▓▓▓▓ ▓▓▓▓   (modified)       │
│ 00010040 00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00   ................│
│ ...                                                                         │
├─────────────────────────────────────────────────────────────────────────────┤
│ Selected: 0x00010020 │ u32: 1065353216 │ f32: 1.0 │ i32: 1065353216        │
│ Bytes (LE): 00 00 80 3F │ As pointer: -> 0x3F800000                         │
├─────────────────────────────────────────────────────────────────────────────┤
│ [◄ Page] [Page ►] │ Bytes per row: [16▼] │ [Freeze] │ [Export] │ [Compare] │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Features

1. **Hex View**: Classic hex dump with ASCII column
2. **Change Highlighting**: Recently modified bytes highlighted
3. **Selection**: Click to select bytes, show interpretations
4. **Edit**: Double-click to modify values
5. **Navigation**: Jump to address, page up/down, follow pointers
6. **Regions**: Named regions for quick navigation

### Search Panel

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Memory Search                                                          [×] │
├─────────────────────────────────────────────────────────────────────────────┤
│ Search Type: [Value ▼]  Data Type: [i32 ▼]  Value: [100      ]  [Search]   │
│                                                                             │
│ ☑ Search changed values only (from previous search)                        │
│ Range: [0x00000000] to [0xFFFFFFFF] (entire memory)                        │
├─────────────────────────────────────────────────────────────────────────────┤
│ Results: 42 matches                                                         │
│ ┌───────────────────────────────────────────────────────────────────────┐  │
│ │ Address    │ Current   │ Previous  │ Type  │                          │  │
│ │ 0x00012340 │ 100       │ 99        │ i32   │ [View] [Bookmark]        │  │
│ │ 0x00015A20 │ 100       │ 100       │ i32   │ [View] [Bookmark]        │  │
│ │ 0x0001F100 │ 100       │ 50        │ i32   │ [View] [Bookmark]        │  │
│ │ ...                                                                    │  │
│ └───────────────────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────────────────┤
│ [New Search] [Filter: Changed] [Filter: Unchanged] [Filter: Increased]     │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Search Workflow (Finding Player Health)

1. **Initial search**: Search for exact value (e.g., health = 100)
   - Returns thousands of matches
2. **Take damage**: Let health change to 80
3. **Filter**: Search for value = 80 among previous results
   - Returns fewer matches
4. **Repeat**: Continue until one address remains
5. **Bookmark**: Save address as "Player Health"

### Bookmarks Panel

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Bookmarks                                                              [×] │
├─────────────────────────────────────────────────────────────────────────────┤
│ ┌───────────────────────────────────────────────────────────────────────┐  │
│ │ Name           │ Address    │ Type  │ Value      │ Actions           │  │
│ │ Player Health  │ 0x00012340 │ i32   │ 80         │ [View] [Edit] [×] │  │
│ │ Player X       │ 0x00012350 │ f32   │ 15.234     │ [View] [Edit] [×] │  │
│ │ Enemy Count    │ 0x0001A000 │ u32   │ 5          │ [View] [Edit] [×] │  │
│ │ Gold           │ 0x0001A010 │ i32   │ 1500       │ [View] [Edit] [×] │  │
│ └───────────────────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────────────────┤
│ [Add Bookmark] [Import] [Export] [Watch Selected]                          │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Watch Window

Real-time display of bookmarked values:

```
┌─────────────────────────────────────┐
│ Watch                          [×] │
├─────────────────────────────────────┤
│ Player Health: 80 (i32)            │
│ Player X: 15.234 (f32)             │
│ Player Y: 3.500 (f32)              │
│ Gold: 1500 (i32)                   │
│ Enemy Count: 5 (u32)               │
└─────────────────────────────────────┘
```

## FFI API

### Memory Region Registration (Game → Host)

Games can register named regions for easier navigation:

```rust
// Register a named memory region
extern "C" fn debug_memory_register_region(
    name_ptr: u32, name_len: u32,
    start_address: u32,
    end_address: u32,
    access: u32,  // 0=RO, 1=RW, 2=Exec
);

// Register the heap region (usually from allocator)
extern "C" fn debug_memory_register_heap(start: u32, end: u32);

// Register the stack region
extern "C" fn debug_memory_register_stack(start: u32, end: u32);
```

### Memory Info (Game → Host)

```rust
// Get total memory size
extern "C" fn debug_memory_size() -> u32;

// Get current heap usage (if tracked by allocator)
extern "C" fn debug_memory_heap_used() -> u32;

// Get peak heap usage
extern "C" fn debug_memory_heap_peak() -> u32;
```

### Pointer Validation (Host → Game)

```rust
// Check if a pointer is valid (within allocated memory)
extern "C" fn debug_memory_is_valid_pointer(ptr: u32) -> i32;

// Get allocation size for a pointer (if tracked)
extern "C" fn debug_memory_allocation_size(ptr: u32) -> u32;
```

## Integration with Debug Inspection

### Automatic Address Discovery

When using the Debug Inspection system, registered variables automatically become memory viewer bookmarks:

```rust
// In game code
debug_register_f32(
    "player.position.x\0",
    &player.position.x as *const f32 as u32,
    DEBUG_CATEGORY_GAMEPLAY,
    DEBUG_FLAGS_READ_WRITE,
);

// Automatically creates bookmark:
// - Name: "player.position.x"
// - Address: 0x00012350 (wherever that pointer points)
// - Type: f32
```

### Cross-Reference

Click "View in Memory" from Debug Panel to jump to that address in Memory Viewer.
Click "Add to Debug Panel" from Memory Viewer to register an address for inspection.

## Memory Comparison

### Compare Current vs Saved State

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Memory Compare: Current vs save_state_001                              [×] │
├─────────────────────────────────────────────────────────────────────────────┤
│ Differences: 1,247 bytes in 83 regions                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│ ┌───────────────────────────────────────────────────────────────────────┐  │
│ │ Region        │ Address    │ Size │ Current        │ Previous        │  │
│ │ Frame Counter │ 0x00010000 │ 4    │ 00 00 01 2C    │ 00 00 00 00     │  │
│ │ Player State  │ 0x00012340 │ 64   │ (complex)      │ (complex)       │  │
│ │ Enemy Array   │ 0x0001A000 │ 512  │ (complex)      │ (complex)       │  │
│ │ ...                                                                    │  │
│ └───────────────────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────────────────┤
│ [Show only differences] [Export diff] [Restore selected]                   │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Use Cases

1. **Find what changed**: Compare before/after to identify what a function modified
2. **Rollback debugging**: Compare state after rollback to identify divergence
3. **Save game analysis**: Understand save file format

## Memory Snapshot

### Snapshot System

```rust
/// A complete memory snapshot
pub struct MemorySnapshot {
    /// Snapshot identifier
    pub id: String,

    /// Timestamp
    pub timestamp: std::time::Instant,

    /// Frame number when taken
    pub frame: u64,

    /// Full memory contents
    pub data: Vec<u8>,

    /// Optional description
    pub description: String,
}
```

### Operations

- **Take Snapshot**: Capture current memory state
- **Compare Snapshots**: Diff two snapshots
- **Restore Snapshot**: Write snapshot back to memory (warning: may corrupt state)
- **Export Snapshot**: Save to file
- **Import Snapshot**: Load from file

## Hotkeys

| Key | Action |
|-----|--------|
| F5 | Toggle Memory Viewer |
| Ctrl+G | Go to address |
| Ctrl+F | Open search |
| Ctrl+B | Add bookmark at selection |
| Ctrl+S | Take snapshot |
| Page Up/Down | Scroll memory |
| Ctrl+C | Copy selected bytes |
| Ctrl+V | Paste bytes (edit mode) |

## Performance Considerations

### Large Memory Handling

WASM memory can be up to 4GB. The viewer must handle this efficiently:

1. **Virtual scrolling**: Only render visible rows
2. **Lazy loading**: Only read visible memory region
3. **Async search**: Search in background thread
4. **Incremental diff**: Only compare changed pages

### Change Detection

Track memory changes efficiently:
1. Copy previous frame's memory (expensive for large memory)
2. Or: Use page-level dirty tracking if available
3. Or: Only track registered regions

**Recommendation**: Track only viewed region + bookmarks.

## Pending Questions

### Q1: Memory write permissions?
Should the viewer allow editing memory?
- A) Read-only for safety
- B) Edit with warning
- C) Edit freely

**Recommendation**: Option B - powerful for debugging but warn about risks.

### Q2: Search performance?
For large memory (100MB+), full scans are slow.
- A) Limit search range
- B) Background search with progress
- C) Index common patterns

**Recommendation**: Option B.

### Q3: Automatic structure detection?
Should we try to auto-detect data structures?
- A) No - just raw bytes
- B) Basic patterns (strings, pointers)
- C) Full structure inference

**Recommendation**: Start with A, add B later.

### Q4: Memory-mapped regions?
WASM has linear memory, but games may have logical regions:
- A) Just show linear addresses
- B) Allow region annotations
- C) Parse WASM debug info for symbols

**Recommendation**: Option B - game registers regions.

### Q5: Integration with source debugging?
Could we show source code references for addresses?
- Requires DWARF debug info
- Complex but very powerful

**Recommendation**: Future enhancement, not MVP.

### Q6: Persistence of bookmarks?
Should bookmarks persist across sessions?
- A) Session only
- B) Per-game persistence
- C) Exportable bookmark files

**Recommendation**: Option C - most flexible.

## Pros

1. **Low-level insight**: See exactly what's in memory
2. **Universal**: Works with any WASM game without game-side code
3. **Search capability**: Find values by searching
4. **Change tracking**: Identify what modified memory
5. **Comparison**: Diff snapshots to understand changes
6. **Edit capability**: Modify values for testing

## Cons

1. **Raw data**: No structure info without game-side registration
2. **Performance**: Large memory can be slow to search/diff
3. **Complexity**: Hex view requires low-level understanding
4. **Editing risk**: Modifying memory can corrupt game state
5. **Address instability**: Addresses may change between builds

## Implementation Complexity

**Estimated effort:** Medium-High

**Key components:**
1. Memory access layer - 0.5 days
2. Hex view renderer (egui) - 2 days
3. Search system - 2 days
4. Bookmark system - 1 day
5. Watch window - 0.5 days
6. Snapshot/compare - 2 days
7. Change highlighting - 1 day
8. Region registration FFI - 0.5 days
9. Debug Panel integration - 1 day
10. Testing - 1.5 days

**Total:** ~12 days

## Console-Agnostic Design

Memory viewing is inherently console-agnostic - all consoles use WASM linear memory. The viewer lives in core and uses egui for UI.

## Way Forward: Implementation Guide

This section provides concrete implementation steps based on the current Emberware codebase architecture.

### Step 1: Add Memory Viewer Types

**File: `core/src/debug/memory.rs` (new file)**

```rust
//! Memory viewer and search system

use std::collections::HashMap;

/// Memory region metadata
#[derive(Clone)]
pub struct MemoryRegion {
    pub name: String,
    pub start: u32,
    pub end: u32,
    pub description: String,
}

/// Bookmarked memory address
#[derive(Clone)]
pub struct Bookmark {
    pub name: String,
    pub address: u32,
    pub data_type: DataType,
    pub notes: String,
}

/// Data type for interpretation
#[derive(Clone, Copy, PartialEq)]
pub enum DataType {
    U8, I8, U16Le, I16Le, U32Le, I32Le, U64Le, I64Le,
    F32Le, F64Le, Pointer, Hex,
}

impl DataType {
    pub fn size(&self) -> usize {
        match self {
            DataType::U8 | DataType::I8 => 1,
            DataType::U16Le | DataType::I16Le => 2,
            DataType::U32Le | DataType::I32Le | DataType::F32Le | DataType::Pointer => 4,
            DataType::U64Le | DataType::I64Le | DataType::F64Le => 8,
            DataType::Hex => 1,
        }
    }

    pub fn format_value(&self, bytes: &[u8]) -> String {
        match self {
            DataType::U8 => format!("{}", bytes[0]),
            DataType::I8 => format!("{}", bytes[0] as i8),
            DataType::U16Le => format!("{}", u16::from_le_bytes([bytes[0], bytes[1]])),
            DataType::I16Le => format!("{}", i16::from_le_bytes([bytes[0], bytes[1]])),
            DataType::U32Le => format!("{}", u32::from_le_bytes(bytes[..4].try_into().unwrap())),
            DataType::I32Le => format!("{}", i32::from_le_bytes(bytes[..4].try_into().unwrap())),
            DataType::F32Le => format!("{:.6}", f32::from_le_bytes(bytes[..4].try_into().unwrap())),
            DataType::Pointer => format!("-> 0x{:08X}", u32::from_le_bytes(bytes[..4].try_into().unwrap())),
            DataType::Hex => format!("{:02X}", bytes[0]),
            _ => "...".to_string(),
        }
    }
}

/// Memory search state
pub struct MemorySearch {
    pub pattern: SearchPattern,
    pub results: Vec<SearchResult>,
    pub previous_values: HashMap<u32, Vec<u8>>,
}

pub enum SearchPattern {
    ExactValue { data_type: DataType, value: Vec<u8> },
    Changed,
    Unchanged,
    Increased,
    Decreased,
}

pub struct SearchResult {
    pub address: u32,
    pub current_value: Vec<u8>,
    pub previous_value: Option<Vec<u8>>,
}

/// Memory viewer state
pub struct MemoryViewer {
    pub view_address: u32,
    pub bytes_per_row: usize,
    pub regions: Vec<MemoryRegion>,
    pub bookmarks: Vec<Bookmark>,
    pub search: Option<MemorySearch>,
    pub previous_memory: Option<Vec<u8>>,
    pub selected_address: Option<u32>,
}

impl MemoryViewer {
    pub fn new() -> Self {
        Self {
            view_address: 0,
            bytes_per_row: 16,
            regions: Vec::new(),
            bookmarks: Vec::new(),
            search: None,
            previous_memory: None,
            selected_address: None,
        }
    }
}
```

### Step 2: Add Memory Viewer UI

**File: `core/src/app/memory_ui.rs` (new file)**

```rust
use crate::debug::memory::{MemoryViewer, DataType, Bookmark};
use egui::{Ui, Color32, RichText};

impl MemoryViewer {
    /// Render the memory viewer panel
    pub fn show(&mut self, ui: &mut Ui, memory: &[u8], prev_memory: Option<&[u8]>) {
        // Address input
        ui.horizontal(|ui| {
            ui.label("Address:");
            let mut addr_str = format!("{:08X}", self.view_address);
            if ui.text_edit_singleline(&mut addr_str).changed() {
                if let Ok(addr) = u32::from_str_radix(&addr_str, 16) {
                    self.view_address = addr;
                }
            }
            if ui.button("Go").clicked() {
                // Already updated
            }
        });

        ui.separator();

        // Hex view
        let start = self.view_address as usize;
        let rows = 16;

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(12.0));

            for row in 0..rows {
                let row_addr = start + row * self.bytes_per_row;
                if row_addr >= memory.len() {
                    break;
                }

                ui.horizontal(|ui| {
                    // Address column
                    ui.label(format!("{:08X}", row_addr));

                    // Hex bytes
                    for col in 0..self.bytes_per_row {
                        let addr = row_addr + col;
                        if addr >= memory.len() {
                            ui.label("  ");
                            continue;
                        }

                        let byte = memory[addr];
                        let changed = prev_memory
                            .map(|p| addr < p.len() && p[addr] != byte)
                            .unwrap_or(false);

                        let text = format!("{:02X}", byte);
                        let text = if changed {
                            RichText::new(text).color(Color32::YELLOW)
                        } else {
                            RichText::new(text)
                        };

                        if ui.selectable_label(
                            self.selected_address == Some(addr as u32),
                            text
                        ).clicked() {
                            self.selected_address = Some(addr as u32);
                        }

                        if col == 7 {
                            ui.label(" ");
                        }
                    }

                    ui.label(" ");

                    // ASCII column
                    let mut ascii = String::new();
                    for col in 0..self.bytes_per_row {
                        let addr = row_addr + col;
                        if addr >= memory.len() {
                            break;
                        }
                        let byte = memory[addr];
                        ascii.push(if byte.is_ascii_graphic() || byte == b' ' {
                            byte as char
                        } else {
                            '.'
                        });
                    }
                    ui.label(ascii);
                });
            }
        });

        // Selection info
        if let Some(addr) = self.selected_address {
            ui.separator();
            if (addr as usize) + 8 <= memory.len() {
                let bytes = &memory[addr as usize..addr as usize + 8];
                ui.horizontal(|ui| {
                    ui.label(format!("0x{:08X}:", addr));
                    ui.label(format!("u32: {}", u32::from_le_bytes(bytes[..4].try_into().unwrap())));
                    ui.label(format!("f32: {:.4}", f32::from_le_bytes(bytes[..4].try_into().unwrap())));
                    ui.label(format!("i32: {}", i32::from_le_bytes(bytes[..4].try_into().unwrap())));
                });
            }
        }
    }
}
```

### Step 3: Integrate into GameSession

**File: `core/src/app/session.rs`**

```rust
use crate::debug::memory::MemoryViewer;

pub struct GameSession<C: Console> {
    pub runtime: Runtime<C>,
    pub resource_manager: C::ResourceManager,
    pub debug_panel: DebugPanel,
    pub memory_viewer: MemoryViewer,  // NEW
    memory_viewer_visible: bool,
}

impl<C: Console> GameSession<C> {
    pub fn render_memory_viewer(&mut self, ctx: &egui::Context) -> bool {
        if !self.memory_viewer_visible {
            return false;
        }

        let game = match self.runtime.game_mut() {
            Some(g) => g,
            None => return false,
        };

        let memory = match game.store().data().game.memory {
            Some(m) => m,
            None => return false,
        };

        let mem_data = memory.data(game.store());

        egui::Window::new("Memory Viewer")
            .default_size([600.0, 400.0])
            .show(ctx, |ui| {
                self.memory_viewer.show(ui, mem_data, None);
            });

        true
    }

    pub fn toggle_memory_viewer(&mut self) {
        self.memory_viewer_visible = !self.memory_viewer_visible;
    }
}
```

### Step 4: Add Memory Region FFI

**File: `core/src/ffi.rs`**

```rust
// Add to register_common_ffi
linker.func_wrap("env", "debug_memory_register_region", debug_memory_register_region)?;
linker.func_wrap("env", "debug_memory_size", debug_memory_size)?;

fn debug_memory_register_region<I: ConsoleInput, S: Send + Default>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    start: u32,
    end: u32,
) {
    let memory = match caller.data().game.memory {
        Some(m) => m,
        None => return,
    };

    let name = {
        let data = memory.data(&caller);
        let ptr = name_ptr as usize;
        let len = name_len as usize;
        if ptr + len <= data.len() {
            String::from_utf8_lossy(&data[ptr..ptr + len]).to_string()
        } else {
            return;
        }
    };

    caller.data_mut().debug_registry.memory_regions.push(MemoryRegion {
        name,
        start,
        end,
        description: String::new(),
    });
}

fn debug_memory_size<I: ConsoleInput, S: Send + Default>(
    caller: Caller<'_, GameStateWithConsole<I, S>>
) -> u32 {
    caller.data().game.memory
        .map(|m| m.data_size(&caller) as u32)
        .unwrap_or(0)
}
```

### Step 5: Wire into Console App

**File: `emberware-z/src/app/mod.rs`**

```rust
impl App {
    fn handle_key_input(&mut self, event: KeyEvent) {
        // ... existing key handling ...

        if event.state.is_pressed() {
            match event.logical_key {
                Key::Named(NamedKey::F6) => {
                    if let Some(session) = &mut self.game_session {
                        session.toggle_memory_viewer();
                    }
                }
                _ => {}
            }
        }
    }

    fn render(&mut self) {
        // ... in the egui context ...

        // Render memory viewer if in Playing mode
        if matches!(mode, AppMode::Playing { .. }) {
            if let Some(session) = &mut self.game_session {
                session.render_memory_viewer(&self.egui_ctx);
            }
        }
    }
}
```

### File Checklist

| File | Changes |
|------|---------|
| `core/src/debug/memory.rs` | New file: MemoryViewer, Bookmark, DataType, Search types |
| `core/src/debug/mod.rs` | Export memory module |
| `core/src/app/memory_ui.rs` | New file: egui hex view renderer |
| `core/src/app/session.rs` | Add MemoryViewer field and rendering |
| `core/src/ffi.rs` | Add memory region FFI functions |
| `emberware-z/src/app/mod.rs` | Wire F6 hotkey and render call |

### Test Cases

1. **Hex view rendering**: Open viewer, verify memory displayed correctly
2. **Address navigation**: Enter address, verify view scrolls
3. **Change highlighting**: Modify memory, verify yellow highlight
4. **Selection**: Click byte, verify interpretation shown
5. **Search exact**: Search for known value, verify found
6. **Search changed**: Modify value, search changed, verify narrowed
7. **Bookmarks**: Add bookmark, verify persists across frames
8. **Region registration**: Register heap region, verify appears in dropdown

## Future Enhancements

1. **Structure templates**: Define struct layouts for automatic parsing
2. **Source integration**: DWARF debug info for symbol names
3. **Memory profiling**: Track allocations over time
4. **Heap visualization**: Graphical view of memory fragmentation
5. **Pointer graphs**: Visualize pointer relationships
6. **Scripted automation**: Lua/JavaScript for custom memory tools
