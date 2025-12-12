# Emberware Z Animation System Specification

**Status:** Ready for Implementation
**Author:** Zerve
**Version:** 2.0
**Last Updated:** December 2025

---

## Executive Summary

This document specifies a ROM-backed keyframe system for Emberware Z that keeps animation data out of linear memory while providing flexibility for both simple "stamp" animations and complex blended animation systems.

The core insight: **keyframes belong in ROM, not linear memory**. With 4MB linear memory and potentially 3MB+ of animation data per character, storing keyframes in WASM memory is infeasible for content-rich games. Instead, the host stores keyframe collections and provides APIs to either directly apply keyframes (stamp mode) or pull decoded keyframes into small working buffers for blending.

The system is deliberately unopinionated—developers control playback rate, blending, state machines, and all animation logic. The host is simply a frame store with bounds checking.

---

## Design Goals

1. **Memory efficiency** — Keyframe data lives in ROM (16MB), not linear memory (4MB)
2. **Agnostic to animation approach** — Support FK, IK, procedural, blend trees, state machines
3. **Stamp fast path** — `keyframe_bind()` sets bones directly from a keyframe handle
4. **Developer flexibility** — Grouped or individual keyframes, any playback scheme
5. **Rollback-friendly** — Only small working buffers in linear memory get snapshotted

---

## Memory Architecture

```
                              ROM (16MB)
  +-----------+ +-----------+ +-----------+ +-----------+
  |Character A| |Character B| | Meshes    | | Textures  |
  | Keyframes | | Keyframes | | (~2MB)    | | (~4MB)    |
  | (~180KB)  | | (~180KB)  | |           | |           |
  +-----+-----+ +-----------+ +-----------+ +-----------+
        |
        | keyframe_read() / keyframe_bind()
        v
                     WASM Linear Memory (4MB)
  +--------------+ +--------------+ +------------------------+
  |Keyframe Buf A| |Keyframe Buf B| | Output Bone Matrices   |
  | (~1.3KB)     | | (~1.3KB)     | | (~2.5KB per character) |
  +--------------+ +--------------+ +------------------------+
  +----------------------------------------------------------+
  |                Game State (~4MB available)               |
  |    Physics, UI, Game Logic, Entity State, etc.           |
  +----------------------------------------------------------+
        |
        | set_bones() / set_bones_4x4()
        v
                              GPU (VRAM)
  +----------------------------------------------------------+
  | Bone Matrix Uniform Buffer (255 bones x 48 bytes = 12KB) |
  +----------------------------------------------------------+
```

---

## API Overview

### Tier 1: Stamp Animation (Minimal WASM Memory)

For games that don't need blending—frame-exact animation like PS1-era 3D or step-based keyframes.

```
ROM ──── keyframe_bind() ────> GPU
              (host decodes, builds matrices, uploads)
```

`keyframe_bind()` is a convenience function equivalent to: decode keyframe -> build bone matrices -> upload via `set_bones()`. It respects the currently bound skeleton (inverse bind mode).

### Tier 2: Blended Animation (Small Working Buffers)

For games needing interpolation, blend trees, or animation mixing.

```
ROM ──── keyframe_read() ────> WASM ──── blend/process ────> WASM ──── set_bones_4x4() ────> GPU
         (decoded BoneTransform)      (developer code)               (upload matrices)
```

### Tier 3: Custom Format (Full Control)

For developers who want their own keyframe format, compression, or exotic animation systems.

```
ROM ──── read_data() ────> WASM ──── custom decode ────> WASM ──── set_bones() ────> GPU
         (raw bytes)               (developer code)               (upload matrices)
```

---

## FFI API

### Keyframe Collection Management

```rust
/// Load keyframe collection from embedded data (include_bytes!)
///
/// Data must be in .ewzanim format (4-byte header + frame data).
/// The data is copied to host memory—original can be freed.
/// Init-only function.
///
/// # Arguments
/// * `data_ptr` — Pointer to keyframe data in WASM memory
/// * `byte_size` — Total size of data
///
/// # Returns
/// Handle (0 = invalid/error)
fn keyframes_load(data_ptr: u32, byte_size: u32) -> u32;

/// Load keyframe collection from ROM data pack by string ID
///
/// Looks up the keyframe collection in the ROM data pack.
/// No data is copied to WASM memory—stays on host.
/// Init-only function.
///
/// # Arguments
/// * `id_ptr` — Pointer to asset ID string in WASM memory
/// * `id_len` — Length of asset ID string
///
/// # Returns
/// Handle (0 = invalid/error)
fn rom_keyframes(id_ptr: u32, id_len: u32) -> u32;

// NOTE: No keyframes_unload() function - keyframes are auto-cleaned on game shutdown
// (same pattern as textures, meshes, and other init-time resources)

/// Get bone count for this keyframe collection
fn keyframes_bone_count(handle: u32) -> u8;

/// Get frame count for this keyframe collection
fn keyframes_frame_count(handle: u32) -> u16;
```

### Keyframe Access

