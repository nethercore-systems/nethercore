# Mode 3: Normalized Blinn-Phong Implementation Plan

Replace Mode 3 (Hybrid: Matcap + PBR) with Mode 3 (Classic Lit: Normalized Blinn-Phong).

**Reference:** Gotanda 2010 - "Practical Implementation at tri-Ace"

---

## ⚠️ Document Status: FINAL REVIEW COMPLETE

**Last Updated:** 2025-12-05 (Final Pass)

**Verified Against Codebase:**

1. ✅ **File Paths**: All paths verified against actual codebase structure
2. ✅ **Shader Generation**: Correctly references `shader_gen.rs` line numbers and template system
3. ✅ **Texture Layout**: 3-slot design verified (Slot 0: Albedo, Slot 1: RSE, Slot 2: Specular)
4. ✅ **Material State**: Mode-specific field interpretation strategy documented (no struct changes)
5. ✅ **Mode 2 Compatibility**: NO changes to Mode 2 - emissive stays in Slot 1.B for both modes
6. ✅ **Implementation Files**: All required file modifications listed with exact locations

**Key Design Decisions:**

- **Emissive stays in Slot 1.B** for both Mode 2 (MRE) and Mode 3 (RSE) - no migration!
- **Specular color** comes ONLY from Slot 2 RGB (defaults to white, no uniform)
- **Mode-specific interpretation**: Same struct fields mean different things per mode
- **Rim power**: Uniform-only (stored in `matcap_blend_modes` byte 0)

**Files Requiring Updates:**
- `emberware-z/shaders/mode3_blinnphong.wgsl` (NEW)
- `emberware-z/src/shader_gen.rs` (line 53, add FS_MODE3_SLOT1/SLOT2, update mode_name)
- `emberware-z/src/graphics/unified_shading_state.rs` (documentation only)
- `emberware-z/src/ffi/mod.rs` (add `material_rim()` and `material_shininess()`)
- `docs/ffi.md` (document new functions)
- `docs/emberware-z.md` (update Mode 3 description and texture table)
- `examples/blinn-phong/` (NEW example)

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
│ Diffuse │ Diffuse │ Diffuse │           Unused              │
│   Red   │  Green  │  Blue   │      (reserved for UI)        │
└─────────┴─────────┴─────────┴───────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│              SLOT 1: MODE 2 (PBR-lite)                      │
├─────────┬───────────┬────────────────┬──────────────────────┤
│    R    │     G     │       B        │          A           │
├─────────┼───────────┼────────────────┼──────────────────────┤
│Metallic │ Roughness │    Emissive    │      Unused          │
└─────────┴───────────┴────────────────┴──────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│              SLOT 1: MODE 3 (Blinn-Phong)                   │
├─────────┬───────────┬────────────────┬──────────────────────┤
│    R    │     G     │       B        │          A           │
├─────────┼───────────┼────────────────┼──────────────────────┤
│   Rim   │ Shininess │    Emissive    │      Unused          │
│Intensity│ (0→1→1-256)│(albedo multiply)│                     │
└─────────┴───────────┴────────────────┴──────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│              SLOT 2: MODE 3 (Blinn-Phong)                   │
├─────────┬───────────┬────────────────┬──────────────────────┤
│    R    │     G     │       B        │          A           │
├─────────┼───────────┼────────────────┼──────────────────────┤
│Specular │ Specular  │   Specular     │      Unused          │
│   Red   │   Green   │     Blue       │                      │
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
@group(1) @binding(0) var slot0: texture_2d<f32>;  // Albedo (RGB, A unused for meshes)
@group(1) @binding(1) var slot1: texture_2d<f32>;  // Mode 2: MRE, Mode 3: Rim+Shininess+Emissive
@group(1) @binding(2) var slot2: texture_2d<f32>;  // Mode 3: Specular RGB (A unused)
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

    // Slot 0: Albedo RGB (A unused for meshes)
    let albedo_sample = textureSample(slot0, tex_sampler, in.uv);
    var albedo = albedo_sample.rgb;

    // Apply vertex color to albedo (same as all other modes)
    //FS_COLOR: albedo *= in.color;

    // Slot 1: Rim (R) + Shininess (G) + Emissive (B)
    let slot1_sample = textureSample(slot1, tex_sampler, in.uv);
    let rim_intensity = slot1_sample.r;
    let shininess_raw = slot1_sample.g;
    let emissive = slot1_sample.b;

    // Slot 2: Specular color RGB (A unused)
    let specular_sample = textureSample(slot2, tex_sampler, in.uv);
    var specular_color = specular_sample.rgb;

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
    // rim_intensity from Slot 1.R, rim_power from uniform (see Section 5.2)
    let rim = rim_lighting(
        N, V,
        sky.sun_color,
        rim_intensity,  // From Slot 1.R
        rim_power       // From uniform (see Section 5.2)
    );

    // Emissive: Albedo × intensity (self-illumination)
    let emissive_glow = albedo * emissive;  // emissive from Slot 1.B
    
    // ===== COMBINE =====
    
    let final_color = total_diffuse + ambient + total_specular + rim + emissive_glow;
    
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

