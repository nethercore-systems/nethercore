# Debug Inspection System: Implementation Spec

Runtime value inspection and modification for fast iteration during Emberware game development.

**Scope:** Cross-console feature (Emberware Z and Classic)

---

## 1. Overview

### Problem

Tuning gameplay values (speeds, damages, hitboxes, frame data) currently requires:
1. Edit source code constant
2. Recompile WASM
3. Reload game
4. Test change
5. Repeat

For numeric tuning, this loop is unnecessarily slow.

### Solution

Games register pointers to values they want to expose. The console provides:
- Debug panel UI for viewing/editing registered values
- Frame control (pause, step, slow-mo) for precise observation
- Export to copy tuned values back to source code

### Design Principles

| Principle | Implementation |
|-----------|----------------|
| Zero ship overhead | Debug code compiles out via `#[cfg(debug_assertions)]` |
| No code path divergence | Same `include_bytes!` workflow, just add inspection |
| Game controls visualization | Console edits data; games draw their own overlays if desired |
| Cross-console | Same FFI for Z and Classic |

---

## 2. Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         GAME (WASM)                         │
│                                                             │
│  fn init() {                                                │
│      debug_register_f32("gravity", &GRAVITY);               │
│      debug_register_rect("punch_hitbox", &PUNCH.hitbox);    │
│  }                                                          │
│                                                             │
│  static mut GRAVITY: f32 = 9.8;                             │
│  static mut PUNCH: MoveData = ...;                          │
│                                                             │
└──────────────────────────┬──────────────────────────────────┘
                           │ FFI calls during init()
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    CONSOLE (Host)                           │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Debug Registry                          │   │
│  │  Vec<RegisteredValue> {                              │   │
│  │    name: String,                                     │   │
│  │    wasm_ptr: u32,                                    │   │
│  │    value_type: ValueType,                            │   │
│  │    constraints: Option<Constraints>,                 │   │
│  │    group_path: Vec<String>,                          │   │
│  │  }                                                   │   │
│  └─────────────────────────────────────────────────────┘   │
│                           │                                 │
│                           ▼                                 │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Debug Panel (egui)                      │   │
│  │  - Renders registered values as editable widgets    │   │
│  │  - Writes to WASM linear memory on change           │   │
│  │  - Organizes by group hierarchy                     │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Frame Controller                        │   │
│  │  - Pause/resume game loop                           │   │
│  │  - Step single frames                               │   │
│  │  - Adjust tick rate (slow-mo)                       │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. FFI API

### 3.1 Value Registration

All registration functions take a name (null-terminated C string) and a pointer into WASM linear memory.

```rust
// Primitives
fn debug_register_i8(name: *const c_char, ptr: *const i8);
fn debug_register_i16(name: *const c_char, ptr: *const i16);
fn debug_register_i32(name: *const c_char, ptr: *const i32);
fn debug_register_u8(name: *const c_char, ptr: *const u8);
fn debug_register_u16(name: *const c_char, ptr: *const u16);
fn debug_register_u32(name: *const c_char, ptr: *const u32);
fn debug_register_f32(name: *const c_char, ptr: *const f32);
fn debug_register_bool(name: *const c_char, ptr: *const bool);

// With range constraints (renders as slider)
fn debug_register_i32_range(name: *const c_char, ptr: *const i32, min: i32, max: i32);
fn debug_register_f32_range(name: *const c_char, ptr: *const f32, min: f32, max: f32);
fn debug_register_u8_range(name: *const c_char, ptr: *const u8, min: u8, max: u8);
// ... other ranged variants as needed

// Compound types (known struct layouts)
fn debug_register_vec2(name: *const c_char, ptr: *const Vec2);   // { x: f32, y: f32 }
fn debug_register_vec3(name: *const c_char, ptr: *const Vec3);   // { x: f32, y: f32, z: f32 }
fn debug_register_rect(name: *const c_char, ptr: *const Rect);   // { x: i16, y: i16, w: i16, h: i16 }
fn debug_register_color(name: *const c_char, ptr: *const Color); // { r: u8, g: u8, b: u8, a: u8 }
```

**Struct layout requirements:**

```rust
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
pub struct Rect {
    pub x: i16,
    pub y: i16,
    pub w: i16,
    pub h: i16,
}

#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
```

### 3.2 Grouping

Organize values into collapsible hierarchy.

```rust
fn debug_group_begin(name: *const c_char);
fn debug_group_end();
```

**Usage:**
```rust
debug_group_begin(c"player");
debug_register_f32(c"speed", &PLAYER_SPEED);
debug_register_i32(c"health", &PLAYER_HEALTH);

debug_group_begin(c"attacks");
debug_register_rect(c"punch_hitbox", &PUNCH_HITBOX);
debug_register_u8(c"punch_damage", &PUNCH_DAMAGE);
debug_group_end();

debug_group_end();
```