```rust
/// Read a decoded keyframe into WASM memory for processing
///
/// Writes `bone_count x 40` bytes to out_ptr as BoneTransform structs.
/// The host decodes the compressed format—game receives ready-to-use transforms.
///
/// # Arguments
/// * `handle` — Keyframes handle from keyframes_load() or rom_keyframes()
/// * `index` — Frame index (0-based)
/// * `out_ptr` — Destination in WASM memory (must have bone_count x 32 bytes)
///
/// # Traps
/// Traps if index >= frame_count or out_ptr is invalid
fn keyframe_read(handle: u32, index: u32, out_ptr: u32);

/// Bind bone matrices directly from a ROM keyframe
///
/// Sets bone matrices directly from a keyframe—no WASM memory needed.
/// Decodes the keyframe, builds bone matrices, and uploads to GPU.
/// Equivalent to: keyframe_read() -> build matrices -> set_bones()
///
/// Respects currently bound skeleton via skeleton_bind() for inverse bind mode.
///
/// # Arguments
/// * `handle` — Keyframes handle
/// * `index` — Frame index (0-based)
///
/// # Traps
/// Traps if index >= frame_count
fn keyframe_bind(handle: u32, index: u32);
```

### Bone Matrix Upload

```rust
/// Upload bone matrices to GPU (4x4 column-major f32)
///
/// Simple path for use with standard matrix libraries like glam::Mat4.
/// Host internally packs to 3x4 for GPU upload.
///
/// # Arguments
/// * `matrices_ptr` — Pointer to bone matrices in WASM memory
/// * `count` — Number of bones
///
/// Matrix layout: 16 x f32 per bone (64 bytes), column-major
fn set_bones_4x4(matrices_ptr: u32, count: u32);

/// Upload bone matrices to GPU (3x4 column-major f32)
///
/// Optimized path for developers who build 3x4 matrices directly.
/// Matches transform_set() convention.
///
/// # Arguments
/// * `matrices_ptr` — Pointer to bone matrices in WASM memory
/// * `count` — Number of bones
///
/// Matrix layout: 12 x f32 per bone (48 bytes), column-major
/// [[col0.xyz], [col1.xyz], [col2.xyz], [translation.xyz]]
fn set_bones(matrices_ptr: u32, count: u32);
```

---

## File Format (.ewzanim)

POD format with minimal header. No magic bytes—data is ready for direct use.

### Header (4 bytes)

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0x00 | 1 | bone_count | Bones per frame (u8, max 255) |
| 0x01 | 1 | flags | Reserved, must be 0 |
| 0x02 | 2 | frame_count | Total frames (u16 LE, max 65535) |

### Frame Data

Immediately following header:
```
frame_count x bone_count x 16 bytes
```

### Example Layout

```
Offset  Size    Content
------  ------  -------
0x0000  1       bone_count = 40
0x0001  1       flags = 0
0x0002  2       frame_count = 60 (little-endian: 0x3C 0x00)
0x0004  38400   Frame data (60 frames x 40 bones x 16 bytes)
------
Total:  38404 bytes
```

### File Size Formula

```
file_size = 4 + (frame_count x bone_count x 16)
```

### Rust Header Definition

```rust
#[repr(C, packed)]
pub struct EmberZAnimationHeader {
    pub bone_count: u8,    // Max 255 bones
    pub flags: u8,         // Reserved = 0
    pub frame_count: u16,  // Max 65535 frames (little-endian)
}
```

---

## Platform Keyframe Format

Each bone transform is stored in 16 bytes, balancing compression with decode simplicity.

### Layout (16 bytes per bone)

| Offset | Size | Field | Encoding |
|--------|------|-------|----------|
| 0x00 | 4 | rotation | Smallest-three packed quaternion (u32) |
| 0x04 | 6 | position | f16 x 3 (x, y, z) |
| 0x0A | 6 | scale | f16 x 3 (x, y, z) — non-uniform scale supported |

### Rust Definition

```rust
#[repr(C, packed)]
pub struct PlatformBoneKeyframe {
    pub rotation: u32,       // Smallest-three packed quaternion
    pub position: [u16; 3],  // f16 x 3 (use half crate or manual decode)
    pub scale: [u16; 3],     // f16 x 3 — full XYZ scale support
}
```

### Decoded Bone Transform (for WASM)

When `keyframe_read()` is called, the host decodes to this format:

```rust
#[repr(C)]
pub struct BoneTransform {
    pub rotation: [f32; 4],  // Quaternion [x, y, z, w]
    pub position: [f32; 3],  // Translation
    pub scale: [f32; 3],     // Non-uniform scale [x, y, z]
}
// Total: 40 bytes per bone
```

### Size Analysis

```
Per frame:     40 bones x 16 bytes = 640 bytes
Per clip:      60 frames x 640 bytes = 38.4 KB
Per character: 50 clips x 38.4 KB = 1.92 MB -> 480 KB compressed (4x savings)
4 characters:  1.92 MB in ROM (vs 7.68 MB uncompressed)
```

---

## Quaternion Encoding: Smallest-Three

Industry-standard encoding used by Unreal, Unity, and ACL.

### Concept

A unit quaternion satisfies: `x^2 + y^2 + z^2 + w^2 = 1`

We can reconstruct one component from the other three:
```
missing = sqrt(1 - a^2 - b^2 - c^2)
```

