# Emberware Z Animation System Specification

**Status:** Proposal / Draft  
**Author:** Zerve  
**Last Updated:** December 2025

---

## Executive Summary

This document proposes a ROM-backed animation system for Emberware Z that keeps animation data out of linear memory while providing flexibility for both simple "stamp" animations and complex blended animation systems.

The core insight: **animation keyframes belong in ROM, not linear memory**. With 4MB linear memory and potentially 3MB+ of animation data per character, storing keyframes in WASM memory is infeasible for content-rich games like fighting games. Instead, the host stores animation clips and provides APIs to either directly apply keyframes (stamp mode) or pull keyframes into small working buffers for blending.

---

## Design Goals

1. **Memory efficiency** — Animation data lives in ROM (12MB), not linear memory (4MB)
2. **Agnostic to animation approach** — Support FK, IK, procedural, blend trees, state machines
3. **Zero-copy fast path** — Stamp animations bypass WASM entirely
4. **Developer flexibility** — Platform format for convenience, raw data API for custom needs
5. **Rollback-friendly** — Only small working buffers in linear memory get snapshotted

---

## Memory Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              ROM (12MB)                                 │
│  ┌───────────────┐ ┌───────────────┐ ┌───────────────┐ ┌─────────────┐  │
│  │ Character A   │ │ Character B   │ │ Meshes        │ │ Textures    │  │
│  │ Clips (~720KB)│ │ Clips (~720KB)│ │ (~2MB)        │ │ (~4MB)      │  │
│  └───────┬───────┘ └───────────────┘ └───────────────┘ └─────────────┘  │
│          │                                                              │
└──────────┼──────────────────────────────────────────────────────────────┘
           │
           │ read_frame() - on demand
           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        WASM Linear Memory (4MB)                         │
│  ┌─────────────────┐ ┌─────────────────┐ ┌───────────────────────────┐  │
│  │ Keyframe Buf A  │ │ Keyframe Buf B  │ │ Output Bone Matrices      │  │
│  │ (~480 bytes)    │ │ (~480 bytes)    │ │ (~1.9KB per character)    │  │
│  └─────────────────┘ └─────────────────┘ └─────────────────────────────┘  │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                    Game State (~4MB available)                  │    │
│  │   Physics, UI, Game Logic, Entity State, etc.                   │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
           │
           │ set_bones() - per frame
           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                              GPU (VRAM)                                 │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ Bone Matrix Uniform Buffer (256 bones × 48 bytes = 12KB max)    │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## API Overview

### Tier 1: Stamp Animation (Zero WASM Memory)

For games that don't need blending—frame-exact animation like PS1-era 3D or step-based keyframes.

```
ROM ──── set_bones_from_clip() ────▶ GPU
              (host unpacks internally)
```

### Tier 2: Blended Animation (Minimal WASM Memory)

For games needing interpolation, blend trees, or animation mixing.

```
ROM ──── read_frame() ────▶ WASM ──── blend/process ────▶ WASM ──── set_bones() ────▶ GPU
         (pull keyframes)            (developer code)              (upload matrices)
```

### Tier 3: Custom Format (Full Control)

For developers who want their own keyframe format, compression, or exotic animation systems.

```
ROM ──── read_data() ────▶ WASM ──── custom decode ────▶ WASM ──── set_bones() ────▶ GPU
         (raw bytes)               (developer code)               (upload matrices)
```

---

## Proposed FFI Functions

> **Open Question:** Function naming conventions. Options include:
> - `clip_*` prefix: `clip_load`, `clip_info`, `clip_read_frame`
> - `anim_*` prefix: `anim_load`, `anim_info`, `anim_read_frame`  
> - `animation_*` prefix: `animation_load`, `animation_info`
> - Mixed: `load_clip`, `read_frame`, `set_bones_from_clip`

### Clip Management

