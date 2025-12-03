# Quad Rendering Improvement — Finalized Implementation Plan

**Purpose:** Replace per-billboard / per-sprite CPU vertex generation with a single, unified, GPU-driven quad system (unit quad + instanced attributes) supporting: spherical and cylindrical billboards (modes 1–4), screen-space sprites, and world-space quads — all under the same unified shading pipeline.

This document is an implementation-ready plan intended for an automated agent or engineer to implement directly.

---

## Summary

* Use a single static unit quad mesh (VBO + IBO).
* Use a small per-instance struct (`QuadInstance`) that carries position, size, UV, color, rotation and a `quad_mode` flag.
* Vertex shader expands the unit quad into world/screen space according to `quad_mode` and `billboard_mode`, computing normals/tangents in-shader as needed.
* Batch draws by `(pipeline, texture_group, material_handle, quad_mode)` and issue `draw_indexed_instanced` calls.
* Maintain compatibility with existing Unified Shading State (USS) and material pipeline.

---

## Goals

* Eliminate CPU-side per-quad vertex generation.
* Support high throughput (thousands+) of quads with minimal CPU overhead.
* Preserve full shading (PBR/matcap/unlit) for billboards and sprites.
* Support the four billboard modes:

  1. Spherical (camera-facing all axes)
  2. Cylindrical Y (rotate only around world Y)
  3. Cylindrical X (rotate only around world X)
  4. Cylindrical Z (rotate only around world Z)
* Support screen-space sprites with rotation and origin.
* Keep per-instance data small (16–32 bytes typical) and GPU-friendly.

---

## Data Layouts (CPU-side)

### Unit quad mesh (static)

A single mesh stored once: positions in local quad space (e.g. (-0.5,-0.5),(0.5,-0.5),(0.5,0.5),(-0.5,0.5)), UVs (0..1), optional vertex color.

### QuadInstance (per-instance)

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct QuadInstance {
    // 12 bytes
    pub position: [f32; 3],   // world or screen-space depending on quad_mode
    // 8 bytes
    pub size: [f32; 2],       // width, height
    // 4 bytes
    pub rotation: f32,        // radians (used for screen-space & world-space rotation)
    // 4 bytes
    pub quad_mode: u32,       // enum: SCREEN_SPACE, WORLD_SPACE, BILLBOARD_SPHERICAL, CYLY, CYLX, CYLZ
    // 16 bytes
    pub uv: [f32; 4],         // u0,v0,u1,v1
    // 4 bytes
    pub color: u32,           // packed RGBA8
}
```

**Notes:**

* Total size ≈ 48 bytes; pad to 16-byte alignment if needed. You can compress `quad_mode` to a smaller field if necessary, but alignment rules for vertex attributes (wgpu) often favor 4-byte types.
* Alternatively, split into multiple smaller attributes: `position: vec3`, `uv: vec4`, `color: u32`, `size+rotation+mode` packed into another `vec4`.

### VRPCommand (now includes quad_mode) changes (command buffer)

Replace `DrawBillboard`/`DrawSprite` with unified `DrawQuad` (or map both into the same path). Store minimal per-draw state:

```rust
pub struct VRPCommand {
    pub mesh_id: MeshId,            // QUAD_MESH
    pub instance_start: u32,        // index into instance buffer
    pub instance_count: u32,
    pub material_handle: MaterialHandle,
    pub texture_group: TextureGroupId,
    pub quad_mode: u32,             // added: billboard/screen/world mode for batching & pipeline selection
    pub depth: f32,
    pub cull_mode: CullMode,
    pub blend_mode: BlendMode,
}
```

---

## Shader (WGSL) — vertex shader core

### Bindings (example)

```wgsl
@group(0) @binding(0) var<storage, read> model_matrices: array<mat4x4<f32>>; // optional for world-space with model indices
@group(0) @binding(1) var<storage, read> view_matrices:  array<mat4x4<f32>>;
@group(0) @binding(2) var<storage, read> proj_matrices:  array<mat4x4<f32>>;
```

### Vertex inputs (unit quad vertex + instance attributes)

```wgsl
struct VertexIn {
  @location(0) position_local: vec2<f32>; // unit quad corner (-0.5..0.5)
  @location(1) uv_local: vec2<f32>;
  // instance attributes (instanced)
  @location(4) instance_position: vec3<f32>;
  @location(5) instance_size: vec2<f32>;
  @location(6) instance_rotation: f32;
  @location(7) instance_quad_mode: u32;
  @location(8) instance_uv: vec4<f32>;
  @location(9) instance_color: u32;
}
```

### Billboard helper pseudo-functions

```wgsl
fn get_camera_basis(view: mat4x4<f32>) -> (vec3<f32>, vec3<f32>, vec3<f32>) {
  let right = vec3<f32>(view[0].x, view[0].y, view[0].z);
  let up    = vec3<f32>(view[1].x, view[1].y, view[1].z);
  let forward = -vec3<f32>(view[2].x, view[2].y, view[2].z);
  return (right, up, forward);
}