**Renders as:**
```
▼ player
    speed: [3.50]
    health: [100]
  ▼ attacks
      punch_hitbox: x[12] y[8] w[32] h[24]
      punch_damage: [15]
```

### 3.3 Frame Control

These are **console-provided**, not game-registered. Games can query state if needed.

```rust
// Query current debug state (useful for games that want to draw differently when paused)
fn debug_is_paused() -> bool;
fn debug_get_time_scale() -> f32;  // 1.0 = normal, 0.5 = half speed, etc.
```

Frame control itself is handled by console UI/hotkeys, not FFI calls.

### 3.4 Value Change Callback (Optional)

Games may want to react when a debug value changes (e.g., recalculate derived data).

```rust
// Register a callback function to be called when ANY debug value changes
fn debug_set_change_callback(callback: extern "C" fn());
```

**Usage:**
```rust
extern "C" fn on_debug_change() {
    unsafe {
        // Recalculate derived values
        TOTAL_DAMAGE = PUNCH_DAMAGE + KICK_DAMAGE;
    }
}

fn init() {
    debug_set_change_callback(on_debug_change);
}
```

---

## 4. Console-Side Implementation

### 4.1 Debug Registry

```rust
pub struct DebugRegistry {
    values: Vec<RegisteredValue>,
    group_stack: Vec<String>,
    change_callback: Option<u32>,  // WASM function pointer
}

pub struct RegisteredValue {
    name: String,
    full_path: String,           // "player/attacks/punch_hitbox"
    wasm_ptr: u32,               // Pointer into WASM linear memory
    value_type: ValueType,
    constraints: Option<Constraints>,
}

pub enum ValueType {
    I8, I16, I32,
    U8, U16, U32,
    F32,
    Bool,
    Vec2,
    Vec3,
    Rect,
    Color,
}

pub struct Constraints {
    min: f64,
    max: f64,
}
```

### 4.2 Memory Access

Reading and writing WASM linear memory:

```rust
impl DebugRegistry {
    fn read_value(&self, memory: &Memory, value: &RegisteredValue) -> DebugValue {
        let data = memory.data(&store);
        let ptr = value.wasm_ptr as usize;
        
        match value.value_type {
            ValueType::F32 => {
                let bytes: [u8; 4] = data[ptr..ptr+4].try_into().unwrap();
                DebugValue::F32(f32::from_le_bytes(bytes))
            }
            ValueType::I32 => {
                let bytes: [u8; 4] = data[ptr..ptr+4].try_into().unwrap();
                DebugValue::I32(i32::from_le_bytes(bytes))
            }
            ValueType::Rect => {
                let x = i16::from_le_bytes(data[ptr..ptr+2].try_into().unwrap());
                let y = i16::from_le_bytes(data[ptr+2..ptr+4].try_into().unwrap());
                let w = i16::from_le_bytes(data[ptr+4..ptr+6].try_into().unwrap());
                let h = i16::from_le_bytes(data[ptr+6..ptr+8].try_into().unwrap());
                DebugValue::Rect { x, y, w, h }
            }
            // ... other types
        }
    }
    
    fn write_value(&self, memory: &mut Memory, value: &RegisteredValue, new_val: DebugValue) {
        let data = memory.data_mut(&mut store);
        let ptr = value.wasm_ptr as usize;
        
        match (value.value_type, new_val) {
            (ValueType::F32, DebugValue::F32(v)) => {
                data[ptr..ptr+4].copy_from_slice(&v.to_le_bytes());
            }
            // ... other types
        }
        
        // Trigger callback if registered
        if let Some(callback_ptr) = self.change_callback {
            self.invoke_callback(callback_ptr);
        }
    }
}
```

### 4.3 Debug Panel UI (egui)