```rust
/// Load animation clip in platform format
/// 
/// # Arguments
/// * `data_ptr` — Pointer to clip data (embedded via include_bytes!)
/// * `byte_size` — Total size of clip data
///
/// # Returns
/// Clip handle (0 = invalid/error)
///
/// The clip data remains on the host side, not copied to linear memory.
fn load_clip(data_ptr: *const u8, byte_size: u32) -> u32;

/// Query clip metadata
///
/// # Arguments
/// * `handle` — Clip handle from load_clip()
/// * `out_ptr` — Pointer to write ClipInfo struct
///
/// # ClipInfo Layout (12 bytes)
/// ```
/// struct ClipInfo {
///     bone_count: u16,    // Number of bones per frame
///     frame_count: u16,   // Total frames in clip
///     fps: f32,           // Intended playback rate
///     flags: u32,         // Reserved for future use
/// }
/// ```
fn clip_info(handle: u32, out_ptr: *mut u8);

/// Unload clip and free host memory
fn unload_clip(handle: u32);
```

### Stamp Mode

```rust
/// Set bone matrices directly from a clip frame
///
/// Host unpacks the keyframe and uploads matrices to GPU.
/// No WASM memory used. Perfect for step-based animation.
///
/// # Arguments
/// * `handle` — Clip handle
/// * `frame` — Frame index (0-based, wraps if out of range)
fn set_bones_from_clip(handle: u32, frame: u32);
```

> **Open Question:** Should `frame` wrap automatically, clamp, or return an error?  
> Wrapping is convenient for looping animations. Clamping is safer for one-shots.

### Blend Mode

```rust
/// Read a keyframe into WASM memory for processing
///
/// # Arguments
/// * `handle` — Clip handle
/// * `frame` — Frame index
/// * `out_ptr` — Destination in WASM memory
///
/// Writes `bone_count × BYTES_PER_BONE` bytes in platform keyframe format.
/// Developer unpacks and processes (blend, IK, etc.) then calls set_bones().
fn read_frame(handle: u32, frame: u32, out_ptr: *mut u8);

/// Read multiple sequential frames (batch prefetch)
///
/// More efficient than multiple read_frame() calls for predictable playback.
///
/// # Arguments
/// * `handle` — Clip handle  
/// * `frame_start` — First frame index
/// * `frame_count` — Number of frames to read
/// * `out_ptr` — Destination buffer (must fit frame_count × bone_count × BYTES_PER_BONE)
fn read_frames(handle: u32, frame_start: u32, frame_count: u32, out_ptr: *mut u8);

/// Upload bone matrices to GPU
///
/// # Arguments
/// * `matrices_ptr` — Pointer to bone matrices in WASM memory
/// * `count` — Number of bones
///
/// Matrix format: 3×4 row-major f32 (48 bytes per bone)
/// Or: 4×4 column-major f32 (64 bytes per bone) — see open question
fn set_bones(matrices_ptr: *const f32, count: u32);
```

> **Open Question:** Matrix format for `set_bones()`:
> - **4×4 f32 (64 bytes)** — Current implementation, simple but wasteful
> - **3×4 f32 (48 bytes)** — Sufficient for affine transforms, 25% smaller
> - **3×4 f16 (24 bytes)** — 62% smaller, but precision concerns far from origin
> 
> Recommendation: Support both via separate functions or a format flag.

### Raw Data API (Power Users)

```rust
/// Load raw binary data with fixed chunk size
///
/// For custom animation formats, procedural data, or any chunked ROM data.
/// Host stores data but doesn't interpret it.
///
/// # Arguments
/// * `data_ptr` — Pointer to raw data
/// * `byte_size` — Total size
/// * `chunk_size` — Size of each indexable chunk (0 = treat as single blob)
///
/// # Returns
/// Data handle
fn load_data(data_ptr: *const u8, byte_size: u32, chunk_size: u32) -> u32;

/// Read a chunk of raw data
///
/// # Arguments
/// * `handle` — Data handle
/// * `index` — Chunk index (0-based)
/// * `out_ptr` — Destination in WASM memory
fn read_data(handle: u32, index: u32, out_ptr: *mut u8);

/// Unload raw data
fn unload_data(handle: u32);
```

---

## Platform Keyframe Format

The platform defines a packed keyframe format for clips loaded via `load_clip()`. This format balances compression with decode simplicity.

### Proposed Format: 12 Bytes Per Bone

```rust
/// Platform keyframe format (12 bytes per bone)
#[repr(C, packed)]
struct PlatformBoneKeyframe {
    /// Rotation as smallest-three packed quaternion
    rotation: u32,      // 4 bytes
    