fn apply_billboard(local: vec2<f32>, size: vec2<f32>, mode: u32, view: mat4x4<f32>) -> vec3<f32> {
  let (camRight, camUp, camFwd) = get_camera_basis(view);
  let lx = local.x * size.x;
  let ly = local.y * size.y;
  if (mode == 1u) { // spherical
      return camRight * lx + camUp * ly;
  } else if (mode == 2u) { // cylindrical Y
      let forward_xz = normalize(vec3<f32>(camFwd.x, 0.0, camFwd.z));
      let right = normalize(cross(vec3<f32>(0.0,1.0,0.0), forward_xz));
      let up = vec3<f32>(0.0,1.0,0.0);
      return right * lx + up * ly;
  }
  // implement other modes similarly
  return vec3<f32>(lx, ly, 0.0);
}
```

### Main vertex flow

```wgsl
@vertex
fn vs_main(in: VertexIn) -> VertexOut {
  // get instance values
  let local = in.position_local; // e.g., (-0.5,-0.5) .. (0.5,0.5)
  let mode = in.instance_quad_mode;

  var offset_world: vec3<f32>;
  if (mode == SCREEN_SPACE_MODE) {
      // handle screen-space: transform to NDC after rotation
  } else if (mode == WORLD_SPACE_MODE) {
      // if normal world quad
      offset_world = in.instance_position.xyz + ... ;
  } else {
      // billboard modes
      offset_world = in.instance_position + apply_billboard(local, in.instance_size, mode, view_matrices[0]);
  }

  let world_pos = vec4<f32>(offset_world, 1.0);
  let clip = proj_matrices[0] * view_matrices[0] * world_pos;
  // set out
}
```

**Notes:**

* Use the correct `view`/`proj` indices if you have multiple cameras; bind the current view/proj for the pass.
* For PBR lighting on billboards, compute normals in-shader (e.g., normal = normalize(camFwd) for spherical), build tangent from right/up if using normal maps.

---

## CPU-side Implementation Steps (Migration Plan)

Each step should be a small, testable change. Keep an automated test suite and a fallback feature flag for the old CPU path during migration.

### Step 0 — Prep & flags

* Add feature flag `gpu_quad_instancing` to toggle new path.
* Add `QuadMode` enum and `QuadInstance` struct to codebase.

### Step 1 — Unit quad mesh

* Create and upload a static unit quad mesh (`MESH_QUAD_STATIC`) with positions and UVs.
* Ensure mesh format works with existing pipelines.

### Step 2 — Instance buffer management

* Implement per-frame instance buffer allocator (ring-buffer or streaming dynamic buffer) sized for expected max instances.
* Provide `push_quad_instance(QuadInstance) -> instance_index`.

### Step 3 — Replace DrawBillboard / DrawSprite paths

* Map `DrawBillboard` and `DrawSprite` to produce `QuadInstance` entries instead of generating vertex/index data.
* For `DrawSprite` (screen-space): compute `position` in screen coordinate space and mark `quad_mode = SCREEN_SPACE`.
* For `DrawBillboard`: compute `instance.position = transform.translation`, `instance.size = (w,h)`, `instance_quad_mode = MODE`.

### Step 4 — VRPCommand change

* Make `VRPCommand` reference `mesh_id = MESH_QUAD_STATIC` and instance buffer ranges (start/count).
* Ensure existing sorting keys include `quad_mode` or material handle as needed.

### Step 5 — Shader updates

* Update vertex shader to accept instance attributes and implement `apply_billboard`, screen-space transform, and in-shader normal generation.
* Add small helper functions for cylindrical modes.

### Step 6 — Batching & replay

* Group commands by `(pipeline, texture_group, material_handle, quad_mode)`.
* For each group, bind pipeline, bind textures, bind instance buffer range, call `draw_indexed_instanced`.

### Step 7 — Validation & compatibility

* Update validation code: if `quad_mode` != None, relax "must have normals" rule.
* Maintain the old CPU path behind the feature flag for quick rollback.

### Step 8 — Tests & benchmarks

* Functional tests for each quad_mode and screen-space sprites.
* Benchmark: compare draw calls, instance uploads, and CPU time for large counts (1k, 10k, 100k quads) before/after.

### Step 9 — Cleanup

* Remove old CPU quad generation code and data paths once stable.
* Document API and examples.

---

## Shader-level Notes & Corner Cases

* **Degenerate camera directions**: handle cases where camera forward is near vertical; fall back to default axes.
* **Depth & sorting**: transparent quads still need per-draw depth sorting; keep sorting stable.
* **Texture atlases**: use `uv_rect` per-instance to pick sprite region.
* **Normal maps**: for billboards, construct TBN from `right`/`up` vectors computed from camera basis; be careful with handedness.
* **Performance**: per-vertex extra math for billboarding is minor compared to CPU cost saved; grouping instances reduces draw calls dramatically.

---

## API migration examples

### Old `DrawBillboard`

```rust
draw_billboard(w, h, mode, transform, color, ...)
```

### New (conceptual)

```rust
let inst = QuadInstance {
    position: transform.translation.xyz(),
    size: [w, h],
    rotation: 0.0, // unused for pure camera-facing modes
    quad_mode: mode as u32,
    uv: [u0,v0,u1,v1],
    color: packed_color,
};
let idx = push_quad_instance(inst);
emit_command(VRPCommand { mesh_id: QUAD_MESH, instance_start: idx, instance_count: 1, ... });
```

---

## Tests & Acceptance Criteria

* Render correctness:

  * Spherical billboards face camera for arbitrary camera orientation.
  * Cylindrical modes preserve the locked axis.
  * Screen-space sprites rotate correctly around `origin`.
  * PBR highlights are plausible on billboards.

* Performance:

  * CPU time for 10k quads should drop significantly vs CPU-generated quads.
  * Draw calls reduced by batching where applicable.

* Robustness:

  * Degenerate cases handled gracefully.
  * Fallback behavior available via feature flag.

---

## Deliverables (for agent)

1. `quad_mesh.rs` — static unit quad creation and mesh registration.
2. `quad_instance.rs` — `QuadInstance` struct, instance buffer allocator, push/pop utilities.
3. `commands.rs` — mapping `DrawBillboard`/`DrawSprite` to `QuadInstance` and new `VRPCommand` fields.
4. `renderer.rs` — batching & instanced replay implementation.
5. `shaders/quad.vert.wgsl` — full vertex shader implementing `apply_billboard` and screen-space logic.
6. Tests + benchmarks.
