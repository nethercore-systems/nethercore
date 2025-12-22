# Procedural Mesh Functions

Generate common 3D primitives at runtime.

All procedural meshes use **vertex format 5** (POS_UV_NORMAL): 8 floats per vertex. Works with all render modes (0-3).

**Constraints:** All functions are init-only. Call in `init()`.

---

## Basic Primitives

### cube

Generates a box mesh.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn cube(size_x: f32, size_y: f32, size_z: f32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t cube(float size_x, float size_y, float size_z);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn cube(size_x: f32, size_y: f32, size_z: f32) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| size_x | `f32` | Half-width (total width = 2 × size_x) |
| size_y | `f32` | Half-height (total height = 2 × size_y) |
| size_z | `f32` | Half-depth (total depth = 2 × size_z) |

**Returns:** Mesh handle

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn init() {
    unsafe {
        UNIT_CUBE = cube(0.5, 0.5, 0.5);      // 1×1×1 cube
        TALL_BOX = cube(1.0, 3.0, 1.0);       // 2×6×2 tall box
        FLAT_TILE = cube(2.0, 0.1, 2.0);      // 4×0.2×4 tile
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t unit_cube = 0;
static uint32_t tall_box = 0;
static uint32_t flat_tile = 0;

NCZX_EXPORT void init(void) {
    unit_cube = cube(0.5f, 0.5f, 0.5f);      // 1×1×1 cube
    tall_box = cube(1.0f, 3.0f, 1.0f);       // 2×6×2 tall box
    flat_tile = cube(2.0f, 0.1f, 2.0f);      // 4×0.2×4 tile
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var unit_cube: u32 = 0;
var tall_box: u32 = 0;
var flat_tile: u32 = 0;

export fn init() void {
    unit_cube = cube(0.5, 0.5, 0.5);      // 1×1×1 cube
    tall_box = cube(1.0, 3.0, 1.0);       // 2×6×2 tall box
    flat_tile = cube(2.0, 0.1, 2.0);      // 4×0.2×4 tile
}
```
{{#endtab}}

{{#endtabs}}

---

### sphere

Generates a UV sphere mesh.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn sphere(radius: f32, segments: u32, rings: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t sphere(float radius, uint32_t segments, uint32_t rings);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn sphere(radius: f32, segments: u32, rings: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| radius | `f32` | Sphere radius |
| segments | `u32` | Horizontal divisions (3-256) |
| rings | `u32` | Vertical divisions (2-256) |

**Returns:** Mesh handle

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn init() {
    unsafe {
        LOW_POLY_SPHERE = sphere(1.0, 8, 6);    // 48 triangles
        SMOOTH_SPHERE = sphere(1.0, 32, 16);    // 960 triangles
        PLANET = sphere(100.0, 64, 32);         // Large, detailed
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t low_poly_sphere = 0;
static uint32_t smooth_sphere = 0;
static uint32_t planet = 0;

NCZX_EXPORT void init(void) {
    low_poly_sphere = sphere(1.0f, 8, 6);    // 48 triangles
    smooth_sphere = sphere(1.0f, 32, 16);    // 960 triangles
    planet = sphere(100.0f, 64, 32);         // Large, detailed
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var low_poly_sphere: u32 = 0;
var smooth_sphere: u32 = 0;
var planet: u32 = 0;

export fn init() void {
    low_poly_sphere = sphere(1.0, 8, 6);    // 48 triangles
    smooth_sphere = sphere(1.0, 32, 16);    // 960 triangles
    planet = sphere(100.0, 64, 32);         // Large, detailed
}
```
{{#endtab}}

{{#endtabs}}

---

### cylinder

Generates a cylinder or cone mesh.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn cylinder(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t cylinder(float radius_bottom, float radius_top, float height, uint32_t segments);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn cylinder(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| radius_bottom | `f32` | Bottom cap radius |
| radius_top | `f32` | Top cap radius (0 for cone) |
| height | `f32` | Cylinder height |
| segments | `u32` | Radial divisions (3-256) |

**Returns:** Mesh handle

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn init() {
    unsafe {
        PILLAR = cylinder(0.5, 0.5, 3.0, 12);      // Uniform cylinder
        CONE = cylinder(1.0, 0.0, 2.0, 16);        // Cone
        TAPERED = cylinder(1.0, 0.5, 2.0, 16);     // Tapered cylinder
        BARREL = cylinder(0.8, 0.6, 1.5, 24);      // Barrel shape
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t pillar = 0;
static uint32_t cone = 0;
static uint32_t tapered = 0;
static uint32_t barrel = 0;

NCZX_EXPORT void init(void) {
    pillar = cylinder(0.5f, 0.5f, 3.0f, 12);      // Uniform cylinder
    cone = cylinder(1.0f, 0.0f, 2.0f, 16);        // Cone
    tapered = cylinder(1.0f, 0.5f, 2.0f, 16);     // Tapered cylinder
    barrel = cylinder(0.8f, 0.6f, 1.5f, 24);      // Barrel shape
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var pillar: u32 = 0;
var cone: u32 = 0;
var tapered: u32 = 0;
var barrel: u32 = 0;

export fn init() void {
    pillar = cylinder(0.5, 0.5, 3.0, 12);      // Uniform cylinder
    cone = cylinder(1.0, 0.0, 2.0, 16);        // Cone
    tapered = cylinder(1.0, 0.5, 2.0, 16);     // Tapered cylinder
    barrel = cylinder(0.8, 0.6, 1.5, 24);      // Barrel shape
}
```
{{#endtab}}

{{#endtabs}}

---

### plane

Generates a subdivided plane mesh (XZ plane, Y=0, facing up).

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn plane(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t plane(float size_x, float size_z, uint32_t subdivisions_x, uint32_t subdivisions_z);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn plane(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| size_x | `f32` | Half-width |
| size_z | `f32` | Half-depth |
| subdivisions_x | `u32` | X divisions (1-256) |
| subdivisions_z | `u32` | Z divisions (1-256) |

**Returns:** Mesh handle

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn init() {
    unsafe {
        GROUND = plane(50.0, 50.0, 1, 1);          // 100×100 simple quad
        TERRAIN = plane(100.0, 100.0, 32, 32);     // Subdivided for LOD
        WATER = plane(20.0, 20.0, 16, 16);         // Animated water
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t ground = 0;
static uint32_t terrain = 0;
static uint32_t water = 0;

NCZX_EXPORT void init(void) {
    ground = plane(50.0f, 50.0f, 1, 1);          // 100×100 simple quad
    terrain = plane(100.0f, 100.0f, 32, 32);     // Subdivided for LOD
    water = plane(20.0f, 20.0f, 16, 16);         // Animated water
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var ground: u32 = 0;
var terrain: u32 = 0;
var water: u32 = 0;

export fn init() void {
    ground = plane(50.0, 50.0, 1, 1);          // 100×100 simple quad
    terrain = plane(100.0, 100.0, 32, 32);     // Subdivided for LOD
    water = plane(20.0, 20.0, 16, 16);         // Animated water
}
```
{{#endtab}}

{{#endtabs}}

---

### torus

Generates a torus (donut) mesh.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t torus(float major_radius, float minor_radius, uint32_t major_segments, uint32_t minor_segments);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| major_radius | `f32` | Distance from center to tube center |
| minor_radius | `f32` | Tube thickness |
| major_segments | `u32` | Segments around ring (3-256) |
| minor_segments | `u32` | Segments around tube (3-256) |

**Returns:** Mesh handle

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn init() {
    unsafe {
        DONUT = torus(2.0, 0.5, 32, 16);           // Classic donut
        RING = torus(3.0, 0.1, 48, 8);             // Thin ring
        TIRE = torus(1.5, 0.6, 24, 12);            // Car tire
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t donut = 0;
static uint32_t ring = 0;
static uint32_t tire = 0;

NCZX_EXPORT void init(void) {
    donut = torus(2.0f, 0.5f, 32, 16);           // Classic donut
    ring = torus(3.0f, 0.1f, 48, 8);             // Thin ring
    tire = torus(1.5f, 0.6f, 24, 12);            // Car tire
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var donut: u32 = 0;
var ring: u32 = 0;
var tire: u32 = 0;

export fn init() void {
    donut = torus(2.0, 0.5, 32, 16);           // Classic donut
    ring = torus(3.0, 0.1, 48, 8);             // Thin ring
    tire = torus(1.5, 0.6, 24, 12);            // Car tire
}
```
{{#endtab}}

{{#endtabs}}

---

### capsule

Generates a capsule (cylinder with hemispherical caps).

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn capsule(radius: f32, height: f32, segments: u32, rings: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t capsule(float radius, float height, uint32_t segments, uint32_t rings);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn capsule(radius: f32, height: f32, segments: u32, rings: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| radius | `f32` | Capsule radius |
| height | `f32` | Cylinder section height (total = height + 2×radius) |
| segments | `u32` | Radial divisions (3-256) |
| rings | `u32` | Hemisphere divisions (1-128) |

**Returns:** Mesh handle

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn init() {
    unsafe {
        PILL = capsule(0.5, 1.0, 16, 8);           // Pill shape
        CHARACTER_COLLIDER = capsule(0.4, 1.2, 8, 4); // Physics capsule
        BULLET = capsule(0.1, 0.3, 12, 6);         // Projectile
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t pill = 0;
static uint32_t character_collider = 0;
static uint32_t bullet = 0;

NCZX_EXPORT void init(void) {
    pill = capsule(0.5f, 1.0f, 16, 8);           // Pill shape
    character_collider = capsule(0.4f, 1.2f, 8, 4); // Physics capsule
    bullet = capsule(0.1f, 0.3f, 12, 6);         // Projectile
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var pill: u32 = 0;
var character_collider: u32 = 0;
var bullet: u32 = 0;

export fn init() void {
    pill = capsule(0.5, 1.0, 16, 8);           // Pill shape
    character_collider = capsule(0.4, 1.2, 8, 4); // Physics capsule
    bullet = capsule(0.1, 0.3, 12, 6);         // Projectile
}
```
{{#endtab}}

{{#endtabs}}

---

## UV-Mapped Variants

These variants are identical but explicitly named for clarity.

### cube_uv

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn cube_uv(size_x: f32, size_y: f32, size_z: f32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t cube_uv(float size_x, float size_y, float size_z);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn cube_uv(size_x: f32, size_y: f32, size_z: f32) u32;
```
{{#endtab}}

{{#endtabs}}

Same as `cube()`. UV coordinates map 0-1 on each face.

---

### sphere_uv

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn sphere_uv(radius: f32, segments: u32, rings: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t sphere_uv(float radius, uint32_t segments, uint32_t rings);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn sphere_uv(radius: f32, segments: u32, rings: u32) u32;
```
{{#endtab}}

{{#endtabs}}

Same as `sphere()`. Equirectangular UV mapping.

---

### cylinder_uv

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn cylinder_uv(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t cylinder_uv(float radius_bottom, float radius_top, float height, uint32_t segments);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn cylinder_uv(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) u32;
```
{{#endtab}}

{{#endtabs}}

Same as `cylinder()`. Radial unwrap for body, polar for caps.

---

### plane_uv

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn plane_uv(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t plane_uv(float size_x, float size_z, uint32_t subdivisions_x, uint32_t subdivisions_z);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn plane_uv(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) u32;
```
{{#endtab}}

{{#endtabs}}

Same as `plane()`. Simple 0-1 grid UV mapping.

---

### torus_uv

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn torus_uv(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t torus_uv(float major_radius, float minor_radius, uint32_t major_segments, uint32_t minor_segments);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn torus_uv(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) u32;
```
{{#endtab}}

{{#endtabs}}

Same as `torus()`. Wrapped UVs on both axes.

---

### capsule_uv

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn capsule_uv(radius: f32, height: f32, segments: u32, rings: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t capsule_uv(float radius, float height, uint32_t segments, uint32_t rings);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn capsule_uv(radius: f32, height: f32, segments: u32, rings: u32) u32;
```
{{#endtab}}

{{#endtabs}}

Same as `capsule()`. Radial for body, polar for hemispheres.

---

## Complete Example

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut GROUND: u32 = 0;
static mut SPHERE: u32 = 0;
static mut CUBE: u32 = 0;
static mut PILLAR: u32 = 0;

fn init() {
    unsafe {
        render_mode(2); // PBR lighting

        // Generate primitives
        GROUND = plane(20.0, 20.0, 1, 1);
        SPHERE = sphere(1.0, 24, 12);
        CUBE = cube(0.5, 0.5, 0.5);
        PILLAR = cylinder(0.3, 0.3, 2.0, 16);
    }
}

fn render() {
    unsafe {
        camera_set(0.0, 5.0, 10.0, 0.0, 0.0, 0.0);

        // Ground
        material_roughness(0.9);
        material_metallic(0.0);
        set_color(0x556644FF);
        push_identity();
        draw_mesh(GROUND);

        // Central sphere
        material_roughness(0.3);
        material_metallic(1.0);
        set_color(0xFFD700FF);
        push_identity();
        push_translate(0.0, 1.0, 0.0);
        draw_mesh(SPHERE);

        // Pillars
        set_color(0x888888FF);
        material_metallic(0.0);
        for i in 0..4 {
            let angle = (i as f32) * 1.57;
            push_identity();
            push_translate(angle.cos() * 5.0, 1.0, angle.sin() * 5.0);
            draw_mesh(PILLAR);
        }

        // Floating cubes
        set_color(0x4488FFFF);
        for i in 0..8 {
            let t = elapsed_time() + (i as f32) * 0.5;
            push_identity();
            push_translate(
                (t * 0.5).cos() * 3.0,
                2.0 + (t * 2.0).sin() * 0.5,
                (t * 0.5).sin() * 3.0
            );
            push_rotate_y(t * 90.0);
            draw_mesh(CUBE);
        }
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
#include <math.h>

static uint32_t ground = 0;
static uint32_t sphere = 0;
static uint32_t cube = 0;
static uint32_t pillar = 0;

NCZX_EXPORT void init(void) {
    render_mode(2); // PBR lighting

    // Generate primitives
    ground = plane(20.0f, 20.0f, 1, 1);
    sphere = sphere(1.0f, 24, 12);
    cube = cube(0.5f, 0.5f, 0.5f);
    pillar = cylinder(0.3f, 0.3f, 2.0f, 16);
}

NCZX_EXPORT void render(void) {
    camera_set(0.0f, 5.0f, 10.0f, 0.0f, 0.0f, 0.0f);

    // Ground
    material_roughness(0.9f);
    material_metallic(0.0f);
    set_color(0x556644FF);
    push_identity();
    draw_mesh(ground);

    // Central sphere
    material_roughness(0.3f);
    material_metallic(1.0f);
    set_color(0xFFD700FF);
    push_identity();
    push_translate(0.0f, 1.0f, 0.0f);
    draw_mesh(sphere);

    // Pillars
    set_color(0x888888FF);
    material_metallic(0.0f);
    for (int i = 0; i < 4; i++) {
        float angle = (float)i * 1.57f;
        push_identity();
        push_translate(cosf(angle) * 5.0f, 1.0f, sinf(angle) * 5.0f);
        draw_mesh(pillar);
    }

    // Floating cubes
    set_color(0x4488FFFF);
    for (int i = 0; i < 8; i++) {
        float t = elapsed_time() + (float)i * 0.5f;
        push_identity();
        push_translate(
            cosf(t * 0.5f) * 3.0f,
            2.0f + sinf(t * 2.0f) * 0.5f,
            sinf(t * 0.5f) * 3.0f
        );
        push_rotate_y(t * 90.0f);
        draw_mesh(cube);
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const std = @import("std");

var ground: u32 = 0;
var sphere: u32 = 0;
var cube: u32 = 0;
var pillar: u32 = 0;

export fn init() void {
    render_mode(2); // PBR lighting

    // Generate primitives
    ground = plane(20.0, 20.0, 1, 1);
    sphere = sphere(1.0, 24, 12);
    cube = cube(0.5, 0.5, 0.5);
    pillar = cylinder(0.3, 0.3, 2.0, 16);
}

export fn render() void {
    camera_set(0.0, 5.0, 10.0, 0.0, 0.0, 0.0);

    // Ground
    material_roughness(0.9);
    material_metallic(0.0);
    set_color(0x556644FF);
    push_identity();
    draw_mesh(ground);

    // Central sphere
    material_roughness(0.3);
    material_metallic(1.0);
    set_color(0xFFD700FF);
    push_identity();
    push_translate(0.0, 1.0, 0.0);
    draw_mesh(sphere);

    // Pillars
    set_color(0x888888FF);
    material_metallic(0.0);
    var i: u32 = 0;
    while (i < 4) : (i += 1) {
        const angle = @as(f32, @floatFromInt(i)) * 1.57;
        push_identity();
        push_translate(@cos(angle) * 5.0, 1.0, @sin(angle) * 5.0);
        draw_mesh(pillar);
    }

    // Floating cubes
    set_color(0x4488FFFF);
    i = 0;
    while (i < 8) : (i += 1) {
        const t = elapsed_time() + @as(f32, @floatFromInt(i)) * 0.5;
        push_identity();
        push_translate(
            @cos(t * 0.5) * 3.0,
            2.0 + @sin(t * 2.0) * 0.5,
            @sin(t * 0.5) * 3.0
        );
        push_rotate_y(t * 90.0);
        draw_mesh(cube);
    }
}
```
{{#endtab}}

{{#endtabs}}

**See Also:** [Meshes](./meshes.md), [rom_mesh](./rom-loading.md#rom_mesh)
