# Skinned Mesh Example - Crash Analysis

## Executive Summary

The skinned-mesh example is crashing on launch. Analysis reveals several potential issues with bone index packing and unsafe static mutations.

---

## Issue #1: Bone Index Packing Format Mismatch

### The Problem

**Vertex Data Layout:**
```rust
// From skinned-mesh/src/lib.rs:245-246
let bone_indices_packed: u32 = 0 | (1 << 8) | (2 << 16) | (0 << 24);
let bone_indices_f32 = f32::from_bits(bone_indices_packed);
vertices[v_idx + 6] = bone_indices_f32;
```

This creates the bit pattern `0x00_02_01_00` and stores it as a single f32.

**GPU Expects:**
```rust
// From vertex.rs:197
format: wgpu::VertexFormat::Uint8x4
```

The GPU reads 4 bytes as `vec4<u32>` in the shader.

**Byte Order Check (Little-Endian):**
- Byte 0 (LSB): 0x00 → bone index 0
- Byte 1: 0x01 → bone index 1
- Byte 2: 0x02 → bone index 2
- Byte 3 (MSB): 0x00 → bone index 0 (unused)

Result: `vec4<u32>(0, 1, 2, 0)` ✓ **CORRECT**

**Verdict:** Bone index packing is **theoretically correct** but relies on:
1. Little-endian byte order (true for x86/ARM)
2. f32::from_bits preserving bit patterns (true)
3. GPU reading the bytes in the expected order

---

## Issue #2: Stride Mismatch Between Code and Comment

**Code Says:**
```rust
// Line 85
/// Stride: 12 + 12 + 4 + 16 = 44 bytes = 11 floats
let mut vertices = [0.0f32; 60 * 11];
```

**Calculated Stride (FORMAT_NORMAL | FORMAT_SKINNED = 12):**
```rust
// From vertex.rs:36
stride = 12 (pos) + 12 (normal) + 4 (bone_indices) + 16 (bone_weights) = 44 bytes
```

44 bytes ÷ 4 bytes/float = **11 floats per vertex** ✓

**Vertex format actually expects:**
- `pos: vec3<f32>` (3 floats)
- `normal: vec3<f32>` (3 floats)
- `bone_indices: vec4<u8>` (1 float reinterpreted as 4 bytes)
- `bone_weights: vec4<f32>` (4 floats)

Total: **11 floats** ✓

**Verdict:** Stride calculation is **CORRECT**.

---

## Issue #3: Unsafe Mutable Static References (UB Risk)

**Compilation Warnings:**
```
warning: creating a mutable reference to mutable static
   --> src/lib.rs:324:30
324 |         mat4_multiply(&mut *(BONE_MATRICES.as_mut_ptr() as *mut [f32; 16]), ...);
```

**The Code:**
```rust
// Lines 324, 341, 355
mat4_multiply(&mut *(BONE_MATRICES.as_mut_ptr() as *mut [f32; 16]), &rot0, &trans0);
mat4_multiply(&mut *(BONE_MATRICES.as_mut_ptr().add(16) as *mut [f32; 16]), ...);
mat4_multiply(&mut *(BONE_MATRICES.as_mut_ptr().add(32) as *mut [f32; 16]), ...);
```

**Why This Is Dangerous:**
1. Creates multiple mutable references to overlapping memory regions
2. Violates Rust's aliasing rules (even in unsafe code)
3. Can trigger Undefined Behavior with optimizations enabled

**The Fix:**
Use array slices or temporary buffers instead of raw pointer casts:

```rust
// BEFORE (UB risk)
mat4_multiply(&mut *(BONE_MATRICES.as_mut_ptr() as *mut [f32; 16]), &rot0, &trans0);

// AFTER (safe)
let mut temp = [0.0f32; 16];
mat4_multiply(&mut temp, &rot0, &trans0);
BONE_MATRICES[0..16].copy_from_slice(&temp);
```

**Verdict:** **HIGH RISK** of undefined behavior causing crashes.

---

## Issue #4: Missing Build Environment Dependencies

**Build Error:**
```
error: failed to run custom build command for `alsa-sys v0.3.1`
The system library `alsa` required by crate `alsa-sys` was not found.
```

**Impact:** Cannot build emberware-z to test the example.

**Fix:** Install ALSA development libraries:
```bash
# Ubuntu/Debian
sudo apt-get install libasound2-dev

# Fedora/RHEL
sudo dnf install alsa-lib-devel

# Arch
sudo pacman -S alsa-lib
```

**Verdict:** Blocking issue for testing.

---

## Issue #5: Potential Shader Compilation Failure

**Skinned Vertex Shader (from shader_gen.rs:85-105):**
```wgsl
for (var i = 0u; i < 4u; i++) {
    let bone_idx = in.bone_indices[i];
    let weight = in.bone_weights[i];

    if (weight > 0.0 && bone_idx < 256u) {
        let bone_matrix = bones[bone_idx];
        skinned_pos += (bone_matrix * vec4<f32>(in.position, 1.0)).xyz * weight;
        skinned_normal += (bone_matrix * vec4<f32>(in.normal, 0.0)).xyz * weight;
    }
}
```

**Potential Issues:**
1. If `bones` storage buffer is not bound correctly
2. If `bone_idx` is out of bounds despite the check
3. If bone matrices contain NaN/Inf values

