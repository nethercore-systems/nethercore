# Animation System Validation Examples

Examples that validate the **inverse bind matrix logic** and test scenarios not covered by existing examples.

## Gap Analysis

Current examples (`skinned-mesh`, `animation-demo`) do NOT use:
- `load_skeleton()` / `rom_skeleton()`
- `skeleton_bind()` with a valid handle
- Inverse bind matrices (FLAG_SKINNING_MODE)

Both use raw bone matrices directly. The GPU path `bone_matrix * inverse_bind * vertex` was untested.

Key difference:
- **Raw mode** (current): `set_bones()` receives final world-space transforms
- **Inverse bind mode** (new): `set_bones()` receives local bone transforms, GPU multiplies by inverse bind

---

## Examples Overview

| Example | Purpose | Key Features |
|---------|---------|--------------|
| `multi-skinned-procedural` | Test inverse bind with procedural meshes | Multiple skeletons, skeleton switching |
| `ik-demo` | Two-bone inverse kinematics | IK solver, procedural bone computation |
| `multi-skinned-rom` | ROM-loaded skinned meshes | `rom_skeleton()`, `rom_keyframes()`, asset generation |
| `skeleton-stress-test` | Performance stress test | 36 robots, walk cycle, frequent skeleton switches |

---

## Understanding Inverse Bind Matrices

**What they are:** Transform vertices from model space to bone-local space at bind pose.

**Why needed:** When animating, bone transforms are in world/model space. To correctly deform a vertex:
1. Transform vertex to bone-local space (inverse bind)
2. Apply animated bone transform
3. Result: vertex in animated world space

**GPU formula:** `final_pos = bone_matrix * inverse_bind * vertex_pos`

**How to compute inverse bind:**
```rust
// For each bone at bind pose:
// 1. Compute bone's world transform at bind pose
// 2. Invert it
// inverse_bind[bone] = inverse(bone_world_at_bind_pose)

fn compute_inverse_bind(bone_world: &Mat4) -> [f32; 12] {
    let inv = bone_world.inverse();
    // Return as 3x4 column-major
    [
        inv.x_axis.x, inv.x_axis.y, inv.x_axis.z,  // col 0
        inv.y_axis.x, inv.y_axis.y, inv.y_axis.z,  // col 1
        inv.z_axis.x, inv.z_axis.y, inv.z_axis.z,  // col 2
        inv.w_axis.x, inv.w_axis.y, inv.w_axis.z,  // col 3 (translation)
    ]
}
```

---

## Example 1: `multi-skinned-procedural`

**Purpose:** Test inverse bind matrices with multiple independently-animated procedural meshes.

**What it validates:**
- `load_skeleton()` - uploading inverse bind matrices
- `skeleton_bind()` - switching between skeletons
- Multiple meshes with different bone counts
- Independent animation state per mesh

**Implementation:**
- 2 procedurally generated arm meshes (3-bone vertical, 4-bone horizontal)
- Each has its own skeleton with inverse bind matrices
- Each animated independently (different speeds, phases)
- Render loop switches skeleton bindings between draws

**Key code pattern:**
```rust
// Inverse bind matrices for 3-bone arm
static ARM1_INVERSE_BIND: [[f32; 12]; 3] = [
    // Bone 0: at origin
    [1.0, 0.0, 0.0,  0.0, 1.0, 0.0,  0.0, 0.0, 1.0,  0.0, 0.0, 0.0],
    // Bone 1: inverse of translate(0, 1.5, 0)
    [1.0, 0.0, 0.0,  0.0, 1.0, 0.0,  0.0, 0.0, 1.0,  0.0, -1.5, 0.0],
    // Bone 2: inverse of translate(0, 3.0, 0)
    [1.0, 0.0, 0.0,  0.0, 1.0, 0.0,  0.0, 0.0, 1.0,  0.0, -3.0, 0.0],
];

// init()
ARM1_SKELETON = load_skeleton(ARM1_INVERSE_BIND.as_ptr() as *const f32, 3);
ARM2_SKELETON = load_skeleton(ARM2_INVERSE_BIND.as_ptr() as *const f32, 4);

// render()
skeleton_bind(ARM1_SKELETON);
set_bones(arm1_bones.as_ptr(), 3);
draw_mesh(ARM1_MESH);

skeleton_bind(ARM2_SKELETON);
set_bones(arm2_bones.as_ptr(), 4);
draw_mesh(ARM2_MESH);
```

---

## Example 2: `ik-demo`

**Purpose:** Demonstrate procedural IK animation using the skeleton system.

**What it validates:**
- IK solver integration with `set_bones()`
- Inverse bind matrices with procedural bone computation
- Real-time target tracking
- 2-bone analytical IK algorithm

**Implementation:**
- Simple 2-bone IK arm reaching toward a target
- Target controlled by left stick or animated in a circle
- Uses proper inverse bind matrices
- Shows constraint handling (reach limits)

**Algorithm (2-bone IK):**
```
Given: shoulder position, target position, upper_len, lower_len
1. Compute distance to target
2. Clamp to reachable range [|upper - lower|, upper + lower]
3. Use law of cosines: cos(elbow) = (a² + b² - c²) / (2ab)
4. Compute shoulder angle using atan2 + offset for elbow bend
5. Build bone matrices (world space transforms)
6. Upload via set_bones() with skeleton bound
```

