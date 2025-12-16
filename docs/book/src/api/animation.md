# Keyframe Animation Functions

GPU-optimized keyframe animation system for skeletal animation.

## Loading Keyframes

### keyframes_load

Loads keyframes from WASM memory.

**Signature:**
```rust
fn keyframes_load(data_ptr: *const u8, byte_size: u32) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| data_ptr | `*const u8` | Pointer to keyframe data |
| byte_size | `u32` | Size of data in bytes |

**Returns:** Keyframe collection handle (non-zero on success)

**Constraints:** Init-only.

**Example:**
```rust
static WALK_DATA: &[u8] = include_bytes!("walk.ewzanim");
static mut WALK_ANIM: u32 = 0;

fn init() {
    unsafe {
        WALK_ANIM = keyframes_load(WALK_DATA.as_ptr(), WALK_DATA.len() as u32);
    }
}
```

---

### rom_keyframes

Loads keyframes from ROM data pack.

**Signature:**
```rust
fn rom_keyframes(id_ptr: *const u8, id_len: u32) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| id_ptr | `*const u8` | Pointer to asset ID string |
| id_len | `u32` | Length of asset ID |

**Returns:** Keyframe collection handle (non-zero on success)

**Constraints:** Init-only.

**Example:**
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

---

## Querying Keyframes

### keyframes_bone_count

Gets the bone count for a keyframe collection.

**Signature:**
```rust
fn keyframes_bone_count(handle: u32) -> u32
```

**Returns:** Number of bones in the animation

**Example:**
```rust
fn init() {
    unsafe {
        WALK_ANIM = rom_keyframes(b"walk".as_ptr(), 4);
        let bones = keyframes_bone_count(WALK_ANIM);
        log_fmt(b"Walk animation has {} bones", bones);
    }
}
```

---

### keyframes_frame_count

Gets the frame count for a keyframe collection.

**Signature:**
```rust
fn keyframes_frame_count(handle: u32) -> u32
```

**Returns:** Number of frames in the animation

**Example:**
```rust
fn render() {
    unsafe {
        let frame_count = keyframes_frame_count(WALK_ANIM);
        let current_frame = (ANIM_TIME as u32) % frame_count;
        keyframe_bind(WALK_ANIM, current_frame);
    }
}
```

---

## Using Keyframes

### keyframe_bind

Binds a keyframe directly from GPU buffer (zero CPU overhead).

**Signature:**
```rust
fn keyframe_bind(handle: u32, index: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| handle | `u32` | Keyframe collection handle |
| index | `u32` | Frame index (0 to frame_count-1) |

**Example:**
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

---

### keyframe_read

Reads a keyframe to WASM memory for CPU-side blending.

**Signature:**
```rust
fn keyframe_read(handle: u32, index: u32, out_ptr: *mut u8)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| handle | `u32` | Keyframe collection handle |
| index | `u32` | Frame index |
| out_ptr | `*mut u8` | Destination buffer (must be large enough for all bone matrices) |

**Example:**
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

**See Also:** [Skinning Functions](./skinning.md), [rom_keyframes](#rom_keyframes)
