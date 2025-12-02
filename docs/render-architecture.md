# **Rendering Architecture Proposal**

## **Goals**

* Keep developer API immediate-mode and easy: set properties, bind textures, draw.
* Internally, quantize and pack all per-draw shader state into a small POD (`PackedUnifiedShadingState`).
* Only cache GPU resources that require bind groups (textures).
* Enable sorting & batching to minimize GPU state changes.
* Make the system portable across WebGPU / WebGL / native.

---

## **Core Concepts**

### **1. Packed Unified Shading State**

Quantized and compact per-draw state, uploaded as uniforms:

```rust
// -- LABEL: PackedSky --
// -- COMMENT: Quantized sky data for GPU upload; small, POD, GPU-aligned.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PackedSky {
    pub horizon_color: u32,        // RGBA8 packed
    pub zenith_color: u32,         // RGBA8 packed
    pub sun_direction: [i16; 4],   // snorm16x4 (w unused or reserved)
    pub sun_color_and_sharpness: u32, // RGB8 + sharpness u8 (packed)
}

// -- LABEL: PackedLight --
// -- COMMENT: One light; pack intensity into alpha byte of color for compactness.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PackedLight {
    pub direction: [i16; 4],     // snorm16x4, .w = enabled flag (32767) or reserved
    pub color_and_intensity: u32, // RGB8 + intensity u8 packed into alpha
}

// -- LABEL: PackedUnifiedShadingState --
// -- COMMENT: Final per-draw state uploaded to GPU; ~88 bytes in typical layout.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PackedUnifiedShadingState {
    // PBR params quantized
    pub metallic: u8,           // 0..255
    pub roughness: u8,          // 0..255
    pub emissive: u8,           // 0..255
    pub pad0: u8,               // pad to 4

    pub color_rgba8: u32,       // base color (RGBA8)
    pub blend_modes: u32,       // 4x u8 packed into u32

    // Sky and lights
    pub sky: PackedSky,         // 16 bytes
    pub lights: [PackedLight; 4],// supports up to 4 immediate lights (64 bytes)
    // total size ~96 bytes; acceptable, aligns well with GPU UBOs
}

```

* All floats/enums are quantized (0–255 or snorm16) to reduce size and enable hashing.
* Sky and lights are included to make each draw self-contained.
* Textures are **not** stored here — they require bind groups.

---

### **2. Command Buffer**

Store one command per draw:

```rust
pub struct VRPCommand { // Maybe this struct should be renamed?
    // ... Preivous fields omitted ...
    // color -> removed, exists in PackedUnifiedShadingState
    // matcap_blend_modes -> removed, exists in PackedUnifiedShadingState
    pub texture_slots: [TextureHandle; 4], // GPU-bindable, stays separately
    pub unified_shading_state_handle: UnifiedShadingStateHandle, // New
}


* `UnifiedShadingStateHandle` references an interned `PackedUnifiedShadingState`.
* Only GPU-bindable resources (textures) are cached separately.
* Supports sorting by `(pipeline, texture_group, material_handle, depth)`.

---

### **3. Immediate-mode API**

Developer calls:

```text
bind_texture(slot, handle)
set_metallic(float)
set_roughness(float)
set_emissive(float)
set_sky(...)
set_light(...)
draw_mesh(mesh_handle, transform)
```

* Each setter marks `CurrentDrawState` dirty.
* On `draw_mesh`, the current state is quantized into `PackedUnifiedShadingState`, a `MaterialHandle` is generated, and a `VRPCommand` is recorded.

---

### **4. Material & Texture Caching**

* **ShadingStateHandler**: maps `PackedUnifiedShadingState` → `MaterialHandle` (hash).
* **TextureBindGroupCache**: maps `[TextureHandle; 4]` → `BindGroup`.
* Only binds GPU resources when the handle changes (lazy binding).

---

### **5. Replay & Sorting**

1. Sort `DrawCommand`s by `(pipeline, texture_group, unified_shading_state_handle)`.
2. Replay: set pipeline, bind textures, upload uniforms if material changed, then draw.
3. This avoids redundant uniform uploads and bind group changes.

---

### **6. Quantization / GPU Upload**

* Floats like metallic/roughness/emissive → u8.
* Colors → RGBA8.
* Normals/directions → snorm16.
* Upload full `PackedUnifiedShadingState` per draw via write buffer.

---

### **7. Advantages of this design**

* Simple developer API, immediate-mode style.
* Minimal memory per draw (~88–96 bytes).
* Sorting & batching possible without breaking API.
* Only GPU-bound resources (textures) require caching.
* Easily portable to WebGPU, WebGL, and native.

---