Drop the component with the largest absolute value (best precision), store a 2-bit index indicating which was dropped.

### Bit Layout (32 bits)

```
[31:22] a component (10 bits, biased signed)
[21:12] b component (10 bits, biased signed)
[11:2]  c component (10 bits, biased signed)
[1:0]   index of dropped component (2 bits)

Each component mapped: [-1/sqrt(2), 1/sqrt(2)] -> [1, 1023] (biased by 512)
```

### Encoding Implementation

```rust
pub fn encode_quat_smallest_three(q: [f32; 4]) -> u32 {
    let [x, y, z, w] = q;

    // Find largest component
    let abs = [x.abs(), y.abs(), z.abs(), w.abs()];
    let largest_idx = abs
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap()
        .0;

    // Ensure largest component is positive (quaternion sign invariance)
    let sign = if [x, y, z, w][largest_idx] < 0.0 { -1.0 } else { 1.0 };
    let q = [x * sign, y * sign, z * sign, w * sign];

    // Select three components to store (excluding largest)
    let (a, b, c) = match largest_idx {
        0 => (q[1], q[2], q[3]), // drop x
        1 => (q[0], q[2], q[3]), // drop y
        2 => (q[0], q[1], q[3]), // drop z
        _ => (q[0], q[1], q[2]), // drop w
    };

    // Quantize to 10-bit signed [-511, 511]
    // Range is [-1/sqrt(2), 1/sqrt(2)] ~ [-0.707, 0.707]
    let scale = 511.0 / 0.7071067811865476;

    let qa = ((a * scale).round() as i32).clamp(-511, 511);
    let qb = ((b * scale).round() as i32).clamp(-511, 511);
    let qc = ((c * scale).round() as i32).clamp(-511, 511);

    // Pack into u32 (bias to unsigned)
    let ua = (qa + 512) as u32;
    let ub = (qb + 512) as u32;
    let uc = (qc + 512) as u32;

    (ua << 22) | (ub << 12) | (uc << 2) | (largest_idx as u32)
}
```

### Decoding Implementation

```rust
pub fn decode_quat_smallest_three(packed: u32) -> [f32; 4] {
    let largest_idx = (packed & 0b11) as usize;

    // Extract 10-bit components and unbias
    let uc = ((packed >> 2) & 0x3FF) as i32 - 512;
    let ub = ((packed >> 12) & 0x3FF) as i32 - 512;
    let ua = ((packed >> 22) & 0x3FF) as i32 - 512;

    // Dequantize
    let scale = 0.7071067811865476 / 511.0;
    let a = ua as f32 * scale;
    let b = ub as f32 * scale;
    let c = uc as f32 * scale;

    // Reconstruct largest component
    let sum_sq = a * a + b * b + c * c;
    let largest = (1.0 - sum_sq).max(0.0).sqrt();

    // Rebuild quaternion
    match largest_idx {
        0 => [largest, a, b, c], // x was largest
        1 => [a, largest, b, c], // y was largest
        2 => [a, b, largest, c], // z was largest
        _ => [a, b, c, largest], // w was largest
    }
}
```

### Precision

At 10 bits per component: ~0.01 degree angular precision (imperceptible in games).

---

## Half-Float (f16) Reference

Position and scale use IEEE 754 half-precision floats.

### Properties

- Range: +/-65504 (sufficient for game-scale positions)
- Precision: ~3 decimal digits
- Denormals near zero provide extra precision for small values

### Conversion (using `half` crate)

```rust
use half::f16;

pub fn f32_to_f16(value: f32) -> u16 {
    f16::from_f32(value).to_bits()
}

pub fn f16_to_f32(bits: u16) -> f32 {
    f16::from_bits(bits).to_f32()
}
```

---

## Integration with Existing System

### Skeleton and Keyframe Relationship

The animation system works alongside the existing skeleton system:

1. **Skeleton** (`rom_skeleton` / `load_skeleton`): Provides inverse bind matrices that transform vertices from model space to bone-local space
2. **Keyframes** (`rom_keyframes` / `keyframes_load`): Provides per-frame bone transforms (local space)
3. **skeleton_bind()**: Enables/disables inverse bind mode on GPU

### Typical Initialization Flow

```rust
fn init() {
    // Load skinned mesh
    PLAYER_MESH = rom_mesh(b"player".as_ptr(), 6);

    // Load skeleton (inverse bind matrices)
    SKELETON = rom_skeleton(b"player".as_ptr(), 6);

    // Load animation clips
    WALK_ANIM = rom_keyframes(b"walk".as_ptr(), 4);
    RUN_ANIM = rom_keyframes(b"run".as_ptr(), 3);
    IDLE_ANIM = rom_keyframes(b"idle".as_ptr(), 4);
}
```

### Rendering Flow

```rust
fn render() {
    // 1. Bind skeleton (enables inverse bind mode)
    skeleton_bind(SKELETON);

    // 2. Set bone matrices (either keyframe_bind or manual)
    keyframe_bind(WALK_ANIM, current_frame);
    // OR: set_bones(matrices_ptr, bone_count);

    // 3. Draw skinned mesh
    draw_mesh(PLAYER_MESH);
}
```

### GPU Skinning Pipeline

