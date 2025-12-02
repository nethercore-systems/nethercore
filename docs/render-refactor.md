TODO: Write this document
TODO: Consider quantized + sorting quantized data 

# Spec: Unified shading inputs + cached texture bind groups

Below may contain small semantic differences than current code. The ideas are the main point to follow.

## Principle

* **Textures & samplers → cached bind groups** (discrete GPU resources). We only ever need 2 samplers in existince EVER.
* **Per-draw floats/enums (metallic/roughness/emissive/blend modes, light/sky params) → small POD uniform or push-constant**. Update these when they change; do not create bind groups for them.
* **Textures are authoritative when present; uniforms are fallbacks** (shader samples texture when available, else uses uniform/constant).

---

# CPU / GPU data model

### CPU-side POD types (semantic)

```rust
// -- LABEL: UnifiedShadingState (CPU-side POD) --
// -- COMMENT: Pure CPU representation. Can be treated as "Material"
// -- WARNING: Need to ensure no alignment issues
// -- NOTE: This can be passed directly to the GPU and inserted into uniforms
#[derive(Clone, Debug)]
pub struct UnifiedShadingState {
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: f32,
    pub color: [f32; 4],  //or u32
    pub blend_modes: [MatcapBlendMode; 4],      // small enums -> cast to u32 for shader
    pub sky: SkyState,                 // handles reference larger structures
    pub lights: LightState,         // handle to chosen light subset
    // Note: texture handles are kept separately due to needing bind groups
    // textures: [TextureHandle; 4]
}

unsafe impl bytemuck::Pod for UnifiedShadingState {}
unsafe impl bytemuck::Zeroable for UnifiedShadingState {}
```

# Texture bind-group caching rules

* Key for cache: exact ordered `[TextureHandle; 4]` (normalize `None` → sentinel handle).
* Cache only textures. **Do not** include floats/enums in this key.
* Create bind-group on first use and reuse thereafter.
---

# Update & draw-time flow (simple)

1. Build or fetch `UnifiedShadingState` for the draw.
2. Determine `textures: [TextureHandle;4]`; `bind_group = texture_cache.get_or_create(textures)`.
3. If `UnifiedShadingState` differs from previously bound shading state:
 - write dynamic UBO or SSBO entry (via `queue.write_buffer` or ring buffer).
4. Bind frame globals, bind group (textures), set material index/push constants, draw.

Update only when values actually change to minimize `write_buffer` calls.

---
