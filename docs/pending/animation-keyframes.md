# Animation Keyframe Storage Spec

**Status:** Pending
**Category:** FFI, Animation, Asset Pipeline

## Problem

Animation-heavy games (fighting games, character action games) can require significant bone matrix data:

- 24-bone skeleton × 48 bytes/matrix = 1,152 bytes per keyframe
- 30 animations × 60 frames × 1,152 bytes = ~2 MB per character

**Emberware Z memory limits (PS1/N64 realistic):**
- ROM total: 12 MB (game code + all assets)
- WASM linear memory: 4 MB (runtime game state)

Storing animation keyframes in linear memory means a single character's animations (~2MB) would consume **half** of the 4MB available for all runtime state. A fighting game with 2+ characters becomes impossible.

Current solutions have tradeoffs:

| Approach | Memory | CPU Cost | Flexibility |
|----------|--------|----------|-------------|
| Embedded `.ewzanim` | High (eats into 12MB limit) | Low | Full |
| `rom_data` per frame | Low | High (allocate + copy each frame) | Full |

Games need flexibility to implement their own animation systems (frame-based, time-based, IK, blending, procedural), but shouldn't pay the memory cost of storing all keyframes in WASM.

## Solution: Host-side Keyframe Storage

Store individual keyframes (bone matrix sets) in the ROM data pack. The game controls all timing, sequencing, and blending logic.

### Design Principles

1. **Game owns animation logic** - No built-in time/interpolation system
2. **Zero-copy fast path** - Direct ROM-to-GPU for simple playback
3. **WASM copy for blending** - When game needs to manipulate matrices
4. **Consistent with existing APIs** - Follows `rom_*` naming and patterns

## Data Pack Format

### PackedKeyframe

```rust
pub struct PackedKeyframe {
    /// Unique identifier (e.g., "ryu_idle_0", "ken_punch_heavy_5")
    pub id: String,
    /// Bone matrices for this keyframe (one per bone, 3x4 row-major)
    pub matrices: Vec<BoneMatrix3x4>,
}
```

### Naming Convention

Games should use consistent naming for keyframe IDs:

```
{character}_{animation}_{frame}
```

Examples:
- `player_idle_0`, `player_idle_1`, ..., `player_idle_30`
- `player_walk_0`, `player_walk_1`, ..., `player_walk_20`
- `enemy_attack_heavy_0`, `enemy_attack_heavy_1`, ...

## FFI Functions

### rom_keyframe (init-only)

```rust
/// Load a keyframe from ROM data pack by ID
///
/// # Arguments
/// * `id_ptr` - Pointer to keyframe ID string in WASM memory
/// * `id_len` - Length of keyframe ID string
///
/// # Returns
/// Keyframe handle (>0) on success. Traps on failure.
///
/// **Init-only:** Can only be called during `init()`.
fn rom_keyframe(id_ptr: u32, id_len: u32) -> u32;
```

Caches the keyframe on the host side. Returns a handle for use with `set_keyframe()`.

### set_keyframe (render)

```rust
/// Set bone matrices from a cached keyframe
///
/// # Arguments
/// * `handle` - Keyframe handle from rom_keyframe()
///
/// Directly uploads keyframe matrices to the bone uniform buffer.
/// More efficient than set_bones() for static keyframe playback.
fn set_keyframe(handle: u32);
```

Zero-copy path: matrices go directly from host cache to GPU.

### rom_keyframe_data (any time)

```rust
/// Copy keyframe matrices to WASM linear memory
///
/// # Arguments
/// * `id_ptr` - Pointer to keyframe ID string in WASM memory
/// * `id_len` - Length of keyframe ID string
/// * `dst_ptr` - Pointer to destination buffer in WASM memory
/// * `max_len` - Maximum bytes to copy
///
/// # Returns
/// Bytes written on success. Traps on failure.
///
/// Use this when you need to blend, modify, or combine keyframes.
/// Buffer must be at least `bone_count * 48` bytes.
fn rom_keyframe_data(id_ptr: u32, id_len: u32, dst_ptr: u32, max_len: u32) -> u32;
```

For games that need matrix data in WASM (blending, IK, procedural).

### rom_keyframe_bone_count (optional)

```rust
/// Get the bone count of a keyframe
///
/// # Arguments
/// * `id_ptr` - Pointer to keyframe ID string in WASM memory
/// * `id_len` - Length of keyframe ID string
///
/// # Returns
/// Number of bones in the keyframe
fn rom_keyframe_bone_count(id_ptr: u32, id_len: u32) -> u32;
```

Useful for validation and buffer allocation.