```rust
pub struct DebugPanel {
    visible: bool,
    registry: DebugRegistry,
    frame_controller: FrameController,
    collapsed_groups: HashSet<String>,
}

impl DebugPanel {
    pub fn render(&mut self, ctx: &egui::Context, memory: &mut Memory) {
        if !self.visible {
            return;
        }
        
        egui::SidePanel::right("debug_panel")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Debug Values");
                ui.separator();
                
                // Frame controls
                self.render_frame_controls(ui);
                ui.separator();
                
                // Registered values
                self.render_value_tree(ui, memory);
                ui.separator();
                
                // Export buttons
                self.render_export_buttons(ui, memory);
            });
    }
    
    fn render_value_tree(&mut self, ui: &mut egui::Ui, memory: &mut Memory) {
        // Build tree from flat list using full_path
        let tree = self.build_tree();
        self.render_tree_node(ui, memory, &tree, "");
    }
    
    fn render_value_widget(&mut self, ui: &mut egui::Ui, memory: &mut Memory, value: &RegisteredValue) {
        let current = self.registry.read_value(memory, value);
        
        let new_val = match (value.value_type, &value.constraints) {
            (ValueType::F32, Some(c)) => {
                let mut v = current.as_f32();
                ui.add(egui::Slider::new(&mut v, c.min as f32..=c.max as f32).text(&value.name));
                DebugValue::F32(v)
            }
            (ValueType::F32, None) => {
                let mut v = current.as_f32();
                ui.add(egui::DragValue::new(&mut v).speed(0.1).prefix(&format!("{}: ", value.name)));
                DebugValue::F32(v)
            }
            (ValueType::Bool, _) => {
                let mut v = current.as_bool();
                ui.checkbox(&mut v, &value.name);
                DebugValue::Bool(v)
            }
            (ValueType::Rect, _) => {
                let (mut x, mut y, mut w, mut h) = current.as_rect();
                ui.horizontal(|ui| {
                    ui.label(&value.name);
                    ui.add(egui::DragValue::new(&mut x).prefix("x:"));
                    ui.add(egui::DragValue::new(&mut y).prefix("y:"));
                    ui.add(egui::DragValue::new(&mut w).prefix("w:"));
                    ui.add(egui::DragValue::new(&mut h).prefix("h:"));
                });
                DebugValue::Rect { x, y, w, h }
            }
            (ValueType::Color, _) => {
                let (r, g, b, a) = current.as_color();
                let mut color = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0];
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
            // ... other types
        };
        
        if new_val != current {
            self.registry.write_value(memory, value, new_val);
        }
    }
}
```

### 4.4 Frame Controller

```rust
pub struct FrameController {
    paused: bool,
    step_requested: bool,
    time_scale: f32,           // 1.0 = normal, 0.1 = 10% speed
    time_scale_options: [f32; 5], // [0.1, 0.25, 0.5, 1.0, 2.0]
}

impl FrameController {
    pub fn should_run_tick(&mut self) -> bool {
        if self.paused {
            if self.step_requested {
                self.step_requested = false;
                return true;
            }
            return false;
        }
        true
    }
    
    pub fn get_effective_delta(&self, base_delta: f32) -> f32 {
        base_delta * self.time_scale
    }
    
    pub fn render_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Pause/Play toggle
            let pause_text = if self.paused { "▶ Play" } else { "⏸ Pause" };
            if ui.button(pause_text).clicked() {
                self.paused = !self.paused;
            }
            
            // Step frame (only when paused)
            ui.add_enabled_ui(self.paused, |ui| {
                if ui.button("⏭ Step").clicked() {
                    self.step_requested = true;
                }
            });
            
            // Time scale dropdown
            egui::ComboBox::from_label("Speed")
                .selected_text(format!("{}x", self.time_scale))
                .show_ui(ui, |ui| {
                    for &scale in &self.time_scale_options {
                        ui.selectable_value(&mut self.time_scale, scale, format!("{}x", scale));
                    }
                });
        });
        
        // Frame counter (if available)
        // ui.label(format!("Frame: {}", current_frame));
    }
}
```

---

## 5. Export System

### 5.1 Export Formats

**Rust constants (primary):**
```rust
// Exported from Debug Panel
const PLAYER_SPEED: f32 = 3.5;
const PLAYER_HEALTH: i32 = 100;
const PUNCH_HITBOX: Rect = Rect { x: 12, y: 8, w: 32, h: 24 };
const PUNCH_DAMAGE: u8 = 15;
```

**Structured Rust (grouped):**
```rust
// player
mod player {
    pub const SPEED: f32 = 3.5;
    pub const HEALTH: i32 = 100;
    
    // player/attacks
    pub mod attacks {
        pub const PUNCH_HITBOX: Rect = Rect { x: 12, y: 8, w: 32, h: 24 };
        pub const PUNCH_DAMAGE: u8 = 15;
    }
}
```

### 5.2 Export Implementation