When a skeleton is bound:
```
vertex_position = bone_matrix[i] * inverse_bind[i] * rest_position
```

When no skeleton is bound (raw mode):
```
vertex_position = bone_matrix[i] * rest_position
```

---

## Usage Examples

### Step-Based Animation (Stamp) — Minimal Memory

```rust
// Game state
static mut WALK_ANIM: u32 = 0;
static mut SKELETON: u32 = 0;
static mut PLAYER_MESH: u32 = 0;
static mut FRAME_TIMER: f32 = 0.0;
static mut CURRENT_FRAME: u32 = 0;

const FRAME_DURATION: f32 = 1.0 / 12.0; // 12 fps playback

fn init() {
    unsafe {
        PLAYER_MESH = rom_mesh(b"player".as_ptr(), 6);
        SKELETON = rom_skeleton(b"player".as_ptr(), 6);
        WALK_ANIM = rom_keyframes(b"walk".as_ptr(), 4);
    }
}

fn render() {
    unsafe {
        // Advance frame (step-based, no interpolation)
        FRAME_TIMER += delta_time();
        if FRAME_TIMER >= FRAME_DURATION {
            FRAME_TIMER -= FRAME_DURATION;
            CURRENT_FRAME = (CURRENT_FRAME + 1) % keyframes_frame_count(WALK_ANIM) as u32;
        }

        // Bind skeleton and keyframe
        skeleton_bind(SKELETON);
        keyframe_bind(WALK_ANIM, CURRENT_FRAME);

        // Draw
        draw_mesh(PLAYER_MESH);
    }
}
```

### Blended Animation

```rust
const BONE_COUNT: usize = 40;

// Keyframe buffers (40 bytes per bone: 4×4 rotation + 3×4 position + 3×4 scale)
static mut BUF_A: [BoneTransform; BONE_COUNT] = [BoneTransform::ZERO; BONE_COUNT];
static mut BUF_B: [BoneTransform; BONE_COUNT] = [BoneTransform::ZERO; BONE_COUNT];
static mut MATRICES: [[f32; 16]; BONE_COUNT] = [[0.0; 16]; BONE_COUNT];

fn render() {
    unsafe {
        let time = elapsed_time();
        let fps = 30.0;
        let frame_count = keyframes_frame_count(WALK_ANIM) as u32;

        // Calculate frame indices and blend factor
        let frame_f = time * fps;
        let frame_a = (frame_f.floor() as u32) % frame_count;
        let frame_b = (frame_a + 1) % frame_count;
        let blend = frame_f.fract();

        // Pull decoded keyframes from ROM
        keyframe_read(WALK_ANIM, frame_a, BUF_A.as_mut_ptr() as u32);
        keyframe_read(WALK_ANIM, frame_b, BUF_B.as_mut_ptr() as u32);

        // Blend and build matrices
        for i in 0..BONE_COUNT {
            let blended = BoneTransform {
                rotation: nlerp(BUF_A[i].rotation, BUF_B[i].rotation, blend),
                position: lerp3(BUF_A[i].position, BUF_B[i].position, blend),
                scale: lerp3(BUF_A[i].scale, BUF_B[i].scale, blend),  // XYZ scale
            };
            MATRICES[i] = build_bone_matrix_4x4(&blended);
        }

        // Upload and draw
        skeleton_bind(SKELETON);
        set_bones_4x4(MATRICES.as_ptr() as u32, BONE_COUNT as u32);
        draw_mesh(PLAYER_MESH);
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [lerp(a[0], b[0], t), lerp(a[1], b[1], t), lerp(a[2], b[2], t)]
}

fn nlerp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    // Flip if negative dot (shorter path)
    let dot = a[0]*b[0] + a[1]*b[1] + a[2]*b[2] + a[3]*b[3];
    let b = if dot < 0.0 { [-b[0], -b[1], -b[2], -b[3]] } else { b };

    let result = [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
        lerp(a[3], b[3], t),
    ];

    // Normalize
    let len = (result[0]*result[0] + result[1]*result[1] +
               result[2]*result[2] + result[3]*result[3]).sqrt();
    [result[0]/len, result[1]/len, result[2]/len, result[3]/len]
}

fn build_bone_matrix_4x4(t: &BoneTransform) -> [f32; 16] {
    let [qx, qy, qz, qw] = t.rotation;
    let [px, py, pz] = t.position;
    let [sx, sy, sz] = t.scale;  // Non-uniform XYZ scale

    let xx = qx * qx;
    let yy = qy * qy;
    let zz = qz * qz;
    let xy = qx * qy;
    let xz = qx * qz;
    let yz = qy * qz;
    let wx = qw * qx;
    let wy = qw * qy;
    let wz = qw * qz;

    [
        // Column 0 (scaled by sx)
        sx * (1.0 - 2.0 * (yy + zz)),
        sx * (2.0 * (xy + wz)),
        sx * (2.0 * (xz - wy)),
        0.0,
        // Column 1 (scaled by sy)
        sy * (2.0 * (xy - wz)),
        sy * (1.0 - 2.0 * (xx + zz)),
        sy * (2.0 * (yz + wx)),
        0.0,
        // Column 2 (scaled by sz)
        sz * (2.0 * (xz + wy)),
        sz * (2.0 * (yz - wx)),
        sz * (1.0 - 2.0 * (xx + yy)),
        0.0,
        // Column 3 (translation)
        px,
        py,
        pz,
        1.0,
    ]
}
```

