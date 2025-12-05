# Mode 3: Normalized Blinn-Phong Implementation Plan

Replace Mode 3 (Hybrid: Matcap + PBR) with Mode 3 (Classic Lit: Normalized Blinn-Phong).

**Reference:** Gotanda 2010 - "Practical Implementation at tri-Ace"

---

## ⚠️ Document Status: UPDATED & CORRECTED

**Last Updated:** 2025-12-05

**Key Corrections from Codebase Review:**

1. **File Paths Corrected**: Shaders are in `emberware-z/shaders/`, NOT `emberware-z/src/graphics/shaders/`
2. **Shader Generation**: Shaders are generated at compile-time from templates via `shader_gen.rs`
3. **Emissive Location**: Currently stored in both uniform (`PackedUnifiedShadingState.emissive`) AND texture Slot 1.B (for Mode 2/3)
4. **Material State**: `PackedUnifiedShadingState` stores material properties as packed u8 values
5. **Template System**: Must update TEMPLATE_MODE3 reference in `shader_gen.rs` and all related snippets
6. **Mode 2 Impact**: Moving emissive to Slot 0.A affects BOTH Mode 2 and Mode 3
7. **Example Required**: Must add `examples/blinn-phong/` to demonstrate new Mode 3

**Additional Files Requiring Updates:**
- `emberware-z/src/shader_gen.rs` (template reference, snippets, mode name)
- `emberware-z/src/graphics/unified_shading_state.rs` (material properties)
- `emberware-z/src/ffi/mod.rs` (new FFI functions)
- `docs/ffi.md` (document new functions)
- `docs/emberware-z.md` (texture slot table, Mode 3 description)

---

## 1. Design Summary

### Why Blinn-Phong Over Hybrid?

| Aspect | Old Mode 3 (Hybrid) | New Mode 3 (Blinn-Phong) |
|--------|---------------------|--------------------------|
| Philosophy | Confused (PBR + Matcap mashup) | Clear (era-authentic lighting) |
| Specular control | Derived from metallic | Painted directly by artist |
| Edge effects | Matcap-dependent | Explicit rim lighting |
| Era feel | Modern (2010s+) | Classic (1990s-2000s) |

### Texture Layout

```
┌─────────────────────────────────────────────────────────────┐
│              SLOT 0: ALBEDO (Shared by Modes 2 & 3)         │
├─────────┬─────────┬─────────┬───────────────────────────────┤
│    R    │    G    │    B    │              A                │
├─────────┼─────────┼─────────┼───────────────────────────────┤
│ Diffuse │ Diffuse │ Diffuse │           Emissive            │
│   Red   │  Green  │  Blue   │   (multiplied with diffuse)   │
└─────────┴─────────┴─────────┴───────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│              SLOT 1: MODE 2 (PBR-lite)                      │
├─────────┬───────────┬────────────────┬──────────────────────┤
│    R    │     G     │       B        │          A           │
├─────────┼───────────┼────────────────┼──────────────────────┤
│Metallic │ Roughness │   (reserved)   │     (reserved)       │
└─────────┴───────────┴────────────────┴──────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│              SLOT 1: MODE 3 (Blinn-Phong)                   │
├─────────┬───────────┬────────────────┬──────────────────────┤
│    R    │     G     │       B        │          A           │
├─────────┼───────────┼────────────────┼──────────────────────┤
│Specular │ Specular  │   Specular     │      Shininess       │
│   Red   │   Green   │     Blue       │  (0→1 maps to 1→256) │
└─────────┴───────────┴────────────────┴──────────────────────┘
```

### Design Decisions (Resolved)

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Normalization | Gotanda linear approx | Cheap, accurate to 1000+, extrapolates fine |
| Geometry term (G) | **Skip** | Era-authentic, classical BP didn't have it, simpler |
| Fresnel | Skip (use rim instead) | Artistic control, era-authentic |
| Shininess mapping | **Linear** 1-256 | Maps cleanly to 8-bit, intuitive (0.5 = 128) |
| Light count | **4 lights + sun** | Same as Mode 2 for consistency |
| Rim color | **Sun color** | Simpler, coherent with scene lighting |
| Vertex color | **Multiplies albedo** | Consistent with all other modes |