```rust
impl DebugPanel {
    fn export_as_rust_flat(&self, memory: &Memory) -> String {
        let mut output = String::from("// Exported from Debug Panel\n\n");
        
        for value in &self.registry.values {
            let current = self.registry.read_value(memory, value);
            let const_name = value.name.to_uppercase().replace(" ", "_");
            
            let line = match (value.value_type, current) {
                (ValueType::F32, DebugValue::F32(v)) => {
                    format!("const {}: f32 = {:.6};\n", const_name, v)
                }
                (ValueType::I32, DebugValue::I32(v)) => {
                    format!("const {}: i32 = {};\n", const_name, v)
                }
                (ValueType::U8, DebugValue::U8(v)) => {
                    format!("const {}: u8 = {};\n", const_name, v)
                }
                (ValueType::Bool, DebugValue::Bool(v)) => {
                    format!("const {}: bool = {};\n", const_name, v)
                }
                (ValueType::Rect, DebugValue::Rect { x, y, w, h }) => {
                    format!("const {}: Rect = Rect {{ x: {}, y: {}, w: {}, h: {} }};\n", 
                            const_name, x, y, w, h)
                }
                (ValueType::Color, DebugValue::Color { r, g, b, a }) => {
                    format!("const {}: Color = Color {{ r: {}, g: {}, b: {}, a: {} }};\n",
                            const_name, r, g, b, a)
                }
                (ValueType::Vec2, DebugValue::Vec2 { x, y }) => {
                    format!("const {}: Vec2 = Vec2 {{ x: {:.6}, y: {:.6} }};\n",
                            const_name, x, y)
                }
                // ... other types
            };
            
            output.push_str(&line);
        }
        
        output
    }
    
    fn render_export_buttons(&self, ui: &mut egui::Ui, memory: &Memory) {
        ui.horizontal(|ui| {
            if ui.button("📋 Copy as Rust").clicked() {
                let text = self.export_as_rust_flat(memory);
                ui.output_mut(|o| o.copied_text = text);
            }
            
            // Future: other formats
            // if ui.button("📋 Copy as RON").clicked() { ... }
        });
    }
}
```

---

## 6. Hotkey Configuration

Console-level configuration (not game-controlled).

### 6.1 Default Hotkeys

| Action | Default Key | Notes |
|--------|-------------|-------|
| Toggle debug panel | F3 | Configurable |
| Pause/Resume | F5 | When panel open |
| Step frame | F6 | When paused |
| Decrease speed | F7 | Cycle through presets |
| Increase speed | F8 | Cycle through presets |

### 6.2 Configuration

```toml
# ~/.emberware/config.toml

[debug]
panel_toggle = "F3"
pause_toggle = "F5"
step_frame = "F6"
speed_decrease = "F7"
speed_increase = "F8"
```

---

## 7. Build Configuration

### 7.1 Game-Side (Conditional Compilation)

Debug registration should compile out in release builds:

```rust
// In game code

#[cfg(debug_assertions)]
mod debug_registration {
    use crate::*;
    
    #[link(wasm_import_module = "env")]
    extern "C" {
        fn debug_register_f32(name: *const i8, ptr: *const f32);
        fn debug_register_i32(name: *const i8, ptr: *const i32);
        fn debug_register_rect(name: *const i8, ptr: *const Rect);
        fn debug_group_begin(name: *const i8);
        fn debug_group_end();
        // ... etc
    }
    
    pub unsafe fn register_all() {
        debug_group_begin(c"player");
        debug_register_f32(c"speed", &PLAYER_SPEED);
        debug_register_i32(c"health", &PLAYER_HEALTH);
        debug_group_end();
        // ... etc
    }
}

fn init() {
    #[cfg(debug_assertions)]
    unsafe {
        debug_registration::register_all();
    }
    
    // ... rest of init
}
```

### 7.2 Console-Side (FFI Stubs)

The console must handle missing FFI imports gracefully for release builds:

```rust
// When linking WASM module, provide no-op stubs for debug functions
// if the game was built in release mode (imports won't exist)

fn link_debug_functions(linker: &mut Linker<GameState>) -> Result<()> {
    // These are optional - don't fail if game doesn't import them
    let _ = linker.func_wrap("env", "debug_register_f32", |_: i32, _: i32| {});
    let _ = linker.func_wrap("env", "debug_register_i32", |_: i32, _: i32| {});
    // ... etc
    
    Ok(())
}
```

**Alternative approach:** Use `linker.define()` with optional semantics, or check if import exists before providing.

---

## 8. Integration with Game Loop

### 8.1 Modified Game Loop

```rust
// In core runtime

pub fn run_frame(&mut self) {
    // Check if we should run this tick
    if !self.debug_panel.frame_controller.should_run_tick() {
        // Still render (so UI is responsive), but don't update game
        self.render_only();
        return;
    }
    
    // Get effective delta time (may be scaled)
    let effective_delta = self.debug_panel.frame_controller
        .get_effective_delta(self.base_delta);
    
    // Store for FFI query
    self.game_state.debug_time_scale = self.debug_panel.frame_controller.time_scale;
    self.game_state.debug_paused = self.debug_panel.frame_controller.paused;
    
    // Normal update
    self.call_wasm_update(effective_delta);
    self.call_wasm_render();
    
    // Render debug overlay on top
    self.debug_panel.render(&self.egui_ctx, &mut self.wasm_memory);
}
```