    /// Position as half-float xyz  
    position: [u16; 3], // 6 bytes (f16 × 3)
    
    /// Uniform scale as half-float
    scale: u16,         // 2 bytes (f16)
}

// Total: 12 bytes per bone
// 40 bones × 12 bytes = 480 bytes per frame
// 60 frames × 480 bytes = 28.8 KB per clip
// 50 clips × 28.8 KB = 1.44 MB per character
```

### Alternative Formats Considered

| Format | Bytes/Bone | Pros | Cons |
|--------|-----------|------|------|
| **A: 12 bytes** (proposed) | 12 | Good compression, simple decode | Uniform scale only |
| **B: 10 bytes** (no scale) | 10 | Smallest practical | No scale support |
| **C: 16 bytes** (full scale) | 16 | Non-uniform scale | 33% larger |
| **D: 8 bytes** (aggressive) | 8 | Tiny | Position precision loss |
| **E: 20 bytes** (f32 pos) | 20 | Full position precision | Less compression |

#### Format A: 12 Bytes (Recommended)

```rust
struct FormatA {
    rotation: u32,      // Smallest-three packed
    position: [f16; 3], // Half-float position
    scale: f16,         // Uniform scale
}
```

#### Format B: 10 Bytes (No Scale)

```rust
struct FormatB {
    rotation: u32,      // Smallest-three packed
    position: [f16; 3], // Half-float position
    // Scale assumed 1.0
}
```

Best for games where bones never scale. Saves 16% vs Format A.

#### Format C: 16 Bytes (Full Scale)

```rust
struct FormatC {
    rotation: u32,      // Smallest-three packed
    position: [f16; 3], // Half-float position  
    scale: [f16; 3],    // Non-uniform scale
}
```

For squash-and-stretch, cartoony animation. 33% larger than Format A.

#### Format D: 8 Bytes (Aggressive)

```rust
struct FormatD {
    rotation: u32,           // Smallest-three packed
    position_xy: u32,        // f16 x, f16 y
    // Z reconstructed from bone length constraint or stored elsewhere
}
```

Experimental. Requires assumptions about skeleton structure.

> **Open Question:** Should we support multiple formats via a format flag in the clip header? Or commit to one format platform-wide?
>
> Single format is simpler for tooling and documentation. Multiple formats add flexibility but complexity.

---

## Quaternion Encoding: Smallest-Three

The "smallest-three" encoding is industry standard (used by Unreal, Unity, ACL).

### Concept

A unit quaternion has constraint: `x² + y² + z² + w² = 1`

Therefore, we can reconstruct one component from the other three:
```
missing = sqrt(1 - a² - b² - c²)
```

We drop the component with the largest absolute value (provides best precision) and store a 2-bit index indicating which was dropped.

### Encoding (32 bits total)

```
Bit layout:
[31:22] x component (10 bits, signed)
[21:12] y component (10 bits, signed)
[11:2]  z component (10 bits, signed)
[1:0]   index of largest component (2 bits)

Each component mapped: [-1/√2, 1/√2] → [-511, 511]
(The dropped component is always ≥ 1/√2 in magnitude)
```

### Encoding Implementation

```rust
/// Encode quaternion to smallest-three format
fn encode_quat_smallest_three(q: [f32; 4]) -> u32 {
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
    // Range is [-1/√2, 1/√2] ≈ [-0.707, 0.707]
    let scale = 511.0 / 0.7071067811865476; // 511 / (1/√2)
    
    let qa = ((a * scale).round() as i32).clamp(-511, 511);
    let qb = ((b * scale).round() as i32).clamp(-511, 511);
    let qc = ((c * scale).round() as i32).clamp(-511, 511);
    
    // Pack into u32
    let ua = (qa + 512) as u32; // Bias to unsigned [1, 1023]
    let ub = (qb + 512) as u32;
    let uc = (qc + 512) as u32;
    
    (ua << 22) | (ub << 12) | (uc << 2) | (largest_idx as u32)
}
```

### Decoding Implementation

```rust
/// Decode smallest-three quaternion to [x, y, z, w]
fn decode_quat_smallest_three(packed: u32) -> [f32; 4] {
    let largest_idx = (packed & 0b11) as usize;
    
    // Extract 10-bit components
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
        0 => [largest, a, b, c],       // x was largest
        1 => [a, largest, b, c],       // y was largest
        2 => [a, b, largest, c],       // z was largest
        _ => [a, b, c, largest],       // w was largest
    }
}
```

### Precision Analysis

At 10 bits per component:
- Angular precision: ~0.01° worst case
- Imperceptible in real-time animation
- Comparable to what AAA engines use

For higher precision (if needed):
- 16 bits per component in 48-bit format: ~0.0001° precision
- Overkill for games, useful for cinematics

---

## Half-Float (f16) Reference

Position and scale use IEEE 754 half-precision floats.

### Properties

- Range: ±65504 (sufficient for game-scale positions)
- Precision: ~3 decimal digits
- Denormals near zero provide extra precision for small values

### Conversion (Rust)

Using the `half` crate:

```rust
use half::f16;