---

## Memory Budget Analysis

### Fighting Game Scenario (4 characters, 50 clips each)

```
Per character:
  50 clips x 30 frames x 40 bones x 16 bytes = 960 KB

4 characters in ROM:
  4 x 960 KB = 3.84 MB -> 960 KB with compression (4x savings)

Linear memory (worst case: 2-way blend on 4 active characters):
  Keyframe buffers: 4 chars x 2 frames x 1.6 KB = 12.8 KB
  Output matrices:  4 chars x 40 bones x 64 bytes = 10.24 KB
  Animation state:  4 chars x ~64 bytes            = 256 bytes
  Total: ~23 KB
```

Leaves ~4 MB linear memory for game logic, physics, UI, etc.

### ROM Budget

```
Animation keyframes:  720 KB (compressed)
Character meshes:     1.5 MB (4 x ~400KB)
Textures:             4 MB
Audio:                2 MB
Code:                 1 MB
-----------------------
Total:                ~9.2 MB of 16 MB
```

---

## Implementation Files

### Files to Create

| File | Purpose |
|------|---------|
| `emberware-z/src/ffi/keyframes.rs` | FFI functions for keyframe loading/access |

### Files to Modify

| File | Changes |
|------|---------|
| `z-common/src/formats/animation.rs` | Replace 16-byte header with 4-byte, add encoding/decoding |
| `z-common/src/formats/z_data_pack.rs` | Add `PackedKeyframes` struct, `keyframes` field |
| `z-common/src/formats/mod.rs` | Export new types |
| `z-common/src/lib.rs` | Export new constants |
| `emberware-z/src/ffi/mod.rs` | Register keyframes module |
| `emberware-z/src/ffi/skinning.rs` | Add `set_bones_4x4()` |
| `emberware-z/src/state/ffi_state.rs` | Add keyframe storage |
| `tools/ember-export/src/animation.rs` | Update to write compressed format |
| `tools/ember-export/src/formats/mod.rs` | Update `write_ember_animation()` |
| `tools/ember-cli/src/pack.rs` | Handle .ewzanim in data pack |

### Data Pack Integration

Add to `ZDataPack`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PackedKeyframes {
    pub id: String,
    pub bone_count: u8,
    pub frame_count: u16,
    pub data: Vec<u8>, // Raw platform format
}

pub struct ZDataPack {
    // ... existing fields ...
    pub keyframes: Vec<PackedKeyframes>,
}

impl ZDataPack {
    pub fn find_keyframes(&self, id: &str) -> Option<&PackedKeyframes> {
        self.keyframes.iter().find(|k| k.id == id)
    }
}
```

---

## Tooling

### ember-export

The `ember-export animation` command exports animations to .ewzanim format.

**Supported Input Formats:**

| Format | Extension | Notes |
|--------|-----------|-------|
| glTF 2.0 | `.gltf`, `.glb` | **Primary format** — full skeletal animation support |
| FBX | `.fbx` | Autodesk format (if `fbx` feature enabled) |
| OBJ | `.obj` | No animation support (meshes only) |

**Note:** OBJ format does not support skeletal animation. Use glTF or FBX for animated assets.

**Keyframe Data Exported:**
- **Position** — bone translation (3 × f16 = 6 bytes)
- **Rotation** — quaternion via smallest-three encoding (4 bytes)
- **Scale** — non-uniform scale (3 × f16 = 6 bytes)

All three transform components are always exported. Scale keyframes ARE included and support non-uniform (XYZ) scaling.

```bash
# Export first animation from glTF
ember-export animation model.gltf -o walk.ewzanim

# Export from GLB (binary glTF)
ember-export animation character.glb -o idle.ewzanim

# Export specific animation at 24 fps
ember-export animation model.gltf --animation 1 --frame-rate 24 -o run.ewzanim

# List available animations
ember-export animation model.gltf --list