---

## 2. Gotanda Normalization

### The Problem with Classical Blinn-Phong

High shininess values produce dim highlights because the specular lobe gets narrow but peak intensity stays the same. A shininess-1000 material looks darker than shininess-10.

### Gotanda's Solution

Normalize the BRDF so total reflected energy stays constant regardless of shininess.

**Linear approximation (Equation 12 from paper):**
```
normalization = shininess × 0.0397436 + 0.0856832
```

- Fitted via least squares for shininess 0-1000
- Negligible error across the range
- Extrapolates fine beyond 1000 (linear, monotonic)
- Much cheaper than the exact integral formula

### What We're Using vs Skipping

| From Gotanda | Using? | Reason |
|--------------|--------|--------|
| Linear normalization (Eq 12) | ✅ Yes | Core brightness fix, very cheap |
| Neumann-Neumann G term | ❌ No | Subtle grazing correction, PS1/N64 didn't have it |
| Schlick Fresnel | ❌ No | Rim lighting provides edge control instead |
| Diffuse Fresnel (1-F) | ❌ No | Subtle effect, not worth complexity |
| Ambient BRDF | ❌ No | Overkill for retro aesthetic |

---

## 3. Shader Implementation

### Binding Structure

All modes share the same binding layout for consistency:

```wgsl
// Group 0: Per-frame uniforms (same as mode2_pbr.wgsl)
@group(0) @binding(0) var<storage, read> model_matrices: array<mat4x4<f32>>;
@group(0) @binding(1) var<storage, read> view_matrices: array<mat4x4<f32>>;
@group(0) @binding(2) var<storage, read> proj_matrices: array<mat4x4<f32>>;
@group(0) @binding(3) var<storage, read> shading_states: array<PackedUnifiedShadingState>;
@group(0) @binding(4) var<storage, read> mvp_shading_indices: array<vec4<u32>>;
@group(0) @binding(5) var<storage, read> bones: array<mat4x4<f32>, 256>;

// Group 1: Textures (shared sampler for all slots)
@group(1) @binding(0) var slot0: texture_2d<f32>;  // Albedo + Emissive (Mode 3)
@group(1) @binding(1) var slot1: texture_2d<f32>;  // Specular + Shininess (Mode 3)
@group(1) @binding(2) var slot2: texture_2d<f32>;  // Unused (white fallback)
@group(1) @binding(3) var slot3: texture_2d<f32>;  // Unused (white fallback)
@group(1) @binding(4) var tex_sampler: sampler;
```

### Core Functions (WGSL)

```wgsl
// Gotanda 2010 linear approximation (Equation 12)
// Fitted for shininess 0-1000, extrapolates fine beyond
fn gotanda_normalization(shininess: f32) -> f32 {
    return shininess * 0.0397436 + 0.0856832;
}

fn normalized_blinn_phong_specular(
    N: vec3<f32>,        // Surface normal (normalized)
    V: vec3<f32>,        // View direction (normalized)
    L: vec3<f32>,        // Light direction (normalized)
    shininess: f32,      // 1-256 range (mapped from texture)
    specular_color: vec3<f32>,
    light_color: vec3<f32>,
) -> vec3<f32> {
    let H = normalize(L + V);
    
    let NdotH = max(dot(N, H), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    
    // Gotanda normalization for energy conservation
    // No geometry term - era-authentic, classical BP didn't have it
    let norm = gotanda_normalization(shininess);
    let spec = norm * pow(NdotH, shininess);
    
    return specular_color * spec * light_color * NdotL;
}

fn rim_lighting(
    N: vec3<f32>,
    V: vec3<f32>,
    rim_color: vec3<f32>,
    rim_intensity: f32,
    rim_power: f32,
) -> vec3<f32> {
    let NdotV = max(dot(N, V), 0.0);
    let rim_factor = pow(1.0 - NdotV, rim_power);
    return rim_color * rim_factor * rim_intensity;
}
```