fn encode_f16(value: f32) -> u16 {
    f16::from_f32(value).to_bits()
}

fn decode_f16(bits: u16) -> f32 {
    f16::from_bits(bits).to_f32()
}
```

### Manual Implementation (for WASM/no_std)

```rust
/// Convert f32 to f16 bits (simplified, handles common cases)
fn f32_to_f16_bits(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = (bits >> 16) & 0x8000;
    let exp = ((bits >> 23) & 0xFF) as i32 - 127 + 15;
    let mantissa = bits & 0x7FFFFF;
    
    if exp <= 0 {
        // Denormal or zero
        if exp < -10 {
            return sign as u16; // Too small, return signed zero
        }
        let m = (mantissa | 0x800000) >> (1 - exp);
        return (sign | (m >> 13)) as u16;
    } else if exp >= 31 {
        // Overflow to infinity
        return (sign | 0x7C00) as u16;
    }
    
    (sign | ((exp as u32) << 10) | (mantissa >> 13)) as u16
}

/// Convert f16 bits to f32
fn f16_bits_to_f32(bits: u16) -> f32 {
    let sign = ((bits & 0x8000) as u32) << 16;
    let exp = (bits >> 10) & 0x1F;
    let mantissa = (bits & 0x3FF) as u32;
    
    let result = if exp == 0 {
        if mantissa == 0 {
            sign // Zero
        } else {
            // Denormal
            let mut m = mantissa;
            let mut e = 1u32;
            while (m & 0x400) == 0 {
                m <<= 1;
                e += 1;
            }
            let exp32 = (127 - 15 - e + 1) << 23;
            sign | exp32 | ((m & 0x3FF) << 13)
        }
    } else if exp == 31 {
        sign | 0x7F800000 | (mantissa << 13) // Inf or NaN
    } else {
        let exp32 = ((exp as u32) - 15 + 127) << 23;
        sign | exp32 | (mantissa << 13)
    };
    
    f32::from_bits(result)
}
```

---

## Complete Keyframe Decode Example

```rust
/// Decode platform keyframe to transform components
fn decode_keyframe(data: &[u8; 12]) -> BoneTransform {
    // Bytes 0-3: rotation (smallest-three)
    let rot_packed = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let rotation = decode_quat_smallest_three(rot_packed);
    
    // Bytes 4-9: position (3 × f16)
    let px = f16_bits_to_f32(u16::from_le_bytes([data[4], data[5]]));
    let py = f16_bits_to_f32(u16::from_le_bytes([data[6], data[7]]));
    let pz = f16_bits_to_f32(u16::from_le_bytes([data[8], data[9]]));
    
    // Bytes 10-11: scale (f16)
    let scale = f16_bits_to_f32(u16::from_le_bytes([data[10], data[11]]));
    
    BoneTransform {
        rotation,
        position: [px, py, pz],
        scale,
    }
}

