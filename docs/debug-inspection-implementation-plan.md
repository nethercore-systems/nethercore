# Debug Inspection System: Implementation Plan

This document provides a phased implementation plan for the Debug Inspection System specified in `debug-inspection-spec.md`. The plan prioritizes robust architecture over quick implementation.

---

## Design Decisions (from Q&A)

| Topic | Decision |
|-------|----------|
| Memory safety | WASM can't access outside its memory; pre-allocate max RAM, disallow buffer growth |
| Threading | Single-threaded; no synchronization needed |
| Registration lifetime | `init()` only; no dynamic registration for now |
| egui location | Core dependency (enables all future consoles automatically) |
| Group errors | Panic on mismatch; auto-close unclosed groups at registration end |
| Time scale + GGRS | Debug features disabled during netplay; single-player/local only |
| Console scope | Console-agnostic in core; no console-specific code needed |
| Callback re-entrancy | Allowed; if WASM changes values, that's fine |
| Release linking | Wasmtime ignores unused definitions; always register debug FFI |
| Float precision | Full round-trip precision for export |
| Fixed-point | Support `fixed` crate types (e.g., `FixedI32<U16>` for Q16.16) |

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         core/src/debug/                             │
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────────────┐│
│  │   registry   │  │    types     │  │          ffi               ││
│  │              │  │              │  │                            ││
│  │ DebugRegistry│  │ ValueType    │  │ debug_register_f32()       ││
│  │ RegisteredVal│  │ DebugValue   │  │ debug_register_i32()       ││
│  │ GroupStack   │  │ Constraints  │  │ debug_group_begin()        ││
│  └──────┬───────┘  └──────────────┘  │ ...                        ││
│         │                            └─────────────┬──────────────┘│
│         │                                          │               │
│         ▼                                          │               │
│  ┌──────────────┐  ┌──────────────┐               │               │
│  │    panel     │  │ frame_ctrl   │               │               │
│  │              │  │              │               │               │
│  │ DebugPanel   │  │ FrameControl │◄──────────────┘               │
│  │ (egui UI)    │  │ pause/step   │                               │
│  │ tree render  │  │ time_scale   │                               │
│  └──────┬───────┘  └──────────────┘                               │
│         │                                                          │
│         ▼                                                          │
│  ┌──────────────┐                                                  │
│  │    export    │                                                  │
│  │              │                                                  │
│  │ Rust const   │                                                  │
│  │ formatting   │                                                  │
│  └──────────────┘                                                  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Core Data Structures

**Goal:** Define the foundational types for the debug registry.

### Files to Create

#### `core/src/debug/mod.rs`

```rust
//! Debug inspection system for runtime value editing
//!
//! Provides FFI functions for games to register values, and an egui panel
//! for viewing and editing those values at runtime.

mod export;
mod ffi;
mod frame_control;
mod panel;
mod registry;
mod types;

pub use export::ExportFormat;
pub use frame_control::FrameController;
pub use panel::DebugPanel;
pub use registry::DebugRegistry;
pub use types::{Constraints, DebugValue, RegisteredValue, ValueType};
```

#### `core/src/debug/types.rs`

Define all value types the system supports:

```rust
/// Supported value types for debug inspection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    // Primitives
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,

    // Compound types (known layouts)
    Vec2,   // { x: f32, y: f32 }
    Vec3,   // { x: f32, y: f32, z: f32 }
    Vec4,   // { x: f32, y: f32, z: f32, w: f32 }
    Rect,   // { x: i16, y: i16, w: i16, h: i16 }
    Color,  // { r: u8, g: u8, b: u8, a: u8 }

    // Fixed-point (compatible with `fixed` crate)
    FixedI16F8,   // Q8.8:  FixedI16<U8>  - 16-bit, 8 fractional
    FixedI32F8,   // Q24.8: FixedI32<U8>  - 32-bit, 8 fractional
    FixedI32F16,  // Q16.16: FixedI32<U16> - 32-bit, 16 fractional (most common)
    FixedI32F24,  // Q8.24: FixedI32<U24> - 32-bit, 24 fractional
}
```

**Implementation notes:**
- Each `ValueType` variant knows its byte size via a `fn byte_size(&self) -> usize` method
- Fixed-point types store raw bits; conversion to/from f64 uses the fractional bit count

#### `core/src/debug/registry.rs`

```rust
/// A value registered for debug inspection
pub struct RegisteredValue {
    /// Display name (from game)
    pub name: String,
    /// Full hierarchical path (e.g., "player/attacks/punch_hitbox")
    pub full_path: String,
    /// Pointer into WASM linear memory
    pub wasm_ptr: u32,
    /// Type of the value
    pub value_type: ValueType,
    /// Optional min/max constraints (renders as slider)
    pub constraints: Option<Constraints>,
}

/// The debug registry storing all registered values
pub struct DebugRegistry {
    /// All registered values (flat list)
    values: Vec<RegisteredValue>,
    /// Current group stack during registration (cleared after init)
    group_stack: Vec<String>,
    /// Whether registration is complete (after init() returns)
    registration_complete: bool,
    /// Optional callback function pointer in WASM
    change_callback: Option<u32>,
}
```