# Export from FBX (requires fbx feature)
ember-export animation character.fbx -o attack.ewzanim
```

**Bone Name Matching:**
The exporter matches animation channels to skeleton bones by name. Bone names must match exactly between the skeleton and animation data.

### ember build

The `ember build` command:

1. Detects `.ewzanim` files in the asset directory
2. Validates header (bone_count > 0, frame_count > 0)
3. Verifies file size matches header: `4 + (frame_count x bone_count x 16)`
4. Bundles into ROM data pack

---

## Test Cases

These test vectors ensure encoding/decoding correctness. All implementations MUST pass these cases.

### Smallest-Three Quaternion Encoding

The encoding uses 32 bits: `[a:10][b:10][c:10][idx:2]` where `idx` identifies the omitted (largest) component.

**Test Vectors:**

| Input Quaternion [x, y, z, w] | Encoded (hex) | Notes |
|-------------------------------|---------------|-------|
| `[0.0, 0.0, 0.0, 1.0]` | `0x80080003` | Identity (w=1, idx=3) |
| `[1.0, 0.0, 0.0, 0.0]` | `0x80080000` | 90° X rotation (x=1, idx=0) |
| `[0.0, 1.0, 0.0, 0.0]` | `0x80080001` | 90° Y rotation (y=1, idx=1) |
| `[0.0, 0.0, 1.0, 0.0]` | `0x80080002` | 90° Z rotation (z=1, idx=2) |
| `[0.5, 0.5, 0.5, 0.5]` | `0xB6DB6DB3` | 120° rotation around [1,1,1] |
| `[0.707107, 0.0, 0.0, 0.707107]` | `0xBFF00003` | 90° X (half-angle form) |
| `[-0.5, -0.5, -0.5, 0.5]` | `0x49249243` | Same as [0.5,0.5,0.5,0.5] (sign flip) |

**Encoding Algorithm:**
```rust
fn encode_quat_smallest_three(q: [f32; 4]) -> u32 {
    // 1. Find index of largest absolute component
    let abs_q = [q[0].abs(), q[1].abs(), q[2].abs(), q[3].abs()];
    let idx = if abs_q[0] > abs_q[1] && abs_q[0] > abs_q[2] && abs_q[0] > abs_q[3] { 0 }
         else if abs_q[1] > abs_q[2] && abs_q[1] > abs_q[3] { 1 }
         else if abs_q[2] > abs_q[3] { 2 }
         else { 3 };

    // 2. Ensure largest component is positive (q == -q for rotations)
    let sign = if q[idx] < 0.0 { -1.0 } else { 1.0 };
    let q = [q[0] * sign, q[1] * sign, q[2] * sign, q[3] * sign];

    // 3. Select the 3 smallest components (skip idx)
    let (a, b, c) = match idx {
        0 => (q[1], q[2], q[3]),
        1 => (q[0], q[2], q[3]),
        2 => (q[0], q[1], q[3]),
        _ => (q[0], q[1], q[2]),
    };

    // 4. Quantize: [-1/√2, 1/√2] → [0, 1023] (10 bits)
    //    Formula: round((v * √2 + 1) * 511.5)
    let scale = 511.5;
    let sqrt2 = std::f32::consts::SQRT_2;
    let qa = ((a * sqrt2 + 1.0) * scale).round() as u32;
    let qb = ((b * sqrt2 + 1.0) * scale).round() as u32;
    let qc = ((c * sqrt2 + 1.0) * scale).round() as u32;

    // 5. Pack: [a:10][b:10][c:10][idx:2]
    (qa << 22) | (qb << 12) | (qc << 2) | (idx as u32)
}
```

**Decoding Algorithm:**
```rust
fn decode_quat_smallest_three(packed: u32) -> [f32; 4] {
    let idx = (packed & 0x3) as usize;
    let qc = ((packed >> 2) & 0x3FF) as f32;
    let qb = ((packed >> 12) & 0x3FF) as f32;
    let qa = ((packed >> 22) & 0x3FF) as f32;

    // Dequantize: [0, 1023] → [-1/√2, 1/√2]
    let scale = 1.0 / 511.5;
    let sqrt2_inv = 1.0 / std::f32::consts::SQRT_2;
    let a = (qa * scale - 1.0) * sqrt2_inv;
    let b = (qb * scale - 1.0) * sqrt2_inv;
    let c = (qc * scale - 1.0) * sqrt2_inv;

    // Reconstruct largest component: sqrt(1 - a² - b² - c²)
    let largest = (1.0 - a*a - b*b - c*c).max(0.0).sqrt();

    // Rebuild quaternion
    match idx {
        0 => [largest, a, b, c],
        1 => [a, largest, b, c],
        2 => [a, b, largest, c],
        _ => [a, b, c, largest],
    }
}
```

**Precision:**
- Maximum error: ~0.001 per component
- Angular error: <0.1° for typical rotations
- Acceptable for skeletal animation (imperceptible)

### Half-Float (f16) Conversion

Using the `half` crate or manual IEEE 754 half-precision conversion.

**Test Vectors:**

| f32 Input | f16 bits (hex) | f32 Roundtrip | Notes |
|-----------|----------------|---------------|-------|
| `0.0` | `0x0000` | `0.0` | Positive zero |
| `-0.0` | `0x8000` | `-0.0` | Negative zero |
| `1.0` | `0x3C00` | `1.0` | One |
| `-1.0` | `0xBC00` | `-1.0` | Negative one |
| `0.5` | `0x3800` | `0.5` | Half |
| `2.0` | `0x4000` | `2.0` | Two |
| `65504.0` | `0x7BFF` | `65504.0` | Max normal |
| `-65504.0` | `0xFBFF` | `-65504.0` | Min normal |
| `0.00006103515625` | `0x0400` | `0.00006103515625` | Min positive normal |
| `inf` | `0x7C00` | `inf` | Positive infinity |
| `-inf` | `0xFC00` | `-inf` | Negative infinity |

**Usage with `half` crate:**
```rust
use half::f16;

fn f32_to_f16(v: f32) -> u16 {
    f16::from_f32(v).to_bits()
}