**Debugging:**
- Check if bone matrices are initialized to identity in init()
- Verify set_bones() is called before draw_mesh()
- Add logging to set_bones() FFI to confirm it's being called

---

## Issue #6: Matrix Multiplication Order (Potential Logic Bug)

**Current Code (Line 324):**
```rust
mat4_multiply(&mut BONE_MATRICES[0..16], &rot0, &trans0);
// Result: BONE_MATRICES[0] = rot0 * trans0
```

**What This Means:**
`M = R × T` means "translate first, then rotate around origin"

**Typical Skeletal Animation:**
For bone 0 at the base of the arm, we want:
1. Translate to the base position: T
2. Rotate around that point: R

This would be: `M = T × R` (not `R × T`)

**Verdict:** **Potentially incorrect** transform order. This won't crash but may cause unexpected visual results.

---

## Recommended Fixes (Priority Order)

### 1. **Fix Unsafe Mutable Static References** (CRITICAL)

Replace all raw pointer casts with safe slice operations:

```rust
fn update_bones(time: f32) {
    unsafe {
        // Bone 0
        let angle0 = sin_approx(time) * 0.3;
        let mut rot0 = [0.0f32; 16];
        let mut trans0 = [0.0f32; 16];
        let mut temp = [0.0f32; 16];

        mat4_rotation_z(&mut rot0, angle0);
        mat4_translation(&mut trans0, 0.0, -SEGMENT_LENGTH * 1.5, 0.0);
        mat4_multiply(&mut temp, &rot0, &trans0);
        BONE_MATRICES[0..16].copy_from_slice(&temp);

        // Bone 1
        let mut bone0_final = [0.0f32; 16];
        bone0_final.copy_from_slice(&BONE_MATRICES[0..16]);

        let angle1 = sin_approx(time + 1.0) * 0.5;
        let mut rot1 = [0.0f32; 16];
        let mut trans1 = [0.0f32; 16];

        mat4_translation(&mut trans1, 0.0, SEGMENT_LENGTH, 0.0);
        mat4_rotation_z(&mut rot1, angle1);
        mat4_multiply(&mut temp, &rot1, &trans1);

        let mut temp2 = [0.0f32; 16];
        mat4_multiply(&mut temp2, &bone0_final, &temp);
        BONE_MATRICES[16..32].copy_from_slice(&temp2);

        // Bone 2
        let mut bone1_final = [0.0f32; 16];
        bone1_final.copy_from_slice(&BONE_MATRICES[16..32]);

        let angle2 = sin_approx(time + 2.0) * 0.4;
        let mut rot2 = [0.0f32; 16];
        let mut trans2 = [0.0f32; 16];

        mat4_translation(&mut trans2, 0.0, SEGMENT_LENGTH, 0.0);
        mat4_rotation_z(&mut rot2, angle2);
        mat4_multiply(&mut temp, &rot2, &trans2);
        mat4_multiply(&mut temp2, &bone1_final, &temp);
        BONE_MATRICES[32..48].copy_from_slice(&temp2);
    }
}
```

### 2. **Add Defensive Validation** (HIGH)

Add checks to ensure bone data is valid:

```rust
#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Validate ARM_MESH is initialized
        if ARM_MESH == 0 {
            let msg = b"ERROR: ARM_MESH not initialized";
            draw_text(msg.as_ptr(), msg.len() as u32, 20.0, 20.0, 48.0, 0xFF0000FF);
            return;
        }

        // Upload bone matrices
        set_bones(BONE_MATRICES.as_ptr(), NUM_BONES as u32);

        // Rest of render code...
    }
}
```

### 3. **Fix Build Dependencies** (BLOCKING)

Install ALSA or use a different audio backend for testing.

### 4. **Add Logging/Debug Output** (MEDIUM)

Add debug text to show bone matrix values:

```rust
// In render(), after set_bones()
let debug = b"Bones uploaded: 3";
draw_text(debug.as_ptr(), debug.len() as u32, 20.0, 600.0, 32.0, 0x00FF00FF);
```

### 5. **Verify Transform Order** (LOW)

Test if `T × R` works better than `R × T` for bone 0.

---

## Testing Plan

1. **Fix unsafe code** (priority 1)
2. **Install ALSA** to unblock build
3. **Rebuild and test** in emberware-z
4. **If still crashes**: Add logging to `set_bones()` FFI
5. **If visual issues**: Check transform order

---

## Potential Root Cause

**Most Likely:** Undefined Behavior from unsafe mutable static references (Issue #3).

**Why:**
- Compiler optimizations may assume no aliasing
- Pointer arithmetic with `add(16)` while holding mutable reference to base
- Can manifest as crashes, corruption, or silent failures

**Evidence:**
- Rust 2024 compatibility warnings explicitly call this out
- UB is unpredictable and may only manifest in release builds or certain conditions

---

## Animation System Flexibility (Separate Finding)

**Good News:** The animation system is genuinely unopinionated:
- Only provides GPU linear blend skinning
- All animation logic (FK, IK, blending, keyframes) is 100% developer-implemented
- Developers can choose frame-based or time-based animation
- No restrictions on skeletal structure or interpolation methods

✅ **System is flexible and doesn't lock developers into specific animation approaches**

---

## Next Steps

1. Apply Fix #1 (unsafe code)
2. Test if crash is resolved
3. If not, add extensive logging to narrow down crash location
4. Consider adding this to TASKS.md as a BUG task if the fix doesn't resolve it