**Key methods:**
- `register(&mut self, name: &str, wasm_ptr: u32, value_type: ValueType, constraints: Option<Constraints>)`
- `group_begin(&mut self, name: &str)`
- `group_end(&mut self)` — panics if stack empty
- `finalize_registration(&mut self)` — auto-closes unclosed groups, sets `registration_complete = true`
- `read_value(&self, memory: &[u8], value: &RegisteredValue) -> DebugValue`
- `write_value(&self, memory: &mut [u8], value: &RegisteredValue, new_val: &DebugValue)`

### Design Considerations

1. **Pointer validation**: On every read/write, validate `wasm_ptr + byte_size <= memory.len()`. Log error and skip if invalid (don't panic—graceful degradation).

2. **Path building**: When `register()` is called, build `full_path` by joining `group_stack` with `/` separator.

3. **Tree structure**: The flat `Vec<RegisteredValue>` is sufficient. The UI builds a tree view by parsing `full_path` strings. This avoids complex nested data structures.

---

## Phase 2: Memory Access Layer

**Goal:** Safe, validated read/write to WASM linear memory.

### `core/src/debug/types.rs` (additions)

```rust
/// Runtime value for reading/editing
#[derive(Debug, Clone, PartialEq)]
pub enum DebugValue {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Bool(bool),
    Vec2 { x: f32, y: f32 },
    Vec3 { x: f32, y: f32, z: f32 },
    Vec4 { x: f32, y: f32, z: f32, w: f32 },
    Rect { x: i16, y: i16, w: i16, h: i16 },
    Color { r: u8, g: u8, b: u8, a: u8 },
    // Fixed-point stored as raw bits + displayed as f64
    FixedI16F8(i16),   // Display: raw as f64 / 256.0
    FixedI32F8(i32),   // Display: raw as f64 / 256.0
    FixedI32F16(i32),  // Display: raw as f64 / 65536.0
    FixedI32F24(i32),  // Display: raw as f64 / 16777216.0
}

impl DebugValue {
    /// Convert to f64 for display (works for all numeric types)
    pub fn as_display_f64(&self) -> Option<f64> { ... }

    /// Create from f64 input (for editing fixed-point)
    pub fn from_display_f64(value_type: ValueType, f: f64) -> Self { ... }
}
```

### Memory Access Implementation

```rust
impl DebugRegistry {
    pub fn read_value(&self, memory: &[u8], value: &RegisteredValue) -> Option<DebugValue> {
        let ptr = value.wasm_ptr as usize;
        let size = value.value_type.byte_size();

        // Bounds check
        if ptr.checked_add(size).map_or(true, |end| end > memory.len()) {
            log::warn!(
                "Debug read out of bounds: {} at 0x{:x} (size {})",
                value.name, ptr, size
            );
            return None;
        }

        let bytes = &memory[ptr..ptr + size];
        Some(match value.value_type {
            ValueType::F32 => DebugValue::F32(f32::from_le_bytes(bytes.try_into().unwrap())),
            ValueType::I32 => DebugValue::I32(i32::from_le_bytes(bytes.try_into().unwrap())),
            ValueType::FixedI32F16 => {
                DebugValue::FixedI32F16(i32::from_le_bytes(bytes.try_into().unwrap()))
            }
            // ... other types
        })
    }

    pub fn write_value(
        &self,
        memory: &mut [u8],
        value: &RegisteredValue,
        new_val: &DebugValue,
    ) -> bool {
        let ptr = value.wasm_ptr as usize;
        let size = value.value_type.byte_size();

        // Bounds check
        if ptr.checked_add(size).map_or(true, |end| end > memory.len()) {
            log::warn!(
                "Debug write out of bounds: {} at 0x{:x} (size {})",
                value.name, ptr, size
            );
            return false;
        }

        let bytes = &mut memory[ptr..ptr + size];
        match new_val {
            DebugValue::F32(v) => bytes.copy_from_slice(&v.to_le_bytes()),
            DebugValue::I32(v) => bytes.copy_from_slice(&v.to_le_bytes()),
            DebugValue::FixedI32F16(v) => bytes.copy_from_slice(&v.to_le_bytes()),
            // ... other types
        }
        true
    }
}
```

---

## Phase 3: FFI Registration Functions

**Goal:** Implement the FFI functions games call to register values.

### `core/src/debug/ffi.rs`

```rust
use wasmtime::{Caller, Linker};
use crate::console::ConsoleInput;
use crate::wasm::GameStateWithConsole;

/// Register all debug FFI functions with the linker
///
/// These are always registered (even for release WASM) because wasmtime
/// ignores definitions the module doesn't import.
pub fn register_debug_ffi<I: ConsoleInput, S: Send + Default + 'static>(
    linker: &mut Linker<GameStateWithConsole<I, S>>,
) -> anyhow::Result<()> {
    // Primitives
    linker.func_wrap("env", "debug_register_i8", debug_register_i8::<I, S>)?;
    linker.func_wrap("env", "debug_register_i16", debug_register_i16::<I, S>)?;
    linker.func_wrap("env", "debug_register_i32", debug_register_i32::<I, S>)?;
    linker.func_wrap("env", "debug_register_u8", debug_register_u8::<I, S>)?;
    linker.func_wrap("env", "debug_register_u16", debug_register_u16::<I, S>)?;
    linker.func_wrap("env", "debug_register_u32", debug_register_u32::<I, S>)?;
    linker.func_wrap("env", "debug_register_f32", debug_register_f32::<I, S>)?;
    linker.func_wrap("env", "debug_register_bool", debug_register_bool::<I, S>)?;

    // Ranged variants
    linker.func_wrap("env", "debug_register_i32_range", debug_register_i32_range::<I, S>)?;
    linker.func_wrap("env", "debug_register_f32_range", debug_register_f32_range::<I, S>)?;
    linker.func_wrap("env", "debug_register_u8_range", debug_register_u8_range::<I, S>)?;

    // Compound types
    linker.func_wrap("env", "debug_register_vec2", debug_register_vec2::<I, S>)?;
    linker.func_wrap("env", "debug_register_vec3", debug_register_vec3::<I, S>)?;
    linker.func_wrap("env", "debug_register_rect", debug_register_rect::<I, S>)?;
    linker.func_wrap("env", "debug_register_color", debug_register_color::<I, S>)?;

    // Fixed-point
    linker.func_wrap("env", "debug_register_fixed_i32f16", debug_register_fixed_i32f16::<I, S>)?;

    // Grouping
    linker.func_wrap("env", "debug_group_begin", debug_group_begin::<I, S>)?;
    linker.func_wrap("env", "debug_group_end", debug_group_end::<I, S>)?;

    // Query functions
    linker.func_wrap("env", "debug_is_paused", debug_is_paused::<I, S>)?;
    linker.func_wrap("env", "debug_get_time_scale", debug_get_time_scale::<I, S>)?;

    // Callback
    linker.func_wrap("env", "debug_set_change_callback", debug_set_change_callback::<I, S>)?;

    Ok(())
}
```

### Helper for Reading C Strings

```rust
/// Read a null-terminated C string from WASM memory
fn read_c_string(memory: &[u8], ptr: u32) -> Option<String> {
    let ptr = ptr as usize;
    if ptr >= memory.len() {
        return None;
    }

    // Find null terminator (with reasonable limit)
    let max_len = 256;
    let end = memory[ptr..]
        .iter()
        .take(max_len)
        .position(|&b| b == 0)?;

    std::str::from_utf8(&memory[ptr..ptr + end])
        .ok()
        .map(String::from)
}
```

### Example FFI Function

```rust
fn debug_register_f32<I: ConsoleInput, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    value_ptr: u32,
) {
    let memory = match caller.data().game.memory {
        Some(m) => m,
        None => return,
    };

    let mem_data = memory.data(&caller);
    let name = match read_c_string(mem_data, name_ptr) {
        Some(n) => n,
        None => {
            log::warn!("debug_register_f32: invalid name pointer");
            return;
        }
    };

    caller.data_mut().debug_registry.register(
        &name,
        value_ptr,
        ValueType::F32,
        None,
    );
}

fn debug_register_f32_range<I: ConsoleInput, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    value_ptr: u32,
    min: f32,
    max: f32,
) {
    // Similar to above, but with constraints
    let constraints = Some(Constraints {
        min: min as f64,
        max: max as f64,
    });
    // ... register with constraints
}
```

### Integration Point

In `core/src/ffi.rs`, add call to `register_debug_ffi`:

```rust
pub fn register_common_ffi<I: ConsoleInput, S: Send + Default + 'static>(
    linker: &mut Linker<GameStateWithConsole<I, S>>,
) -> Result<()> {
    // ... existing registrations ...

    // Debug inspection FFI
    crate::debug::ffi::register_debug_ffi(linker)?;

    Ok(())
}
```

---

## Phase 4: Frame Controller

**Goal:** Implement pause, step, and time scale controls.

### `core/src/debug/frame_control.rs`

```rust
/// Frame control state for debug inspection
#[derive(Debug, Clone)]
pub struct FrameController {
    /// Whether the game is paused
    paused: bool,
    /// Single-step requested (consumed after one tick)
    step_requested: bool,
    /// Time scale multiplier (1.0 = normal)
    time_scale: f32,
    /// Available time scale presets
    time_scale_presets: &'static [f32],
    /// Current preset index
    time_scale_index: usize,
    /// Whether debug features are enabled (disabled during netplay)
    enabled: bool,
}

impl Default for FrameController {
    fn default() -> Self {
        Self {
            paused: false,
            step_requested: false,
            time_scale: 1.0,
            time_scale_presets: &[0.1, 0.25, 0.5, 1.0, 2.0],
            time_scale_index: 3, // 1.0x
            enabled: true,
        }
    }
}

impl FrameController {
    /// Check if we should run a tick this frame
    ///
    /// Returns `true` if the game should advance, `false` if paused.
    pub fn should_run_tick(&mut self) -> bool {
        if !self.enabled {
            return true; // Always run when disabled (netplay)
        }

        if self.paused {
            if self.step_requested {
                self.step_requested = false;
                return true;
            }
            return false;
        }
        true
    }

    /// Get the number of ticks to run this frame based on time scale
    ///
    /// For time_scale < 1.0, this returns 0 or 1 based on accumulator.
    /// For time_scale >= 1.0, this returns 1 or more.
    ///
    /// Note: This is only for single-player/local. During netplay,
    /// GGRS controls tick count and this returns 1.
    pub fn ticks_to_run(&self, accumulator_ticks: f32) -> u32 {
        if !self.enabled {
            return 1;
        }

        // For slow-mo, we skip ticks
        // For fast-forward, we run multiple ticks
        (accumulator_ticks * self.time_scale).floor().max(0.0) as u32
    }

    /// Toggle pause state
    pub fn toggle_pause(&mut self) {
        if self.enabled {
            self.paused = !self.paused;
        }
    }

    /// Request a single frame step (only works when paused)
    pub fn request_step(&mut self) {
        if self.enabled && self.paused {
            self.step_requested = true;
        }
    }

    /// Cycle time scale to next slower preset
    pub fn decrease_time_scale(&mut self) {
        if self.enabled && self.time_scale_index > 0 {
            self.time_scale_index -= 1;
            self.time_scale = self.time_scale_presets[self.time_scale_index];
        }
    }

    /// Cycle time scale to next faster preset
    pub fn increase_time_scale(&mut self) {
        if self.enabled && self.time_scale_index < self.time_scale_presets.len() - 1 {
            self.time_scale_index += 1;
            self.time_scale = self.time_scale_presets[self.time_scale_index];
        }
    }

    /// Disable debug features (called when entering netplay)
    pub fn disable(&mut self) {
        self.enabled = false;
        self.paused = false;
        self.step_requested = false;
        self.time_scale = 1.0;
        self.time_scale_index = 3;
    }

    /// Enable debug features (called when exiting netplay)
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    // Getters for FFI
    pub fn is_paused(&self) -> bool { self.paused && self.enabled }
    pub fn time_scale(&self) -> f32 { if self.enabled { self.time_scale } else { 1.0 } }
    pub fn is_enabled(&self) -> bool { self.enabled }
}
```

---

## Phase 5: Debug Panel UI

**Goal:** Implement the egui panel for viewing and editing values.

### `core/src/debug/panel.rs`

```rust
use egui::{Context, Ui};
use hashbrown::HashSet;

/// Debug panel state and rendering
pub struct DebugPanel {
    /// Whether the panel is visible
    visible: bool,
    /// Collapsed group paths
    collapsed: HashSet<String>,
    /// Search/filter text (future feature, initially empty)
    filter: String,
}

impl Default for DebugPanel {
    fn default() -> Self {
        Self {
            visible: false,
            collapsed: HashSet::new(),
            filter: String::new(),
        }
    }
}

impl DebugPanel {
    /// Toggle panel visibility
    pub fn toggle_visible(&mut self) {
        self.visible = !self.visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Render the debug panel
    ///
    /// Returns `true` if any value was changed (to trigger callback).
    pub fn render(
        &mut self,
        ctx: &Context,
        registry: &DebugRegistry,
        frame_ctrl: &mut FrameController,
        memory: &mut [u8],
    ) -> bool {
        if !self.visible {
            return false;
        }

        let mut changed = false;

        egui::SidePanel::right("debug_inspection_panel")
            .default_width(320.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Debug Inspector");
                ui.separator();

                // Frame controls
                changed |= self.render_frame_controls(ui, frame_ctrl);
                ui.separator();

                // Value tree
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        changed |= self.render_value_tree(ui, registry, memory);
                    });

                ui.separator();

                // Export buttons
                self.render_export_buttons(ui, registry, memory);
            });

        changed
    }

    fn render_frame_controls(&mut self, ui: &mut Ui, frame_ctrl: &mut FrameController) -> bool {
        if !frame_ctrl.is_enabled() {
            ui.colored_label(
                egui::Color32::YELLOW,
                "⚠ Debug controls disabled during netplay"
            );
            return false;
        }

        ui.horizontal(|ui| {
            // Pause/Play button
            let pause_text = if frame_ctrl.is_paused() { "▶ Play" } else { "⏸ Pause" };
            if ui.button(pause_text).clicked() {
                frame_ctrl.toggle_pause();
            }

            // Step button (only enabled when paused)
            ui.add_enabled_ui(frame_ctrl.is_paused(), |ui| {
                if ui.button("⏭ Step").clicked() {
                    frame_ctrl.request_step();
                }
            });

            // Time scale
            ui.label("Speed:");
            if ui.button("−").clicked() {
                frame_ctrl.decrease_time_scale();
            }
            ui.label(format!("{:.2}x", frame_ctrl.time_scale()));
            if ui.button("+").clicked() {
                frame_ctrl.increase_time_scale();
            }
        });

        false
    }

    fn render_value_tree(
        &mut self,
        ui: &mut Ui,
        registry: &DebugRegistry,
        memory: &mut [u8],
    ) -> bool {
        let mut changed = false;
        let tree = build_tree(registry.values());
        changed |= self.render_tree_node(ui, registry, memory, &tree, "");
        changed
    }

    fn render_tree_node(
        &mut self,
        ui: &mut Ui,
        registry: &DebugRegistry,
        memory: &mut [u8],
        node: &TreeNode,
        path: &str,
    ) -> bool {
        let mut changed = false;

        // Render child groups
        for (name, child) in &node.children {
            let child_path = if path.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", path, name)
            };

            let is_collapsed = self.collapsed.contains(&child_path);
            let header = egui::CollapsingHeader::new(name)
                .default_open(!is_collapsed)
                .show(ui, |ui| {
                    changed |= self.render_tree_node(ui, registry, memory, child, &child_path);
                });

            // Track collapse state
            if header.fully_closed() {
                self.collapsed.insert(child_path);
            } else {
                self.collapsed.remove(&child_path);
            }
        }

        // Render leaf values
        for value in &node.values {
            changed |= self.render_value_widget(ui, registry, memory, value);
        }

        changed
    }

    fn render_value_widget(
        &mut self,
        ui: &mut Ui,
        registry: &DebugRegistry,
        memory: &mut [u8],
        value: &RegisteredValue,
    ) -> bool {
        let current = match registry.read_value(memory, value) {
            Some(v) => v,
            None => {
                ui.colored_label(egui::Color32::RED, format!("{}: <invalid>", value.name));
                return false;
            }
        };

        let new_val = match (&value.value_type, &value.constraints, &current) {
            // Float with range -> slider
            (ValueType::F32, Some(c), DebugValue::F32(v)) => {
                let mut val = *v;
                ui.horizontal(|ui| {
                    ui.label(&value.name);
                    ui.add(egui::Slider::new(&mut val, (c.min as f32)..=(c.max as f32)));
                });
                DebugValue::F32(val)
            }

            // Float without range -> drag value
            (ValueType::F32, None, DebugValue::F32(v)) => {
                let mut val = *v;
                ui.horizontal(|ui| {
                    ui.label(&value.name);
                    ui.add(egui::DragValue::new(&mut val).speed(0.1));
                });
                DebugValue::F32(val)
            }

            // Integer with range -> slider
            (ValueType::I32, Some(c), DebugValue::I32(v)) => {
                let mut val = *v;
                ui.horizontal(|ui| {
                    ui.label(&value.name);
                    ui.add(egui::Slider::new(&mut val, (c.min as i32)..=(c.max as i32)));
                });
                DebugValue::I32(val)
            }

            // Bool -> checkbox
            (ValueType::Bool, _, DebugValue::Bool(v)) => {
                let mut val = *v;
                ui.checkbox(&mut val, &value.name);
                DebugValue::Bool(val)
            }

            // Color -> color picker
            (ValueType::Color, _, DebugValue::Color { r, g, b, a }) => {
                let mut color = [
                    *r as f32 / 255.0,
                    *g as f32 / 255.0,
                    *b as f32 / 255.0,
                    *a as f32 / 255.0,
                ];
                ui.horizontal(|ui| {
                    ui.label(&value.name);
                    ui.color_edit_button_rgba_unmultiplied(&mut color);
                });
                DebugValue::Color {
                    r: (color[0] * 255.0) as u8,
                    g: (color[1] * 255.0) as u8,
                    b: (color[2] * 255.0) as u8,
                    a: (color[3] * 255.0) as u8,
                }
            }

            // Rect -> four drag values
            (ValueType::Rect, _, DebugValue::Rect { x, y, w, h }) => {
                let (mut vx, mut vy, mut vw, mut vh) = (*x, *y, *w, *h);
                ui.horizontal(|ui| {
                    ui.label(&value.name);
                    ui.add(egui::DragValue::new(&mut vx).prefix("x:"));
                    ui.add(egui::DragValue::new(&mut vy).prefix("y:"));
                    ui.add(egui::DragValue::new(&mut vw).prefix("w:"));
                    ui.add(egui::DragValue::new(&mut vh).prefix("h:"));
                });
                DebugValue::Rect { x: vx, y: vy, w: vw, h: vh }
            }

            // Fixed-point Q16.16 -> drag value with f64 display
            (ValueType::FixedI32F16, _, DebugValue::FixedI32F16(raw)) => {
                let mut display = *raw as f64 / 65536.0;
                ui.horizontal(|ui| {
                    ui.label(&value.name);
                    ui.add(egui::DragValue::new(&mut display).speed(0.01));
                    ui.weak("(Q16.16)");
                });
                DebugValue::FixedI32F16((display * 65536.0).round() as i32)
            }

            // ... other types follow same pattern

            _ => {
                // Fallback: display as read-only
                ui.horizontal(|ui| {
                    ui.label(&value.name);
                    ui.weak(format!("{:?}", current));
                });
                current
            }
        };

        // Write back if changed
        if new_val != current {
            registry.write_value(memory, value, &new_val);
            return true;
        }

        false
    }

    fn render_export_buttons(
        &self,
        ui: &mut Ui,
        registry: &DebugRegistry,
        memory: &[u8],
    ) {
        ui.horizontal(|ui| {
            if ui.button("📋 Copy as Rust").clicked() {
                let text = crate::debug::export::export_rust_flat(registry, memory);
                ui.output_mut(|o| o.copied_text = text);
            }
        });
    }
}

// ============================================================================
// Tree building helpers
// ============================================================================

struct TreeNode<'a> {
    children: Vec<(String, TreeNode<'a>)>,
    values: Vec<&'a RegisteredValue>,
}

fn build_tree<'a>(values: &'a [RegisteredValue]) -> TreeNode<'a> {
    let mut root = TreeNode {
        children: Vec::new(),
        values: Vec::new(),
    };

    for value in values {
        insert_into_tree(&mut root, value, &value.full_path);
    }

    root
}

fn insert_into_tree<'a>(node: &mut TreeNode<'a>, value: &'a RegisteredValue, path: &str) {
    if let Some(slash_pos) = path.find('/') {
        let (group, rest) = path.split_at(slash_pos);
        let rest = &rest[1..]; // skip the '/'

        // Find or create child
        let child = node.children
            .iter_mut()
            .find(|(name, _)| name == group)
            .map(|(_, child)| child);

        if let Some(child) = child {
            insert_into_tree(child, value, rest);
        } else {
            let mut new_child = TreeNode {
                children: Vec::new(),
                values: Vec::new(),
            };
            insert_into_tree(&mut new_child, value, rest);
            node.children.push((group.to_string(), new_child));
        }
    } else {
        // Leaf value
        node.values.push(value);
    }
}
```

---

## Phase 6: Export System

**Goal:** Implement export to Rust const declarations.

### `core/src/debug/export.rs`

```rust
use crate::debug::{DebugRegistry, DebugValue, RegisteredValue, ValueType};

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// Flat Rust const declarations
    RustFlat,
    /// Grouped Rust modules (future)
    RustModules,
}

/// Export all values as flat Rust const declarations
pub fn export_rust_flat(registry: &DebugRegistry, memory: &[u8]) -> String {
    let mut output = String::from("// Exported from Emberware Debug Inspector\n\n");

    for value in registry.values() {
        if let Some(debug_val) = registry.read_value(memory, value) {
            let const_name = value.name.to_uppercase().replace(' ', "_").replace('-', "_");
            let line = format_rust_const(&const_name, &value.value_type, &debug_val);
            output.push_str(&line);
            output.push('\n');
        }
    }

    output
}

fn format_rust_const(name: &str, value_type: &ValueType, value: &DebugValue) -> String {
    match (value_type, value) {
        (ValueType::F32, DebugValue::F32(v)) => {
            // Full round-trip precision
            format!("const {}: f32 = {:?};", name, v)
        }
        (ValueType::F64, DebugValue::F64(v)) => {
            format!("const {}: f64 = {:?};", name, v)
        }
        (ValueType::I8, DebugValue::I8(v)) => {
            format!("const {}: i8 = {};", name, v)
        }
        (ValueType::I16, DebugValue::I16(v)) => {
            format!("const {}: i16 = {};", name, v)
        }
        (ValueType::I32, DebugValue::I32(v)) => {
            format!("const {}: i32 = {};", name, v)
        }
        (ValueType::I64, DebugValue::I64(v)) => {
            format!("const {}: i64 = {};", name, v)
        }
        (ValueType::U8, DebugValue::U8(v)) => {
            format!("const {}: u8 = {};", name, v)
        }
        (ValueType::U16, DebugValue::U16(v)) => {
            format!("const {}: u16 = {};", name, v)
        }
        (ValueType::U32, DebugValue::U32(v)) => {
            format!("const {}: u32 = {};", name, v)
        }
        (ValueType::U64, DebugValue::U64(v)) => {
            format!("const {}: u64 = {};", name, v)
        }
        (ValueType::Bool, DebugValue::Bool(v)) => {
            format!("const {}: bool = {};", name, v)
        }
        (ValueType::Vec2, DebugValue::Vec2 { x, y }) => {
            format!("const {}: Vec2 = Vec2 {{ x: {:?}, y: {:?} }};", name, x, y)
        }
        (ValueType::Vec3, DebugValue::Vec3 { x, y, z }) => {
            format!(
                "const {}: Vec3 = Vec3 {{ x: {:?}, y: {:?}, z: {:?} }};",
                name, x, y, z
            )
        }
        (ValueType::Rect, DebugValue::Rect { x, y, w, h }) => {
            format!(
                "const {}: Rect = Rect {{ x: {}, y: {}, w: {}, h: {} }};",
                name, x, y, w, h
            )
        }
        (ValueType::Color, DebugValue::Color { r, g, b, a }) => {
            format!(
                "const {}: Color = Color {{ r: {}, g: {}, b: {}, a: {} }};",
                name, r, g, b, a
            )
        }
        (ValueType::FixedI32F16, DebugValue::FixedI32F16(raw)) => {
            // Export as raw bits with comment showing decimal
            let decimal = *raw as f64 / 65536.0;
            format!(
                "const {}: FixedI32<U16> = FixedI32::from_bits({}); // ≈ {:.6}",
                name, raw, decimal
            )
        }
        _ => format!("// {}: unsupported type", name),
    }
}
```

---

## Phase 7: Integration with GameState

**Goal:** Add `DebugRegistry` to `GameStateWithConsole` and integrate with runtime.

### Modify `core/src/wasm/state.rs`

Add the debug registry to the game state:

```rust
use crate::debug::{DebugRegistry, DebugPanel, FrameController};

pub struct GameStateWithConsole<I: ConsoleInput, S> {
    pub game: GameState<I>,
    pub console: S,
    /// Debug inspection registry (always present, empty in release WASM)
    pub debug_registry: DebugRegistry,
}

impl<I: ConsoleInput, S: Default> GameStateWithConsole<I, S> {
    pub fn new() -> Self {
        Self {
            game: GameState::new(),
            console: S::default(),
            debug_registry: DebugRegistry::new(),
        }
    }
}
```

### Modify `core/src/wasm/mod.rs` (GameInstance)

Add method to finalize registration after init():

```rust
impl<I: ConsoleInput, S: Default + 'static> GameInstance<I, S> {
    pub fn init(&mut self) -> Result<()> {
        // ... existing init logic ...

        // Finalize debug registration
        self.store.data_mut().debug_registry.finalize_registration();

        Ok(())
    }
}
```

### Add Debug State to App

In whatever struct manages the application state (likely in `core/src/app/`), add:

```rust
pub struct AppState {
    // ... existing fields ...

    /// Debug panel UI state
    pub debug_panel: DebugPanel,
    /// Frame controller (pause/step/time scale)
    pub frame_controller: FrameController,
}
```

---

## Phase 8: Hotkey Configuration

**Goal:** Add debug hotkey configuration to the config system.

### Modify `core/src/app/config.rs`

```rust
/// Debug hotkey configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebugConfig {
    /// Key to toggle debug panel (default: F3)
    #[serde(default = "default_panel_toggle")]
    pub panel_toggle: String,
    /// Key to toggle pause (default: F5)
    #[serde(default = "default_pause_toggle")]
    pub pause_toggle: String,
    /// Key to step one frame (default: F6)
    #[serde(default = "default_step_frame")]
    pub step_frame: String,
    /// Key to decrease time scale (default: F7)
    #[serde(default = "default_speed_decrease")]
    pub speed_decrease: String,
    /// Key to increase time scale (default: F8)
    #[serde(default = "default_speed_increase")]
    pub speed_increase: String,
}

fn default_panel_toggle() -> String { "F3".to_string() }
fn default_pause_toggle() -> String { "F5".to_string() }
fn default_step_frame() -> String { "F6".to_string() }
fn default_speed_decrease() -> String { "F7".to_string() }
fn default_speed_increase() -> String { "F8".to_string() }

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            panel_toggle: default_panel_toggle(),
            pause_toggle: default_pause_toggle(),
            step_frame: default_step_frame(),
            speed_decrease: default_speed_decrease(),
            speed_increase: default_speed_increase(),
        }
    }
}

// Add to main Config struct:
pub struct Config {
    pub video: VideoConfig,
    pub audio: AudioConfig,
    pub input: InputConfig,
    #[serde(default)]
    pub debug: DebugConfig,
}
```

---

## Phase 9: Runtime Integration

**Goal:** Integrate frame controller with the game loop.

### Modify `core/src/runtime.rs`

The runtime needs to respect the frame controller:

```rust
impl<C: Console> Runtime<C> {
    /// Run a single frame with debug frame control
    pub fn frame_with_debug(
        &mut self,
        frame_ctrl: &mut FrameController,
    ) -> Result<(u32, f32)> {
        // Check if debug features should be disabled (netplay active)
        if self.session.as_ref().map_or(false, |s| s.is_networked()) {
            frame_ctrl.disable();
        }

        // Check if we should run
        if !frame_ctrl.should_run_tick() {
            // Still need to render UI, but skip game tick
            return Ok((0, 0.0));
        }

        // Normal frame logic with time scale consideration
        // ... (existing frame() logic, but tick count affected by time_scale)

        self.frame()
    }
}
```

**Alternative approach:** Keep `Runtime::frame()` unchanged and have the app layer check `frame_ctrl.should_run_tick()` before calling it. This keeps the runtime simpler.

---

## Phase 10: Change Callback

**Goal:** Invoke the WASM callback when values change.

### In the panel render loop

After detecting a change:

```rust
// In DebugPanel::render(), after writing a changed value:
if changed {
    if let Some(callback_ptr) = registry.change_callback() {
        // Need access to the WASM instance to call the callback
        // This requires passing the TypedFunc or similar
        // Option 1: Store callback as TypedFunc in registry
        // Option 2: Return "changed" flag and let caller invoke
    }
}
```

**Recommended approach:** Have `DebugPanel::render()` return a bool indicating whether any value changed. The app layer then invokes the callback if needed. This keeps the panel decoupled from WASM execution.

```rust
// In app layer:
let changed = debug_panel.render(ctx, &registry, &mut frame_ctrl, memory);
if changed {
    if let Some(callback) = game.debug_change_callback() {
        callback.call(&mut store, ())?;
    }
}
```

---

## File Summary

### Files to Create

| File | Description |
|------|-------------|
| `core/src/debug/mod.rs` | Module root, re-exports |
| `core/src/debug/types.rs` | `ValueType`, `DebugValue`, `Constraints` |
| `core/src/debug/registry.rs` | `DebugRegistry`, `RegisteredValue` |
| `core/src/debug/panel.rs` | `DebugPanel` egui rendering |
| `core/src/debug/frame_control.rs` | `FrameController` |
| `core/src/debug/ffi.rs` | FFI function implementations |
| `core/src/debug/export.rs` | Export formatting |

### Files to Modify

| File | Changes |
|------|---------|
| `core/src/lib.rs` | Add `pub mod debug;` |
| `core/src/ffi.rs` | Call `register_debug_ffi()` |
| `core/src/wasm/state.rs` | Add `debug_registry` field |
| `core/src/wasm/mod.rs` | Finalize registration after init |
| `core/src/app/config.rs` | Add `DebugConfig` |
| `core/src/app/types.rs` | Add debug state fields if needed |

---

## Testing Strategy

### Unit Tests

1. **Registry tests** (`registry.rs`)
   - Register values and verify storage
   - Group nesting produces correct paths
   - Mismatched `group_end()` panics
   - Unclosed groups auto-close on finalize

2. **Memory access tests** (`registry.rs`)
   - Read/write each value type
   - Out-of-bounds returns `None`/`false`
   - Fixed-point conversion accuracy

3. **Export tests** (`export.rs`)
   - Each type formats correctly
   - Float precision is round-trip safe

4. **Frame controller tests** (`frame_control.rs`)
   - Pause blocks ticks
   - Step advances exactly once
   - Time scale presets cycle correctly
   - Disable/enable works

### Integration Tests

1. **FFI registration** (`core/src/integration.rs`)
   - WAT module calls `debug_register_*` functions
   - Values appear in registry
   - `debug_is_paused()` returns correct state

2. **End-to-end**
   - Load test game with debug values
   - Verify panel renders
   - Edit value and verify memory changes
   - Export and verify output

---

## Implementation Order

1. **Phase 1-2**: Types and registry (foundation)
2. **Phase 3**: FFI functions (enables testing with real WASM)
3. **Phase 4**: Frame controller (standalone, easy to test)
4. **Phase 5**: Panel UI (depends on 1-4)
5. **Phase 6**: Export (depends on 1-2)
6. **Phase 7**: GameState integration (connects everything)
7. **Phase 8**: Config (polish)
8. **Phase 9-10**: Runtime integration and callbacks (final integration)

Each phase can be merged independently after testing.

---

## Open Implementation Details

These should be resolved during implementation:

1. **Tree node ordering**: Should groups appear before values, or in registration order?
   - Recommendation: Registration order (matches game code structure)

2. **Callback storage**: Store as `u32` pointer or `TypedFunc`?
   - Recommendation: Store `u32`, resolve to `TypedFunc` when invoking

3. **Panel position**: Right side panel or floating window?
   - Recommendation: Right panel (spec shows this), but could be configurable

4. **Keyboard focus**: Should hotkeys work when egui has focus?
   - Recommendation: Yes, debug hotkeys should always work

---

## Future Considerations (Out of Scope)

Per the spec, these are documented but not implemented:

- Visual rect overlay on game view
- Draggable rect editing in game view
- Value change graphs over time
- State snapshot bookmarks
- Network debug sync across P2P
- Watch expressions (computed values)
- Search/filter in large registries
- Named presets for value configurations
- `#[derive(DebugWatch)]` macro

---

**End of Implementation Plan**