**File:** `emberware-z/src/shader_gen.rs`

The shader generation system needs several updates:

1. **Update template reference (line 53)**:
   ```rust
   const TEMPLATE_MODE3: &str = include_str!("../shaders/mode3_blinnphong.wgsl");
   ```

2. **Add new snippet constants for Mode 3 texture sampling** (add after existing FS_* constants around line 100):
   ```rust
   // Mode 3: Sample Slot 1 (RSE: Rim, Shininess, Emissive)
   const FS_MODE3_SLOT1: &str =
       "let slot1_sample = textureSample(slot1, tex_sampler, in.uv);\n    \
        rim_intensity = slot1_sample.r;\n    \
        shininess_raw = slot1_sample.g;\n    \
        emissive = slot1_sample.b;";

   // Mode 3: Sample Slot 2 (Specular RGB)
   const FS_MODE3_SLOT2: &str =
       "let specular_sample = textureSample(slot2, tex_sampler, in.uv);\n    \
        specular_color = specular_sample.rgb;";
   ```

3. **Update mode_name function** (around line 254):
   ```rust
   fn mode_name(mode: u8) -> &'static str {
       match mode {
           0 => "Unlit",
           1 => "Matcap",
           2 => "PBR",
           3 => "Blinn-Phong",  // Changed from "Hybrid"
           _ => "Unknown",
       }
   }
   ```

4. **Update placeholder replacement for Modes 2 & 3** (in generate_shader function around line 212):
   ```rust
   2 | 3 => {
       // Mode 2 (PBR) and Mode 3 (Blinn-Phong) - both use "albedo" variable
       shader = shader.replace("//FS_COLOR", if has_color { FS_ALBEDO_COLOR } else { "" });
       shader = shader.replace("//FS_UV", if has_uv { FS_ALBEDO_UV } else { "" });

       // Mode-specific texture sampling
       if mode == 2 {
           // Mode 2: MRE texture (Metallic-Roughness-Emissive) in Slot 1
           if has_uv {
               shader = shader.replace("//FS_MRE",
                   "let mre_sample = textureSample(slot1, tex_sampler, in.uv);\n    \
                    mre = vec3<f32>(mre_sample.r, mre_sample.g, mre_sample.b);");
           } else {
               shader = shader.replace("//FS_MRE", "");
           }
       } else {
           // Mode 3: Slot 1 (RSE: Rim+Shininess+Emissive) + Slot 2 (Specular RGB)
           if has_uv {
               shader = shader.replace("//FS_MODE3_SLOT1", FS_MODE3_SLOT1);
               shader = shader.replace("//FS_MODE3_SLOT2", FS_MODE3_SLOT2);
           } else {
               shader = shader.replace("//FS_MODE3_SLOT1", "");
               shader = shader.replace("//FS_MODE3_SLOT2", "");
           }
       }
   }
   ```

