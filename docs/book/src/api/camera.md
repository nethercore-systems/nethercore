# Camera Functions

Camera position, target, and projection control.

## Camera Setup

### camera_set

Sets the camera position and look-at target.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void camera_set(float x, float y, float z, float target_x, float target_y, float target_z);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| x, y, z | `f32` | Camera position in world space |
| target_x, target_y, target_z | `f32` | Point the camera looks at |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Fixed camera looking at origin
    camera_set(0.0, 5.0, 10.0, 0.0, 0.0, 0.0);

    // Third-person follow camera
    camera_set(
        player.x,
        player.y + 3.0,
        player.z + 8.0,
        player.x,
        player.y + 1.0,
        player.z
    );
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    /* Fixed camera looking at origin */
    camera_set(0.0f, 5.0f, 10.0f, 0.0f, 0.0f, 0.0f);

    /* Third-person follow camera */
    camera_set(
        player_x,
        player_y + 3.0f,
        player_z + 8.0f,
        player_x,
        player_y + 1.0f,
        player_z
    );
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Fixed camera looking at origin
    camera_set(0.0, 5.0, 10.0, 0.0, 0.0, 0.0);

    // Third-person follow camera
    camera_set(
        player_x,
        player_y + 3.0,
        player_z + 8.0,
        player_x,
        player_y + 1.0,
        player_z
    );
}
```
{{#endtab}}

{{#endtabs}}

---

### camera_fov

Sets the camera field of view.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn camera_fov(fov_degrees: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void camera_fov(float fov_degrees);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn camera_fov(fov_degrees: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| fov_degrees | `f32` | Vertical FOV in degrees (1-179) |

**Default:** 60 degrees

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Normal gameplay
    camera_fov(60.0);

    // Zoom in for aiming
    if aiming {
        camera_fov(30.0);
    }

    // Wide angle for racing
    camera_fov(90.0);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    /* Normal gameplay */
    camera_fov(60.0f);

    /* Zoom in for aiming */
    if (aiming) {
        camera_fov(30.0f);
    }

    /* Wide angle for racing */
    camera_fov(90.0f);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Normal gameplay
    camera_fov(60.0);

    // Zoom in for aiming
    if (aiming) {
        camera_fov(30.0);
    }

    // Wide angle for racing
    camera_fov(90.0);
}
```
{{#endtab}}

{{#endtabs}}

---

## Custom Matrices

For advanced camera control, you can set the view and projection matrices directly.

### push_view_matrix

Sets a custom view matrix (camera transform).

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn push_view_matrix(
    m0: f32, m1: f32, m2: f32, m3: f32,
    m4: f32, m5: f32, m6: f32, m7: f32,
    m8: f32, m9: f32, m10: f32, m11: f32,
    m12: f32, m13: f32, m14: f32, m15: f32
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void push_view_matrix(
    float m0, float m1, float m2, float m3,
    float m4, float m5, float m6, float m7,
    float m8, float m9, float m10, float m11,
    float m12, float m13, float m14, float m15
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// 16 individual f32 parameters for the 4x4 matrix
```
{{#endtab}}

{{#endtabs}}

**Parameters:** 16 floats representing a 4x4 column-major matrix.

**Matrix Layout (column-major):**
```
| m0  m4  m8  m12 |
| m1  m5  m9  m13 |
| m2  m6  m10 m14 |
| m3  m7  m11 m15 |
```

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Using glam for matrix math
    let eye = Vec3::new(0.0, 5.0, 10.0);
    let target = Vec3::new(0.0, 0.0, 0.0);
    let up = Vec3::Y;
    let view = Mat4::look_at_rh(eye, target, up);

    let cols = view.to_cols_array();
    push_view_matrix(
        cols[0], cols[1], cols[2], cols[3],
        cols[4], cols[5], cols[6], cols[7],
        cols[8], cols[9], cols[10], cols[11],
        cols[12], cols[13], cols[14], cols[15]
    );
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    /* Using a math library for matrix calculation */
    float view[16];
    mat4_look_at(view, eye, target, up);

    push_view_matrix(
        view[0], view[1], view[2], view[3],
        view[4], view[5], view[6], view[7],
        view[8], view[9], view[10], view[11],
        view[12], view[13], view[14], view[15]
    );
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Using a math library for matrix calculation
    const view = look_at(eye, target, up);

    // Pass 16 individual floats to push_view_matrix
}
```
{{#endtab}}

{{#endtabs}}

---

### push_projection_matrix

Sets a custom projection matrix.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn push_projection_matrix(
    m0: f32, m1: f32, m2: f32, m3: f32,
    m4: f32, m5: f32, m6: f32, m7: f32,
    m8: f32, m9: f32, m10: f32, m11: f32,
    m12: f32, m13: f32, m14: f32, m15: f32
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void push_projection_matrix(
    float m0, float m1, float m2, float m3,
    float m4, float m5, float m6, float m7,
    float m8, float m9, float m10, float m11,
    float m12, float m13, float m14, float m15
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// 16 individual f32 parameters for the 4x4 matrix
```
{{#endtab}}

{{#endtabs}}

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Custom perspective projection
    let aspect = 16.0 / 9.0;
    let fov = 60.0_f32.to_radians();
    let near = 0.1;
    let far = 1000.0;
    let proj = Mat4::perspective_rh(fov, aspect, near, far);

    let cols = proj.to_cols_array();
    push_projection_matrix(
        cols[0], cols[1], cols[2], cols[3],
        cols[4], cols[5], cols[6], cols[7],
        cols[8], cols[9], cols[10], cols[11],
        cols[12], cols[13], cols[14], cols[15]
    );

    // Orthographic projection for 2D
    let ortho = Mat4::orthographic_rh(0.0, 960.0, 540.0, 0.0, -1.0, 1.0);
    // ... push_projection_matrix with ortho values
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    /* Custom perspective projection */
    float aspect = 16.0f / 9.0f;
    float fov = 60.0f * 3.14159f / 180.0f;
    float near = 0.1f;
    float far = 1000.0f;
    float proj[16];
    mat4_perspective(proj, fov, aspect, near, far);

    push_projection_matrix(
        proj[0], proj[1], proj[2], proj[3],
        proj[4], proj[5], proj[6], proj[7],
        proj[8], proj[9], proj[10], proj[11],
        proj[12], proj[13], proj[14], proj[15]
    );
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Custom perspective projection
    const aspect = 16.0 / 9.0;
    const fov = 60.0 * std.math.pi / 180.0;
    const near = 0.1;
    const far = 1000.0;
    const proj = perspective(fov, aspect, near, far);

    // Pass 16 individual floats to push_projection_matrix
}
```
{{#endtab}}

{{#endtabs}}

---

## Camera Patterns

### Orbiting Camera

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut ORBIT_ANGLE: f32 = 0.0;
static mut ORBIT_DISTANCE: f32 = 10.0;
static mut ORBIT_HEIGHT: f32 = 5.0;

fn update() {
    unsafe {
        // Rotate with right stick
        ORBIT_ANGLE += right_stick_x(0) * 2.0 * delta_time();

        // Zoom with triggers
        ORBIT_DISTANCE -= trigger_right(0) * 5.0 * delta_time();
        ORBIT_DISTANCE += trigger_left(0) * 5.0 * delta_time();
        ORBIT_DISTANCE = ORBIT_DISTANCE.clamp(5.0, 20.0);
    }
}

fn render() {
    unsafe {
        let cam_x = ORBIT_ANGLE.cos() * ORBIT_DISTANCE;
        let cam_z = ORBIT_ANGLE.sin() * ORBIT_DISTANCE;
        camera_set(cam_x, ORBIT_HEIGHT, cam_z, 0.0, 0.0, 0.0);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static float orbit_angle = 0.0f;
static float orbit_distance = 10.0f;
static float orbit_height = 5.0f;

EWZX_EXPORT void update(void) {
    /* Rotate with right stick */
    orbit_angle += right_stick_x(0) * 2.0f * delta_time();

    /* Zoom with triggers */
    orbit_distance -= trigger_right(0) * 5.0f * delta_time();
    orbit_distance += trigger_left(0) * 5.0f * delta_time();
    orbit_distance = nczx_clampf(orbit_distance, 5.0f, 20.0f);
}

EWZX_EXPORT void render(void) {
    float cam_x = cosf(orbit_angle) * orbit_distance;
    float cam_z = sinf(orbit_angle) * orbit_distance;
    camera_set(cam_x, orbit_height, cam_z, 0.0f, 0.0f, 0.0f);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var orbit_angle: f32 = 0.0;
var orbit_distance: f32 = 10.0;
var orbit_height: f32 = 5.0;

export fn update() void {
    // Rotate with right stick
    orbit_angle += right_stick_x(0) * 2.0 * delta_time();

    // Zoom with triggers
    orbit_distance -= trigger_right(0) * 5.0 * delta_time();
    orbit_distance += trigger_left(0) * 5.0 * delta_time();
    orbit_distance = zx.clampf(orbit_distance, 5.0, 20.0);
}

export fn render() void {
    const cam_x = @cos(orbit_angle) * orbit_distance;
    const cam_z = @sin(orbit_angle) * orbit_distance;
    camera_set(cam_x, orbit_height, cam_z, 0.0, 0.0, 0.0);
}
```
{{#endtab}}

{{#endtabs}}

### First-Person Camera

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut CAM_X: f32 = 0.0;
static mut CAM_Y: f32 = 1.7; // Eye height
static mut CAM_Z: f32 = 0.0;
static mut CAM_YAW: f32 = 0.0;
static mut CAM_PITCH: f32 = 0.0;

fn update() {
    unsafe {
        // Look with right stick
        CAM_YAW += right_stick_x(0) * 3.0 * delta_time();
        CAM_PITCH -= right_stick_y(0) * 2.0 * delta_time();
        CAM_PITCH = CAM_PITCH.clamp(-1.4, 1.4); // Limit look up/down

        // Move with left stick
        let forward_x = CAM_YAW.sin();
        let forward_z = CAM_YAW.cos();
        let right_x = forward_z;
        let right_z = -forward_x;

        let speed = 5.0 * delta_time();
        CAM_X += left_stick_y(0) * forward_x * speed;
        CAM_Z += left_stick_y(0) * forward_z * speed;
        CAM_X += left_stick_x(0) * right_x * speed;
        CAM_Z += left_stick_x(0) * right_z * speed;
    }
}

fn render() {
    unsafe {
        let look_x = CAM_X + CAM_YAW.sin() * CAM_PITCH.cos();
        let look_y = CAM_Y + CAM_PITCH.sin();
        let look_z = CAM_Z + CAM_YAW.cos() * CAM_PITCH.cos();
        camera_set(CAM_X, CAM_Y, CAM_Z, look_x, look_y, look_z);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static float cam_x = 0.0f;
static float cam_y = 1.7f; /* Eye height */
static float cam_z = 0.0f;
static float cam_yaw = 0.0f;
static float cam_pitch = 0.0f;

EWZX_EXPORT void update(void) {
    /* Look with right stick */
    cam_yaw += right_stick_x(0) * 3.0f * delta_time();
    cam_pitch -= right_stick_y(0) * 2.0f * delta_time();
    cam_pitch = nczx_clampf(cam_pitch, -1.4f, 1.4f);

    /* Move with left stick */
    float forward_x = sinf(cam_yaw);
    float forward_z = cosf(cam_yaw);
    float right_x = forward_z;
    float right_z = -forward_x;

    float speed = 5.0f * delta_time();
    cam_x += left_stick_y(0) * forward_x * speed;
    cam_z += left_stick_y(0) * forward_z * speed;
    cam_x += left_stick_x(0) * right_x * speed;
    cam_z += left_stick_x(0) * right_z * speed;
}

EWZX_EXPORT void render(void) {
    float look_x = cam_x + sinf(cam_yaw) * cosf(cam_pitch);
    float look_y = cam_y + sinf(cam_pitch);
    float look_z = cam_z + cosf(cam_yaw) * cosf(cam_pitch);
    camera_set(cam_x, cam_y, cam_z, look_x, look_y, look_z);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var cam_x: f32 = 0.0;
var cam_y: f32 = 1.7; // Eye height
var cam_z: f32 = 0.0;
var cam_yaw: f32 = 0.0;
var cam_pitch: f32 = 0.0;

export fn update() void {
    // Look with right stick
    cam_yaw += right_stick_x(0) * 3.0 * delta_time();
    cam_pitch -= right_stick_y(0) * 2.0 * delta_time();
    cam_pitch = zx.clampf(cam_pitch, -1.4, 1.4);

    // Move with left stick
    const forward_x = @sin(cam_yaw);
    const forward_z = @cos(cam_yaw);
    const right_x = forward_z;
    const right_z = -forward_x;

    const speed = 5.0 * delta_time();
    cam_x += left_stick_y(0) * forward_x * speed;
    cam_z += left_stick_y(0) * forward_z * speed;
    cam_x += left_stick_x(0) * right_x * speed;
    cam_z += left_stick_x(0) * right_z * speed;
}

export fn render() void {
    const look_x = cam_x + @sin(cam_yaw) * @cos(cam_pitch);
    const look_y = cam_y + @sin(cam_pitch);
    const look_z = cam_z + @cos(cam_yaw) * @cos(cam_pitch);
    camera_set(cam_x, cam_y, cam_z, look_x, look_y, look_z);
}
```
{{#endtab}}

{{#endtabs}}

### Split-Screen Cameras

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    let count = player_count();

    for p in 0..count {
        // Set viewport (would need custom projection)
        setup_viewport_for_player(p, count);

        // Each player's camera follows them
        camera_set(
            players[p].x,
            players[p].y + 5.0,
            players[p].z + 10.0,
            players[p].x,
            players[p].y,
            players[p].z
        );

        draw_scene();
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    uint32_t count = player_count();

    for (uint32_t p = 0; p < count; p++) {
        /* Set viewport (would need custom projection) */
        setup_viewport_for_player(p, count);

        /* Each player's camera follows them */
        camera_set(
            players[p].x,
            players[p].y + 5.0f,
            players[p].z + 10.0f,
            players[p].x,
            players[p].y,
            players[p].z
        );

        draw_scene();
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    const count = player_count();

    var p: u32 = 0;
    while (p < count) : (p += 1) {
        // Set viewport (would need custom projection)
        setup_viewport_for_player(p, count);

        // Each player's camera follows them
        camera_set(
            players[p].x,
            players[p].y + 5.0,
            players[p].z + 10.0,
            players[p].x,
            players[p].y,
            players[p].z
        );

        draw_scene();
    }
}
```
{{#endtab}}

{{#endtabs}}
