# Procedural Mesh Functions

Generate common 3D primitives at runtime.

All procedural meshes use **vertex format 5** (POS_UV_NORMAL): 8 floats per vertex. Works with all render modes (0-3).

**Constraints:** All functions are init-only. Call in `init()`.

---

## Basic Primitives

### cube

Generates a box mesh.

**Signature:**
```rust
fn cube(size_x: f32, size_y: f32, size_z: f32) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| size_x | `f32` | Half-width (total width = 2 × size_x) |
| size_y | `f32` | Half-height (total height = 2 × size_y) |
| size_z | `f32` | Half-depth (total depth = 2 × size_z) |

**Returns:** Mesh handle

**Example:**
```rust
fn init() {
    unsafe {
        UNIT_CUBE = cube(0.5, 0.5, 0.5);      // 1×1×1 cube
        TALL_BOX = cube(1.0, 3.0, 1.0);       // 2×6×2 tall box
        FLAT_TILE = cube(2.0, 0.1, 2.0);      // 4×0.2×4 tile
    }
}
```

---

### sphere

Generates a UV sphere mesh.

**Signature:**
```rust
fn sphere(radius: f32, segments: u32, rings: u32) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| radius | `f32` | Sphere radius |
| segments | `u32` | Horizontal divisions (3-256) |
| rings | `u32` | Vertical divisions (2-256) |

**Returns:** Mesh handle

**Example:**
```rust
fn init() {
    unsafe {
        LOW_POLY_SPHERE = sphere(1.0, 8, 6);    // 48 triangles
        SMOOTH_SPHERE = sphere(1.0, 32, 16);    // 960 triangles
        PLANET = sphere(100.0, 64, 32);         // Large, detailed
    }
}
```

---

### cylinder

Generates a cylinder or cone mesh.

**Signature:**
```rust
fn cylinder(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| radius_bottom | `f32` | Bottom cap radius |
| radius_top | `f32` | Top cap radius (0 for cone) |
| height | `f32` | Cylinder height |
| segments | `u32` | Radial divisions (3-256) |

**Returns:** Mesh handle

**Example:**
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

---

### plane

Generates a subdivided plane mesh (XZ plane, Y=0, facing up).

**Signature:**
```rust
fn plane(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| size_x | `f32` | Half-width |
| size_z | `f32` | Half-depth |
| subdivisions_x | `u32` | X divisions (1-256) |
| subdivisions_z | `u32` | Z divisions (1-256) |

**Returns:** Mesh handle

**Example:**
```rust
fn init() {
    unsafe {
        GROUND = plane(50.0, 50.0, 1, 1);          // 100×100 simple quad
        TERRAIN = plane(100.0, 100.0, 32, 32);     // Subdivided for LOD
        WATER = plane(20.0, 20.0, 16, 16);         // Animated water
    }
}
```

---

### torus

Generates a torus (donut) mesh.

**Signature:**
```rust
fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| major_radius | `f32` | Distance from center to tube center |
| minor_radius | `f32` | Tube thickness |
| major_segments | `u32` | Segments around ring (3-256) |
| minor_segments | `u32` | Segments around tube (3-256) |

**Returns:** Mesh handle

**Example:**
```rust
fn init() {
    unsafe {
        DONUT = torus(2.0, 0.5, 32, 16);           // Classic donut
        RING = torus(3.0, 0.1, 48, 8);             // Thin ring
        TIRE = torus(1.5, 0.6, 24, 12);            // Car tire
    }
}
```

---

### capsule

Generates a capsule (cylinder with hemispherical caps).

**Signature:**
```rust
fn capsule(radius: f32, height: f32, segments: u32, rings: u32) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| radius | `f32` | Capsule radius |
| height | `f32` | Cylinder section height (total = height + 2×radius) |
| segments | `u32` | Radial divisions (3-256) |
| rings | `u32` | Hemisphere divisions (1-128) |

**Returns:** Mesh handle

**Example:**
```rust
fn init() {
    unsafe {
        PILL = capsule(0.5, 1.0, 16, 8);           // Pill shape
        CHARACTER_COLLIDER = capsule(0.4, 1.2, 8, 4); // Physics capsule
        BULLET = capsule(0.1, 0.3, 12, 6);         // Projectile
    }
}
```

---

## UV-Mapped Variants

These variants are identical but explicitly named for clarity.

### cube_uv

```rust
fn cube_uv(size_x: f32, size_y: f32, size_z: f32) -> u32
```

Same as `cube()`. UV coordinates map 0-1 on each face.

---

### sphere_uv

```rust
fn sphere_uv(radius: f32, segments: u32, rings: u32) -> u32
```

Same as `sphere()`. Equirectangular UV mapping.

---

### cylinder_uv

```rust
fn cylinder_uv(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) -> u32
```

Same as `cylinder()`. Radial unwrap for body, polar for caps.

---

### plane_uv

```rust
fn plane_uv(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) -> u32
```

Same as `plane()`. Simple 0-1 grid UV mapping.

---

### torus_uv

```rust
fn torus_uv(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> u32
```

Same as `torus()`. Wrapped UVs on both axes.

---

### capsule_uv

```rust
fn capsule_uv(radius: f32, height: f32, segments: u32, rings: u32) -> u32
```

Same as `capsule()`. Radial for body, polar for hemispheres.

---

## Complete Example

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

**See Also:** [Meshes](./meshes.md), [rom_mesh](./rom-loading.md#rom_mesh)
