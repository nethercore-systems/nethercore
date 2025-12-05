# Mode 3: Normalized Blinn-Phong Implementation Plan

Replace Mode 3 (Hybrid: Matcap + PBR) with Mode 3 (Classic Lit: Normalized Blinn-Phong).

**Reference:** Gotanda 2010 - "Practical Implementation at tri-Ace"

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

### Fragment Shader

```wgsl
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let N = normalize(in.world_normal);
    let V = normalize(camera.position - in.world_position);
    
    // ===== TEXTURE SAMPLING =====
    
    // Slot 0: Albedo RGB + Emissive A
    let albedo_e = textureSample(slot0_texture, slot0_sampler, in.uv);
    var albedo = albedo_e.rgb;
    let emissive_intensity = albedo_e.a;
    
    // Apply vertex color to albedo (same as all other modes)
    //FS_VERTEX_COLOR: albedo *= in.vertex_color.rgb;
    
    // Slot 1: Specular RGB + Shininess A
    let spec_shin = textureSample(slot1_texture, slot1_sampler, in.uv);
    let specular_color = spec_shin.rgb;
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
    let rim = rim_lighting(
        N, V,
        sky.sun_color,
        bp_material.rim_intensity,
        bp_material.rim_power
    );
    
    // Emissive: Albedo × intensity (self-illumination)
    let emissive = albedo * emissive_intensity;
    
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

## 5. Files to Modify

| Action | File | Changes |
|--------|------|---------|
| **Create** | `emberware-z/src/graphics/shaders/mode3_blinnphong.wgsl` | New shader |
| **Modify** | `emberware-z/src/graphics/shaders/mode2_pbr.wgsl` | Move emissive to Slot 0.A |
| **Modify** | `emberware-z/src/graphics/pipeline.rs` | Shader selection for Mode 3 |
| **Modify** | `emberware-z/src/graphics/state.rs` | Add BlinnPhong material state |
| **Delete** | `emberware-z/src/graphics/shaders/mode3_hybrid.wgsl` | Remove old shader |
| **Modify** | `docs/emberware-z.md` | Update Mode 3 documentation |
| **Modify** | `CLAUDE.md` | Update shader architecture docs |

---

## 6. Example Materials

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

## 8. Testing Checklist

- [ ] Gotanda normalization produces consistent brightness across shininess range
- [ ] No visible energy blow-up at grazing angles (despite no G term)
- [ ] Rim lighting works independently of specular
- [ ] All 4 lights + sun contribute correctly
- [ ] Emissive from Slot 0.A functions correctly
- [ ] Mode 2 still works after emissive relocation
- [ ] Vertex color multiplies albedo correctly
- [ ] Formats without normals fall back to Mode 0 with warning
- [ ] Gold material looks warm/orange (not PBR-derived)
- [ ] Comparison renders show Mode 2 vs Mode 3 are intentionally different

---

## 9. TODO: Future

- [ ] FFI functions for material uniforms (specular_intensity, rim_intensity, rim_power)
- [ ] Rust uniform struct (BlinnPhongMaterialUniform)
- [ ] Default values and ranges for uniforms
- [ ] FFI documentation in docs/ffi.md