struct BoneTransform {
    rotation: [f32; 4],  // Quaternion [x, y, z, w]
    position: [f32; 3],  // Translation
    scale: f32,          // Uniform scale
}
```

---

## Building Bone Matrices

After decoding keyframes and blending, the developer must build matrices for `set_bones()`.

### Quaternion + Position + Scale → 3×4 Matrix

```rust
/// Build 3×4 affine matrix from transform components
/// Output is row-major: [[r00, r01, r02, tx], [r10, r11, r12, ty], [r20, r21, r22, tz]]
fn build_bone_matrix(t: &BoneTransform) -> [f32; 12] {
    let [qx, qy, qz, qw] = t.rotation;
    let [px, py, pz] = t.position;
    let s = t.scale;
    
    // Rotation matrix from quaternion (scaled)
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
        // Row 0
        s * (1.0 - 2.0 * (yy + zz)),
        s * (2.0 * (xy - wz)),
        s * (2.0 * (xz + wy)),
        px,
        // Row 1
        s * (2.0 * (xy + wz)),
        s * (1.0 - 2.0 * (xx + zz)),
        s * (2.0 * (yz - wx)),
        py,
        // Row 2
        s * (2.0 * (xz - wy)),
        s * (2.0 * (yz + wx)),
        s * (1.0 - 2.0 * (xx + yy)),
        pz,
    ]
}
```

> **Open Question:** Row-major vs column-major for `set_bones()` input?
> 
> Current `set_bones()` uses column-major 4×4. Options:
> 1. Keep column-major 4×4 (64 bytes) — compatible with existing code
> 2. Add `set_bones_3x4()` for row-major 3×4 (48 bytes) — new efficient path
> 3. Switch entirely to 3×4 — breaking change

---

## Animation Blending Example

Typical frame update for a blended animation system:

```rust
// Game state
static mut ANIM_STATE: AnimState = AnimState::new();
static mut KEYFRAME_BUF_A: [u8; 480] = [0u8; 480]; // 40 bones × 12 bytes
static mut KEYFRAME_BUF_B: [u8; 480] = [0u8; 480];
static mut BONE_MATRICES: [f32; 480] = [0.0; 480];  // 40 bones × 12 floats (3×4)

const BONE_COUNT: usize = 40;