**Important notes:**
- Mode 2 keeps existing FS_MRE logic unchanged (emissive stays in Slot 1.B)
- Mode 3 uses two separate placeholders: FS_MODE3_SLOT1 and FS_MODE3_SLOT2
- Both modes share FS_COLOR and FS_ALBEDO_UV for vertex color and albedo sampling
- No changes to Mode 0 or Mode 1 shader generation

### 5.2 Unified Shading State Changes

**Material Property Architecture:**

Mode 3 uses a texture-based workflow (like Mode 2) with uniform fallbacks:

| Property | Source | Fallback | Notes |
|----------|--------|----------|-------|
| **Specular color** (RGB) | Slot 2 RGB | White (1.0, 1.0, 1.0) default | No uniform - defaults to white for neutral highlights |
| **Shininess** | Slot 1 G | Uniform (1 byte) | Shared slot with rim & emissive |
| **Rim intensity** | Slot 1 R | Uniform (1 byte) | Shared slot with shininess & emissive |
| **Rim power** | Uniform only | — | Controls rim falloff curve |
| **Emissive** | Slot 1 B | Uniform (1 byte) | **Same as Mode 2!** |

**Current `PackedUnifiedShadingState` structure** (`emberware-z/src/graphics/unified_shading_state.rs:29`):

```rust
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedUnifiedShadingState {
    pub metallic: u8,             // Byte 0
    pub roughness: u8,            // Byte 1
    pub emissive: u8,             // Byte 2
    pub pad0: u8,                 // Byte 3
    pub color_rgba8: u32,         // 4 bytes (used by all modes)
    pub blend_mode: u32,          // 4 bytes (used by all modes)
    pub matcap_blend_modes: u32,  // 4 bytes (Mode 1: matcap modes, Mode 3: rim_power)
    pub sky: PackedSky,           // 16 bytes
    pub lights: [PackedLight; 4], // 32 bytes
}
```

**Mode-Specific Field Interpretation Strategy:**

The same struct fields are interpreted differently per render mode (no struct changes needed):

| Field | Bytes | Mode 2 (PBR) | Mode 3 (Blinn-Phong) |
|-------|-------|--------------|----------------------|
| `metallic` | 0 | Metallic | **Rim intensity** (uniform fallback for Slot 1.R) |
| `roughness` | 1 | Roughness | **Shininess** (uniform fallback for Slot 1.G) |
| `emissive` | 2 | Emissive | **Emissive** (same meaning! uniform fallback for Slot 1.B) |
| `pad0` | 3 | Unused | Unused |
| `matcap_blend_modes` | 4 bytes | Unused | **Rim power** (byte 0, uniform-only, no texture) |

**Key insights:**
1. **Emissive stays in Slot 1.B for both modes** - no migration needed!
2. Mode 3 reinterprets `metallic` → `rim_intensity` and `roughness` → `shininess`
3. Specular color comes ONLY from Slot 2 RGB (defaults to white if not bound)
4. Rim power is uniform-only (stored in `matcap_blend_modes` byte 0)

**Shader unpacking for Mode 3** (in `mode3_blinnphong.wgsl`):