## Usage Examples

### Simple Frame-based Playback

```rust
// In init()
static IDLE_FRAMES: [u32; 30] = [0; 30];
for i in 0..30 {
    let id = format!("player_idle_{}", i);
    IDLE_FRAMES[i] = rom_keyframe(id.as_ptr() as u32, id.len() as u32);
}

// In render()
let frame = (tick / 2) % 30;  // 30fps animation at 60fps tick rate
set_keyframe(IDLE_FRAMES[frame]);
draw_mesh(PLAYER_MESH);
```

### Blending Two Animations

```rust
// Allocate buffers for two keyframes (24 bones typical for 5th-gen characters)
static mut BONES_A: [BoneMatrix3x4; 24] = [...];
static mut BONES_B: [BoneMatrix3x4; 24] = [...];
static mut BLENDED: [BoneMatrix3x4; 24] = [...];

// In render()
let walk_id = format!("player_walk_{}", walk_frame);
let run_id = format!("player_run_{}", run_frame);

rom_keyframe_data(walk_id.as_ptr() as u32, walk_id.len() as u32,
                  BONES_A.as_mut_ptr() as u32, size_of_val(&BONES_A) as u32);
rom_keyframe_data(run_id.as_ptr() as u32, run_id.len() as u32,
                  BONES_B.as_mut_ptr() as u32, size_of_val(&BONES_B) as u32);

// Blend in WASM (lerp each matrix based on speed)
for i in 0..24 {
    BLENDED[i] = lerp_matrix(BONES_A[i], BONES_B[i], blend_factor);
}

set_bones(BLENDED.as_ptr() as u32, 24);
draw_mesh(PLAYER_MESH);
```

### IK with Base Keyframe

```rust
// Get base pose from ROM
rom_keyframe_data(base_pose_id, ..., BONES.as_mut_ptr(), ...);

// Apply IK to arm bones (modify in WASM)
apply_ik(&mut BONES[ARM_START..ARM_END], target_position);

// Upload modified skeleton
set_bones(BONES.as_ptr() as u32, 24);
```

## Memory Comparison

For a character with 30 animations, 60 frames average, 24-bone skeleton:

| Approach | Linear Memory (4MB limit) | ROM/Host |
|----------|---------------------------|----------|
| Embedded animations | ~2 MB (50% of limit!) | 0 |
| Host keyframes (simple) | ~7 KB (handles only) | ~2 MB |
| Host keyframes (blending) | ~9 KB (handles + 2 buffers) | ~2 MB |

Host-side keyframes keep animation data in ROM, leaving linear memory free for game state. A fighting game with 6 characters:
- **Embedded**: 12 MB animations → impossible (exceeds 4MB linear memory)
- **Host keyframes**: ~54 KB linear + 12 MB in ROM → works fine

## Asset Pipeline

### ember-export Changes

The `ember-export` tool should support exporting animations as keyframes:

```bash
# Export animation as individual keyframes
ember-export animation model.glb --animation "Idle" --output-keyframes player_idle
# Creates: player_idle_0.ewzkey, player_idle_1.ewzkey, ...

# Or export as single file with multiple keyframes
ember-export animation model.glb --animation "Idle" --output player_idle.ewzanim
```

### ember.toml Integration

```toml
[[assets.keyframes]]
id_prefix = "player_idle"
path = "assets/player_idle.ewzanim"  # Contains all frames

# Or individual files
[[assets.keyframes]]
id = "player_special_0"
path = "assets/player_special_frame0.ewzkey"
```

## Implementation Plan

### Phase 1: Core FFI
- [ ] Add `PackedKeyframe` to `z-common/src/formats/z_data_pack.rs`
- [ ] Add keyframe storage to `ZFFIState`
- [ ] Implement `rom_keyframe` in `ffi/rom.rs`
- [ ] Implement `set_keyframe` in `ffi/skinning.rs`
- [ ] Implement `rom_keyframe_data` in `ffi/rom.rs`

### Phase 2: Asset Pipeline
- [ ] Add keyframe export to `ember-export`
- [ ] Add keyframe packing to `ember pack`
- [ ] Update `ember.toml` schema

### Phase 3: Documentation
- [ ] Add to FFI reference docs
- [ ] Create animation tutorial
- [ ] Example: simple animation playback
- [ ] Example: blending system

## Open Questions

1. **Keyframe metadata** - Should we store frame timing hints, or leave entirely to game?
2. **Compression** - Should keyframes support delta compression for sequential frames?
3. **Batch loading** - Should there be `rom_keyframes(prefix, count)` for efficiency?
