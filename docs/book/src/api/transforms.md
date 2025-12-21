# Transform Functions

Matrix stack operations for positioning, rotating, and scaling objects.

## Conventions

- **Y-up** right-handed coordinate system
- **Column-major** matrix storage (wgpu/WGSL compatible)
- **Column vectors**: `v' = M * v`
- **Angles in degrees** for FFI (converted to radians internally)

## Transform Stack

### push_identity

Resets the current transform to identity (no transformation).

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn push_identity()
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void push_identity(void);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn push_identity() void;
```
{{#endtab}}

{{#endtabs}}

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Reset before drawing each object
    push_identity();
    draw_mesh(object_a);

    push_identity();
    push_translate(10.0, 0.0, 0.0);
    draw_mesh(object_b);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    /* Reset before drawing each object */
    push_identity();
    draw_mesh(object_a);

    push_identity();
    push_translate(10.0f, 0.0f, 0.0f);
    draw_mesh(object_b);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Reset before drawing each object
    push_identity();
    draw_mesh(object_a);

    push_identity();
    push_translate(10.0, 0.0, 0.0);
    draw_mesh(object_b);
}
```
{{#endtab}}

{{#endtabs}}

---

### transform_set

Sets the current transform from a 4x4 matrix.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn transform_set(matrix_ptr: *const f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void transform_set(const float* matrix_ptr);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn transform_set(matrix_ptr: [*]const f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| matrix_ptr | `*const f32` | Pointer to 16 floats (4x4 column-major) |

**Matrix Layout (column-major, 16 floats):**
```
[col0.x, col0.y, col0.z, col0.w,
 col1.x, col1.y, col1.z, col1.w,
 col2.x, col2.y, col2.z, col2.w,
 col3.x, col3.y, col3.z, col3.w]
```

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Using glam
    let transform = Mat4::from_scale_rotation_translation(
        Vec3::ONE,
        Quat::from_rotation_y(angle),
        Vec3::new(x, y, z)
    );

    let cols = transform.to_cols_array();
    transform_set(cols.as_ptr());
    draw_mesh(model);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    /* Using a math library */
    float transform[16];
    mat4_from_translation_rotation_scale(transform, pos, rot, scale);

    transform_set(transform);
    draw_mesh(model);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Using a math library
    const transform = from_scale_rotation_translation(scale, rotation, translation);

    transform_set(&transform);
    draw_mesh(model);
}
```
{{#endtab}}

{{#endtabs}}

---

## Translation

### push_translate

Applies a translation to the current transform.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn push_translate(x: f32, y: f32, z: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void push_translate(float x, float y, float z);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn push_translate(x: f32, y: f32, z: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| x | `f32` | X offset (right is positive) |
| y | `f32` | Y offset (up is positive) |
| z | `f32` | Z offset (toward camera is positive) |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Position object at (10, 5, 0)
    push_identity();
    push_translate(10.0, 5.0, 0.0);
    draw_mesh(object);

    // Stack translations (additive)
    push_identity();
    push_translate(5.0, 0.0, 0.0);  // Move right 5
    push_translate(0.0, 3.0, 0.0);  // Then move up 3
    draw_mesh(object);  // At (5, 3, 0)
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    /* Position object at (10, 5, 0) */
    push_identity();
    push_translate(10.0f, 5.0f, 0.0f);
    draw_mesh(object);

    /* Stack translations (additive) */
    push_identity();
    push_translate(5.0f, 0.0f, 0.0f);  /* Move right 5 */
    push_translate(0.0f, 3.0f, 0.0f);  /* Then move up 3 */
    draw_mesh(object);  /* At (5, 3, 0) */
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Position object at (10, 5, 0)
    push_identity();
    push_translate(10.0, 5.0, 0.0);
    draw_mesh(object);

    // Stack translations (additive)
    push_identity();
    push_translate(5.0, 0.0, 0.0);  // Move right 5
    push_translate(0.0, 3.0, 0.0);  // Then move up 3
    draw_mesh(object);  // At (5, 3, 0)
}
```
{{#endtab}}

{{#endtabs}}

---

## Rotation

### push_rotate_x

Rotates around the X axis.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn push_rotate_x(angle_deg: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void push_rotate_x(float angle_deg);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn push_rotate_x(angle_deg: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| angle_deg | `f32` | Rotation angle in degrees |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    push_identity();
    push_rotate_x(45.0); // Tilt forward 45 degrees
    draw_mesh(object);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    push_identity();
    push_rotate_x(45.0f); /* Tilt forward 45 degrees */
    draw_mesh(object);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    push_identity();
    push_rotate_x(45.0); // Tilt forward 45 degrees
    draw_mesh(object);
}
```
{{#endtab}}

{{#endtabs}}

---

### push_rotate_y

Rotates around the Y axis.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn push_rotate_y(angle_deg: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void push_rotate_y(float angle_deg);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn push_rotate_y(angle_deg: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    push_identity();
    push_rotate_y(elapsed_time() * 90.0); // Spin 90 deg/sec
    draw_mesh(spinning_object);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    push_identity();
    push_rotate_y(elapsed_time() * 90.0f); /* Spin 90 deg/sec */
    draw_mesh(spinning_object);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    push_identity();
    push_rotate_y(elapsed_time() * 90.0); // Spin 90 deg/sec
    draw_mesh(spinning_object);
}
```
{{#endtab}}

{{#endtabs}}

---

### push_rotate_z

Rotates around the Z axis.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn push_rotate_z(angle_deg: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void push_rotate_z(float angle_deg);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn push_rotate_z(angle_deg: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    push_identity();
    push_rotate_z(45.0); // Roll 45 degrees
    draw_mesh(object);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    push_identity();
    push_rotate_z(45.0f); /* Roll 45 degrees */
    draw_mesh(object);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    push_identity();
    push_rotate_z(45.0); // Roll 45 degrees
    draw_mesh(object);
}
```
{{#endtab}}

{{#endtabs}}

---

### push_rotate

Rotates around an arbitrary axis.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn push_rotate(angle_deg: f32, axis_x: f32, axis_y: f32, axis_z: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void push_rotate(float angle_deg, float axis_x, float axis_y, float axis_z);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn push_rotate(angle_deg: f32, axis_x: f32, axis_y: f32, axis_z: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| angle_deg | `f32` | Rotation angle in degrees |
| axis_x, axis_y, axis_z | `f32` | Rotation axis (will be normalized) |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    push_identity();
    // Rotate around diagonal axis
    push_rotate(45.0, 1.0, 1.0, 0.0);
    draw_mesh(object);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    push_identity();
    /* Rotate around diagonal axis */
    push_rotate(45.0f, 1.0f, 1.0f, 0.0f);
    draw_mesh(object);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    push_identity();
    // Rotate around diagonal axis
    push_rotate(45.0, 1.0, 1.0, 0.0);
    draw_mesh(object);
}
```
{{#endtab}}

{{#endtabs}}

---

## Scale

### push_scale

Applies non-uniform scaling.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn push_scale(x: f32, y: f32, z: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void push_scale(float x, float y, float z);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn push_scale(x: f32, y: f32, z: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| x | `f32` | Scale factor on X axis |
| y | `f32` | Scale factor on Y axis |
| z | `f32` | Scale factor on Z axis |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    push_identity();
    push_scale(2.0, 1.0, 1.0); // Stretch horizontally
    draw_mesh(object);

    push_identity();
    push_scale(1.0, 0.5, 1.0); // Squash vertically
    draw_mesh(squashed);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    push_identity();
    push_scale(2.0f, 1.0f, 1.0f); /* Stretch horizontally */
    draw_mesh(object);

    push_identity();
    push_scale(1.0f, 0.5f, 1.0f); /* Squash vertically */
    draw_mesh(squashed);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    push_identity();
    push_scale(2.0, 1.0, 1.0); // Stretch horizontally
    draw_mesh(object);

    push_identity();
    push_scale(1.0, 0.5, 1.0); // Squash vertically
    draw_mesh(squashed);
}
```
{{#endtab}}

{{#endtabs}}

---

### push_scale_uniform

Applies uniform scaling (same factor on all axes).

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn push_scale_uniform(s: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void push_scale_uniform(float s);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn push_scale_uniform(s: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| s | `f32` | Uniform scale factor |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    push_identity();
    push_scale_uniform(2.0); // Double size
    draw_mesh(big_object);

    push_identity();
    push_scale_uniform(0.5); // Half size
    draw_mesh(small_object);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    push_identity();
    push_scale_uniform(2.0f); /* Double size */
    draw_mesh(big_object);

    push_identity();
    push_scale_uniform(0.5f); /* Half size */
    draw_mesh(small_object);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    push_identity();
    push_scale_uniform(2.0); // Double size
    draw_mesh(big_object);

    push_identity();
    push_scale_uniform(0.5); // Half size
    draw_mesh(small_object);
}
```
{{#endtab}}

{{#endtabs}}

---

## Transform Order

Transforms are applied in **reverse order** of function calls (right-to-left matrix multiplication).

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    push_identity();
    push_translate(5.0, 0.0, 0.0);  // Applied LAST
    push_rotate_y(45.0);             // Applied SECOND
    push_scale_uniform(2.0);         // Applied FIRST
    draw_mesh(object);

    // Equivalent to: Translate * Rotate * Scale * vertex
    // Object is: 1) scaled, 2) rotated, 3) translated
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    push_identity();
    push_translate(5.0f, 0.0f, 0.0f);  /* Applied LAST */
    push_rotate_y(45.0f);               /* Applied SECOND */
    push_scale_uniform(2.0f);           /* Applied FIRST */
    draw_mesh(object);

    /* Equivalent to: Translate * Rotate * Scale * vertex */
    /* Object is: 1) scaled, 2) rotated, 3) translated */
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    push_identity();
    push_translate(5.0, 0.0, 0.0);  // Applied LAST
    push_rotate_y(45.0);             // Applied SECOND
    push_scale_uniform(2.0);         // Applied FIRST
    draw_mesh(object);

    // Equivalent to: Translate * Rotate * Scale * vertex
    // Object is: 1) scaled, 2) rotated, 3) translated
}
```
{{#endtab}}

{{#endtabs}}

### Common Patterns

**Object at position with rotation:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
push_identity();
push_translate(obj.x, obj.y, obj.z);  // Position
push_rotate_y(obj.rotation);           // Then rotate
draw_mesh(obj.mesh);
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
push_identity();
push_translate(obj_x, obj_y, obj_z);  /* Position */
push_rotate_y(obj_rotation);           /* Then rotate */
draw_mesh(obj_mesh);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
push_identity();
push_translate(obj.x, obj.y, obj.z);  // Position
push_rotate_y(obj.rotation);           // Then rotate
draw_mesh(obj.mesh);
```
{{#endtab}}

{{#endtabs}}

**Hierarchical transforms (parent-child):**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Tank body
    push_identity();
    push_translate(tank.x, tank.y, tank.z);
    push_rotate_y(tank.body_angle);
    draw_mesh(tank_body);

    // Turret (inherits body transform, then adds its own)
    push_translate(0.0, 1.0, 0.0);     // Offset from body
    push_rotate_y(tank.turret_angle);  // Independent rotation
    draw_mesh(tank_turret);

    // Barrel (inherits turret transform)
    push_translate(0.0, 0.5, 2.0);
    push_rotate_x(tank.barrel_pitch);
    draw_mesh(tank_barrel);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    /* Tank body */
    push_identity();
    push_translate(tank_x, tank_y, tank_z);
    push_rotate_y(tank_body_angle);
    draw_mesh(tank_body);

    /* Turret (inherits body transform, then adds its own) */
    push_translate(0.0f, 1.0f, 0.0f);     /* Offset from body */
    push_rotate_y(tank_turret_angle);     /* Independent rotation */
    draw_mesh(tank_turret);

    /* Barrel (inherits turret transform) */
    push_translate(0.0f, 0.5f, 2.0f);
    push_rotate_x(tank_barrel_pitch);
    draw_mesh(tank_barrel);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Tank body
    push_identity();
    push_translate(tank.x, tank.y, tank.z);
    push_rotate_y(tank.body_angle);
    draw_mesh(tank_body);

    // Turret (inherits body transform, then adds its own)
    push_translate(0.0, 1.0, 0.0);     // Offset from body
    push_rotate_y(tank.turret_angle);  // Independent rotation
    draw_mesh(tank_turret);

    // Barrel (inherits turret transform)
    push_translate(0.0, 0.5, 2.0);
    push_rotate_x(tank.barrel_pitch);
    draw_mesh(tank_barrel);
}
```
{{#endtab}}

{{#endtabs}}

**Rotating around a pivot point:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    push_identity();
    push_translate(pivot_x, pivot_y, pivot_z);  // Move to pivot
    push_rotate_y(angle);                        // Rotate
    push_translate(-pivot_x, -pivot_y, -pivot_z); // Move back
    draw_mesh(object);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    push_identity();
    push_translate(pivot_x, pivot_y, pivot_z);  /* Move to pivot */
    push_rotate_y(angle);                        /* Rotate */
    push_translate(-pivot_x, -pivot_y, -pivot_z); /* Move back */
    draw_mesh(object);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    push_identity();
    push_translate(pivot_x, pivot_y, pivot_z);  // Move to pivot
    push_rotate_y(angle);                        // Rotate
    push_translate(-pivot_x, -pivot_y, -pivot_z); // Move back
    draw_mesh(object);
}
```
{{#endtab}}

{{#endtabs}}

---

## Complete Example

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut ANGLE: f32 = 0.0;

fn update() {
    unsafe {
        ANGLE += 90.0 * delta_time(); // 90 degrees per second
    }
}

fn render() {
    unsafe {
        camera_set(0.0, 5.0, 10.0, 0.0, 0.0, 0.0);

        // Spinning cube at origin
        push_identity();
        push_rotate_y(ANGLE);
        draw_mesh(cube);

        // Orbiting cube
        push_identity();
        push_rotate_y(ANGLE * 0.5);    // Orbital rotation
        push_translate(5.0, 0.0, 0.0);  // Distance from center
        push_rotate_y(ANGLE * 2.0);     // Spin on own axis
        push_scale_uniform(0.5);
        draw_mesh(cube);

        // Static cube for reference
        push_identity();
        push_translate(-5.0, 0.0, 0.0);
        draw_mesh(cube);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static float angle = 0.0f;

EWZX_EXPORT void update(void) {
    angle += 90.0f * delta_time(); /* 90 degrees per second */
}

EWZX_EXPORT void render(void) {
    camera_set(0.0f, 5.0f, 10.0f, 0.0f, 0.0f, 0.0f);

    /* Spinning cube at origin */
    push_identity();
    push_rotate_y(angle);
    draw_mesh(cube);

    /* Orbiting cube */
    push_identity();
    push_rotate_y(angle * 0.5f);    /* Orbital rotation */
    push_translate(5.0f, 0.0f, 0.0f);  /* Distance from center */
    push_rotate_y(angle * 2.0f);     /* Spin on own axis */
    push_scale_uniform(0.5f);
    draw_mesh(cube);

    /* Static cube for reference */
    push_identity();
    push_translate(-5.0f, 0.0f, 0.0f);
    draw_mesh(cube);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var angle: f32 = 0.0;

export fn update() void {
    angle += 90.0 * delta_time(); // 90 degrees per second
}

export fn render() void {
    camera_set(0.0, 5.0, 10.0, 0.0, 0.0, 0.0);

    // Spinning cube at origin
    push_identity();
    push_rotate_y(angle);
    draw_mesh(cube);

    // Orbiting cube
    push_identity();
    push_rotate_y(angle * 0.5);    // Orbital rotation
    push_translate(5.0, 0.0, 0.0);  // Distance from center
    push_rotate_y(angle * 2.0);     // Spin on own axis
    push_scale_uniform(0.5);
    draw_mesh(cube);

    // Static cube for reference
    push_identity();
    push_translate(-5.0, 0.0, 0.0);
    draw_mesh(cube);
}
```
{{#endtab}}

{{#endtabs}}