### Fragment Shader (Sketch - see Section 5 for complete details)

**Note:** This is a conceptual overview. Actual implementation must match the binding structure in existing shaders (see `mode2_pbr.wgsl` for reference).

```wgsl
@fragment
fn fs(in: VertexOutput) -> @location(0) vec4<f32> {
    let N = normalize(in.world_normal);
    let V = normalize(in.camera_position - in.world_position);

    // ===== TEXTURE SAMPLING =====

    // Slot 0: Albedo RGB + Emissive A
    let albedo_sample = textureSample(slot0, tex_sampler, in.uv);
    var albedo = albedo_sample.rgb;
    let emissive_tex = albedo_sample.a;

    // Apply vertex color to albedo (same as all other modes)
    //FS_COLOR: albedo *= in.color;

    // Slot 1: Specular RGB + Shininess A
    let spec_shin = textureSample(slot1, tex_sampler, in.uv);
    var specular_color = spec_shin.rgb;
    let shininess_raw = spec_shin.a;
    
    // Linear mapping: 0→1, 0.5→128, 1→256
    let shininess = mix(1.0, 256.0, shininess_raw);
    
    // ===== LIGHTING =====
    
    var total_diffuse = vec3<f32>(0.0);
    var total_specular = vec3<f32>(0.0);
    
    // Sun light (from procedural sky)
    let sun_L = normalize(sky.sun_direction);
    let sun_NdotL = max(dot(N, sun_L), 0.0);
    total_diffuse += albedo * sky.sun_color * sun_NdotL;
    total_specular += normalized_blinn_phong_specular(
        N, V, sun_L, shininess, specular_color, sky.sun_color
    );
    
    // 4 additional lights (same count as Mode 2)
    for (var i = 0u; i < 4u; i++) {
        if (lights[i].intensity > 0.0) {
            let L = normalize(lights[i].direction);
            let light_color = lights[i].color * lights[i].intensity;
            let NdotL = max(dot(N, L), 0.0);
            
            total_diffuse += albedo * light_color * NdotL;
            total_specular += normalized_blinn_phong_specular(
                N, V, L, shininess, specular_color, light_color
            );
        }
    }
    
    // Ambient from procedural sky (sampled at normal direction)
    let ambient = albedo * sample_sky(N) * 0.3;
    
    // Rim lighting (uses sun color)
    // Note: rim_intensity and rim_power come from unpacked shading state
    // See Section 5.2 for material property storage details
    let rim = rim_lighting(
        N, V,
        sky.sun_color,
        rim_intensity,  // From shading state (see Section 5.2)
        rim_power       // From shading state (see Section 5.2)
    );

    // Emissive: Albedo × intensity (self-illumination)
    let emissive = albedo * emissive_tex;
    
    // ===== COMBINE =====
    
    let final_color = total_diffuse + ambient + total_specular + rim + emissive;
    
    return vec4<f32>(final_color, 1.0);
}
```

---

## 4. Performance Comparison

| Operation | Mode 2 (GGX/PBR) | Mode 3 (Blinn-Phong) |
|-----------|------------------|----------------------|
| Distribution | GGX (expensive) | pow(NdotH, n) (cheap) |
| Fresnel | Schlick pow5 | None (rim instead) |
| Geometry | Smith GGX | None |
| Normalization | Complex | Linear (1 mul + 1 add) |
| Lights | 4 + sun | 4 + sun |

Mode 3 is significantly cheaper per light than Mode 2.

---

## 5. Implementation Details

### 5.1 Shader Generator Changes (`shader_gen.rs`)