### 8.2 Rollback Considerations

**Important:** Debug modifications happen to *live* WASM memory. During rollback:

- Values are restored to snapshot state (debug changes are lost on rollback)
- This is correct behavior—debug changes are ephemeral exploration
- If you want to keep a change, export and rebuild

For single-player/development, this is fine. The debug panel is a tuning tool, not a save system.

---

## 9. Files to Create/Modify

### 9.1 Core (Cross-Console)

| Action | File | Description |
|--------|------|-------------|
| **Create** | `core/src/debug/mod.rs` | Debug module root |
| **Create** | `core/src/debug/registry.rs` | Value registration and storage |
| **Create** | `core/src/debug/panel.rs` | egui panel rendering |
| **Create** | `core/src/debug/frame_control.rs` | Pause, step, time scale |
| **Create** | `core/src/debug/export.rs` | Export formatting |
| **Create** | `core/src/debug/ffi.rs` | FFI function implementations |
| **Modify** | `core/src/lib.rs` | Add debug module |
| **Modify** | `core/src/ffi.rs` | Link debug FFI functions |
| **Modify** | `core/src/runtime.rs` | Integrate frame controller |

### 9.2 Console-Specific

| Action | File | Description |
|--------|------|-------------|
| **Modify** | `emberware-z/src/app.rs` | Render debug panel, handle hotkeys |
| **Modify** | `emberware-z/src/config.rs` | Add debug hotkey config |
| **Modify** | `emberware-classic/src/app.rs` | Same as Z |
| **Modify** | `emberware-classic/src/config.rs` | Same as Z |

### 9.3 Documentation

| Action | File | Description |
|--------|------|-------------|
| **Create** | `docs/debug-inspection.md` | Developer guide for using debug features |
| **Modify** | `docs/ffi.md` | Add debug FFI reference |
| **Modify** | `docs/developer-guide.md` | Link to debug docs, workflow tips |

### 9.4 Shared Types (Optional Helper Crate)

| Action | File | Description |
|--------|------|-------------|
| **Create** | `emberware-debug/Cargo.toml` | Optional helper crate for games |
| **Create** | `emberware-debug/src/lib.rs` | Safe wrappers, helper macros |

---

## 10. Example: Complete Game Integration

### 10.1 Game Data Structures

```rust
// game/src/data.rs

#[repr(C)]
pub struct Rect {
    pub x: i16,
    pub y: i16,
    pub w: i16,
    pub h: i16,
}

#[repr(C)]
pub struct MoveData {
    pub startup: u8,
    pub active: u8,
    pub recovery: u8,
    pub damage: u8,
    pub hitbox: Rect,
    pub hurtbox: Rect,
}

#[repr(C)]
pub struct CharacterData {
    pub walk_speed: f32,
    pub run_speed: f32,
    pub jump_force: f32,
    pub gravity: f32,
    pub health: i32,
    pub standing_light: MoveData,
    pub standing_heavy: MoveData,
    pub crouching_light: MoveData,
    pub crouching_heavy: MoveData,
}

// Global state
pub static mut RYU: CharacterData = CharacterData {
    walk_speed: 2.5,
    run_speed: 5.0,
    jump_force: 12.0,
    gravity: 0.6,
    health: 1000,
    standing_light: MoveData {
        startup: 4,
        active: 3,
        recovery: 8,
        damage: 30,
        hitbox: Rect { x: 20, y: -40, w: 50, h: 30 },
        hurtbox: Rect { x: -20, y: -80, w: 40, h: 80 },
    },
    // ... etc
};
```

### 10.2 Debug Registration