```wgsl
// Get shading state for this draw
let shading = shading_states[in.shading_state_index];

// Unpack uniform fallback values (reinterpreted for Mode 3)
let rim_intensity_uniform = unpack_unorm8_from_u32(shading.metallic);  // Reinterpret metallic → rim
let shininess_uniform = unpack_unorm8_from_u32(shading.roughness);     // Reinterpret roughness → shininess
let emissive_uniform = unpack_unorm8_from_u32(shading.emissive);       // Same meaning as Mode 2

// Initialize with uniform defaults
var rim_intensity = rim_intensity_uniform;
var shininess_raw = shininess_uniform;
var emissive = emissive_uniform;
var specular_color = vec3<f32>(1.0, 1.0, 1.0);  // Default to white

// Texture sampling (if UV format present)
//FS_MODE3_SLOT1  // Overrides rim_intensity, shininess_raw, emissive from Slot 1 RGB
//FS_MODE3_SLOT2  // Overrides specular_color from Slot 2 RGB

// Map shininess 0-1 → 1-256
let shininess = mix(1.0, 256.0, shininess_raw);

// Unpack rim power (uniform-only, from matcap_blend_modes byte 0)
let rim_power_raw = unpack_unorm8_from_u32(shading.matcap_blend_modes & 0xFFu);
let rim_power = rim_power_raw * 32.0;  // Map 0-1 → 0-32 range
```

**Benefits:**
- ✅ No struct size change (64 bytes, same as before)
- ✅ Zero runtime cost (just different field interpretation)
- ✅ Emissive stays in Slot 1.B for both modes (no Mode 2 changes!)
- ✅ Clear texture-first workflow with uniform fallbacks
- ✅ Specular defaults to white (neutral highlights) when Slot 2 not bound

### 5.3 Texture Slot Usage

Mode 3 uses 3 texture slots (0, 1, 2). The binding layout (see `create_texture_bind_group_layout` in `pipeline.rs`) requires all 4 texture slots to be bound.

**Solution:** Bind white 1×1 fallback textures to unused slots:

| Mode | Slot 0 | Slot 1 | Slot 2 | Slot 3 |
|------|--------|--------|--------|--------|
| **2 (PBR)** | Albedo RGB | MRE (Metallic-Roughness-Emissive) | White fallback | White fallback |
| **3 (Blinn-Phong)** | Albedo RGB | RSE (Rim-Shininess-Emissive) | Specular RGB | White fallback |

**Default texture behavior:**
- **Slot 0** (Albedo): Defaults to white (1.0, 1.0, 1.0) if not bound → uses material color
- **Slot 1** (Mode 3: RSE): Defaults to white (1.0, 1.0, 1.0) if not bound → uses uniform fallbacks
- **Slot 2** (Mode 3: Specular): Defaults to white (1.0, 1.0, 1.0) if not bound → neutral white specular highlights
- **Slot 3**: Always white fallback (unused by Modes 2 & 3)

The white fallback texture (value `1.0` in all channels) ensures:
- Textures multiply cleanly (white = no effect)
- Missing textures fall back to uniform values
- Specular highlights remain neutral when Slot 2 is not set

This matches the existing pattern used in Mode 0 and Mode 1.

### 5.4 FFI Function Changes

**Existing functions (Mode 2 & 3 reinterpret the same fields):**

```rust
// Mode 2: Sets metallic uniform fallback
// Mode 3: Sets rim_intensity uniform fallback (Slot 1.R)
fn material_metallic(value: f32);

// Mode 2: Sets roughness uniform fallback
// Mode 3: Sets shininess uniform fallback (Slot 1.G)
fn material_roughness(value: f32);

// Mode 2: Sets emissive uniform fallback (Slot 1.B)
// Mode 3: Sets emissive uniform fallback (Slot 1.B) - same meaning!
fn material_emissive(value: f32);
```

**New Mode 3-specific functions for clarity** (add to `emberware-z/src/ffi/mod.rs`):

```rust
/// Set rim lighting parameters (Mode 3 only)
/// intensity: 0.0-1.0 (uniform fallback for Slot 1.R)
/// power: 0.0-1.0 (mapped to 0-32 internally, uniform-only, no texture)
fn material_rim(intensity: f32, power: f32);

/// Set shininess (Mode 3 only, alias for material_roughness)
/// Maps 0.0-1.0 to shininess range 1-256
/// This is an alias for material_roughness() for clarity in Mode 3
fn material_shininess(value: f32);
```

**How they map to `PackedUnifiedShadingState`:**

