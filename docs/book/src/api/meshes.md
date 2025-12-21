# Mesh Functions

Loading and drawing 3D meshes.

## Retained Meshes

Retained meshes are loaded once in `init()` and drawn multiple times in `render()`.

### load_mesh

Loads a non-indexed mesh from vertex data.

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn load_mesh(data_ptr: *const u8, vertex_count: u32, format: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT uint32_t load_mesh(const uint8_t* data_ptr, uint32_t vertex_count, uint32_t format);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn load_mesh(data_ptr: [*]const u8, vertex_count: u32, format: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| data_ptr | `*const u8` | Pointer to vertex data |
| vertex_count | `u32` | Number of vertices |
| format | `u32` | Vertex format flags |

**Returns:** Mesh handle (non-zero on success)

**Constraints:** Init-only.

---

### load_mesh_indexed

Loads an indexed mesh (more efficient for shared vertices).

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn load_mesh_indexed(
    data_ptr: *const u8,
    vertex_count: u32,
    index_ptr: *const u16,
    index_count: u32,
    format: u32
) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT uint32_t load_mesh_indexed(
    const uint8_t* data_ptr,
    uint32_t vertex_count,
    const uint16_t* index_ptr,
    uint32_t index_count,
    uint32_t format
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn load_mesh_indexed(
    data_ptr: [*]const u8,
    vertex_count: u32,
    index_ptr: [*]const u16,
    index_count: u32,
    format: u32,
) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| data_ptr | `*const u8` | Pointer to vertex data |
| vertex_count | `u32` | Number of vertices |
| index_ptr | `*const u16` | Pointer to u16 index data |
| index_count | `u32` | Number of indices |
| format | `u32` | Vertex format flags |

**Returns:** Mesh handle

**Constraints:** Init-only.

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut CUBE_MESH: u32 = 0;

// Cube with 8 vertices, 36 indices (12 triangles)
const CUBE_VERTS: [f32; 8 * 6] = [
    // Position (xyz) + Normal (xyz)
    -1.0, -1.0, -1.0,  0.0, 0.0, -1.0,
     1.0, -1.0, -1.0,  0.0, 0.0, -1.0,
    // ... more vertices
];

const CUBE_INDICES: [u16; 36] = [
    0, 1, 2, 2, 3, 0, // Front face
    // ... more indices
];

fn init() {
    unsafe {
        CUBE_MESH = load_mesh_indexed(
            CUBE_VERTS.as_ptr() as *const u8,
            8,
            CUBE_INDICES.as_ptr(),
            36,
            4 // FORMAT_POS_NORMAL
        );
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t cube_mesh = 0;

// Cube with 8 vertices, 36 indices (12 triangles)
static const float CUBE_VERTS[8 * 6] = {
    // Position (xyz) + Normal (xyz)
    -1.0f, -1.0f, -1.0f,  0.0f, 0.0f, -1.0f,
     1.0f, -1.0f, -1.0f,  0.0f, 0.0f, -1.0f,
    // ... more vertices
};

static const uint16_t CUBE_INDICES[36] = {
    0, 1, 2, 2, 3, 0, // Front face
    // ... more indices
};

EWZX_EXPORT void init(void) {
    cube_mesh = load_mesh_indexed(
        (const uint8_t*)CUBE_VERTS,
        8,
        CUBE_INDICES,
        36,
        4 // FORMAT_POS_NORMAL
    );
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var cube_mesh: u32 = 0;

// Cube with 8 vertices, 36 indices (12 triangles)
const CUBE_VERTS = [_]f32{
    // Position (xyz) + Normal (xyz)
    -1.0, -1.0, -1.0, 0.0, 0.0, -1.0,
    1.0,  -1.0, -1.0, 0.0, 0.0, -1.0,
    // ... more vertices
};

const CUBE_INDICES = [_]u16{
    0, 1, 2, 2, 3, 0, // Front face
    // ... more indices
};

export fn init() void {
    cube_mesh = load_mesh_indexed(
        @ptrCast(&CUBE_VERTS),
        8,
        &CUBE_INDICES,
        36,
        4, // FORMAT_POS_NORMAL
    );
}
```
{{#endtab}}

{{#endtabs}}

---

### load_mesh_packed

Loads a packed mesh with half-precision floats (smaller memory footprint).

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn load_mesh_packed(data_ptr: *const u8, vertex_count: u32, format: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT uint32_t load_mesh_packed(const uint8_t* data_ptr, uint32_t vertex_count, uint32_t format);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn load_mesh_packed(data_ptr: [*]const u8, vertex_count: u32, format: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Constraints:** Init-only. Uses f16 for positions and snorm16 for normals.

---

### load_mesh_indexed_packed

Loads an indexed packed mesh.

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn load_mesh_indexed_packed(
    data_ptr: *const u8,
    vertex_count: u32,
    index_ptr: *const u16,
    index_count: u32,
    format: u32
) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT uint32_t load_mesh_indexed_packed(
    const uint8_t* data_ptr,
    uint32_t vertex_count,
    const uint16_t* index_ptr,
    uint32_t index_count,
    uint32_t format
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn load_mesh_indexed_packed(
    data_ptr: [*]const u8,
    vertex_count: u32,
    index_ptr: [*]const u16,
    index_count: u32,
    format: u32,
) u32;
```
{{#endtab}}

{{#endtabs}}

**Constraints:** Init-only.

---

### draw_mesh

Draws a retained mesh with the current transform and render state.

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn draw_mesh(handle: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void draw_mesh(uint32_t handle);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn draw_mesh(handle: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| handle | `u32` | Mesh handle from `load_mesh*()` or procedural generators |

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Draw at origin
    push_identity();
    draw_mesh(cube);

    // Draw at different position
    push_identity();
    push_translate(5.0, 0.0, 0.0);
    draw_mesh(cube);

    // Draw with different color
    set_color(0xFF0000FF);
    push_identity();
    push_translate(-5.0, 0.0, 0.0);
    draw_mesh(cube);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    // Draw at origin
    push_identity();
    draw_mesh(cube);

    // Draw at different position
    push_identity();
    push_translate(5.0f, 0.0f, 0.0f);
    draw_mesh(cube);

    // Draw with different color
    set_color(0xFF0000FF);
    push_identity();
    push_translate(-5.0f, 0.0f, 0.0f);
    draw_mesh(cube);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Draw at origin
    push_identity();
    draw_mesh(cube);

    // Draw at different position
    push_identity();
    push_translate(5.0, 0.0, 0.0);
    draw_mesh(cube);

    // Draw with different color
    set_color(0xFF0000FF);
    push_identity();
    push_translate(-5.0, 0.0, 0.0);
    draw_mesh(cube);
}
```
{{#endtab}}

{{#endtabs}}

---

## Immediate Mode Drawing

For dynamic geometry that changes every frame.

### draw_triangles

Draws non-indexed triangles immediately (not retained).

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn draw_triangles(data_ptr: *const u8, vertex_count: u32, format: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void draw_triangles(const uint8_t* data_ptr, uint32_t vertex_count, uint32_t format);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn draw_triangles(data_ptr: [*]const u8, vertex_count: u32, format: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| data_ptr | `*const u8` | Pointer to vertex data |
| vertex_count | `u32` | Number of vertices (must be multiple of 3) |
| format | `u32` | Vertex format flags |

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Dynamic triangle
    let verts: [f32; 18] = [
        // Position (xyz) + Color (rgb)
        0.0, 1.0, 0.0,  1.0, 0.0, 0.0, // Top (red)
        -1.0, -1.0, 0.0,  0.0, 1.0, 0.0, // Left (green)
        1.0, -1.0, 0.0,  0.0, 0.0, 1.0, // Right (blue)
    ];

    push_identity();
    draw_triangles(verts.as_ptr() as *const u8, 3, 2); // FORMAT_POS_COLOR
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    // Dynamic triangle
    float verts[18] = {
        // Position (xyz) + Color (rgb)
        0.0f, 1.0f, 0.0f,  1.0f, 0.0f, 0.0f, // Top (red)
        -1.0f, -1.0f, 0.0f,  0.0f, 1.0f, 0.0f, // Left (green)
        1.0f, -1.0f, 0.0f,  0.0f, 0.0f, 1.0f, // Right (blue)
    };

    push_identity();
    draw_triangles((const uint8_t*)verts, 3, 2); // FORMAT_POS_COLOR
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Dynamic triangle
    const verts = [_]f32{
        // Position (xyz) + Color (rgb)
        0.0, 1.0, 0.0, 1.0, 0.0, 0.0, // Top (red)
        -1.0, -1.0, 0.0, 0.0, 1.0, 0.0, // Left (green)
        1.0, -1.0, 0.0, 0.0, 0.0, 1.0, // Right (blue)
    };

    push_identity();
    draw_triangles(@ptrCast(&verts), 3, 2); // FORMAT_POS_COLOR
}
```
{{#endtab}}

{{#endtabs}}

---

### draw_triangles_indexed

Draws indexed triangles immediately.

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn draw_triangles_indexed(
    data_ptr: *const u8,
    vertex_count: u32,
    index_ptr: *const u16,
    index_count: u32,
    format: u32
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void draw_triangles_indexed(
    const uint8_t* data_ptr,
    uint32_t vertex_count,
    const uint16_t* index_ptr,
    uint32_t index_count,
    uint32_t format
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn draw_triangles_indexed(
    data_ptr: [*]const u8,
    vertex_count: u32,
    index_ptr: [*]const u16,
    index_count: u32,
    format: u32,
) void;
```
{{#endtab}}

{{#endtabs}}

---

## Vertex Formats

Vertex format is specified as a bitmask of flags:

| Flag | Value | Components | Bytes |
|------|-------|------------|-------|
| Position | 0 | xyz (3 floats) | 12 |
| UV | 1 | uv (2 floats) | 8 |
| Color | 2 | rgb (3 floats) | 12 |
| Normal | 4 | xyz (3 floats) | 12 |
| Skinned | 8 | bone indices + weights | 16 |

**Common Combinations:**

| Format | Value | Components | Stride |
|--------|-------|------------|--------|
| POS | 0 | Position only | 12 bytes |
| POS_UV | 1 | Position + UV | 20 bytes |
| POS_COLOR | 2 | Position + Color | 24 bytes |
| POS_UV_COLOR | 3 | Position + UV + Color | 32 bytes |
| POS_NORMAL | 4 | Position + Normal | 24 bytes |
| POS_UV_NORMAL | 5 | Position + UV + Normal | 32 bytes |
| POS_COLOR_NORMAL | 6 | Position + Color + Normal | 36 bytes |
| POS_UV_COLOR_NORMAL | 7 | Position + UV + Color + Normal | 44 bytes |

**With Skinning (add 8):**

| Format | Value | Stride |
|--------|-------|--------|
| POS_NORMAL_SKINNED | 12 | 40 bytes |
| POS_UV_NORMAL_SKINNED | 13 | 48 bytes |

---

## Vertex Data Layout

Data is laid out per-vertex in this order:
1. Position (xyz) - 3 floats
2. UV (uv) - 2 floats (if enabled)
3. Color (rgb) - 3 floats (if enabled)
4. Normal (xyz) - 3 floats (if enabled)
5. Skinning (indices + weights) - 4 bytes + 4 bytes (if enabled)

**Example: POS_UV_NORMAL (format 5)**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// Each vertex: 8 floats (32 bytes)
let vertex: [f32; 8] = [
    0.0, 1.0, 0.0,  // Position
    0.5, 1.0,       // UV
    0.0, 1.0, 0.0,  // Normal
];
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Each vertex: 8 floats (32 bytes)
float vertex[8] = {
    0.0f, 1.0f, 0.0f,  // Position
    0.5f, 1.0f,        // UV
    0.0f, 1.0f, 0.0f,  // Normal
};
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Each vertex: 8 floats (32 bytes)
const vertex = [_]f32{
    0.0, 1.0, 0.0, // Position
    0.5, 1.0,      // UV
    0.0, 1.0, 0.0, // Normal
};
```
{{#endtab}}

{{#endtabs}}

---

## Complete Example

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut TRIANGLE: u32 = 0;
static mut QUAD: u32 = 0;

// Triangle with position + color
const TRI_VERTS: [f32; 3 * 6] = [
    // pos xyz, color rgb
    0.0, 1.0, 0.0,  1.0, 0.0, 0.0,
    -1.0, -1.0, 0.0,  0.0, 1.0, 0.0,
    1.0, -1.0, 0.0,  0.0, 0.0, 1.0,
];

// Quad with position + UV + normal (indexed)
const QUAD_VERTS: [f32; 4 * 8] = [
    // pos xyz, uv, normal xyz
    -1.0, -1.0, 0.0,  0.0, 0.0,  0.0, 0.0, 1.0,
     1.0, -1.0, 0.0,  1.0, 0.0,  0.0, 0.0, 1.0,
     1.0,  1.0, 0.0,  1.0, 1.0,  0.0, 0.0, 1.0,
    -1.0,  1.0, 0.0,  0.0, 1.0,  0.0, 0.0, 1.0,
];

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

fn init() {
    unsafe {
        // Non-indexed triangle
        TRIANGLE = load_mesh(
            TRI_VERTS.as_ptr() as *const u8,
            3,
            2 // POS_COLOR
        );

        // Indexed quad
        QUAD = load_mesh_indexed(
            QUAD_VERTS.as_ptr() as *const u8,
            4,
            QUAD_INDICES.as_ptr(),
            6,
            5 // POS_UV_NORMAL
        );
    }
}

fn render() {
    unsafe {
        camera_set(0.0, 0.0, 5.0, 0.0, 0.0, 0.0);

        // Draw triangle
        push_identity();
        push_translate(-2.0, 0.0, 0.0);
        draw_mesh(TRIANGLE);

        // Draw textured quad
        texture_bind(my_texture);
        push_identity();
        push_translate(2.0, 0.0, 0.0);
        draw_mesh(QUAD);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t triangle = 0;
static uint32_t quad = 0;

// Triangle with position + color
static const float TRI_VERTS[3 * 6] = {
    // pos xyz, color rgb
    0.0f, 1.0f, 0.0f,  1.0f, 0.0f, 0.0f,
    -1.0f, -1.0f, 0.0f,  0.0f, 1.0f, 0.0f,
    1.0f, -1.0f, 0.0f,  0.0f, 0.0f, 1.0f,
};

// Quad with position + UV + normal (indexed)
static const float QUAD_VERTS[4 * 8] = {
    // pos xyz, uv, normal xyz
    -1.0f, -1.0f, 0.0f,  0.0f, 0.0f,  0.0f, 0.0f, 1.0f,
     1.0f, -1.0f, 0.0f,  1.0f, 0.0f,  0.0f, 0.0f, 1.0f,
     1.0f,  1.0f, 0.0f,  1.0f, 1.0f,  0.0f, 0.0f, 1.0f,
    -1.0f,  1.0f, 0.0f,  0.0f, 1.0f,  0.0f, 0.0f, 1.0f,
};

static const uint16_t QUAD_INDICES[6] = {0, 1, 2, 2, 3, 0};

EWZX_EXPORT void init(void) {
    // Non-indexed triangle
    triangle = load_mesh(
        (const uint8_t*)TRI_VERTS,
        3,
        2 // POS_COLOR
    );

    // Indexed quad
    quad = load_mesh_indexed(
        (const uint8_t*)QUAD_VERTS,
        4,
        QUAD_INDICES,
        6,
        5 // POS_UV_NORMAL
    );
}

EWZX_EXPORT void render(void) {
    camera_set(0.0f, 0.0f, 5.0f, 0.0f, 0.0f, 0.0f);

    // Draw triangle
    push_identity();
    push_translate(-2.0f, 0.0f, 0.0f);
    draw_mesh(triangle);

    // Draw textured quad
    texture_bind(my_texture);
    push_identity();
    push_translate(2.0f, 0.0f, 0.0f);
    draw_mesh(quad);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var triangle: u32 = 0;
var quad: u32 = 0;

// Triangle with position + color
const TRI_VERTS = [_]f32{
    // pos xyz, color rgb
    0.0, 1.0, 0.0, 1.0, 0.0, 0.0,
    -1.0, -1.0, 0.0, 0.0, 1.0, 0.0,
    1.0, -1.0, 0.0, 0.0, 0.0, 1.0,
};

// Quad with position + UV + normal (indexed)
const QUAD_VERTS = [_]f32{
    // pos xyz, uv, normal xyz
    -1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    1.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0,
    1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0,
    -1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0,
};

const QUAD_INDICES = [_]u16{ 0, 1, 2, 2, 3, 0 };

export fn init() void {
    // Non-indexed triangle
    triangle = load_mesh(
        @ptrCast(&TRI_VERTS),
        3,
        2, // POS_COLOR
    );

    // Indexed quad
    quad = load_mesh_indexed(
        @ptrCast(&QUAD_VERTS),
        4,
        &QUAD_INDICES,
        6,
        5, // POS_UV_NORMAL
    );
}

export fn render() void {
    camera_set(0.0, 0.0, 5.0, 0.0, 0.0, 0.0);

    // Draw triangle
    push_identity();
    push_translate(-2.0, 0.0, 0.0);
    draw_mesh(triangle);

    // Draw textured quad
    texture_bind(my_texture);
    push_identity();
    push_translate(2.0, 0.0, 0.0);
    draw_mesh(quad);
}
```
{{#endtab}}

{{#endtabs}}

**See Also:** [Procedural Meshes](./procedural.md), [rom_mesh](./rom-loading.md#rom_mesh)