**Key code pattern:**
```rust
fn solve_two_bone_ik(shoulder: [f32; 3], target: [f32; 3]) -> [[f32; 12]; 2] {
    let dist = distance(shoulder, target);
    let clamped_dist = dist.clamp(MIN_REACH, MAX_REACH);

    // Law of cosines for elbow angle
    let cos_elbow = (a2 + b2 - c2) / (2.0 * UPPER_LEN * LOWER_LEN);
    let elbow_angle = acosf(cos_elbow);

    // Shoulder angle to reach target
    let base_angle = atan2f(dy, dx);
    let offset = asinf((LOWER_LEN * sinf(elbow_angle)) / clamped_dist);
    let shoulder_angle = base_angle - offset;

    // Build bone matrices...
    [bone0_matrix, bone1_matrix]
}
```

---

## Example 3: `multi-skinned-rom`

**Purpose:** Test inverse bind matrices with ROM-loaded assets.

**What it validates:**
- `rom_skeleton()` - loading inverse bind from data pack
- `rom_mesh()` - loading skinned mesh from data pack
- `rom_keyframes()` - loading animations from data pack
- Multiple characters with independent ROM-backed animations
- Proper skeleton/keyframe binding coordination

**Implementation:**
- 2 ROM-loaded skinned characters (generated by `gen-multi-skinned-assets` tool)
- Each has skeleton + mesh + animation loaded from data pack
- Independent playback with different timing offsets

**Asset Generation Tool:** `tools/gen-multi-skinned-assets`

Generates `.ewzskel`, `.ewzmesh`, `.ewzanim` files programmatically:
- arm1: 3-bone vertical arm with wave animation
- arm2: 4-bone horizontal arm with wave animation

**ember.toml:**
```toml
[game]
id = "multi-skinned-rom"
title = "Multi Skinned ROM"

[[assets.skeletons]]
id = "arm1_skel"
path = "assets/arm1.ewzskel"

[[assets.meshes]]
id = "arm1_mesh"
path = "assets/arm1.ewzmesh"

[[assets.animations]]
id = "wave1"
path = "assets/wave1.ewzanim"
```

**Key code pattern:**
```rust
// init()
ARM1_MESH = rom_mesh(b"arm1_mesh".as_ptr(), 9);
ARM1_SKELETON = rom_skeleton(b"arm1_skel".as_ptr(), 9);
ARM1_ANIM = rom_keyframes(b"wave1".as_ptr(), 5);

// render()
// IMPORTANT: Bind skeleton BEFORE keyframe_bind
skeleton_bind(ARM1_SKELETON);
keyframe_bind(ARM1_ANIM, frame);
draw_mesh(ARM1_MESH);
```

---

## Example 4: `skeleton-stress-test`

**Purpose:** Stress test animation system with many independently animated skinned meshes.

**What it validates:**
- Performance with many skeleton bindings per frame
- GPU buffer management under load
- Correct rendering with frequent skeleton switches
- Shared skeleton/mesh with different animation phases

**Implementation:**
- 6×6 grid of robot characters (36 total)
- All share same skeleton/mesh definition
- Each has different animation phase offset (staggered walk cycle)
- Simple procedural walk cycle animation

### Robot Skeleton (7 bones)

```
       [0] torso (root)
          |
    +-----+-----+
    |           |
[1] L_hip    [4] R_hip
    |           |
[2] L_knee   [5] R_knee
    |           |
[3] L_foot   [6] R_foot
```

### Walk Cycle Animation

```rust
fn compute_walk_cycle(phase: f32) -> [[f32; 12]; 7] {
    let t = phase * TWO_PI;

    // Torso: slight bob up/down (2x frequency)
    let torso_bob = sinf(t * 2.0) * 0.02;
    let torso_sway = sinf(t) * 0.015;

    // Left leg (phase 0 = left foot forward)
    let l_hip_angle = sinf(t) * 0.35;
    let l_knee_bend = (1.0 - cosf(t)) * 0.25;

    // Right leg (180° out of phase)
    let r_hip_angle = sinf(t + PI) * 0.35;
    let r_knee_bend = (1.0 - cosf(t + PI)) * 0.25;

    // Build hierarchical bone transforms...
}
```

### Render Loop

```rust
fn render() {
    for row in 0..GRID_SIZE {
        for col in 0..GRID_SIZE {
            let i = row * GRID_SIZE + col;
            let phase = (ANIM_TIME + PHASE_OFFSETS[i]) % 1.0;

            skeleton_bind(ROBOT_SKELETON);
            compute_walk_cycle(phase, &mut BONE_MATRICES);
            set_bones(BONE_MATRICES.as_ptr(), 7);

            push_translate(x, 0.0, z);
            draw_mesh(ROBOT_MESH);
        }
    }
}
```

---

## File Structure

```
examples/
├── multi-skinned-procedural/
│   ├── Cargo.toml
│   └── src/lib.rs
├── ik-demo/
│   ├── Cargo.toml
│   └── src/lib.rs
├── multi-skinned-rom/
│   ├── Cargo.toml
│   ├── ember.toml
│   ├── assets/           # Generated by tool
│   └── src/lib.rs
└── skeleton-stress-test/
    ├── Cargo.toml
    └── src/lib.rs

tools/
└── gen-multi-skinned-assets/
    ├── Cargo.toml
    └── src/main.rs
```

---

## Testing Checklist

After implementation, verify:
- [ ] Meshes deform correctly when skeleton bound (vs not bound)
- [ ] Multiple skeletons can be switched mid-frame
- [ ] Inverse bind matrices produce correct deformation
- [ ] IK reaches targets accurately
- [ ] No visual artifacts when mixing animation sources
- [ ] Walk cycle looks natural (legs alternate, torso bobs)
- [ ] Stress test maintains acceptable FPS with 36 robots

---

## Controls (All Examples)

| Input | Action |
|-------|--------|
| A button | Toggle animation pause |
| D-pad Up | Increase animation speed |
| D-pad Down | Decrease animation speed |
| Left stick | Move IK target (ik-demo) / Rotate view (others) |