The shader generation system needs several updates:

1. **Update template reference (line 53)**:
   ```rust
   const TEMPLATE_MODE3: &str = include_str!("../shaders/mode3_blinnphong.wgsl");
   ```

2. **Update FS_ALBEDO_UV snippet (line 112)** to extract emissive from Slot 0 alpha:
   ```rust
   const FS_ALBEDO_UV: &str = "let albedo_sample = textureSample(slot0, tex_sampler, in.uv); albedo *= albedo_sample.rgb; let emissive_tex = albedo_sample.a;";
   ```

3. **Add new snippet for Blinn-Phong texture sampling**:
   ```rust
   const FS_SPECULAR_SHININESS: &str = "let spec_shin = textureSample(slot1, tex_sampler, in.uv); specular_color = spec_shin.rgb; shininess_raw = spec_shin.a;";
   ```

4. **Update FS_MRE snippet (line 219)** - Mode 2 now samples emissive from Slot 0.A instead of Slot 1.B:
   ```rust
   // OLD: let mre_sample = textureSample(slot1, tex_sampler, in.uv);\n    mre = vec3<f32>(mre_sample.r, mre_sample.g, mre_sample.b);
   // NEW: let mre_sample = textureSample(slot1, tex_sampler, in.uv);\n    mre.r = mre_sample.r;\n    mre.g = mre_sample.g;
   ```

5. **Update mode_name function (line 254)**:
   ```rust
   3 => "Blinn-Phong",  // Changed from "Hybrid"
   ```

6. **Add placeholder replacement for Mode 3** (in generate_shader function around line 212):
   ```rust
   2 | 3 => {
       // Mode 2 (PBR) and Mode 3 (Blinn-Phong) - use "albedo" variable
       shader = shader.replace("//FS_COLOR", if has_color { FS_ALBEDO_COLOR } else { "" });
       shader = shader.replace("//FS_UV", if has_uv { FS_ALBEDO_UV } else { "" });

       // Mode-specific texture sampling
       if mode == 2 {
           // Mode 2: MR texture (Metallic-Roughness)
           if has_uv {
               shader = shader.replace("//FS_MR", "let mr_sample = textureSample(slot1, tex_sampler, in.uv);\n    mr.x = mr_sample.r;\n    mr.y = mr_sample.g;");
           } else {
               shader = shader.replace("//FS_MR", "");
           }
       } else {
           // Mode 3: Specular-Shininess texture
           if has_uv {
               shader = shader.replace("//FS_SPECULAR_SHININESS", FS_SPECULAR_SHININESS);
           } else {
               shader = shader.replace("//FS_SPECULAR_SHININESS", "");
           }
       }
   }
   ```

   **Note:** This replaces the old Mode 2/3 block that used `//FS_MRE`.

### 5.2 Unified Shading State Changes

**Material Property Architecture:**

Mode 3 uses a texture-based workflow (like Mode 2) with uniform fallbacks:

| Property | Source | Fallback |
|----------|--------|----------|
| **Specular color** (RGB) | Slot 1 texture (RGB channels) | Uniform (3 bytes) |
| **Shininess** | Slot 1 texture (A channel) | Uniform (1 byte) |
| **Rim intensity** | Uniform only | — |
| **Rim power** | Uniform only | — |
| **Emissive** | Slot 0 texture (A channel) | Uniform (1 byte) |

**Current `PackedUnifiedShadingState` layout:**
```rust
pub metallic: u8,              // 4 bytes
pub roughness: u8,
pub emissive: u8,
pub pad0: u8,
pub color_rgba8: u32,          // 4 bytes (used by all modes)
pub blend_mode: u32,           // 4 bytes (used by all modes)
pub matcap_blend_modes: u32,   // 4 bytes (Mode 1 only)
pub sky: PackedSky,            // 16 bytes
pub lights: [PackedLight; 4],  // 32 bytes
```

**Proposed: Mode-Specific Field Interpretation**

