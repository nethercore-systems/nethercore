# Matrix Index Packing Design

## Overview
This document describes a compact and efficient method for providing model, view, and projection matrices to the GPU without issuing per‑draw uniform uploads. The approach packs three matrix indices into a single 32‑bit value, enabling large matrix pools and minimal per‑draw overhead.

## Goals
- Avoid per‑draw uniform uploads of model/view/projection matrices.
- Allow bulk upload of matrix arrays once per frame.
- Keep per‑draw data extremely small (4 bytes).
- Work efficiently across WebGPU/WebGL/native.
- Support large numbers of model matrices with minimal cost.

## Core Idea
Each draw call uses a single `u32` key that encodes:
- **model matrix index** (16 bits)
- **view matrix index** (8 bits)
- **projection matrix index** (8 bits)

This key indexes into three storage buffers bound once per frame:
- `model_matrices: array<mat4x4<f32>>`
- `view_matrices: array<mat4x4<f32>>`
- `proj_matrices: array<mat4x4<f32>>`

### Bit Layout

```
31         24 23      16 15                        0
+------------+----------+---------------------------+
| proj_index | view_idx |        model_index        |
+------------+----------+---------------------------+
```

## Rust Packing Helpers

```rust
pub fn pack_mvp_indices(model: u32, view: u32, proj: u32) -> u32 {
    (model & 0xFFFF) | ((view & 0xFF) << 16) | ((proj & 0xFF) << 24)
}

pub fn unpack_mvp_indices(key: u32) -> (u32, u32, u32) {
    let model = key & 0xFFFF;
    let view  = (key >> 16) & 0xFF;
    let proj  = (key >> 24) & 0xFF;
    (model, view, proj)
}
```

## WGSL Shader Side

```wgsl
@group(0) @binding(0) var<storage, read> model_matrices: array<mat4x4<f32>>;
@group(0) @binding(1) var<storage, read> view_matrices:  array<mat4x4<f32>>;
@group(0) @binding(2) var<storage, read> proj_matrices:  array<mat4x4<f32>>;

fn unpack_mvp(key: u32) -> vec3<u32> {
  let model = key & 0xFFFFu;
  let view  = (key >> 16u) & 0xFFu;
  let proj  = (key >> 24u) & 0xFFu;
  return vec3<u32>(model, view, proj);
}

fn load_matrices(key: u32) -> mat4x4<f32> {
  let ids = unpack_mvp(key);
  let model = model_matrices[ids.x];
  let view  = view_matrices[ids.y];
  let proj  = proj_matrices[ids.z];
  return proj * view * model;
}
```

## Upload Strategy
- Build `Vec<Mat4>` for model/view/projection matrices each frame via ZFFIState and FFI calls.
- Upload each matrix buffer **once** with a single bulk transfer at the start of the frame.
- Store the packed `u32` in CurrentDrawState.

## Integration into Draw Pipeline
1. Fill matrix pools and store indices.
2. Pack indices into a single `u32`.
3. Record commands containing the packed key.
4. Upload matrix buffers once.
5. Replay render commands:
   - bind pipeline, textures, material
   - provide the packed index key
   - draw

## Benefits
- **4 bytes per draw** instead of 192 bytes (3 matrices).
- Zero per‑draw matrix uploads.
- Perfect for batched, sorted, immediate‑mode rendering.
- Scales to tens of thousands of instances.
- Simple, portable, GPU‑friendly.

## Limits
- Model matrices: up to **65,536** entries per frame.
- View/projection matrices: up to **256** each.
- These limits can be extended by adjusting bit allocation if needed.