| FFI Function | Field | Mode 2 Meaning | Mode 3 Meaning |
|--------------|-------|----------------|----------------|
| `material_metallic(v)` | `metallic: u8` | Metallic | **Rim intensity** (uniform fallback for Slot 1.R) |
| `material_roughness(v)` | `roughness: u8` | Roughness | **Shininess** (uniform fallback for Slot 1.G) |
| `material_emissive(v)` | `emissive: u8` | Emissive | **Emissive** (same meaning! uniform fallback for Slot 1.B) |
| `material_shininess(v)` | `roughness: u8` | N/A | **Shininess** (alias for `material_roughness()`) |
| `material_rim(i, p)` | `metallic: u8` (intensity)<br>`matcap_blend_modes` byte 0 (power) | N/A | **Rim intensity + power** (Mode 3 only) |

**Specular color in Mode 3:**
- Comes ONLY from Slot 2 RGB texture
- No uniform fallback (defaults to white if Slot 2 not bound)
- No FFI function needed (texture-only)

**Recommendation:** Mode 3 code should use the new `material_rim()` and `material_shininess()` functions for clarity, though `material_metallic()` and `material_roughness()` will work (field reinterpretation).

---

## 6. Files to Modify

| Action | File | Changes |
|--------|------|---------|
| **Create** | `emberware-z/shaders/mode3_blinnphong.wgsl` | **NEW** shader template with Gotanda normalization, rim lighting, 3-slot texture layout |
| **Modify** | `emberware-z/src/shader_gen.rs` | Update TEMPLATE_MODE3 (line 53), add FS_MODE3_SLOT1/SLOT2 snippets, update mode_name function, update placeholder replacement |
| **Modify** | `emberware-z/src/graphics/unified_shading_state.rs` | Document mode-specific field interpretation (no struct changes) |
| **Modify** | `emberware-z/src/ffi/mod.rs` | Add `material_rim(intensity, power)` and `material_shininess(value)` functions |
| **Delete** | `emberware-z/shaders/mode3_hybrid.wgsl` | Remove old hybrid shader (matcap + PBR) |
| **Modify** | `docs/emberware-z.md` | Update Mode 3 description, texture slot table, add Blinn-Phong lighting details |
| **Modify** | `docs/ffi.md` | Document new `material_rim()` and `material_shininess()` functions |
| **Create** | `examples/blinn-phong/` | **NEW** example demonstrating Mode 3 features (specular, shininess, rim) |
| **Modify** | `CLAUDE.md` (optional) | Update shader architecture section if present |

**Key insight:** Mode 2 (`mode2_pbr.wgsl`) requires NO changes - emissive stays in Slot 1.B!

---

## 7. Mode 2 Compatibility

**IMPORTANT:** Mode 2 (PBR) requires **NO shader changes** for this implementation!

### Why Mode 2 is Unchanged

Mode 2 and Mode 3 share the same `PackedUnifiedShadingState` structure but interpret fields differently:

| Field | Mode 2 (PBR) | Mode 3 (Blinn-Phong) |
|-------|--------------|----------------------|
| `metallic` | Metallic | Rim intensity |
| `roughness` | Roughness | Shininess |
| `emissive` | Emissive | Emissive (same!) |

**Texture slot layout:**
- Mode 2: Slot 1 contains MRE (Metallic, Roughness, Emissive)
- Mode 3: Slot 1 contains RSE (Rim, Shininess, Emissive)
- **Both use Slot 1.B for emissive** - no migration needed!

### Verification Checklist

When implementing Mode 3, verify Mode 2 still works:
- [ ] `mode2_pbr.wgsl` shader unchanged
- [ ] Existing Mode 2 examples render identically
- [ ] MRE texture (Slot 1) sampling works correctly
- [ ] Emissive from Slot 1.B renders as before

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
    ├── albedo.png       (Slot 0: RGB diffuse color, A unused)
    ├── rse.png          (Slot 1: R=rim intensity, G=shininess, B=emissive)
    ├── specular.png     (Slot 2: RGB specular color, A unused)
    └── README.md        (Material authoring guide)
