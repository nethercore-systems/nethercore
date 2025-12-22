# Skeletal Animation Functions

GPU-based skeletal animation with bone transforms.

## Skeleton Loading

### load_skeleton

Loads inverse bind matrices for a skeleton.

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn load_skeleton(inverse_bind_ptr: *const f32, bone_count: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t load_skeleton(const float* inverse_bind_ptr, uint32_t bone_count);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn load_skeleton(inverse_bind_ptr: [*]const f32, bone_count: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| inverse_bind_ptr | `*const f32` | Pointer to 3x4 matrices (12 floats each, column-major) |
| bone_count | `u32` | Number of bones (max 256) |

**Returns:** Skeleton handle (non-zero on success)

**Constraints:** Init-only.

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut SKELETON: u32 = 0;
static INVERSE_BIND: &[u8] = include_bytes!("skeleton.nczxskel");

fn init() {
    unsafe {
        // Parse bone count from header
        let bone_count = u32::from_le_bytes([
            INVERSE_BIND[0], INVERSE_BIND[1],
            INVERSE_BIND[2], INVERSE_BIND[3]
        ]);

        // Matrix data starts after 8-byte header
        let matrices_ptr = INVERSE_BIND[8..].as_ptr() as *const f32;
        SKELETON = load_skeleton(matrices_ptr, bone_count);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t SKELETON = 0;

NCZX_EXPORT void init(void) {
    // Load skeleton binary data
    const uint8_t* inverse_bind = /* load skeleton.nczxskel */;

    // Parse bone count from header
    uint32_t bone_count = inverse_bind[0] | (inverse_bind[1] << 8) |
                         (inverse_bind[2] << 16) | (inverse_bind[3] << 24);

    // Matrix data starts after 8-byte header
    const float* matrices_ptr = (const float*)(inverse_bind + 8);
    SKELETON = load_skeleton(matrices_ptr, bone_count);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var SKELETON: u32 = 0;

export fn init() void {
    // Load skeleton binary data
    const inverse_bind = @embedFile("skeleton.nczxskel");

    // Parse bone count from header
    const bone_count = std.mem.readIntLittle(u32, inverse_bind[0..4]);

    // Matrix data starts after 8-byte header
    const matrices_ptr = @ptrCast([*]const f32, @alignCast(@alignOf(f32), inverse_bind[8..]));
    SKELETON = load_skeleton(matrices_ptr, bone_count);
}
```
{{#endtab}}

{{#endtabs}}

---

### skeleton_bind

Binds a skeleton for inverse bind mode rendering.

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn skeleton_bind(skeleton: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void skeleton_bind(uint32_t skeleton);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn skeleton_bind(skeleton: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| skeleton | `u32` | Skeleton handle, or 0 to disable inverse bind mode |

**Skinning Modes:**

| `skeleton_bind()` | `set_bones()` receives | GPU applies |
|-------------------|------------------------|-------------|
| `0` or not called | Final skinning matrices | Nothing extra |
| Valid handle | Model-space bone transforms | `bone × inverse_bind` |

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    unsafe {
        // Enable inverse bind mode
        skeleton_bind(SKELETON);

        // Upload model-space transforms (GPU applies inverse bind)
        set_bones(animation_bones.as_ptr(), bone_count);
        draw_mesh(character_mesh);

        // Disable for other meshes
        skeleton_bind(0);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
    // Enable inverse bind mode
    skeleton_bind(SKELETON);

    // Upload model-space transforms (GPU applies inverse bind)
    set_bones(animation_bones, bone_count);
    draw_mesh(character_mesh);

    // Disable for other meshes
    skeleton_bind(0);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Enable inverse bind mode
    skeleton_bind(SKELETON);

    // Upload model-space transforms (GPU applies inverse bind)
    set_bones(animation_bones.ptr, bone_count);
    draw_mesh(character_mesh);

    // Disable for other meshes
    skeleton_bind(0);
}
```
{{#endtab}}

{{#endtabs}}

---

## Bone Transforms

### set_bones

Uploads bone transforms as 3x4 matrices.

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn set_bones(matrices_ptr: *const f32, count: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void set_bones(const float* matrices_ptr, uint32_t count);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn set_bones(matrices_ptr: [*]const f32, count: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| matrices_ptr | `*const f32` | Pointer to array of 3x4 matrices (12 floats each) |
| count | `u32` | Number of bones (max 256) |

**3x4 Matrix Layout (column-major, 12 floats):**
```
[col0.x, col0.y, col0.z,   // X axis
 col1.x, col1.y, col1.z,   // Y axis
 col2.x, col2.y, col2.z,   // Z axis
 tx,     ty,     tz]       // Translation
// Implicit 4th row: [0, 0, 0, 1]
```

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut BONE_MATRICES: [f32; 64 * 12] = [0.0; 64 * 12]; // 64 bones max

fn update() {
    unsafe {
        // Update bone transforms from animation
        for i in 0..BONE_COUNT {
            let offset = i * 12;
            // Set identity with translation
            BONE_MATRICES[offset + 0] = 1.0;  // col0.x
            BONE_MATRICES[offset + 4] = 1.0;  // col1.y
            BONE_MATRICES[offset + 8] = 1.0;  // col2.z
            BONE_MATRICES[offset + 9] = bone_positions[i].x;
            BONE_MATRICES[offset + 10] = bone_positions[i].y;
            BONE_MATRICES[offset + 11] = bone_positions[i].z;
        }
    }
}

fn render() {
    unsafe {
        set_bones(BONE_MATRICES.as_ptr(), BONE_COUNT as u32);
        draw_mesh(SKINNED_MESH);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static float BONE_MATRICES[64 * 12]; // 64 bones max

NCZX_EXPORT void update(void) {
    // Update bone transforms from animation
    for (int i = 0; i < BONE_COUNT; i++) {
        int offset = i * 12;
        // Set identity with translation
        BONE_MATRICES[offset + 0] = 1.0f;  // col0.x
        BONE_MATRICES[offset + 4] = 1.0f;  // col1.y
        BONE_MATRICES[offset + 8] = 1.0f;  // col2.z
        BONE_MATRICES[offset + 9] = bone_positions[i].x;
        BONE_MATRICES[offset + 10] = bone_positions[i].y;
        BONE_MATRICES[offset + 11] = bone_positions[i].z;
    }
}

NCZX_EXPORT void render(void) {
    set_bones(BONE_MATRICES, BONE_COUNT);
    draw_mesh(SKINNED_MESH);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var BONE_MATRICES: [64 * 12]f32 = undefined; // 64 bones max

export fn update() void {
    // Update bone transforms from animation
    for (0..BONE_COUNT) |i| {
        const offset = i * 12;
        // Set identity with translation
        BONE_MATRICES[offset + 0] = 1.0;  // col0.x
        BONE_MATRICES[offset + 4] = 1.0;  // col1.y
        BONE_MATRICES[offset + 8] = 1.0;  // col2.z
        BONE_MATRICES[offset + 9] = bone_positions[i].x;
        BONE_MATRICES[offset + 10] = bone_positions[i].y;
        BONE_MATRICES[offset + 11] = bone_positions[i].z;
    }
}

export fn render() void {
    set_bones(&BONE_MATRICES, BONE_COUNT);
    draw_mesh(SKINNED_MESH);
}
```
{{#endtab}}

{{#endtabs}}

---

### set_bones_4x4

Uploads bone transforms as 4x4 matrices (converted to 3x4 internally).

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn set_bones_4x4(matrices_ptr: *const f32, count: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void set_bones_4x4(const float* matrices_ptr, uint32_t count);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn set_bones_4x4(matrices_ptr: [*]const f32, count: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| matrices_ptr | `*const f32` | Pointer to array of 4x4 matrices (16 floats each) |
| count | `u32` | Number of bones (max 256) |

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Using glam Mat4 arrays
    let mut bone_mats: [Mat4; 64] = [Mat4::IDENTITY; 64];

    // Animate bones
    for i in 0..bone_count {
        bone_mats[i] = compute_bone_transform(i);
    }

    // Upload (host converts 4x4 → 3x4)
    set_bones_4x4(bone_mats.as_ptr() as *const f32, bone_count);
    draw_mesh(skinned_mesh);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
    // Using 4x4 matrix arrays (16 floats each)
    float bone_mats[64 * 16];

    // Initialize to identity
    for (int i = 0; i < bone_count; i++) {
        identity_matrix_4x4(&bone_mats[i * 16]);
    }

    // Animate bones
    for (int i = 0; i < bone_count; i++) {
        compute_bone_transform(i, &bone_mats[i * 16]);
    }

    // Upload (host converts 4x4 → 3x4)
    set_bones_4x4(bone_mats, bone_count);
    draw_mesh(skinned_mesh);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Using 4x4 matrix arrays (16 floats each)
    var bone_mats: [64 * 16]f32 = undefined;

    // Initialize to identity
    for (0..bone_count) |i| {
        identity_matrix_4x4(bone_mats[i * 16 .. (i + 1) * 16]);
    }

    // Animate bones
    for (0..bone_count) |i| {
        compute_bone_transform(i, bone_mats[i * 16 .. (i + 1) * 16]);
    }

    // Upload (host converts 4x4 → 3x4)
    set_bones_4x4(&bone_mats, bone_count);
    draw_mesh(skinned_mesh);
}
```
{{#endtab}}

{{#endtabs}}

---

## Skinned Vertex Format

Add `FORMAT_SKINNED` (8) to your vertex format for skinned meshes:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
const FORMAT_SKINNED: u32 = 8;

// Common skinned formats
const FORMAT_SKINNED_UV_NORMAL: u32 = FORMAT_SKINNED | FORMAT_UV | FORMAT_NORMAL; // 13
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
#define FORMAT_SKINNED 8

// Common skinned formats
#define FORMAT_SKINNED_UV_NORMAL (FORMAT_SKINNED | FORMAT_UV | FORMAT_NORMAL) // 13
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const FORMAT_SKINNED: u32 = 8;

// Common skinned formats
const FORMAT_SKINNED_UV_NORMAL: u32 = FORMAT_SKINNED | FORMAT_UV | FORMAT_NORMAL; // 13
```
{{#endtab}}

{{#endtabs}}

**Skinned vertex data layout:**
```
position (3 floats)
uv (2 floats, if FORMAT_UV)
color (3 floats, if FORMAT_COLOR)
normal (3 floats, if FORMAT_NORMAL)
bone_indices (4 u8, packed as 4 bytes)
bone_weights (4 floats)
```

**Example vertex (FORMAT_SKINNED_UV_NORMAL):**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// 52 bytes per vertex: 3 + 2 + 3 + 4bytes + 4 floats
let vertex = [
    0.0, 1.0, 0.0,     // position
    0.5, 0.5,          // uv
    0.0, 1.0, 0.0,     // normal
    // bone_indices: [0, 1, 255, 255] as 4 bytes
    // bone_weights: [0.7, 0.3, 0.0, 0.0] as 4 floats
];
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// 52 bytes per vertex: 3 + 2 + 3 + 4bytes + 4 floats
float vertex[] = {
    0.0f, 1.0f, 0.0f,     // position
    0.5f, 0.5f,           // uv
    0.0f, 1.0f, 0.0f,     // normal
    // bone_indices: [0, 1, 255, 255] as 4 bytes
    // bone_weights: [0.7f, 0.3f, 0.0f, 0.0f] as 4 floats
};
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// 52 bytes per vertex: 3 + 2 + 3 + 4bytes + 4 floats
const vertex = [_]f32{
    0.0, 1.0, 0.0,     // position
    0.5, 0.5,          // uv
    0.0, 1.0, 0.0,     // normal
    // bone_indices: [0, 1, 255, 255] as 4 bytes
    // bone_weights: [0.7, 0.3, 0.0, 0.0] as 4 floats
};
```
{{#endtab}}

{{#endtabs}}

---

## Complete Example

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut SKELETON: u32 = 0;
static mut CHARACTER_MESH: u32 = 0;
static mut BONE_MATRICES: [f32; 32 * 12] = [0.0; 32 * 12];
const BONE_COUNT: usize = 32;

fn init() {
    unsafe {
        // Load skeleton
        SKELETON = rom_skeleton(b"player_rig".as_ptr(), 10);

        // Load skinned mesh
        CHARACTER_MESH = rom_mesh(b"player".as_ptr(), 6);

        // Initialize bones to identity
        for i in 0..BONE_COUNT {
            let o = i * 12;
            BONE_MATRICES[o + 0] = 1.0;
            BONE_MATRICES[o + 4] = 1.0;
            BONE_MATRICES[o + 8] = 1.0;
        }
    }
}

fn update() {
    unsafe {
        // Animate bones (your animation logic here)
        animate_walk_cycle(&mut BONE_MATRICES, elapsed_time());
    }
}

fn render() {
    unsafe {
        // Bind skeleton for inverse bind mode
        skeleton_bind(SKELETON);

        // Upload bone transforms
        set_bones(BONE_MATRICES.as_ptr(), BONE_COUNT as u32);

        // Draw character
        texture_bind(character_texture);
        push_identity();
        push_translate(player_x, player_y, player_z);
        draw_mesh(CHARACTER_MESH);

        // Unbind skeleton
        skeleton_bind(0);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t SKELETON = 0;
static uint32_t CHARACTER_MESH = 0;
static float BONE_MATRICES[32 * 12];
#define BONE_COUNT 32

NCZX_EXPORT void init(void) {
    // Load skeleton
    SKELETON = rom_skeleton("player_rig", 10);

    // Load skinned mesh
    CHARACTER_MESH = rom_mesh("player", 6);

    // Initialize bones to identity
    for (int i = 0; i < BONE_COUNT; i++) {
        int o = i * 12;
        BONE_MATRICES[o + 0] = 1.0f;
        BONE_MATRICES[o + 4] = 1.0f;
        BONE_MATRICES[o + 8] = 1.0f;
    }
}

NCZX_EXPORT void update(void) {
    // Animate bones (your animation logic here)
    animate_walk_cycle(BONE_MATRICES, elapsed_time());
}

NCZX_EXPORT void render(void) {
    // Bind skeleton for inverse bind mode
    skeleton_bind(SKELETON);

    // Upload bone transforms
    set_bones(BONE_MATRICES, BONE_COUNT);

    // Draw character
    texture_bind(character_texture);
    push_identity();
    push_translate(player_x, player_y, player_z);
    draw_mesh(CHARACTER_MESH);

    // Unbind skeleton
    skeleton_bind(0);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var SKELETON: u32 = 0;
var CHARACTER_MESH: u32 = 0;
var BONE_MATRICES: [32 * 12]f32 = undefined;
const BONE_COUNT: usize = 32;

export fn init() void {
    // Load skeleton
    SKELETON = rom_skeleton("player_rig", 10);

    // Load skinned mesh
    CHARACTER_MESH = rom_mesh("player", 6);

    // Initialize bones to identity
    for (0..BONE_COUNT) |i| {
        const o = i * 12;
        BONE_MATRICES[o + 0] = 1.0;
        BONE_MATRICES[o + 4] = 1.0;
        BONE_MATRICES[o + 8] = 1.0;
    }
}

export fn update() void {
    // Animate bones (your animation logic here)
    animate_walk_cycle(&BONE_MATRICES, elapsed_time());
}

export fn render() void {
    // Bind skeleton for inverse bind mode
    skeleton_bind(SKELETON);

    // Upload bone transforms
    set_bones(&BONE_MATRICES, BONE_COUNT);

    // Draw character
    texture_bind(character_texture);
    push_identity();
    push_translate(player_x, player_y, player_z);
    draw_mesh(CHARACTER_MESH);

    // Unbind skeleton
    skeleton_bind(0);
}
```
{{#endtab}}

{{#endtabs}}

**See Also:** [Animation Functions](./animation.md), [rom_skeleton](./rom-loading.md#rom_skeleton)
