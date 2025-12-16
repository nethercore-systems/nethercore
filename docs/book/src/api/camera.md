# Camera Functions

Camera position, target, and projection control.

## Camera Setup

### camera_set

Sets the camera position and look-at target.

**Signature:**
```rust
fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| x, y, z | `f32` | Camera position in world space |
| target_x, target_y, target_z | `f32` | Point the camera looks at |

**Example:**
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

---

### camera_fov

Sets the camera field of view.

**Signature:**
```rust
fn camera_fov(fov_degrees: f32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| fov_degrees | `f32` | Vertical FOV in degrees (1-179) |

**Default:** 60 degrees

**Example:**
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

---

## Custom Matrices

For advanced camera control, you can set the view and projection matrices directly.

### push_view_matrix

Sets a custom view matrix (camera transform).

**Signature:**
```rust
fn push_view_matrix(
    m0: f32, m1: f32, m2: f32, m3: f32,
    m4: f32, m5: f32, m6: f32, m7: f32,
    m8: f32, m9: f32, m10: f32, m11: f32,
    m12: f32, m13: f32, m14: f32, m15: f32
)
```

**Parameters:** 16 floats representing a 4x4 column-major matrix.

**Matrix Layout (column-major):**
```
| m0  m4  m8  m12 |
| m1  m5  m9  m13 |
| m2  m6  m10 m14 |
| m3  m7  m11 m15 |
```

**Example:**
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

---

### push_projection_matrix

Sets a custom projection matrix.

**Signature:**
```rust
fn push_projection_matrix(
    m0: f32, m1: f32, m2: f32, m3: f32,
    m4: f32, m5: f32, m6: f32, m7: f32,
    m8: f32, m9: f32, m10: f32, m11: f32,
    m12: f32, m13: f32, m14: f32, m15: f32
)
```

**Example:**
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

---

## Camera Patterns

### Orbiting Camera

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

### First-Person Camera

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

### Split-Screen Cameras

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
