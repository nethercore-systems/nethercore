# Keyframe Animation Functions

GPU-optimized keyframe animation system for skeletal animation.

## Loading Keyframes

### keyframes_load

Loads keyframes from WASM memory.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn keyframes_load(data_ptr: *const u8, byte_size: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t keyframes_load(const uint8_t* data_ptr, uint32_t byte_size);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn keyframes_load(data_ptr: [*]const u8, byte_size: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| data_ptr | `*const u8` | Pointer to keyframe data |
| byte_size | `u32` | Size of data in bytes |

**Returns:** Keyframe collection handle (non-zero on success)

**Constraints:** Init-only.

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static WALK_DATA: &[u8] = include_bytes!("walk.nczxanim");
static mut WALK_ANIM: u32 = 0;

fn init() {
    unsafe {
        WALK_ANIM = keyframes_load(WALK_DATA.as_ptr(), WALK_DATA.len() as u32);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static const uint8_t WALK_DATA[] = { /* embedded data */ };
static uint32_t WALK_ANIM = 0;

NCZX_EXPORT void init(void) {
    WALK_ANIM = keyframes_load(WALK_DATA, sizeof(WALK_DATA));
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const WALK_DATA = @embedFile("walk.nczxanim");
var WALK_ANIM: u32 = 0;

export fn init() void {
    WALK_ANIM = keyframes_load(WALK_DATA.ptr, WALK_DATA.len);
}
```
{{#endtab}}

{{#endtabs}}

---

### rom_keyframes

Loads keyframes from ROM data pack.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn rom_keyframes(id_ptr: *const u8, id_len: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t rom_keyframes(const uint8_t* id_ptr, uint32_t id_len);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn rom_keyframes(id_ptr: [*]const u8, id_len: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| id_ptr | `*const u8` | Pointer to asset ID string |
| id_len | `u32` | Length of asset ID |

**Returns:** Keyframe collection handle (non-zero on success)

**Constraints:** Init-only.

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut WALK_ANIM: u32 = 0;
static mut IDLE_ANIM: u32 = 0;
static mut ATTACK_ANIM: u32 = 0;

fn init() {
    unsafe {
        WALK_ANIM = rom_keyframes(b"walk".as_ptr(), 4);
        IDLE_ANIM = rom_keyframes(b"idle".as_ptr(), 4);
        ATTACK_ANIM = rom_keyframes(b"attack".as_ptr(), 6);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t WALK_ANIM = 0;
static uint32_t IDLE_ANIM = 0;
static uint32_t ATTACK_ANIM = 0;

NCZX_EXPORT void init(void) {
    WALK_ANIM = rom_keyframes((const uint8_t*)"walk", 4);
    IDLE_ANIM = rom_keyframes((const uint8_t*)"idle", 4);
    ATTACK_ANIM = rom_keyframes((const uint8_t*)"attack", 6);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var WALK_ANIM: u32 = 0;
var IDLE_ANIM: u32 = 0;
var ATTACK_ANIM: u32 = 0;

export fn init() void {
    WALK_ANIM = rom_keyframes("walk", 4);
    IDLE_ANIM = rom_keyframes("idle", 4);
    ATTACK_ANIM = rom_keyframes("attack", 6);
}
```
{{#endtab}}

{{#endtabs}}

---

## Querying Keyframes

### keyframes_bone_count

Gets the bone count for a keyframe collection.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn keyframes_bone_count(handle: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t keyframes_bone_count(uint32_t handle);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn keyframes_bone_count(handle: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Returns:** Number of bones in the animation

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn init() {
    unsafe {
        WALK_ANIM = rom_keyframes(b"walk".as_ptr(), 4);
        let bones = keyframes_bone_count(WALK_ANIM);
        log_fmt(b"Walk animation has {} bones", bones);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void init(void) {
    WALK_ANIM = rom_keyframes((const uint8_t*)"walk", 4);
    uint32_t bones = keyframes_bone_count(WALK_ANIM);
    log_fmt((const uint8_t*)"Walk animation has {} bones", bones);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn init() void {
    WALK_ANIM = rom_keyframes("walk", 4);
    const bones = keyframes_bone_count(WALK_ANIM);
    log_fmt("Walk animation has {} bones", bones);
}
```
{{#endtab}}

{{#endtabs}}

---

### keyframes_frame_count

Gets the frame count for a keyframe collection.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn keyframes_frame_count(handle: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t keyframes_frame_count(uint32_t handle);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn keyframes_frame_count(handle: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Returns:** Number of frames in the animation

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    unsafe {
        let frame_count = keyframes_frame_count(WALK_ANIM);
        let current_frame = (ANIM_TIME as u32) % frame_count;
        keyframe_bind(WALK_ANIM, current_frame);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
    uint32_t frame_count = keyframes_frame_count(WALK_ANIM);
    uint32_t current_frame = ((uint32_t)ANIM_TIME) % frame_count;
    keyframe_bind(WALK_ANIM, current_frame);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    const frame_count = keyframes_frame_count(WALK_ANIM);
    const current_frame = @as(u32, @intFromFloat(ANIM_TIME)) % frame_count;
    keyframe_bind(WALK_ANIM, current_frame);
}
```
{{#endtab}}

{{#endtabs}}

---

## Using Keyframes

### keyframe_bind

Binds a keyframe directly from GPU buffer (zero CPU overhead).

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn keyframe_bind(handle: u32, index: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void keyframe_bind(uint32_t handle, uint32_t index);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn keyframe_bind(handle: u32, index: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| handle | `u32` | Keyframe collection handle |
| index | `u32` | Frame index (0 to frame_count-1) |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut ANIM_FRAME: f32 = 0.0;

fn update() {
    unsafe {
        ANIM_FRAME += delta_time() * 30.0; // 30 FPS animation
    }
}

fn render() {
    unsafe {
        let frame_count = keyframes_frame_count(WALK_ANIM);
        let frame = (ANIM_FRAME as u32) % frame_count;

        // Bind frame - GPU reads directly, no CPU decode!
        keyframe_bind(WALK_ANIM, frame);
        draw_mesh(CHARACTER_MESH);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static float ANIM_FRAME = 0.0f;

NCZX_EXPORT void update(void) {
    ANIM_FRAME += delta_time() * 30.0f; // 30 FPS animation
}

NCZX_EXPORT void render(void) {
    uint32_t frame_count = keyframes_frame_count(WALK_ANIM);
    uint32_t frame = ((uint32_t)ANIM_FRAME) % frame_count;

    // Bind frame - GPU reads directly, no CPU decode!
    keyframe_bind(WALK_ANIM, frame);
    draw_mesh(CHARACTER_MESH);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var ANIM_FRAME: f32 = 0.0;

export fn update() void {
    ANIM_FRAME += delta_time() * 30.0; // 30 FPS animation
}

export fn render() void {
    const frame_count = keyframes_frame_count(WALK_ANIM);
    const frame = @as(u32, @intFromFloat(ANIM_FRAME)) % frame_count;

    // Bind frame - GPU reads directly, no CPU decode!
    keyframe_bind(WALK_ANIM, frame);
    draw_mesh(CHARACTER_MESH);
}
```
{{#endtab}}

{{#endtabs}}

---

### keyframe_read

Reads a keyframe to WASM memory for CPU-side blending.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn keyframe_read(handle: u32, index: u32, out_ptr: *mut u8)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void keyframe_read(uint32_t handle, uint32_t index, uint8_t* out_ptr);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn keyframe_read(handle: u32, index: u32, out_ptr: [*]u8) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| handle | `u32` | Keyframe collection handle |
| index | `u32` | Frame index |
| out_ptr | `*mut u8` | Destination buffer (must be large enough for all bone matrices) |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    unsafe {
        let frame_count = keyframes_frame_count(WALK_ANIM);
        let frame_a = (ANIM_TIME as u32) % frame_count;
        let frame_b = (frame_a + 1) % frame_count;
        let blend = ANIM_TIME.fract();

        // Read frames for interpolation
        let mut buf_a = [0u8; 64 * 12 * 4]; // 64 bones × 12 floats × 4 bytes
        let mut buf_b = [0u8; 64 * 12 * 4];

        keyframe_read(WALK_ANIM, frame_a, buf_a.as_mut_ptr());
        keyframe_read(WALK_ANIM, frame_b, buf_b.as_mut_ptr());

        // Interpolate on CPU
        let blended = interpolate_bones(&buf_a, &buf_b, blend);

        // Upload blended result
        set_bones(blended.as_ptr(), bone_count);
        draw_mesh(CHARACTER_MESH);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
    uint32_t frame_count = keyframes_frame_count(WALK_ANIM);
    uint32_t frame_a = ((uint32_t)ANIM_TIME) % frame_count;
    uint32_t frame_b = (frame_a + 1) % frame_count;
    float blend = ANIM_TIME - (float)((uint32_t)ANIM_TIME);

    // Read frames for interpolation
    uint8_t buf_a[64 * 12 * 4]; // 64 bones × 12 floats × 4 bytes
    uint8_t buf_b[64 * 12 * 4];

    keyframe_read(WALK_ANIM, frame_a, buf_a);
    keyframe_read(WALK_ANIM, frame_b, buf_b);

    // Interpolate on CPU
    uint8_t blended[64 * 12 * 4];
    interpolate_bones(buf_a, buf_b, blend, blended);

    // Upload blended result
    set_bones(blended, bone_count);
    draw_mesh(CHARACTER_MESH);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    const frame_count = keyframes_frame_count(WALK_ANIM);
    const frame_a = @as(u32, @intFromFloat(ANIM_TIME)) % frame_count;
    const frame_b = (frame_a + 1) % frame_count;
    const blend = ANIM_TIME - @floor(ANIM_TIME);

    // Read frames for interpolation
    var buf_a: [64 * 12 * 4]u8 = undefined; // 64 bones × 12 floats × 4 bytes
    var buf_b: [64 * 12 * 4]u8 = undefined;

    keyframe_read(WALK_ANIM, frame_a, &buf_a);
    keyframe_read(WALK_ANIM, frame_b, &buf_b);

    // Interpolate on CPU
    var blended: [64 * 12 * 4]u8 = undefined;
    interpolate_bones(&buf_a, &buf_b, blend, &blended);

    // Upload blended result
    set_bones(&blended, bone_count);
    draw_mesh(CHARACTER_MESH);
}
```
{{#endtab}}

{{#endtabs}}

---

## Animation Paths

| Path | Function | Use Case | Performance |
|------|----------|----------|-------------|
| **Static** | `keyframe_bind()` | Pre-baked ROM animations | Zero CPU work |
| **Immediate** | `set_bones()` | Procedural, IK, blended | Minimal overhead |

**Static keyframes:** Data uploaded to GPU once in `init()`. `keyframe_bind()` just sets buffer offset.

**Immediate bones:** Matrices appended to per-frame buffer, uploaded before rendering.

---

## Complete Example

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut SKELETON: u32 = 0;
static mut CHARACTER: u32 = 0;
static mut WALK_ANIM: u32 = 0;
static mut IDLE_ANIM: u32 = 0;
static mut ANIM_TIME: f32 = 0.0;
static mut IS_WALKING: bool = false;

fn init() {
    unsafe {
        SKELETON = rom_skeleton(b"player_rig".as_ptr(), 10);
        CHARACTER = rom_mesh(b"player".as_ptr(), 6);
        WALK_ANIM = rom_keyframes(b"walk".as_ptr(), 4);
        IDLE_ANIM = rom_keyframes(b"idle".as_ptr(), 4);
    }
}

fn update() {
    unsafe {
        // Check movement input
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);
        IS_WALKING = stick_x.abs() > 0.1 || stick_y.abs() > 0.1;

        // Advance animation
        let anim_speed = if IS_WALKING { 30.0 } else { 15.0 };
        ANIM_TIME += delta_time() * anim_speed;
    }
}

fn render() {
    unsafe {
        skeleton_bind(SKELETON);

        // Choose animation
        let anim = if IS_WALKING { WALK_ANIM } else { IDLE_ANIM };
        let frame_count = keyframes_frame_count(anim);
        let frame = (ANIM_TIME as u32) % frame_count;

        // Bind keyframe (GPU-side, no CPU decode)
        keyframe_bind(anim, frame);

        // Draw character
        texture_bind(player_texture);
        push_identity();
        push_translate(player_x, player_y, player_z);
        draw_mesh(CHARACTER);

        skeleton_bind(0);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t SKELETON = 0;
static uint32_t CHARACTER = 0;
static uint32_t WALK_ANIM = 0;
static uint32_t IDLE_ANIM = 0;
static float ANIM_TIME = 0.0f;
static bool IS_WALKING = false;

NCZX_EXPORT void init(void) {
    SKELETON = rom_skeleton((const uint8_t*)"player_rig", 10);
    CHARACTER = rom_mesh((const uint8_t*)"player", 6);
    WALK_ANIM = rom_keyframes((const uint8_t*)"walk", 4);
    IDLE_ANIM = rom_keyframes((const uint8_t*)"idle", 4);
}

NCZX_EXPORT void update(void) {
    // Check movement input
    float stick_x = left_stick_x(0);
    float stick_y = left_stick_y(0);
    IS_WALKING = (stick_x > 0.1f || stick_x < -0.1f) || (stick_y > 0.1f || stick_y < -0.1f);

    // Advance animation
    float anim_speed = IS_WALKING ? 30.0f : 15.0f;
    ANIM_TIME += delta_time() * anim_speed;
}

NCZX_EXPORT void render(void) {
    skeleton_bind(SKELETON);

    // Choose animation
    uint32_t anim = IS_WALKING ? WALK_ANIM : IDLE_ANIM;
    uint32_t frame_count = keyframes_frame_count(anim);
    uint32_t frame = ((uint32_t)ANIM_TIME) % frame_count;

    // Bind keyframe (GPU-side, no CPU decode)
    keyframe_bind(anim, frame);

    // Draw character
    texture_bind(player_texture);
    push_identity();
    push_translate(player_x, player_y, player_z);
    draw_mesh(CHARACTER);

    skeleton_bind(0);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var SKELETON: u32 = 0;
var CHARACTER: u32 = 0;
var WALK_ANIM: u32 = 0;
var IDLE_ANIM: u32 = 0;
var ANIM_TIME: f32 = 0.0;
var IS_WALKING: bool = false;

export fn init() void {
    SKELETON = rom_skeleton("player_rig", 10);
    CHARACTER = rom_mesh("player", 6);
    WALK_ANIM = rom_keyframes("walk", 4);
    IDLE_ANIM = rom_keyframes("idle", 4);
}

export fn update() void {
    // Check movement input
    const stick_x = left_stick_x(0);
    const stick_y = left_stick_y(0);
    IS_WALKING = @abs(stick_x) > 0.1 or @abs(stick_y) > 0.1;

    // Advance animation
    const anim_speed = if (IS_WALKING) 30.0 else 15.0;
    ANIM_TIME += delta_time() * anim_speed;
}

export fn render() void {
    skeleton_bind(SKELETON);

    // Choose animation
    const anim = if (IS_WALKING) WALK_ANIM else IDLE_ANIM;
    const frame_count = keyframes_frame_count(anim);
    const frame = @as(u32, @intFromFloat(ANIM_TIME)) % frame_count;

    // Bind keyframe (GPU-side, no CPU decode)
    keyframe_bind(anim, frame);

    // Draw character
    texture_bind(player_texture);
    push_identity();
    push_translate(player_x, player_y, player_z);
    draw_mesh(CHARACTER);

    skeleton_bind(0);
}
```
{{#endtab}}

{{#endtabs}}

**See Also:** [Skinning Functions](./skinning.md), [rom_keyframes](#rom_keyframes)