```

**Texture authoring notes:**
- **albedo.png** (Slot 0): Standard diffuse/base color RGB, alpha unused for meshes
- **rse.png** (Slot 1): Per-pixel control over rim (R), shininess (G), emissive (B)
- **specular.png** (Slot 2): Specular highlight color (e.g., gold=warm orange, silver=neutral white)

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

## 10. Migration Notes

### For Games Using Old Mode 3 (Hybrid PBR+Matcap)

The old Mode 3 used a hybrid approach with matcaps + PBR direct lighting. The new Mode 3 is pure Blinn-Phong.

**Texture slot changes:**

| Old (Hybrid) | New (Blinn-Phong) | Migration Action |
|--------------|-------------------|------------------|
| Slot 0: Albedo RGB | Slot 0: Albedo RGB | **No change** |
| Slot 1: MRE (Metallic-Roughness-Emissive) | Slot 1: RSE (Rim-Shininess-Emissive) | **Reauthor texture** |
| Slot 2: Environment matcap | Slot 2: Specular RGB | **Replace with specular color** |
| Slot 3: Ambient matcap | Slot 3: Unused | **Remove** |

### Texture Conversion Guide

**Slot 1 (MRE → RSE):**
- **R channel (Metallic → Rim intensity)**: Delete metallic data, paint rim intensity (edges bright, center dark)
- **G channel (Roughness → Shininess)**: Invert roughness to get shininess: `shininess = 1.0 - roughness`
- **B channel (Emissive)**: **No change** - emissive stays in same channel!

**Slot 2 (Matcap → Specular):**
- Delete environment matcap texture
- Create new specular color texture (e.g., warm orange for gold, neutral white for silver)
- Or leave unbound for default white specular

**Slot 3:**
- Delete ambient matcap texture (unused in new Mode 3)

### Uniform Changes

No changes needed - Mode 3 reinterprets existing fields:
- `material_metallic()` → now sets rim intensity
- `material_roughness()` → now sets shininess
- `material_emissive()` → still sets emissive (same meaning!)

**Recommendation:** Use new `material_rim()` and `material_shininess()` functions for clarity.

---

## 11. Testing Checklist

### Mode 3 (Blinn-Phong) Tests
- [ ] Gotanda normalization produces consistent brightness across shininess range (1-256)
- [ ] No visible energy blow-up at grazing angles (despite no G term)
- [ ] Rim lighting works independently of specular
- [ ] All 4 lights + sun contribute correctly
- [ ] **Emissive from Slot 1.B** functions correctly (both uniform and texture)
- [ ] **Specular from Slot 2 RGB** displays correctly (gold=warm, silver=neutral)
- [ ] **RSE texture (Slot 1)** samples correctly: R=rim, G=shininess, B=emissive
- [ ] Shininess variation (soft to sharp) behaves as expected
- [ ] Rim intensity and rim power controls work as expected
- [ ] Vertex color multiplies albedo correctly
- [ ] Formats without normals fall back to Mode 0 with warning
- [ ] Gold material looks warm/orange (not PBR-derived metallic)
- [ ] White fallback for Slot 2 produces neutral white specular
- [ ] All 8 Mode 3 shader permutations compile successfully (naga validation)

### Mode 2 (PBR) Regression Tests
- [ ] Mode 2 still works **without any changes** to `mode2_pbr.wgsl`
- [ ] **MRE texture (Slot 1)** still works correctly: R=metallic, G=roughness, B=emissive
- [ ] Emissive from Slot 1.B renders identically to before
- [ ] Existing Mode 2 games/examples render unchanged
- [ ] No regressions in metallic/roughness rendering

### Comparison Tests
- [ ] Mode 2 vs Mode 3 renders show intentional differences (PBR vs Blinn-Phong)
- [ ] Example game demonstrates all Mode 3 features
- [ ] Same material settings produce visually different results (as expected)

---

## 12. TODO: Implementation Tasks

### Phase 1: Shader Implementation (Required)
- [ ] **Create** `emberware-z/shaders/mode3_blinnphong.wgsl` shader template
  - [ ] Implement Gotanda normalization function
  - [ ] Implement normalized Blinn-Phong specular function
  - [ ] Implement rim lighting function
  - [ ] Add texture sampling for Slot 1 (RSE) and Slot 2 (Specular)
  - [ ] Add unpacking logic for mode-specific field interpretation
  - [ ] Test with 8 vertex format permutations (NORMAL flag required)
- [ ] **Update** `emberware-z/src/shader_gen.rs` (line 53):
  - [ ] Change TEMPLATE_MODE3 to reference `mode3_blinnphong.wgsl`
  - [ ] Add FS_MODE3_SLOT1 snippet (sample Slot 1 RGB → rim, shininess, emissive)
  - [ ] Add FS_MODE3_SLOT2 snippet (sample Slot 2 RGB → specular color)
  - [ ] Update mode_name function: "Hybrid" → "Blinn-Phong"
  - [ ] Update placeholder replacement for Mode 3 (use FS_MODE3_SLOT1/SLOT2)
- [ ] **Verify** Mode 2 compatibility: NO changes to `mode2_pbr.wgsl`
- [ ] **Delete** `emberware-z/shaders/mode3_hybrid.wgsl` (old shader)

### Phase 2: Material System (Required)
- [ ] **Update** `emberware-z/src/graphics/unified_shading_state.rs`
  - [ ] Add documentation comments explaining mode-specific field interpretation
  - [ ] No struct changes needed (reuse existing fields)
- [ ] **Add FFI functions** to `emberware-z/src/ffi/mod.rs`:
  - [ ] `material_rim(intensity: f32, power: f32)` - sets rim intensity (metallic field) + power (matcap_blend_modes byte 0)
  - [ ] `material_shininess(value: f32)` - alias for material_roughness(), for Mode 3 clarity
- [ ] **Update** `docs/ffi.md`:
  - [ ] Document new `material_rim()` function
  - [ ] Document `material_shininess()` alias
  - [ ] Explain mode-specific field interpretation

### Phase 3: Example Game (Required)
- [ ] **Create** `examples/blinn-phong/` directory structure
- [ ] **Implement** example game showcasing:
  - [ ] Multiple shininess values (10, 50, 128, 200)
  - [ ] Specular color variation (gold, silver, copper)
  - [ ] Rim lighting controls
  - [ ] 4 lights + sun lighting
  - [ ] Texture-driven materials (RSE + Specular textures)
- [ ] **Create** asset files:
  - [ ] Sphere/character meshes
  - [ ] albedo.png (Slot 0: RGB diffuse)
  - [ ] rse.png (Slot 1: R=rim, G=shininess, B=emissive)
  - [ ] specular.png (Slot 2: RGB specular color)
- [ ] **Write** material authoring guide (README in assets/)

### Phase 4: Documentation (Required)
- [ ] **Update** `docs/emberware-z.md`:
  - [ ] Mode 3 description: "Hybrid" → "Normalized Blinn-Phong"
  - [ ] Texture slot table: update Slot 1/2 descriptions
  - [ ] Add Blinn-Phong lighting details
  - [ ] Add Gotanda normalization reference
- [ ] **Add** migration guide for old Mode 3 users (Section 10)
- [ ] **Update** `CLAUDE.md` (if shader architecture is documented)

### Phase 5: Testing & Validation (Required)
- [ ] Run all shader compilation tests (40 permutations)
- [ ] Verify Mode 2 regression tests pass (NO changes to Mode 2!)
- [ ] Test all Mode 3 features (see Section 11 checklist)
- [ ] Verify no regressions in existing examples
- [ ] Test white fallback behavior for unbound Slot 2