Reuse existing fields with different meanings per mode (no struct changes):

```rust
// Fields 0-3: Mode-specific material properties
pub material_param_0: u8,  // Mode 2: metallic,    Mode 3: specular_r
pub material_param_1: u8,  // Mode 2: roughness,   Mode 3: specular_g
pub material_param_2: u8,  // Mode 2: emissive,    Mode 3: specular_b
pub material_param_3: u8,  // Mode 2: pad (unused), Mode 3: shininess

// Bytes from matcap_blend_modes (Mode 1 only, Mode 3 can repurpose)
// matcap_blend_modes: u32 (byte layout: [mode1, mode2, mode3, mode4])
//   Mode 3 uses bytes 0-1: rim_intensity (byte 0), rim_power (byte 1)
//   Mode 1 uses all 4 bytes for matcap blend modes
```

**In shader code:**

```wgsl
// Mode 3 unpacking in fragment shader:
let shading = shading_states[in.shading_state_index];

// Unpack specular color + shininess (uniform defaults)
let specular_r = unpack_unorm8_from_u32(shading.material_param_0);
let specular_g = unpack_unorm8_from_u32(shading.material_param_1);
let specular_b = unpack_unorm8_from_u32(shading.material_param_2);
let shininess_uniform = unpack_unorm8_from_u32(shading.material_param_3);

var specular_color = vec3<f32>(specular_r, specular_g, specular_b);
var shininess_raw = shininess_uniform;

// Texture overrides (from Slot 1 if UV format present)
//FS_SPECULAR_SHININESS  // Replaces specular_color and shininess_raw

// Map shininess 0-1 → 1-256
let shininess = mix(1.0, 256.0, shininess_raw);

// Unpack rim parameters (from matcap_blend_modes bytes 0-1)
let rim_intensity = unpack_unorm8_from_u32(shading.matcap_blend_modes & 0xFFu);
let rim_power = unpack_unorm8_from_u32((shading.matcap_blend_modes >> 8u) & 0xFFu) * 32.0;  // Map to 0-32 range
```

**Benefits:**
- ✅ No struct size change
- ✅ Zero runtime cost
- ✅ Clear separation: texture = primary, uniform = fallback
- ✅ Rim parameters stay uniform-only (no texture complexity)

### 5.3 Texture Slot Usage

Mode 3 only uses Slots 0 and 1. However, the binding layout (see `create_texture_bind_group_layout` in `pipeline.rs`) requires all 4 texture slots to be bound.

**Solution:** Bind white 1×1 fallback textures to unused slots:

| Mode | Slot 0 | Slot 1 | Slot 2 | Slot 3 |
|------|--------|--------|--------|--------|
| **3 (Blinn-Phong)** | Albedo+Emissive (UV) | Specular+Shininess (UV) | White fallback | White fallback |

The white fallback texture (value `1.0` in all channels) ensures unused slots don't affect rendering. This matches the existing pattern used in Mode 0 and Mode 2.

### 5.4 FFI Function Changes

**New functions for Mode 3 material control:**

```rust
/// Set specular color (uniform fallback, overridden by Slot 1 texture RGB)
fn material_specular(r: f32, g: f32, b: f32);

/// Set shininess (uniform fallback, overridden by Slot 1 texture A)
/// Maps 0.0-1.0 to shininess range 1-256
fn material_shininess(value: f32);

/// Set rim lighting parameters (uniform-only, no texture override)
/// intensity: 0.0-1.0, power: 0.0-1.0 (mapped to 0-32 internally)
fn material_rim(intensity: f32, power: f32);
```

**How they map to `PackedUnifiedShadingState`:**