```rust
// game/src/debug.rs

#[cfg(debug_assertions)]
pub mod debug {
    use crate::data::*;
    
    #[link(wasm_import_module = "env")]
    extern "C" {
        fn debug_register_f32(name: *const i8, ptr: *const f32);
        fn debug_register_f32_range(name: *const i8, ptr: *const f32, min: f32, max: f32);
        fn debug_register_i32(name: *const i8, ptr: *const i32);
        fn debug_register_u8(name: *const i8, ptr: *const u8);
        fn debug_register_u8_range(name: *const i8, ptr: *const u8, min: u8, max: u8);
        fn debug_register_rect(name: *const i8, ptr: *const Rect);
        fn debug_group_begin(name: *const i8);
        fn debug_group_end();
    }
    
    unsafe fn register_move(name: *const i8, move_data: &MoveData) {
        debug_group_begin(name);
        debug_register_u8_range(c"startup", &move_data.startup, 1, 30);
        debug_register_u8_range(c"active", &move_data.active, 1, 20);
        debug_register_u8_range(c"recovery", &move_data.recovery, 1, 60);
        debug_register_u8_range(c"damage", &move_data.damage, 0, 255);
        debug_register_rect(c"hitbox", &move_data.hitbox);
        debug_register_rect(c"hurtbox", &move_data.hurtbox);
        debug_group_end();
    }
    
    pub unsafe fn register_all() {
        debug_group_begin(c"ryu");
        
        // Movement
        debug_group_begin(c"movement");
        debug_register_f32_range(c"walk_speed", &RYU.walk_speed, 0.0, 10.0);
        debug_register_f32_range(c"run_speed", &RYU.run_speed, 0.0, 15.0);
        debug_register_f32_range(c"jump_force", &RYU.jump_force, 0.0, 20.0);
        debug_register_f32_range(c"gravity", &RYU.gravity, 0.0, 2.0);
        debug_group_end();
        
        // Stats
        debug_register_i32(c"health", &RYU.health);
        
        // Moves
        debug_group_begin(c"moves");
        register_move(c"st_light", &RYU.standing_light);
        register_move(c"st_heavy", &RYU.standing_heavy);
        register_move(c"cr_light", &RYU.crouching_light);
        register_move(c"cr_heavy", &RYU.crouching_heavy);
        debug_group_end();
        
        debug_group_end();
    }
}
```

### 10.3 Init Integration

```rust
// game/src/lib.rs

mod data;

#[cfg(debug_assertions)]
mod debug;

#[no_mangle]
pub extern "C" fn init() {
    // Debug registration (compiles out in release)
    #[cfg(debug_assertions)]
    unsafe {
        debug::register_all();
    }
    
    // ... rest of init
}
```

### 10.4 Optional: Game-Side Debug Visualization

```rust
// game/src/render.rs

fn render() {
    // Normal game rendering...
    render_characters();
    render_projectiles();
    render_ui();
    
    // Debug visualization (game-controlled, not console)
    #[cfg(debug_assertions)]
    unsafe {
        if debug_is_paused() || SHOW_HITBOXES {
            render_hitbox_overlay(&RYU.standing_light.hitbox, 0xFF0000FF);  // Red
            render_hitbox_overlay(&RYU.standing_light.hurtbox, 0x00FF00FF); // Green
        }
    }
}

fn render_hitbox_overlay(rect: &Rect, color: u32) {
    // Use existing draw_rect FFI to visualize
    // Transform from character-local to screen space
    let screen_x = PLAYER_X + rect.x as f32;
    let screen_y = PLAYER_Y + rect.y as f32;
    draw_rect(screen_x, screen_y, rect.w as f32, rect.h as f32, color);
}
```

---