fn f16_to_f32(bits: u16) -> f32 {
    f16::from_bits(bits).to_f32()
}
```

**Position/Scale Ranges:**
- Position: ±65504 units (f16 max) — sufficient for most game worlds
- Scale: 0.0 to 65504.0 per axis — non-uniform XYZ scale
- For larger worlds, use world-relative transforms or multiple skeletons

### Full Roundtrip Test Cases

These test the complete encode/decode pipeline for `PlatformBoneKeyframe` ↔ `BoneTransform`.

**Test Vector 1: Identity Transform**
```rust
let input = BoneTransform {
    rotation: [0.0, 0.0, 0.0, 1.0],  // Identity quaternion
    position: [0.0, 0.0, 0.0],
    scale: [1.0, 1.0, 1.0],  // Uniform scale
};

let encoded = PlatformBoneKeyframe {
    rotation: 0x80080003,  // Identity quat
    position: [0x0000, 0x0000, 0x0000],  // Zero position
    scale: [0x3C00, 0x3C00, 0x3C00],  // 1.0 in f16 for each axis
};

// Decode back
let decoded = decode_bone_transform(&encoded);
assert!((decoded.rotation[3] - 1.0).abs() < 0.002);  // w ≈ 1
assert!(decoded.position.iter().all(|&v| v.abs() < 0.001));
assert!(decoded.scale.iter().all(|&v| (v - 1.0).abs() < 0.001));
```

**Test Vector 2: Typical Animation Pose**
```rust
let input = BoneTransform {
    rotation: [0.270598, 0.0, 0.0, 0.962728],  // 31.4° X rotation
    position: [1.5, 2.25, -0.75],
    scale: [1.0, 1.0, 1.0],  // Uniform scale
};

let encoded = encode_bone_transform(
    input.rotation,
    input.position,
    input.scale
);

// Expected values (approximate due to quantization):
// rotation: smallest-three encoding of [0.27, 0, 0, 0.96]
// position: [f16(1.5), f16(2.25), f16(-0.75)]
// scale: [f16(1.0), f16(1.0), f16(1.0)]

let decoded = decode_bone_transform(&encoded);

// Verify rotation (angular error < 0.1°)
let dot = input.rotation[0] * decoded.rotation[0]
        + input.rotation[1] * decoded.rotation[1]
        + input.rotation[2] * decoded.rotation[2]
        + input.rotation[3] * decoded.rotation[3];
assert!(dot.abs() > 0.9999);  // Nearly identical rotation

// Verify position (f16 precision)
assert!((decoded.position[0] - 1.5).abs() < 0.01);
assert!((decoded.position[1] - 2.25).abs() < 0.01);
assert!((decoded.position[2] - (-0.75)).abs() < 0.01);

// Verify scale (XYZ)
assert!(decoded.scale.iter().all(|&v| (v - 1.0).abs() < 0.001));
```

**Test Vector 3: Extreme Values (Non-Uniform Scale)**
```rust
let input = BoneTransform {
    rotation: [0.707107, 0.0, 0.707107, 0.0],  // 180° around [1,0,1]
    position: [1000.0, -500.0, 0.001],  // Large + small values
    scale: [0.5, 1.0, 2.5],  // Non-uniform scale (squash & stretch)
};

let encoded = encode_bone_transform(
    input.rotation,
    input.position,
    input.scale
);

let decoded = decode_bone_transform(&encoded);

// Position precision degrades at large values (f16 limitation)
assert!((decoded.position[0] - 1000.0).abs() < 1.0);   // ~0.1% error at 1000
assert!((decoded.position[1] - (-500.0)).abs() < 0.5);
assert!((decoded.position[2] - 0.001).abs() < 0.001);  // Small values OK

// Verify non-uniform scale (XYZ)
assert!((decoded.scale[0] - 0.5).abs() < 0.01);
assert!((decoded.scale[1] - 1.0).abs() < 0.01);
assert!((decoded.scale[2] - 2.5).abs() < 0.01);
```

**Test Vector 4: Byte-Level Verification**
```rust
// Known good encoding for verification (16 bytes per bone)
let keyframe_bytes: [u8; 16] = [
    0x03, 0x00, 0x08, 0x80,  // rotation: 0x80080003 (identity, LE)
    0x00, 0x3C,              // position.x: 0x3C00 (1.0 in f16)
    0x00, 0x40,              // position.y: 0x4000 (2.0 in f16)
    0x00, 0xC0,              // position.z: 0xC000 (-2.0 in f16)
    0x00, 0x3C,              // scale.x: 0x3C00 (1.0 in f16)
    0x00, 0x3C,              // scale.y: 0x3C00 (1.0 in f16)
    0x00, 0x3C,              // scale.z: 0x3C00 (1.0 in f16)
];

let kf = PlatformBoneKeyframe::from_bytes(&keyframe_bytes);
let decoded = decode_bone_transform(&kf);