| FFI Function | Field | Mode 2 Meaning | Mode 3 Meaning |
|--------------|-------|----------------|----------------|
| `material_metallic(v)` | `material_param_0` | Metallic | Specular R (if `material_specular` not called) |
| `material_roughness(v)` | `material_param_1` | Roughness | Specular G (if `material_specular` not called) |
| `material_emissive(v)` | `material_param_2` | Emissive | Specular B (if `material_specular` not called) |
| `material_specular(r,g,b)` | `material_param_0-2` | N/A | Specular RGB (Mode 3 only) |
| `material_shininess(v)` | `material_param_3` | N/A | Shininess (Mode 3 only) |
| `material_rim(i, p)` | `matcap_blend_modes` bytes 0-1 | N/A | Rim intensity/power (Mode 3 only) |

**Recommendation:** Add Mode 3-specific functions for clarity. Developers calling `material_specular()` in Mode 3 is clearer than repurposing `material_metallic()`.

---

## 6. Files to Modify

| Action | File | Changes |
|--------|------|---------|
| **Create** | `emberware-z/shaders/mode3_blinnphong.wgsl` | **NEW** shader template ⚠️ Path corrected! |
| **Modify** | `emberware-z/shaders/mode2_pbr.wgsl` | Move emissive from Slot 1.B to Slot 0.A ⚠️ Path corrected! |
| **Modify** | `emberware-z/src/shader_gen.rs` | Update TEMPLATE_MODE3, FS_ALBEDO_UV, FS_MRE snippets, mode_name |
| **Modify** | `emberware-z/src/graphics/unified_shading_state.rs` | Add BlinnPhong material fields (specular, shininess, rim) |
| **Modify** | `emberware-z/src/ffi/mod.rs` | Add FFI functions for BP material (rim_intensity, rim_power) |
| **Delete** | `emberware-z/shaders/mode3_hybrid.wgsl` | Remove old shader ⚠️ Path corrected! |
| **Modify** | `docs/emberware-z.md` | Update Mode 3 docs, texture slot table (line ~171, ~535) |
| **Create** | `examples/blinn-phong/` | **NEW** example demonstrating Mode 3 Blinn-Phong |
| **Modify** | `CLAUDE.md` (optional) | Update if shader architecture is documented |

---

## 7. Mode 2 PBR Changes (Emissive Migration)

Mode 2 must also move emissive from Slot 1.B to Slot 0.A for consistency.

### Current State (Mode 2)

**Emissive is currently:**
1. Stored in `PackedUnifiedShadingState.emissive` (u8) as a uniform default
2. Can be overridden by sampling Slot 1.B (MRE texture blue channel)
3. Applied as `let glow = albedo * mre.b;` where `mre.b` comes from texture or uniform

**How it's sampled currently** (shader_gen.rs line 219):
```rust
if has_uv {
    shader = shader.replace("//FS_MRE",
        "let mre_sample = textureSample(slot1, tex_sampler, in.uv);\n    mre = vec3<f32>(mre_sample.r, mre_sample.g, mre_sample.b);");
}
```