fn update_animation() {
    unsafe {
        let state = &mut ANIM_STATE;
        
        // Advance time
        state.time += DELTA_TIME;
        
        // Calculate frame indices and blend factor
        let frame_f = state.time * state.fps;
        let frame_a = frame_f.floor() as u32;
        let frame_b = frame_a + 1;
        let blend = frame_f.fract();
        
        // Pull keyframes from ROM
        read_frame(state.clip_handle, frame_a, KEYFRAME_BUF_A.as_mut_ptr());
        read_frame(state.clip_handle, frame_b, KEYFRAME_BUF_B.as_mut_ptr());
        
        // Blend and build matrices
        for i in 0..BONE_COUNT {
            let offset = i * 12;
            
            // Decode keyframes
            let kf_a = decode_keyframe(&KEYFRAME_BUF_A[offset..offset+12]);
            let kf_b = decode_keyframe(&KEYFRAME_BUF_B[offset..offset+12]);
            
            // Blend (lerp position/scale, slerp rotation)
            let blended = BoneTransform {
                rotation: slerp(kf_a.rotation, kf_b.rotation, blend),
                position: lerp3(kf_a.position, kf_b.position, blend),
                scale: lerp(kf_a.scale, kf_b.scale, blend),
            };
            
            // Build matrix
            let mat = build_bone_matrix(&blended);
            BONE_MATRICES[i*12..(i+1)*12].copy_from_slice(&mat);
        }
        
        // Upload to GPU
        set_bones(BONE_MATRICES.as_ptr(), BONE_COUNT as u32);
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [lerp(a[0], b[0], t), lerp(a[1], b[1], t), lerp(a[2], b[2], t)]
}

fn slerp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    // Simplified slerp (nlerp for small angles is often sufficient)
    let dot = a[0]*b[0] + a[1]*b[1] + a[2]*b[2] + a[3]*b[3];
    
    // Flip if negative dot (take shorter path)
    let b = if dot < 0.0 { [-b[0], -b[1], -b[2], -b[3]] } else { b };
    
    // Linear interpolate and normalize
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
```

---

## Stamp Animation Example

For games that don't need blending:

```rust
static mut CURRENT_FRAME: u32 = 0;
static mut FRAME_TIMER: f32 = 0.0;

const FRAME_DURATION: f32 = 1.0 / 12.0; // 12 fps animation

fn update_animation_stamp() {
    unsafe {
        FRAME_TIMER += DELTA_TIME;
        
        if FRAME_TIMER >= FRAME_DURATION {
            FRAME_TIMER -= FRAME_DURATION;
            CURRENT_FRAME += 1;
            
            // Wrap at end of clip
            let info = get_clip_info(CLIP_HANDLE);
            if CURRENT_FRAME >= info.frame_count {
                CURRENT_FRAME = 0;
            }
        }
        
        // Direct ROM → GPU, no WASM memory used
        set_bones_from_clip(CLIP_HANDLE, CURRENT_FRAME);
    }
}
```

---

## Clip File Format (.ewzanim)

> **Open Question:** File extension and magic bytes
> - `.ewzanim` — explicit but long
> - `.eza` — short
> - `.clip` — generic

### Header (16 bytes)

```rust
#[repr(C, packed)]
struct ClipHeader {
    magic: [u8; 4],       // "EWZA" or similar
    version: u8,          // Format version (1)
    flags: u8,            // Bit 0: has scale, Bit 1: looping, etc.
    bone_count: u16,      // Bones per frame
    frame_count: u16,     // Total frames
    fps: u16,             // Fixed-point FPS (fps * 256)
    reserved: [u8; 4],    // Future use
}
```

### Frame Data

Immediately following header:
```
frame_count × bone_count × 12 bytes (or 10 if no scale flag)
```

### Example File Layout

```
Offset  Size    Content
------  ------  -------
0x0000  4       Magic "EWZA"
0x0004  1       Version (1)
0x0005  1       Flags (0x01 = has scale)
0x0006  2       Bone count (40)
0x0008  2       Frame count (60)  
0x000A  2       FPS (3840 = 15.0 * 256)
0x000C  4       Reserved
0x0010  28800   Frame data (60 frames × 40 bones × 12 bytes)
```

---

## Memory Budget Analysis

### Fighting Game Scenario (Breakpoint)

4 unique characters, ~50 animation clips each, 40 bones, average 30 frames per clip:

```
Per character:
  50 clips × 30 frames × 40 bones × 12 bytes = 720 KB

4 characters:
  4 × 720 KB = 2.88 MB in ROM

Linear memory (worst case, 2-way blend on 4 active characters):
  Keyframe buffers: 4 chars × 2 frames × 480 bytes = 3.84 KB
  Output matrices:  4 chars × 40 bones × 48 bytes = 7.68 KB
  Animation state:  4 chars × ~64 bytes = 256 bytes
  Total: ~12 KB
```

This leaves ~4 MB linear memory for game logic, physics, UI, etc.

### ROM Budget

```
Animation clips:  2.88 MB
Character meshes: 1.5 MB (4 × ~400KB each)
Textures:         4 MB
Audio:            2 MB
Code:             1 MB
-----------------------
Total:            ~11.4 MB of 12 MB
```

Feasible, with room for optimization.

---

## Open Questions Summary

1. **Function naming convention** — `clip_*`, `anim_*`, or mixed?

2. **Frame index behavior** — Wrap, clamp, or error on out-of-range?

3. **Matrix format for `set_bones()`** — Keep 4×4 or add/switch to 3×4?

4. **Multiple keyframe formats** — Single platform format or format flag?

5. **Scale support** — Always include (12 bytes) or optional (10 bytes)?

6. **File extension** — `.ewzanim`, `.eza`, `.clip`, other?

7. **Batch read granularity** — Expose `read_frames()` or rely on single-frame reads?

8. **Clip metadata** — What additional info in `clip_info()`? Events? Markers?

---

## Next Steps

1. Prototype `set_bones_from_clip()` to validate zero-copy path
2. Implement keyframe encoding/decoding in Rust
3. Create test clip with known values for precision validation  
4. Write `ember build` integration for `.ewzanim` generation
5. Document recommended workflow for artists/animators

---

## References

- [ACL: Animation Compression Library](https://github.com/nfrechette/acl) — Industry-standard compression techniques
- [Quaternion Compression](https://gafferongames.com/post/snapshot_compression/) — Smallest-three explanation
- [half crate](https://crates.io/crates/half) — Rust f16 implementation