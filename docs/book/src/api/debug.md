# Debug Functions

Runtime value inspection via the F3 debug panel.

## Overview

The debug system allows you to register game variables for live editing and monitoring. Press **F3** to open the debug panel during development.

**Features:**
- Live value editing (sliders, color pickers)
- Read-only watches
- Grouped organization
- Frame control (pause, step, time scale)
- Zero overhead in release builds

---

## Value Registration

Register editable values in `init()`. The debug panel will show controls for these values.

### debug_register_i32

Registers an editable 32-bit signed integer.

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn debug_register_i32(name_ptr: *const u8, name_len: u32, ptr: *const i32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void debug_register_i32(const uint8_t* name_ptr, uint32_t name_len, const int32_t* ptr);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn debug_register_i32(name_ptr: [*]const u8, name_len: u32, ptr: *const i32) void;
```
{{#endtab}}

{{#endtabs}}

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut ENEMY_COUNT: i32 = 5;

fn init() {
    unsafe {
        debug_register_i32(b"Enemy Count".as_ptr(), 11, &ENEMY_COUNT);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static int32_t enemy_count = 5;

NCZX_EXPORT void init(void) {
    debug_register_i32("Enemy Count", 11, &enemy_count);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var enemy_count: i32 = 5;

export fn init() void {
    debug_register_i32("Enemy Count", 11, &enemy_count);
}
```
{{#endtab}}

{{#endtabs}}

---

### debug_register_f32

Registers an editable 32-bit float.

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn debug_register_f32(name_ptr: *const u8, name_len: u32, ptr: *const f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void debug_register_f32(const uint8_t* name_ptr, uint32_t name_len, const float* ptr);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn debug_register_f32(name_ptr: [*]const u8, name_len: u32, ptr: *const f32) void;
```
{{#endtab}}

{{#endtabs}}

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut GRAVITY: f32 = 9.8;
static mut JUMP_FORCE: f32 = 15.0;

fn init() {
    unsafe {
        debug_register_f32(b"Gravity".as_ptr(), 7, &GRAVITY);
        debug_register_f32(b"Jump Force".as_ptr(), 10, &JUMP_FORCE);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static float gravity = 9.8f;
static float jump_force = 15.0f;

NCZX_EXPORT void init(void) {
    debug_register_f32("Gravity", 7, &gravity);
    debug_register_f32("Jump Force", 10, &jump_force);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var gravity: f32 = 9.8;
var jump_force: f32 = 15.0;

export fn init() void {
    debug_register_f32("Gravity", 7, &gravity);
    debug_register_f32("Jump Force", 10, &jump_force);
}
```
{{#endtab}}

{{#endtabs}}

---

### debug_register_bool

Registers an editable boolean (checkbox).

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn debug_register_bool(name_ptr: *const u8, name_len: u32, ptr: *const u8)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void debug_register_bool(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn debug_register_bool(name_ptr: [*]const u8, name_len: u32, ptr: *const u8) void;
```
{{#endtab}}

{{#endtabs}}

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut GOD_MODE: u8 = 0; // 0 = false, 1 = true

fn init() {
    unsafe {
        debug_register_bool(b"God Mode".as_ptr(), 8, &GOD_MODE);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint8_t god_mode = 0; // 0 = false, 1 = true

NCZX_EXPORT void init(void) {
    debug_register_bool("God Mode", 8, &god_mode);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var god_mode: u8 = 0; // 0 = false, 1 = true

export fn init() void {
    debug_register_bool("God Mode", 8, &god_mode);
}
```
{{#endtab}}

{{#endtabs}}

---

### debug_register_u8 / u16 / u32

Registers unsigned integers.

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn debug_register_u8(name_ptr: *const u8, name_len: u32, ptr: *const u8)
fn debug_register_u16(name_ptr: *const u8, name_len: u32, ptr: *const u16)
fn debug_register_u32(name_ptr: *const u8, name_len: u32, ptr: *const u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void debug_register_u8(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);
NCZX_IMPORT void debug_register_u16(const uint8_t* name_ptr, uint32_t name_len, const uint16_t* ptr);
NCZX_IMPORT void debug_register_u32(const uint8_t* name_ptr, uint32_t name_len, const uint32_t* ptr);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn debug_register_u8(name_ptr: [*]const u8, name_len: u32, ptr: *const u8) void;
pub extern fn debug_register_u16(name_ptr: [*]const u8, name_len: u32, ptr: *const u16) void;
pub extern fn debug_register_u32(name_ptr: [*]const u8, name_len: u32, ptr: *const u32) void;
```
{{#endtab}}

{{#endtabs}}

---

### debug_register_i8 / i16

Registers signed integers.

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn debug_register_i8(name_ptr: *const u8, name_len: u32, ptr: *const i8)
fn debug_register_i16(name_ptr: *const u8, name_len: u32, ptr: *const i16)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void debug_register_i8(const uint8_t* name_ptr, uint32_t name_len, const int8_t* ptr);
NCZX_IMPORT void debug_register_i16(const uint8_t* name_ptr, uint32_t name_len, const int16_t* ptr);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn debug_register_i8(name_ptr: [*]const u8, name_len: u32, ptr: *const i8) void;
pub extern fn debug_register_i16(name_ptr: [*]const u8, name_len: u32, ptr: *const i16) void;
```
{{#endtab}}

{{#endtabs}}

---

## Range-Constrained Registration

### debug_register_i32_range

Registers an integer with min/max bounds (slider).

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn debug_register_i32_range(
    name_ptr: *const u8, name_len: u32,
    ptr: *const i32,
    min: i32, max: i32
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void debug_register_i32_range(
    const uint8_t* name_ptr, uint32_t name_len,
    const int32_t* ptr,
    int32_t min, int32_t max
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn debug_register_i32_range(
    name_ptr: [*]const u8, name_len: u32,
    ptr: *const i32,
    min: i32, max: i32
) void;
```
{{#endtab}}

{{#endtabs}}

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut DIFFICULTY: i32 = 2;

fn init() {
    unsafe {
        debug_register_i32_range(b"Difficulty".as_ptr(), 10, &DIFFICULTY, 1, 5);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static int32_t difficulty = 2;

NCZX_EXPORT void init(void) {
    debug_register_i32_range("Difficulty", 10, &difficulty, 1, 5);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var difficulty: i32 = 2;

export fn init() void {
    debug_register_i32_range("Difficulty", 10, &difficulty, 1, 5);
}
```
{{#endtab}}

{{#endtabs}}

---

### debug_register_f32_range

Registers a float with min/max bounds.

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn debug_register_f32_range(
    name_ptr: *const u8, name_len: u32,
    ptr: *const f32,
    min: f32, max: f32
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void debug_register_f32_range(
    const uint8_t* name_ptr, uint32_t name_len,
    const float* ptr,
    float min, float max
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn debug_register_f32_range(
    name_ptr: [*]const u8, name_len: u32,
    ptr: *const f32,
    min: f32, max: f32
) void;
```
{{#endtab}}

{{#endtabs}}

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut PLAYER_SPEED: f32 = 5.0;

fn init() {
    unsafe {
        debug_register_f32_range(b"Speed".as_ptr(), 5, &PLAYER_SPEED, 0.0, 20.0);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static float player_speed = 5.0f;

NCZX_EXPORT void init(void) {
    debug_register_f32_range("Speed", 5, &player_speed, 0.0f, 20.0f);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var player_speed: f32 = 5.0;

export fn init() void {
    debug_register_f32_range("Speed", 5, &player_speed, 0.0, 20.0);
}
```
{{#endtab}}

{{#endtabs}}

---

### debug_register_u8_range / u16_range / i16_range

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn debug_register_u8_range(name_ptr: *const u8, name_len: u32, ptr: *const u8, min: u32, max: u32)
fn debug_register_u16_range(name_ptr: *const u8, name_len: u32, ptr: *const u16, min: u32, max: u32)
fn debug_register_i16_range(name_ptr: *const u8, name_len: u32, ptr: *const i16, min: i32, max: i32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void debug_register_u8_range(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr, uint32_t min, uint32_t max);
NCZX_IMPORT void debug_register_u16_range(const uint8_t* name_ptr, uint32_t name_len, const uint16_t* ptr, uint32_t min, uint32_t max);
NCZX_IMPORT void debug_register_i16_range(const uint8_t* name_ptr, uint32_t name_len, const int16_t* ptr, int32_t min, int32_t max);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn debug_register_u8_range(name_ptr: [*]const u8, name_len: u32, ptr: *const u8, min: u32, max: u32) void;
pub extern fn debug_register_u16_range(name_ptr: [*]const u8, name_len: u32, ptr: *const u16, min: u32, max: u32) void;
pub extern fn debug_register_i16_range(name_ptr: [*]const u8, name_len: u32, ptr: *const i16, min: i32, max: i32) void;
```
{{#endtab}}

{{#endtabs}}

---

## Compound Types

### debug_register_vec2

Registers a 2D vector (two f32s).

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn debug_register_vec2(name_ptr: *const u8, name_len: u32, ptr: *const f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void debug_register_vec2(const uint8_t* name_ptr, uint32_t name_len, const float* ptr);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn debug_register_vec2(name_ptr: [*]const u8, name_len: u32, ptr: *const f32) void;
```
{{#endtab}}

{{#endtabs}}

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut PLAYER_POS: [f32; 2] = [0.0, 0.0];

fn init() {
    unsafe {
        debug_register_vec2(b"Player Pos".as_ptr(), 10, PLAYER_POS.as_ptr());
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static float player_pos[2] = {0.0f, 0.0f};

NCZX_EXPORT void init(void) {
    debug_register_vec2("Player Pos", 10, player_pos);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var player_pos: [2]f32 = .{0.0, 0.0};

export fn init() void {
    debug_register_vec2("Player Pos", 10, &player_pos);
}
```
{{#endtab}}

{{#endtabs}}

---

### debug_register_vec3

Registers a 3D vector (three f32s).

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn debug_register_vec3(name_ptr: *const u8, name_len: u32, ptr: *const f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void debug_register_vec3(const uint8_t* name_ptr, uint32_t name_len, const float* ptr);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn debug_register_vec3(name_ptr: [*]const u8, name_len: u32, ptr: *const f32) void;
```
{{#endtab}}

{{#endtabs}}

---

### debug_register_rect

Registers a rectangle (x, y, width, height as four f32s).

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn debug_register_rect(name_ptr: *const u8, name_len: u32, ptr: *const f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void debug_register_rect(const uint8_t* name_ptr, uint32_t name_len, const float* ptr);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn debug_register_rect(name_ptr: [*]const u8, name_len: u32, ptr: *const f32) void;
```
{{#endtab}}

{{#endtabs}}

---

### debug_register_color

Registers a color (RGBA as four u8s).

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn debug_register_color(name_ptr: *const u8, name_len: u32, ptr: *const u8)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void debug_register_color(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn debug_register_color(name_ptr: [*]const u8, name_len: u32, ptr: *const u8) void;
```
{{#endtab}}

{{#endtabs}}

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut TINT_COLOR: [u8; 4] = [255, 255, 255, 255];

fn init() {
    unsafe {
        debug_register_color(b"Tint".as_ptr(), 4, TINT_COLOR.as_ptr());
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint8_t tint_color[4] = {255, 255, 255, 255};

NCZX_EXPORT void init(void) {
    debug_register_color("Tint", 4, tint_color);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var tint_color: [4]u8 = .{255, 255, 255, 255};

export fn init() void {
    debug_register_color("Tint", 4, &tint_color);
}
```
{{#endtab}}

{{#endtabs}}

---

## Fixed-Point Registration

For games using fixed-point math.

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn debug_register_fixed_i16_q8(name_ptr: *const u8, name_len: u32, ptr: *const i16)
fn debug_register_fixed_i32_q8(name_ptr: *const u8, name_len: u32, ptr: *const i32)
fn debug_register_fixed_i32_q16(name_ptr: *const u8, name_len: u32, ptr: *const i32)
fn debug_register_fixed_i32_q24(name_ptr: *const u8, name_len: u32, ptr: *const i32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void debug_register_fixed_i16_q8(const uint8_t* name_ptr, uint32_t name_len, const int16_t* ptr);
NCZX_IMPORT void debug_register_fixed_i32_q8(const uint8_t* name_ptr, uint32_t name_len, const int32_t* ptr);
NCZX_IMPORT void debug_register_fixed_i32_q16(const uint8_t* name_ptr, uint32_t name_len, const int32_t* ptr);
NCZX_IMPORT void debug_register_fixed_i32_q24(const uint8_t* name_ptr, uint32_t name_len, const int32_t* ptr);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn debug_register_fixed_i16_q8(name_ptr: [*]const u8, name_len: u32, ptr: *const i16) void;
pub extern fn debug_register_fixed_i32_q8(name_ptr: [*]const u8, name_len: u32, ptr: *const i32) void;
pub extern fn debug_register_fixed_i32_q16(name_ptr: [*]const u8, name_len: u32, ptr: *const i32) void;
pub extern fn debug_register_fixed_i32_q24(name_ptr: [*]const u8, name_len: u32, ptr: *const i32) void;
```
{{#endtab}}

{{#endtabs}}

---

## Watch Functions (Read-Only)

Watches display values without allowing editing.

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn debug_watch_i8(name_ptr: *const u8, name_len: u32, ptr: *const i8)
fn debug_watch_i16(name_ptr: *const u8, name_len: u32, ptr: *const i16)
fn debug_watch_i32(name_ptr: *const u8, name_len: u32, ptr: *const i32)
fn debug_watch_u8(name_ptr: *const u8, name_len: u32, ptr: *const u8)
fn debug_watch_u16(name_ptr: *const u8, name_len: u32, ptr: *const u16)
fn debug_watch_u32(name_ptr: *const u8, name_len: u32, ptr: *const u32)
fn debug_watch_f32(name_ptr: *const u8, name_len: u32, ptr: *const f32)
fn debug_watch_bool(name_ptr: *const u8, name_len: u32, ptr: *const u8)
fn debug_watch_vec2(name_ptr: *const u8, name_len: u32, ptr: *const f32)
fn debug_watch_vec3(name_ptr: *const u8, name_len: u32, ptr: *const f32)
fn debug_watch_rect(name_ptr: *const u8, name_len: u32, ptr: *const f32)
fn debug_watch_color(name_ptr: *const u8, name_len: u32, ptr: *const u8)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void debug_watch_i8(const uint8_t* name_ptr, uint32_t name_len, const int8_t* ptr);
NCZX_IMPORT void debug_watch_i16(const uint8_t* name_ptr, uint32_t name_len, const int16_t* ptr);
NCZX_IMPORT void debug_watch_i32(const uint8_t* name_ptr, uint32_t name_len, const int32_t* ptr);
NCZX_IMPORT void debug_watch_u8(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);
NCZX_IMPORT void debug_watch_u16(const uint8_t* name_ptr, uint32_t name_len, const uint16_t* ptr);
NCZX_IMPORT void debug_watch_u32(const uint8_t* name_ptr, uint32_t name_len, const uint32_t* ptr);
NCZX_IMPORT void debug_watch_f32(const uint8_t* name_ptr, uint32_t name_len, const float* ptr);
NCZX_IMPORT void debug_watch_bool(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);
NCZX_IMPORT void debug_watch_vec2(const uint8_t* name_ptr, uint32_t name_len, const float* ptr);
NCZX_IMPORT void debug_watch_vec3(const uint8_t* name_ptr, uint32_t name_len, const float* ptr);
NCZX_IMPORT void debug_watch_rect(const uint8_t* name_ptr, uint32_t name_len, const float* ptr);
NCZX_IMPORT void debug_watch_color(const uint8_t* name_ptr, uint32_t name_len, const uint8_t* ptr);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn debug_watch_i8(name_ptr: [*]const u8, name_len: u32, ptr: *const i8) void;
pub extern fn debug_watch_i16(name_ptr: [*]const u8, name_len: u32, ptr: *const i16) void;
pub extern fn debug_watch_i32(name_ptr: [*]const u8, name_len: u32, ptr: *const i32) void;
pub extern fn debug_watch_u8(name_ptr: [*]const u8, name_len: u32, ptr: *const u8) void;
pub extern fn debug_watch_u16(name_ptr: [*]const u8, name_len: u32, ptr: *const u16) void;
pub extern fn debug_watch_u32(name_ptr: [*]const u8, name_len: u32, ptr: *const u32) void;
pub extern fn debug_watch_f32(name_ptr: [*]const u8, name_len: u32, ptr: *const f32) void;
pub extern fn debug_watch_bool(name_ptr: [*]const u8, name_len: u32, ptr: *const u8) void;
pub extern fn debug_watch_vec2(name_ptr: [*]const u8, name_len: u32, ptr: *const f32) void;
pub extern fn debug_watch_vec3(name_ptr: [*]const u8, name_len: u32, ptr: *const f32) void;
pub extern fn debug_watch_rect(name_ptr: [*]const u8, name_len: u32, ptr: *const f32) void;
pub extern fn debug_watch_color(name_ptr: [*]const u8, name_len: u32, ptr: *const u8) void;
```
{{#endtab}}

{{#endtabs}}

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut FRAME_COUNT: u32 = 0;
static mut FPS: f32 = 0.0;

fn init() {
    unsafe {
        debug_watch_u32(b"Frame".as_ptr(), 5, &FRAME_COUNT);
        debug_watch_f32(b"FPS".as_ptr(), 3, &FPS);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t frame_count = 0;
static float fps = 0.0f;

NCZX_EXPORT void init(void) {
    debug_watch_u32("Frame", 5, &frame_count);
    debug_watch_f32("FPS", 3, &fps);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var frame_count: u32 = 0;
var fps: f32 = 0.0;

export fn init() void {
    debug_watch_u32("Frame", 5, &frame_count);
    debug_watch_f32("FPS", 3, &fps);
}
```
{{#endtab}}

{{#endtabs}}

---

## Grouping

### debug_group_begin

Starts a collapsible group in the debug panel.

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn debug_group_begin(name_ptr: *const u8, name_len: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void debug_group_begin(const uint8_t* name_ptr, uint32_t name_len);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn debug_group_begin(name_ptr: [*]const u8, name_len: u32) void;
```
{{#endtab}}

{{#endtabs}}

---

### debug_group_end

Ends the current group.

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn debug_group_end()
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void debug_group_end(void);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn debug_group_end() void;
```
{{#endtab}}

{{#endtabs}}

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn init() {
    unsafe {
        debug_group_begin(b"Player".as_ptr(), 6);
        debug_register_vec3(b"Position".as_ptr(), 8, PLAYER_POS.as_ptr());
        debug_register_f32(b"Health".as_ptr(), 6, &PLAYER_HEALTH);
        debug_register_f32(b"Speed".as_ptr(), 5, &PLAYER_SPEED);
        debug_group_end();

        debug_group_begin(b"Physics".as_ptr(), 7);
        debug_register_f32(b"Gravity".as_ptr(), 7, &GRAVITY);
        debug_register_f32(b"Friction".as_ptr(), 8, &FRICTION);
        debug_group_end();
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void init(void) {
    debug_group_begin("Player", 6);
    debug_register_vec3("Position", 8, player_pos);
    debug_register_f32("Health", 6, &player_health);
    debug_register_f32("Speed", 5, &player_speed);
    debug_group_end();

    debug_group_begin("Physics", 7);
    debug_register_f32("Gravity", 7, &gravity);
    debug_register_f32("Friction", 8, &friction);
    debug_group_end();
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn init() void {
    debug_group_begin("Player", 6);
    debug_register_vec3("Position", 8, &player_pos);
    debug_register_f32("Health", 6, &player_health);
    debug_register_f32("Speed", 5, &player_speed);
    debug_group_end();

    debug_group_begin("Physics", 7);
    debug_register_f32("Gravity", 7, &gravity);
    debug_register_f32("Friction", 8, &friction);
    debug_group_end();
}
```
{{#endtab}}

{{#endtabs}}

---

## Frame Control

### debug_is_paused

Check if game is paused via debug panel.

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn debug_is_paused() -> i32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT int32_t debug_is_paused(void);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn debug_is_paused() i32;
```
{{#endtab}}

{{#endtabs}}

**Returns:** 1 if paused, 0 otherwise

---

### debug_get_time_scale

Get the current time scale.

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn debug_get_time_scale() -> f32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT float debug_get_time_scale(void);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn debug_get_time_scale() f32;
```
{{#endtab}}

{{#endtabs}}

**Returns:** Time scale (1.0 = normal, 0.5 = half speed, 2.0 = double)

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn update() {
    unsafe {
        if debug_is_paused() != 0 {
            return; // Skip update when paused
        }

        let dt = delta_time() * debug_get_time_scale();
        // Use scaled delta time
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void update(void) {
    if (debug_is_paused() != 0) {
        return; // Skip update when paused
    }

    float dt = delta_time() * debug_get_time_scale();
    // Use scaled delta time
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    if (debug_is_paused() != 0) {
        return; // Skip update when paused
    }

    const dt = delta_time() * debug_get_time_scale();
    // Use scaled delta time
}
```
{{#endtab}}

{{#endtabs}}

---

## Debug Keyboard Shortcuts

| Key | Action |
|-----|--------|
| F3 | Toggle debug panel |
| F5 | Pause/unpause |
| F6 | Step one frame (while paused) |
| F7 | Decrease time scale |
| F8 | Increase time scale |

---

## Complete Example

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// Game state
static mut PLAYER_X: f32 = 0.0;
static mut PLAYER_Y: f32 = 0.0;
static mut PLAYER_VEL_X: f32 = 0.0;
static mut PLAYER_VEL_Y: f32 = 0.0;
static mut PLAYER_HEALTH: f32 = 100.0;

// Tuning parameters
static mut MOVE_SPEED: f32 = 5.0;
static mut JUMP_FORCE: f32 = 12.0;
static mut GRAVITY: f32 = 25.0;
static mut FRICTION: f32 = 0.9;

// Debug
static mut GOD_MODE: u8 = 0;
static mut SHOW_HITBOXES: u8 = 0;
static mut ENEMY_COUNT: i32 = 5;

fn init() {
    unsafe {
        // Player group
        debug_group_begin(b"Player".as_ptr(), 6);
        debug_watch_f32(b"X".as_ptr(), 1, &PLAYER_X);
        debug_watch_f32(b"Y".as_ptr(), 1, &PLAYER_Y);
        debug_watch_f32(b"Vel X".as_ptr(), 5, &PLAYER_VEL_X);
        debug_watch_f32(b"Vel Y".as_ptr(), 5, &PLAYER_VEL_Y);
        debug_register_f32_range(b"Health".as_ptr(), 6, &PLAYER_HEALTH, 0.0, 100.0);
        debug_group_end();

        // Physics group
        debug_group_begin(b"Physics".as_ptr(), 7);
        debug_register_f32_range(b"Move Speed".as_ptr(), 10, &MOVE_SPEED, 1.0, 20.0);
        debug_register_f32_range(b"Jump Force".as_ptr(), 10, &JUMP_FORCE, 5.0, 30.0);
        debug_register_f32_range(b"Gravity".as_ptr(), 7, &GRAVITY, 10.0, 50.0);
        debug_register_f32_range(b"Friction".as_ptr(), 8, &FRICTION, 0.5, 1.0);
        debug_group_end();

        // Debug options
        debug_group_begin(b"Debug".as_ptr(), 5);
        debug_register_bool(b"God Mode".as_ptr(), 8, &GOD_MODE);
        debug_register_bool(b"Show Hitboxes".as_ptr(), 13, &SHOW_HITBOXES);
        debug_register_i32_range(b"Enemy Count".as_ptr(), 11, &ENEMY_COUNT, 0, 20);
        debug_group_end();
    }
}

fn update() {
    unsafe {
        // Respect debug pause
        if debug_is_paused() != 0 {
            return;
        }

        let dt = delta_time() * debug_get_time_scale();

        // Use tunable values
        PLAYER_VEL_Y += GRAVITY * dt;
        PLAYER_VEL_X *= FRICTION;

        if button_held(0, BUTTON_RIGHT) != 0 {
            PLAYER_VEL_X = MOVE_SPEED;
        }
        if button_held(0, BUTTON_LEFT) != 0 {
            PLAYER_VEL_X = -MOVE_SPEED;
        }
        if button_pressed(0, BUTTON_A) != 0 {
            PLAYER_VEL_Y = -JUMP_FORCE;
        }

        PLAYER_X += PLAYER_VEL_X * dt;
        PLAYER_Y += PLAYER_VEL_Y * dt;
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Game state
static float player_x = 0.0f;
static float player_y = 0.0f;
static float player_vel_x = 0.0f;
static float player_vel_y = 0.0f;
static float player_health = 100.0f;

// Tuning parameters
static float move_speed = 5.0f;
static float jump_force = 12.0f;
static float gravity = 25.0f;
static float friction = 0.9f;

// Debug
static uint8_t god_mode = 0;
static uint8_t show_hitboxes = 0;
static int32_t enemy_count = 5;

NCZX_EXPORT void init(void) {
    // Player group
    debug_group_begin("Player", 6);
    debug_watch_f32("X", 1, &player_x);
    debug_watch_f32("Y", 1, &player_y);
    debug_watch_f32("Vel X", 5, &player_vel_x);
    debug_watch_f32("Vel Y", 5, &player_vel_y);
    debug_register_f32_range("Health", 6, &player_health, 0.0f, 100.0f);
    debug_group_end();

    // Physics group
    debug_group_begin("Physics", 7);
    debug_register_f32_range("Move Speed", 10, &move_speed, 1.0f, 20.0f);
    debug_register_f32_range("Jump Force", 10, &jump_force, 5.0f, 30.0f);
    debug_register_f32_range("Gravity", 7, &gravity, 10.0f, 50.0f);
    debug_register_f32_range("Friction", 8, &friction, 0.5f, 1.0f);
    debug_group_end();

    // Debug options
    debug_group_begin("Debug", 5);
    debug_register_bool("God Mode", 8, &god_mode);
    debug_register_bool("Show Hitboxes", 13, &show_hitboxes);
    debug_register_i32_range("Enemy Count", 11, &enemy_count, 0, 20);
    debug_group_end();
}

NCZX_EXPORT void update(void) {
    // Respect debug pause
    if (debug_is_paused() != 0) {
        return;
    }

    float dt = delta_time() * debug_get_time_scale();

    // Use tunable values
    player_vel_y += gravity * dt;
    player_vel_x *= friction;

    if (button_held(0, NCZX_BUTTON_RIGHT) != 0) {
        player_vel_x = move_speed;
    }
    if (button_held(0, NCZX_BUTTON_LEFT) != 0) {
        player_vel_x = -move_speed;
    }
    if (button_pressed(0, NCZX_BUTTON_A) != 0) {
        player_vel_y = -jump_force;
    }

    player_x += player_vel_x * dt;
    player_y += player_vel_y * dt;
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Game state
var player_x: f32 = 0.0;
var player_y: f32 = 0.0;
var player_vel_x: f32 = 0.0;
var player_vel_y: f32 = 0.0;
var player_health: f32 = 100.0;

// Tuning parameters
var move_speed: f32 = 5.0;
var jump_force: f32 = 12.0;
var gravity: f32 = 25.0;
var friction: f32 = 0.9;

// Debug
var god_mode: u8 = 0;
var show_hitboxes: u8 = 0;
var enemy_count: i32 = 5;

export fn init() void {
    // Player group
    debug_group_begin("Player", 6);
    debug_watch_f32("X", 1, &player_x);
    debug_watch_f32("Y", 1, &player_y);
    debug_watch_f32("Vel X", 5, &player_vel_x);
    debug_watch_f32("Vel Y", 5, &player_vel_y);
    debug_register_f32_range("Health", 6, &player_health, 0.0, 100.0);
    debug_group_end();

    // Physics group
    debug_group_begin("Physics", 7);
    debug_register_f32_range("Move Speed", 10, &move_speed, 1.0, 20.0);
    debug_register_f32_range("Jump Force", 10, &jump_force, 5.0, 30.0);
    debug_register_f32_range("Gravity", 7, &gravity, 10.0, 50.0);
    debug_register_f32_range("Friction", 8, &friction, 0.5, 1.0);
    debug_group_end();

    // Debug options
    debug_group_begin("Debug", 5);
    debug_register_bool("God Mode", 8, &god_mode);
    debug_register_bool("Show Hitboxes", 13, &show_hitboxes);
    debug_register_i32_range("Enemy Count", 11, &enemy_count, 0, 20);
    debug_group_end();
}

export fn update() void {
    // Respect debug pause
    if (debug_is_paused() != 0) {
        return;
    }

    const dt = delta_time() * debug_get_time_scale();

    // Use tunable values
    player_vel_y += gravity * dt;
    player_vel_x *= friction;

    if (button_held(0, Button.right) != 0) {
        player_vel_x = move_speed;
    }
    if (button_held(0, Button.left) != 0) {
        player_vel_x = -move_speed;
    }
    if (button_pressed(0, Button.a) != 0) {
        player_vel_y = -jump_force;
    }

    player_x += player_vel_x * dt;
    player_y += player_vel_y * dt;
}
```
{{#endtab}}

{{#endtabs}}

**See Also:** [System Functions](./system.md)