**Why this needs to change:**
- Slot 0 alpha channel is currently unused (always multiplied as opacity but rarely used)
- Emissive is more naturally associated with the albedo texture
- Frees up Slot 1.B for other purposes (though Mode 2 won't use it)
- Consistency with Mode 3 which needs Slot 1 for specular+shininess

### Shader Changes (`mode2_pbr.wgsl`)

1. **Update comment (line 4)**:
   ```wgsl
   // OLD: MRE texture in slot 1 (R=Metallic, G=Roughness, B=Emissive)
   // NEW: MR texture in slot 1 (R=Metallic, G=Roughness), Emissive in Slot 0.A
   ```

2. **Update texture binding comment (line 54)**:
   ```wgsl
   // OLD: @group(1) @binding(1) var slot1: texture_2d<f32>;  // MRE (Metallic-Roughness-Emissive)
   // NEW: @group(1) @binding(1) var slot1: texture_2d<f32>;  // MR (Metallic-Roughness)
   ```

3. **Update fragment shader sampling**:
   ```wgsl
   // Unpack material properties from shading state
   let metallic = unpack_unorm8_from_u32(mre_packed & 0xFFu);
   let roughness = unpack_unorm8_from_u32((mre_packed >> 8u) & 0xFFu);
   let emissive = unpack_unorm8_from_u32((mre_packed >> 16u) & 0xFFu);

   // Get albedo + emissive from Slot 0
   var albedo = material_color.rgb;
   //FS_COLOR
   //FS_UV  // This now extracts emissive_tex from Slot 0.A

   // Sample MR texture (Metallic-Roughness only, no Emissive)
   var mr = vec2<f32>(metallic, roughness);
   //FS_MR  // NEW placeholder (was FS_MRE): only overrides mr.r and mr.g

   // Emissive: texture overrides uniform if present
   // emissive_tex comes from FS_UV snippet extracting Slot 0.A
   let emissive_final = max(emissive, emissive_tex);
   let glow = albedo * emissive_final;
   ```

   **Note:** The `emissive_tex` variable is created by the updated `FS_ALBEDO_UV` snippet (see Section 5.1).

---

## 8. Example Game

Create `examples/blinn-phong/` to demonstrate Mode 3:

### Features to Showcase

1. **Specular variation**: Different shininess values (10, 50, 128, 200)
2. **Specular color**: Gold (warm), silver (neutral), copper (orange-tinted)
3. **Rim lighting**: Character silhouettes, edge highlights
4. **Multiple lights**: Demonstrate 4 lights + sun
5. **Texture control**: Show both uniform and texture-driven specular/shininess
6. **Material presets**: Wood, metal, plastic, skin, cloth

### Example Structure

```
examples/blinn-phong/
├── Cargo.toml
├── src/
│   └── lib.rs
└── assets/
    ├── sphere.obj
    ├── character.obj
    ├── albedo.png       (RGB: diffuse color)
    ├── specular.png     (RGB: specular color, A: shininess)
    └── README.md        (Material authoring guide)
```

---

## 9. Example Materials

### Authoring Guide

| Material | Specular Color | Shininess | Rim | Notes |
|----------|---------------|-----------|-----|-------|
| Gold armor | `(0.9, 0.6, 0.2)` | 0.8 | 0.2 | Warm orange specular |
| Silver metal | `(0.9, 0.9, 0.9)` | 0.85 | 0.15 | Neutral white specular |
| Leather | `(0.3, 0.2, 0.1)` | 0.3 | 0.1 | Dark, broad highlights |
| Wet skin | `(0.8, 0.8, 0.8)` | 0.7 | 0.3 | Bright, with rim |
| Matte plastic | `(0.4, 0.4, 0.4)` | 0.5 | 0.0 | Subtle highlights |
| Silk cloth | `(0.5, 0.5, 0.6)` | 0.4 | 0.2 | Slight blue tint |

### Shininess Reference (Linear Mapping)

| Texture Value | Mapped Shininess | Visual | Use For |
|---------------|------------------|--------|---------|
| 0.0 - 0.2 | 1-52 | Very broad, soft | Cloth, skin, rough stone |
| 0.2 - 0.4 | 52-103 | Broad | Leather, wood, rubber |
| 0.4 - 0.6 | 103-154 | Medium | Plastic, painted metal |
| 0.6 - 0.8 | 154-205 | Tight | Polished metal, wet surfaces |
| 0.8 - 1.0 | 205-256 | Very tight | Chrome, mirrors, glass |

---

## 7. Migration Notes

### For Games Using Old Mode 3 (Hybrid)

1. **Slot 0.A:** Now carries emissive (was unused)
2. **Slot 1:** Changes from MRE to Specular RGB + Shininess A
3. **Slots 2-3:** No longer used (remove matcap textures)

### Texture Conversion Guide

| Old (Hybrid) | New (Blinn-Phong) |
|--------------|-------------------|
| Slot 1.R (Metallic) | → Paint specular color based on material type |
| Slot 1.G (Roughness) | → Invert to shininess: `1.0 - roughness` |
| Slot 1.B (Emissive) | → Move to Slot 0.A |
| Slot 2 (Env Matcap) | → Delete (unused) |
| Slot 3 (Matcap) | → Delete (unused) |

---

## 11. Testing Checklist

### Mode 3 (Blinn-Phong) Tests
- [ ] Gotanda normalization produces consistent brightness across shininess range (1-256)
- [ ] No visible energy blow-up at grazing angles (despite no G term)
- [ ] Rim lighting works independently of specular
- [ ] All 4 lights + sun contribute correctly
- [ ] Emissive from Slot 0.A functions correctly (both uniform and texture)
- [ ] Specular color variation (gold, silver, copper) displays correctly
- [ ] Shininess variation (soft to sharp) behaves as expected
- [ ] Vertex color multiplies albedo correctly
- [ ] Formats without normals fall back to Mode 0 with warning
- [ ] Gold material looks warm/orange (not PBR-derived metallic)
- [ ] All 40 shader permutations compile successfully (naga validation)

### Mode 2 (PBR) Regression Tests
- [ ] Mode 2 still works after emissive relocation to Slot 0.A
- [ ] Emissive from Slot 0.A renders identically to old Slot 1.B
- [ ] MR texture (no emissive channel) still works correctly
- [ ] Existing Mode 2 games/examples render unchanged

### Comparison Tests
- [ ] Mode 2 vs Mode 3 renders show intentional differences (PBR vs Blinn-Phong)
- [ ] Example game demonstrates all Mode 3 features

---

## 12. TODO: Implementation Tasks

### Core Implementation (Required)
- [ ] Create `emberware-z/shaders/mode3_blinnphong.wgsl` shader template
- [ ] Update `emberware-z/shaders/mode2_pbr.wgsl` (emissive → Slot 0.A)
- [ ] Update `emberware-z/src/shader_gen.rs`:
  - [ ] TEMPLATE_MODE3 reference
  - [ ] FS_ALBEDO_UV snippet (extract emissive from alpha)
  - [ ] FS_MRE → FS_MR rename and update
  - [ ] FS_SPECULAR_SHININESS snippet (Mode 3)
  - [ ] mode_name: "Hybrid" → "Blinn-Phong"
- [ ] Update `docs/emberware-z.md`:
  - [ ] Mode 3 description (line ~171, ~293)
  - [ ] Texture slot table (line ~535)
  - [ ] Lighting section

### Material System (Required)
- [ ] Implement mode-specific field interpretation (Section 5.2)
  - [ ] Mode 2: metallic/roughness/emissive interpretation
  - [ ] Mode 3: specular RGB/shininess interpretation + rim from matcap_blend_modes
- [ ] Add FFI functions to `emberware-z/src/ffi/mod.rs`:
  - [ ] `material_specular(r: f32, g: f32, b: f32)` - sets specular color uniform
  - [ ] `material_shininess(value: f32)` - sets shininess uniform (0.0-1.0)
  - [ ] `material_rim(intensity: f32, power: f32)` - sets rim parameters

### Example Game (Required)
- [ ] Create `examples/blinn-phong/` directory structure
- [ ] Implement example showcasing:
  - [ ] Multiple shininess values
  - [ ] Specular color variation
  - [ ] Rim lighting
  - [ ] 4 lights + sun
  - [ ] Texture-driven specular/shininess
- [ ] Add asset files (sphere/character mesh, textures)
- [ ] Write material authoring guide (README)

### Documentation (Required)
- [ ] Update `docs/ffi.md` with new material functions
- [ ] Add migration guide for old Mode 3 users
- [ ] Update CLAUDE.md if shader architecture is documented

### Cleanup (Required)
- [ ] Delete `emberware-z/shaders/mode3_hybrid.wgsl`
- [ ] Run all shader compilation tests
- [ ] Verify no regressions in existing examples