## 11. UI Mockup

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              GAME VIEW                                       │
│                                                                             │
│                         [Game renders here]                                 │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────┐
│ Debug Panel                        [X]  │
├─────────────────────────────────────────┤
│ ⏸ Pause  ⏭ Step   Speed: [1.0x ▼]     │
├─────────────────────────────────────────┤
│ ▼ ryu                                   │
│   ▼ movement                            │
│       walk_speed  [====2.5====]  2.50   │
│       run_speed   [=====5.0====] 5.00   │
│       jump_force  [====12.0===] 12.00   │
│       gravity     [===0.6===]    0.60   │
│   health          [1000        ]        │
│   ▼ moves                               │
│     ▼ st_light                          │
│         startup   [==4==]         4     │
│         active    [==3==]         3     │
│         recovery  [==8==]         8     │
│         damage    [==30=]        30     │
│         hitbox    x[20] y[-40] w[50] h[30] │
│         hurtbox   x[-20] y[-80] w[40] h[80]│
│     ▶ st_heavy (collapsed)              │
│     ▶ cr_light (collapsed)              │
│     ▶ cr_heavy (collapsed)              │
├─────────────────────────────────────────┤
│ [📋 Copy as Rust]                       │
└─────────────────────────────────────────┘
```

---

## 12. Testing Checklist

### 12.1 FFI Registration

- [ ] `debug_register_f32` correctly registers and reads f32 values
- [ ] `debug_register_i32` correctly registers and reads i32 values
- [ ] `debug_register_u8` correctly registers and reads u8 values
- [ ] `debug_register_bool` correctly registers and reads bool values
- [ ] `debug_register_rect` correctly registers and reads Rect structs
- [ ] `debug_register_vec2` correctly registers and reads Vec2 structs
- [ ] `debug_register_color` correctly registers and reads Color structs
- [ ] Range-constrained variants enforce min/max in UI
- [ ] `debug_group_begin`/`debug_group_end` creates correct hierarchy
- [ ] Nested groups render correctly
- [ ] Registering 100+ values doesn't cause performance issues

### 12.2 Value Editing

- [ ] Editing f32 via drag value updates WASM memory immediately
- [ ] Editing f32 via slider updates WASM memory immediately
- [ ] Editing i32 updates WASM memory immediately
- [ ] Editing bool via checkbox updates WASM memory immediately
- [ ] Editing Rect fields updates WASM memory immediately
- [ ] Editing Color via color picker updates WASM memory immediately
- [ ] Changes are visible in game on next frame
- [ ] Change callback is invoked when values change

### 12.3 Frame Control

- [ ] Pause stops game updates but UI remains responsive
- [ ] Step advances exactly one frame when paused
- [ ] Time scale 0.5x runs game at half speed
- [ ] Time scale 0.1x runs game at 10% speed
- [ ] Time scale 2.0x runs game at double speed
- [ ] `debug_is_paused()` FFI returns correct state
- [ ] `debug_get_time_scale()` FFI returns correct value

### 12.4 Export

- [ ] "Copy as Rust" produces valid Rust const syntax
- [ ] Exported f32 values have sufficient precision
- [ ] Exported Rect values have correct field names
- [ ] Exported Color values have correct field names
- [ ] Copy to clipboard works on all platforms

### 12.5 Build Configuration

- [ ] Debug build includes all registration code
- [ ] Release build excludes all registration code
- [ ] Release WASM is smaller than debug WASM
- [ ] Console handles missing debug imports gracefully
- [ ] No runtime errors when loading release WASM

### 12.6 Hotkeys

- [ ] F3 (default) toggles debug panel visibility
- [ ] F5 (default) toggles pause when panel open
- [ ] F6 (default) steps frame when paused
- [ ] F7/F8 (default) adjust time scale
- [ ] Custom hotkeys from config.toml work correctly

### 12.7 Rollback Interaction

- [ ] Debug-modified values are rolled back correctly
- [ ] Debug panel state persists across rollbacks
- [ ] Frame stepping works correctly with GGRS active
- [ ] No desync caused by debug modifications (values roll back)

### 12.8 Cross-Console

- [ ] Debug panel works in Emberware Z
- [ ] Debug panel works in Emberware Classic
- [ ] Same FFI signatures work for both consoles
- [ ] UI renders appropriately for each console's resolution

---

## 13. Future Extensions

These are out of scope for initial implementation but worth considering:

| Feature | Description | Complexity |
|---------|-------------|------------|
| Visual rect overlay | Console draws registered Rects on game view | Medium |
| Draggable rect editing | Click and drag Rects in game view | High |
| Value graphs | Plot value changes over time | Medium |
| State snapshots | Save/restore debug bookmarks | Medium |
| Network debug sync | Sync debug values across P2P session | High |
| Watch expressions | Computed values from registered data | Medium |
| Search/filter | Filter values by name in large registries | Low |
| Presets | Save/load named value configurations | Medium |
| Derive macro | `#[derive(DebugWatch)]` for automatic registration | Medium |

---

## 14. Open Questions

Resolved during spec creation:

| Question | Resolution |
|----------|------------|
| Visual overlays? | Game-controlled; console provides data only |
| Build config? | `#[cfg(debug_assertions)]`; compiles out in release |
| Grouping API? | `debug_group_begin`/`debug_group_end` pattern |
| Export formats? | Rust primary; extensible for future languages |
| Hotkey config? | Console-level config file |

---

## 15. Implementation Plan

Phased implementation plan with complete code, file paths, and integration points.

### 15.1 Design Decisions

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
| Float precision | Full round-trip precision for export (use `{:?}` format) |
| Fixed-point | Support `fixed` crate types (e.g., `FixedI32<U16>` for Q16.16) |

### 15.2 Existing Code Context

| File | Contains | Relevant Patterns |
|------|----------|-------------------|
| `core/src/ffi.rs` | Common FFI functions | How to register FFI with `Linker`, how to access `GameStateWithConsole` via `Caller` |
| `core/src/wasm/state.rs` | `GameState`, `GameStateWithConsole` | Where to add `debug_registry` field |
| `core/src/wasm/mod.rs` | `GameInstance` | Where to finalize registration after `init()`, how to access WASM memory |
| `core/src/console.rs` | `Console` trait, `ConsoleInput` | Generic bounds needed for FFI functions |
| `core/src/app/config.rs` | `Config` struct | Pattern for adding `DebugConfig` |
| `core/src/rollback/session.rs` | `RollbackSession`, `SessionType` | How to check if networked (for disabling debug) |

