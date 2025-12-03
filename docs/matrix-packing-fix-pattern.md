# Matrix Packing Fix Pattern

**Implementation:** Using instance indices with MVP indices storage buffer (no push constants required)

## Pattern for FFI Draw Functions

Replace this pattern:
```rust
state.render_pass.record_*(
    // ...params...
    state.current_transform,  // ❌ Old: Mat4 (64 bytes)
    // ...more params...
);
```

With this pattern:
```rust
// Add current transform to model matrix pool
let model_idx = state.add_model_matrix(state.current_transform)
    .expect("Model matrix pool overflow");

// Pack matrix indices into single u32 (model: 16 bits, view: 8 bits, proj: 8 bits)
let mvp_index = crate::graphics::MvpIndex::new(
    model_idx,
    state.current_view_idx,
    state.current_proj_idx,
);

state.render_pass.record_*(
    // ...params...
    mvp_index,  // ✅ New: MvpIndex (4 bytes, 16× reduction!)
    // ...more params...
);
```

**How it works:** The packed `MvpIndex` is uploaded to an MVP indices storage buffer. Each draw uses `instance_index` to fetch its packed indices from the buffer, unpacks them in the shader, and fetches matrices from the matrix storage buffers. This approach works on all GPUs (no push constants required).

## Pattern for Deferred Command Expansion

Replace VRPCommand construction:
```rust
VRPCommand {
    format,
    transform: Mat4::IDENTITY,  // ❌ Old field
    // ...
}
```

With:
```rust
VRPCommand {
    format,
    mvp_index: MvpIndex::new(0, 0, 0),  // ✅ New field (identity = index 0)
    // ...
}
```

## Locations to Fix

### emberware-z/src/ffi/mod.rs
- Line ~1203: `draw_triangles` function
- Line ~1386: `draw_triangles_indexed` function

### emberware-z/src/graphics/mod.rs
- Line ~1325: DrawBillboard expansion
- Line ~1452: DrawSprite expansion
- Line ~1523: DrawRect expansion
- Line ~1613: DrawText expansion
- Line ~2176: `self.view_buffer` → `self.view_matrix_buffer`