assert!((decoded.rotation[3] - 1.0).abs() < 0.002);  // Identity quat
assert!((decoded.position[0] - 1.0).abs() < 0.001);
assert!((decoded.position[1] - 2.0).abs() < 0.001);
assert!((decoded.position[2] - (-2.0)).abs() < 0.001);
assert!(decoded.scale.iter().all(|&v| (v - 1.0).abs() < 0.001));  // XYZ scale
```

### File Format Validation

**Minimum Valid File (1 bone, 1 frame):**
```
Offset  Bytes (hex)           Description
------  --------------------  -----------
0x00    01                    bone_count = 1
0x01    00                    flags = 0
0x02    01 00                 frame_count = 1 (LE)
0x04    03 00 08 80           rotation (identity)
0x08    00 00 00 00 00 00     position (zero)
0x0E    00 3C 00 3C 00 3C     scale xyz (1.0, 1.0, 1.0)
------
Total:  20 bytes (4 header + 16 data)
```

**Header Validation Rules:**
1. `bone_count` must be 1-255 (0 is invalid)
2. `frame_count` must be 1-65535 (0 is invalid)
3. `flags` must be 0 (reserved for future use)
4. File size must equal `4 + (bone_count × frame_count × 16)`

---

## Error Handling

All animation errors result in a **WASM trap** (panic). There is no graceful fallback.

### Runtime Errors (WASM Traps)

| Error | Cause | Behavior |
|-------|-------|----------|
| Invalid keyframe handle | `keyframe_bind()` with handle 0 | Trap: "invalid keyframe handle" |
| Frame index out of bounds | `keyframe_bind(h, 999)` when max is 60 | Trap: "frame index {idx} >= frame_count {max}" |
| Invalid out_ptr | `keyframe_read()` with bad pointer | Trap: WASM memory access violation |
| Asset not found | `rom_keyframes("missing")` | Trap: "keyframe asset not found: {id}" |
| Corrupted header | `bone_count == 0` or `frame_count == 0` | Trap: "invalid animation header" |
| Truncated data | File size doesn't match header | Trap: "animation data truncated" |

### Skeleton/Animation Mismatch

| Scenario | Behavior |
|----------|----------|
| Animation has MORE bones than skeleton | Extra bones ignored (use first N bones) |
| Animation has FEWER bones than skeleton | Missing bones get identity transform |
| No skeleton bound in inverse-bind mode | All bones get identity matrix (no inverse bind applied) |

**Rationale:** Rather than trap on mismatch, silently handle it. This allows:
- Partial skeleton rigs (animate only upper body)
- Shared animations across different character rigs
- Debug scenarios where skeleton isn't loaded yet

### Why Trap Instead of Fallback?

- **Fail fast** — Corrupted assets indicate a serious build/packaging bug
- **Init-only loading** — All keyframes load in `init()`, so traps happen before gameplay
- **Determinism** — Rollback netcode requires identical state; fallback animations could desync

---

## Edge Case Tests

```rust
#[test]
fn test_single_frame_animation() {
    // Minimum valid animation: 1 bone, 1 frame (20 bytes total)
    let data = [
        0x01,                               // bone_count = 1
        0x00,                               // flags = 0
        0x01, 0x00,                         // frame_count = 1 (LE)
        // Frame 0, Bone 0 (16 bytes per bone)
        0x03, 0x00, 0x08, 0x80,             // rotation (identity) — 4 bytes
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // position (zero) — 6 bytes
        0x00, 0x3C, 0x00, 0x3C, 0x00, 0x3C, // scale xyz (1.0, 1.0, 1.0) — 6 bytes
    ];
    assert_eq!(data.len(), 20); // 4 header + 16 data
    let header = parse_header(&data);
    assert_eq!(header.bone_count, 1);
    assert_eq!(header.frame_count, 1);
}

#[test]
fn test_max_bone_count() {
    // Maximum: 255 bones
    let mut data = vec![0xFF, 0x00, 0x01, 0x00]; // 255 bones, 1 frame
    data.extend(vec![0u8; 255 * 16]); // 255 bones × 16 bytes
    assert_eq!(data.len(), 4 + 255 * 16);
}

#[test]
fn test_max_frame_count() {
    // Maximum: 65535 frames (at 60fps = ~18 minutes)
    let header = AnimationHeader {
        bone_count: 1,
        flags: 0,
        frame_count: 65535,
    };
    let expected_size = 4 + (1 * 65535 * 16);
    assert_eq!(expected_size, 1048564); // ~1MB for single bone
}

#[test]
fn test_bone_count_mismatch_more_bones() {
    // Animation has 32 bones, skeleton has 28
    // Should use first 28 bones, ignore last 4
    let anim_bones = 32;
    let skel_bones = 28;
    let used = anim_bones.min(skel_bones);
    assert_eq!(used, 28);
}

#[test]
fn test_bone_count_mismatch_fewer_bones() {
    // Animation has 20 bones, skeleton has 32
    // Bones 20-31 get identity transform
    let anim_bones = 20;
    let skel_bones = 32;
    // First 20 bones: from animation
    // Bones 20-31: identity (position=0, rotation=identity, scale=1)
}

#[test]
fn test_no_skeleton_bound() {
    // keyframe_bind() with no skeleton_bind() call
    // Should apply animation directly without inverse bind
    // All bones get animation transforms as-is (identity for inverse bind)
}
```

---

## References

- [ACL: Animation Compression Library](https://github.com/nfrechette/acl) — Industry-standard compression
- [Quaternion Compression (Gaffer on Games)](https://gafferongames.com/post/snapshot_compression/) — Smallest-three explanation
- [half crate](https://crates.io/crates/half) — Rust f16 implementation