### 15.3 Implementation Phases

**Phase 1: Core Types** (`core/src/debug/types.rs`)
- `ValueType` enum with byte sizes, type names, fractional bits
- `DebugValue` enum for runtime values
- `Constraints` struct for min/max
- Fixed-point support (Q8.8, Q16.16, Q24.8, Q8.24)

**Phase 2: Registry** (`core/src/debug/registry.rs`)
- `DebugRegistry` struct with values, group stack, callbacks
- `RegisteredValue` struct with name, full_path, wasm_ptr, value_type, constraints
- `register()`, `group_begin()`, `group_end()`, `finalize_registration()`
- `read_value()`, `write_value()` with bounds checking

**Phase 3: FFI Functions** (`core/src/debug/ffi.rs`)
- All `debug_register_*` functions
- `debug_group_begin`, `debug_group_end`
- `debug_is_paused`, `debug_get_time_scale`
- `debug_set_change_callback`

**Phase 4: Frame Controller** (`core/src/debug/frame_control.rs`)
- Pause/resume, single-step, time scale
- `disable()` method for netplay

**Phase 5: Debug Panel** (`core/src/debug/panel.rs`)
- egui `SidePanel::right` for UI
- Tree-building from `full_path` strings
- Value widgets based on `ValueType` and `Constraints`

**Phase 6: Export** (`core/src/debug/export.rs`)
- `export_rust_flat()` function
- Use `{:?}` for float formatting (round-trip safe)

**Phase 7: GameState Integration**
- Add `debug_registry: DebugRegistry` to `GameStateWithConsole`
- Call `finalize_registration()` after init

**Phase 8: Config** (`core/src/app/config.rs`)
- Add `DebugConfig` with hotkey strings

**Phase 9: Runtime Integration**
- Check `session.is_networked()` to disable debug features
- Integrate `FrameController` with game loop

**Phase 10: Change Callback**
- Invoke WASM callback via stored function pointer

### 15.4 Console-Agnostic Integration

Debug state lives in:
- `DebugRegistry` → `GameStateWithConsole` (accessed by FFI via `Caller`)
- `DebugPanel`, `FrameController` → `GameSession` (UI/control state)

**Console wiring (2-3 lines per console):**
```rust
// In egui render:
if let Some(session) = &mut self.game_session {
    session.render_debug_panel(ctx);
}

// In key handler:
if let Some(session) = &mut self.game_session {
    if session.handle_debug_hotkey(&event.logical_key) {
        return;
    }
}
```

### 15.5 File Checklist

| Phase | Action | File |
|-------|--------|------|
| 1 | Create | `core/src/debug/types.rs` |
| 2 | Create | `core/src/debug/registry.rs` |
| 3 | Create | `core/src/debug/ffi.rs` |
| 3 | Modify | `core/src/ffi.rs` (add `register_debug_ffi()` call) |
| 4 | Create | `core/src/debug/frame_control.rs` |
| 5 | Create | `core/src/debug/panel.rs` |
| 6 | Create | `core/src/debug/export.rs` |
| 7 | Modify | `core/src/wasm/state.rs` (add `debug_registry` field) |
| 7 | Modify | `core/src/wasm/mod.rs` (finalize in init) |
| 8 | Modify | `core/src/app/config.rs` (add `DebugConfig`) |
| 9 | Modify | `core/src/app/session.rs` (add panel, frame controller) |
| - | Create | `core/src/debug/mod.rs` |
| - | Modify | `core/src/lib.rs` (add `pub mod debug`) |

### 15.6 Integration Test WAT

```wat
(module
    (import "env" "debug_register_f32_range" (func $debug_register_f32_range (param i32 i32 f32 f32)))
    (import "env" "debug_register_i32" (func $debug_register_i32 (param i32 i32)))
    (import "env" "debug_group_begin" (func $debug_group_begin (param i32)))
    (import "env" "debug_group_end" (func $debug_group_end))

    (memory (export "memory") 1)

    (data (i32.const 0) "player\00")
    (data (i32.const 16) "speed\00")
    (data (i32.const 32) "health\00")
    (data (i32.const 256) "\00\00\20\41")  ;; speed: 10.0f
    (data (i32.const 260) "\64\00\00\00")  ;; health: 100

    (func (export "init")
        (call $debug_group_begin (i32.const 0))
        (call $debug_register_f32_range (i32.const 16) (i32.const 256) (f32.const 0.0) (f32.const 20.0))
        (call $debug_register_i32 (i32.const 32) (i32.const 260))
        (call $debug_group_end)
    )
    (func (export "update"))
    (func (export "render"))
)
```

---

**End of Spec**
