# Emberware Z Material Overrides Specification

> **Status:** Ready for Implementation
> **Author:** Zerve
> **Last Updated:** December 2025
> **Related:** [transparency-implementation-spec.md](transparency-implementation-spec.md)

## Overview

Material overrides allow per-draw selection between **uniform values** and **texture sampling** for material properties. This enables efficient rendering of objects that share geometry but have different material properties, without requiring separate textures.

### Key Benefits

- **Memory Efficiency:** Render solid-colored objects without allocating 1x1 textures
- **Runtime Flexibility:** Dynamically change material properties without texture swaps
- **Batch-Friendly:** Objects with different uniform materials can share the same pipeline
- **Retro Authenticity:** 5th-gen consoles frequently used flat-shaded materials

### Override Semantics

Override flags are for **temporarily overriding textures when present**. When rendering primitives without textures, the shader naturally falls back to uniform values—the override flags are not needed for that case.

**Example use case:** A mesh has full PBR textures, but you want to flash it red when hit. Set `use_uniform_color(1)` and `set_color(0xFF0000FF)` for the damage frame, then `use_uniform_color(0)` to return to textured rendering.

---

## Texture Slot Layout

All render modes use the same slot indices:

| Slot | Purpose |
|------|---------|
| 0 | Albedo (base color) |
| 1 | Material map (Mode 2: M,R,E channels; Mode 3: S,D,E channels) |
| 2 | Specular color (Mode 3 only, unused in other modes) |
| 3 | Environment map |

---

## Existing Uniform Storage

The `PackedUnifiedShadingState` struct **already contains** uniform material values. This spec only adds **flag bits** to toggle between uniform vs texture sampling.

**uniform_set_0** (4 packed u8 values):
- Byte 0: metallic (Mode 2) / specular_damping (Mode 3)
- Byte 1: roughness (Mode 2) / shininess (Mode 3)
- Byte 2: emissive intensity (all modes)
- Byte 3: rim_intensity

**uniform_set_1** (4 packed u8 values):
- Byte 0: rim_power
- Bytes 1-3: specular RGB (Mode 3 only)

---

## Flag Layout

Material override flags occupy **bits 2-7** of `PackedUnifiedShadingState.flags`:

```
flags (32 bits):
├─ Bit 0:      skinning_mode              (existing: 0=raw, 1=inverse bind)
├─ Bit 1:      texture_filter             (existing: 0=nearest, 1=linear)
├─ Bit 2:      use_uniform_color          (NEW: 0=texture/vertex, 1=uniform)
├─ Bit 3:      use_uniform_metallic       (NEW: 0=texture, 1=uniform)
├─ Bit 4:      use_uniform_roughness      (NEW: 0=texture, 1=uniform)
├─ Bit 5:      use_uniform_emissive       (NEW: 0=texture, 1=uniform)
├─ Bit 6:      use_uniform_specular       (NEW: 0=texture, 1=uniform)
├─ Bit 7:      use_matcap_reflection      (NEW: 0=sky reflection, 1=matcap)
├─ Bits 8-11:  uniform_alpha              (dither, see transparency spec)
├─ Bits 12-13: dither_offset_x            (dither, see transparency spec)
├─ Bits 14-15: dither_offset_y            (dither, see transparency spec)
└─ Bits 16-31: reserved
```

### Flag Constants

```wgsl
// Material override flags (bits 2-7)
const FLAG_USE_UNIFORM_COLOR: u32 = 4u;       // Bit 2 (1 << 2)
const FLAG_USE_UNIFORM_METALLIC: u32 = 8u;    // Bit 3 (1 << 3)
const FLAG_USE_UNIFORM_ROUGHNESS: u32 = 16u;  // Bit 4 (1 << 4)
const FLAG_USE_UNIFORM_EMISSIVE: u32 = 32u;   // Bit 5 (1 << 5)
const FLAG_USE_UNIFORM_SPECULAR: u32 = 64u;   // Bit 6 (1 << 6)
const FLAG_USE_MATCAP_REFLECTION: u32 = 128u; // Bit 7 (1 << 7)
```

---

## Override Flags

### `use_uniform_color` (Bit 2)

Controls the source of base color/albedo.

| Value | Behavior |
|-------|----------|
| 0 | Sample from texture slot 0 (or use vertex color if no UV) |
| 1 | Use `color_rgba8` uniform value |

**Applies to:** All render modes (0-3)

**No-UV Primitives:** When rendering geometry without UV coordinates (e.g., immediate-mode triangles, debug primitives), the shader automatically falls back to the uniform color value if no texture is bound. The `use_uniform_color` flag is not required in this case—it's for overriding textures when they ARE present.

**Use Cases:**
- Solid-colored geometry (walls, floors, primitives)
- Color-tinted objects where texture isn't needed
- Debug visualization
- Temporary override (e.g., flash red when hit)

### `use_uniform_metallic` (Bit 3)

Controls the source of metallic value (Mode 2) or specular damping (Mode 3).

| Value | Mode 2 (MR) | Mode 3 (SS) |
|-------|-------------|-------------|
| 0 | Sample metallic from texture R channel | Sample spec_damping from texture |
| 1 | Use `material_metallic` uniform | Use `material_spec_damping` uniform |

**Applies to:** Mode 2 (Metallic-Roughness), Mode 3 (Specular-Shininess)

### `use_uniform_roughness` (Bit 4)

Controls the source of roughness (Mode 2) or shininess (Mode 3).

| Value | Mode 2 (MR) | Mode 3 (SS) |
|-------|-------------|-------------|
| 0 | Sample roughness from texture G channel | Sample shininess from texture |
| 1 | Use `material_roughness` uniform | Use `material_shininess` uniform |

**Applies to:** Mode 2 (Metallic-Roughness), Mode 3 (Specular-Shininess)

### `use_uniform_emissive` (Bit 5)

Controls the source of emissive intensity.

| Value | Behavior |
|-------|----------|
| 0 | Sample emissive intensity from material texture B channel |
| 1 | Use `material_emissive` uniform (f32 intensity, 0.0-1.0) |

**Applies to:** Mode 2 (MR), Mode 3 (SS)

**Note:** Emissive is a single intensity value multiplied with the albedo color, NOT an RGB color. This keeps the emissive glow consistent with the object's base color.

**Use Cases:**
- Glowing objects (intensity multiplied with albedo)
- Animated emissive effects (pulsing, flickering)
- Screen/monitor surfaces

### `use_uniform_specular` (Bit 6)

Controls the source of specular color (Mode 3 only).

| Value | Behavior |
|-------|----------|
| 0 | Sample from specular texture |
| 1 | Use `material_specular` uniform (RGB color) |

**Applies to:** Mode 3 (Specular-Shininess) only

### `use_matcap_reflection` (Bit 7)

Controls the reflection source for Mode 1 (Matcap).

| Value | Behavior |
|-------|----------|
| 0 | Use procedural sky for environment reflection |
| 1 | Use matcap texture for stylized reflection |

**Applies to:** Mode 1 (Matcap) only

**Use Cases:**
- Stylized metal/chrome effects
- Toon shading with custom reflection spheres
- Artistic control over reflections

---

## Interaction with Render Modes

**Important:** Modes 0 and 1 do not use PBR/classic material properties. The metallic, roughness, emissive, and specular override flags have **no effect** in these modes—only `use_uniform_color` (and `use_matcap_reflection` for Mode 1) apply.

### Mode 0: Unlit / Simple Lambert

| Flag | Effect |
|------|--------|
| `use_uniform_color` | Uniform color vs texture/vertex color |
| `use_uniform_metallic` | **No effect** |
| `use_uniform_roughness` | **No effect** |
| `use_uniform_emissive` | **No effect** |
| `use_uniform_specular` | **No effect** |
| `use_matcap_reflection` | **No effect** |

### Mode 1: Matcap

| Flag | Effect |
|------|--------|
| `use_uniform_color` | Base color tint source |
| `use_matcap_reflection` | Sky reflection vs matcap texture |
| `use_uniform_metallic` | **No effect** |
| `use_uniform_roughness` | **No effect** |
| `use_uniform_emissive` | **No effect** |
| `use_uniform_specular` | **No effect** |

### Mode 2: Metallic-Roughness (PBR)

| Flag | Effect |
|------|--------|
| `use_uniform_color` | Albedo source |
| `use_uniform_metallic` | Metallic value source |
| `use_uniform_roughness` | Roughness value source |
| `use_uniform_emissive` | Emissive color source |
| `use_uniform_specular` | No effect (mode uses metallic workflow) |
| `use_matcap_reflection` | No effect |

### Mode 3: Specular-Shininess (Classic)

| Flag | Effect |
|------|--------|
| `use_uniform_color` | Diffuse color source |
| `use_uniform_metallic` | Specular damping source |
| `use_uniform_roughness` | Shininess source |
| `use_uniform_emissive` | Emissive color source |
| `use_uniform_specular` | Specular color source |
| `use_matcap_reflection` | No effect |

---

## FFI Functions

### Setting Override Flags

```c
// Color override (0 = texture/vertex, 1 = uniform)
void use_uniform_color(u32 enabled);

// Metallic override (Mode 2) / Spec damping override (Mode 3)
void use_uniform_metallic(u32 enabled);
void use_uniform_specular_damping(u32 enabled);  // Alias for use_uniform_metallic

// Roughness override (Mode 2) / Shininess override (Mode 3)
void use_uniform_roughness(u32 enabled);
void use_uniform_shininess(u32 enabled);  // Alias for use_uniform_roughness

// Emissive override (Modes 2, 3)
void use_uniform_emissive(u32 enabled);

// Specular color override (Mode 3 only)
void use_uniform_specular(u32 enabled);

// Matcap vs sky reflection (Mode 1 only)
void use_matcap_reflection(u32 enabled);
```

### Setting Uniform Values

These functions set the uniform values used when the corresponding override flag is enabled:

```c
// Base color (used when use_uniform_color = 1)
void set_color(u32 rgba8);  // Existing function, 0xRRGGBBAA format

// PBR material properties (Mode 2)
void material_metallic(f32 metallic);      // 0.0 - 1.0
void material_roughness(f32 roughness);    // 0.0 - 1.0
void material_emissive(f32 intensity);     // 0.0 - 1.0 (multiplied with albedo)

// Classic material properties (Mode 3) - aliases for same underlying storage
void material_specular_damping(f32 damping);  // 0.0 - 1.0 (alias for metallic slot)
void material_shininess(f32 shininess);       // 0.0 - 1.0 (alias for roughness slot)
void material_specular_color(f32 r, f32 g, f32 b);  // Specular RGB (Mode 3 only)
```

**Note on Aliases:** `material_metallic` and `material_specular_damping` write to the same storage location (uniform_set_0 byte 0). Similarly, `material_roughness` and `material_shininess` share byte 1. Use whichever name matches your render mode for clarity.

---

## Shader Implementation

### WGSL Helper Functions

```wgsl
// Check if a material override flag is set
fn has_flag(flags: u32, flag: u32) -> bool {
    return (flags & flag) != 0u;
}

// Get base color from uniform or texture
fn get_base_color(shading: PackedUnifiedShadingState, uv: vec2<f32>) -> vec4<f32> {
    if has_flag(shading.flags, FLAG_USE_UNIFORM_COLOR) {
        return unpack_rgba8(shading.color_rgba8);
    } else {
        return sample_filtered(slot0, shading.flags, uv);
    }
}

// Get metallic from uniform or texture (Mode 2)
fn get_metallic(shading: PackedUnifiedShadingState, uv: vec2<f32>) -> f32 {
    if has_flag(shading.flags, FLAG_USE_UNIFORM_METALLIC) {
        return shading.material_metallic;
    } else {
        return sample_filtered(slot_mr, shading.flags, uv).r;
    }
}

// Get roughness from uniform or texture (Mode 2)
fn get_roughness(shading: PackedUnifiedShadingState, uv: vec2<f32>) -> f32 {
    if has_flag(shading.flags, FLAG_USE_UNIFORM_ROUGHNESS) {
        return shading.material_roughness;
    } else {
        return sample_filtered(slot_mr, shading.flags, uv).g;
    }
}

// Get emissive from uniform or texture
fn get_emissive(shading: PackedUnifiedShadingState, uv: vec2<f32>) -> vec3<f32> {
    if has_flag(shading.flags, FLAG_USE_UNIFORM_EMISSIVE) {
        return unpack_rgb8(shading.material_emissive);
    } else {
        return sample_filtered(slot_emissive, shading.flags, uv).rgb;
    }
}

// Get specular color from uniform or texture (Mode 3)
fn get_specular_color(shading: PackedUnifiedShadingState, uv: vec2<f32>) -> vec3<f32> {
    if has_flag(shading.flags, FLAG_USE_UNIFORM_SPECULAR) {
        return unpack_rgb8(shading.material_specular);
    } else {
        return sample_filtered(slot_specular, shading.flags, uv).rgb;
    }
}
```

### Example: Mode 2 Fragment Shader

```wgsl
@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
    let shading = shading_states[in.shading_state_index];

    // Material properties - uniform or texture based on flags
    let albedo = get_base_color(shading, in.uv);
    let metallic = get_metallic(shading, in.uv);
    let roughness = get_roughness(shading, in.uv);
    let emissive = get_emissive(shading, in.uv);

    // PBR calculations...
    var color = calculate_pbr(albedo.rgb, metallic, roughness, in.normal, in.world_pos);
    color += emissive;

    // Dither transparency (always active)
    if should_discard_dither(in.clip_position.xy, shading.flags) {
        discard;
    }

    return vec4<f32>(color, albedo.a);
}
```

---

## Rust Implementation

### PackedUnifiedShadingState Updates

```rust
// In unified_shading_state.rs

impl PackedUnifiedShadingState {
    // Flag bit constants
    const FLAG_SKINNING_MODE: u32 = 1 << 0;
    const FLAG_TEXTURE_FILTER: u32 = 1 << 1;
    const FLAG_USE_UNIFORM_COLOR: u32 = 1 << 2;
    const FLAG_USE_UNIFORM_METALLIC: u32 = 1 << 3;
    const FLAG_USE_UNIFORM_ROUGHNESS: u32 = 1 << 4;
    const FLAG_USE_UNIFORM_EMISSIVE: u32 = 1 << 5;
    const FLAG_USE_UNIFORM_SPECULAR: u32 = 1 << 6;
    const FLAG_USE_MATCAP_REFLECTION: u32 = 1 << 7;

    /// Set material override flag
    pub fn set_override_flag(&mut self, flag: u32, enabled: bool) {
        if enabled {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
    }

    /// Check if a material override flag is set
    pub fn has_override_flag(&self, flag: u32) -> bool {
        (self.flags & flag) != 0
    }
}
```

### ZFFIState Updates

```rust
// In ffi_state.rs

impl ZFFIState {
    /// Set use_uniform_color flag
    pub fn set_use_uniform_color(&mut self, enabled: bool) {
        self.current_shading_state.set_override_flag(
            PackedUnifiedShadingState::FLAG_USE_UNIFORM_COLOR,
            enabled
        );
    }

    /// Set use_uniform_metallic flag
    pub fn set_use_uniform_metallic(&mut self, enabled: bool) {
        self.current_shading_state.set_override_flag(
            PackedUnifiedShadingState::FLAG_USE_UNIFORM_METALLIC,
            enabled
        );
    }

    // ... similar for other override flags
}
```

### FFI Registration

```rust
// In ffi/material.rs (new file)

pub fn register(linker: &mut Linker<GameStateWithConsole<ZInput, ZFFIState>>) -> Result<()> {
    linker.func_wrap("env", "use_uniform_color", use_uniform_color)?;
    linker.func_wrap("env", "use_uniform_metallic", use_uniform_metallic)?;
    linker.func_wrap("env", "use_uniform_roughness", use_uniform_roughness)?;
    linker.func_wrap("env", "use_uniform_emissive", use_uniform_emissive)?;
    linker.func_wrap("env", "use_uniform_specular", use_uniform_specular)?;
    linker.func_wrap("env", "use_matcap_reflection", use_matcap_reflection)?;
    linker.func_wrap("env", "material_metallic", material_metallic)?;
    linker.func_wrap("env", "material_roughness", material_roughness)?;
    linker.func_wrap("env", "material_emissive", material_emissive)?;
    linker.func_wrap("env", "material_specular", material_specular)?;
    linker.func_wrap("env", "material_shininess", material_shininess)?;
    linker.func_wrap("env", "material_spec_damping", material_spec_damping)?;
    Ok(())
}

fn use_uniform_color(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, enabled: u32) {
    caller.data_mut().console.set_use_uniform_color(enabled != 0);
}

fn material_metallic(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, value: f32) {
    let clamped = value.clamp(0.0, 1.0);
    caller.data_mut().console.current_shading_state.material_metallic = clamped;
}

// ... similar for other functions
```

---

## Default Values

| Property | Default | Meaning |
|----------|---------|---------|
| `use_uniform_color` | 0 | Use texture/vertex color |
| `use_uniform_metallic` | 0 | Use texture |
| `use_uniform_roughness` | 0 | Use texture |
| `use_uniform_emissive` | 0 | Use texture |
| `use_uniform_specular` | 0 | Use texture |
| `use_matcap_reflection` | 0 | Use sky reflection |
| `material_metallic` | 0.0 | Non-metallic |
| `material_roughness` | 0.5 | Medium roughness |
| `material_emissive` | 0x000000 | No emission |
| `material_specular` | 0xFFFFFF | White specular |
| `material_shininess` | 32.0 | Medium shininess |

**Backwards Compatibility:** All override flags default to 0, so existing code continues to sample from textures.

---

## Unit Test Specifications

### Flag Packing Tests

```rust
#[test]
fn test_override_flag_bit_positions() {
    // Verify each flag occupies correct bit position
    assert_eq!(PackedUnifiedShadingState::FLAG_USE_UNIFORM_COLOR, 1 << 2);
    assert_eq!(PackedUnifiedShadingState::FLAG_USE_UNIFORM_METALLIC, 1 << 3);
    assert_eq!(PackedUnifiedShadingState::FLAG_USE_UNIFORM_ROUGHNESS, 1 << 4);
    assert_eq!(PackedUnifiedShadingState::FLAG_USE_UNIFORM_EMISSIVE, 1 << 5);
    assert_eq!(PackedUnifiedShadingState::FLAG_USE_UNIFORM_SPECULAR, 1 << 6);
    assert_eq!(PackedUnifiedShadingState::FLAG_USE_MATCAP_REFLECTION, 1 << 7);
}

#[test]
fn test_override_flags_independence() {
    // Setting one flag shouldn't affect others
    let mut state = PackedUnifiedShadingState::default();

    state.set_override_flag(PackedUnifiedShadingState::FLAG_USE_UNIFORM_COLOR, true);
    assert!(state.has_override_flag(PackedUnifiedShadingState::FLAG_USE_UNIFORM_COLOR));
    assert!(!state.has_override_flag(PackedUnifiedShadingState::FLAG_USE_UNIFORM_METALLIC));

    state.set_override_flag(PackedUnifiedShadingState::FLAG_USE_UNIFORM_METALLIC, true);
    assert!(state.has_override_flag(PackedUnifiedShadingState::FLAG_USE_UNIFORM_COLOR));
    assert!(state.has_override_flag(PackedUnifiedShadingState::FLAG_USE_UNIFORM_METALLIC));
}

#[test]
fn test_override_flags_do_not_affect_existing_flags() {
    // Material overrides shouldn't touch skinning_mode or texture_filter
    let mut state = PackedUnifiedShadingState::default();
    state.flags |= 0b11; // Set both existing flags

    state.set_override_flag(PackedUnifiedShadingState::FLAG_USE_UNIFORM_COLOR, true);

    assert_eq!(state.flags & 0b11, 0b11); // Existing flags preserved
}

#[test]
fn test_override_flag_toggle() {
    let mut state = PackedUnifiedShadingState::default();

    // Enable
    state.set_override_flag(PackedUnifiedShadingState::FLAG_USE_UNIFORM_EMISSIVE, true);
    assert!(state.has_override_flag(PackedUnifiedShadingState::FLAG_USE_UNIFORM_EMISSIVE));

    // Disable
    state.set_override_flag(PackedUnifiedShadingState::FLAG_USE_UNIFORM_EMISSIVE, false);
    assert!(!state.has_override_flag(PackedUnifiedShadingState::FLAG_USE_UNIFORM_EMISSIVE));
}
```

### Default Value Tests

```rust
#[test]
fn test_default_override_flags_disabled() {
    let state = PackedUnifiedShadingState::default();

    // All override flags should be 0 (use texture)
    assert!(!state.has_override_flag(PackedUnifiedShadingState::FLAG_USE_UNIFORM_COLOR));
    assert!(!state.has_override_flag(PackedUnifiedShadingState::FLAG_USE_UNIFORM_METALLIC));
    assert!(!state.has_override_flag(PackedUnifiedShadingState::FLAG_USE_UNIFORM_ROUGHNESS));
    assert!(!state.has_override_flag(PackedUnifiedShadingState::FLAG_USE_UNIFORM_EMISSIVE));
    assert!(!state.has_override_flag(PackedUnifiedShadingState::FLAG_USE_UNIFORM_SPECULAR));
    assert!(!state.has_override_flag(PackedUnifiedShadingState::FLAG_USE_MATCAP_REFLECTION));
}

#[test]
fn test_default_material_values() {
    let state = PackedUnifiedShadingState::default();

    assert_eq!(state.material_metallic, 0.0);
    assert_eq!(state.material_roughness, 0.5);
    assert_eq!(state.material_emissive, 0x000000);
    assert_eq!(state.material_specular, 0xFFFFFF);
    assert_eq!(state.material_shininess, 32.0);
}
```

### Material Value Clamping Tests

```rust
#[test]
fn test_metallic_clamping() {
    let mut state = ZFFIState::default();

    // Test lower bound
    material_metallic_impl(&mut state, -0.5);
    assert_eq!(state.current_shading_state.material_metallic, 0.0);

    // Test upper bound
    material_metallic_impl(&mut state, 1.5);
    assert_eq!(state.current_shading_state.material_metallic, 1.0);

    // Test valid range
    material_metallic_impl(&mut state, 0.7);
    assert_eq!(state.current_shading_state.material_metallic, 0.7);
}

#[test]
fn test_roughness_clamping() {
    let mut state = ZFFIState::default();

    material_roughness_impl(&mut state, -1.0);
    assert_eq!(state.current_shading_state.material_roughness, 0.0);

    material_roughness_impl(&mut state, 2.0);
    assert_eq!(state.current_shading_state.material_roughness, 1.0);
}

#[test]
fn test_shininess_clamping() {
    let mut state = ZFFIState::default();

    // Shininess range: 1.0 - 256.0
    material_shininess_impl(&mut state, 0.0);
    assert_eq!(state.current_shading_state.material_shininess, 1.0);

    material_shininess_impl(&mut state, 500.0);
    assert_eq!(state.current_shading_state.material_shininess, 256.0);
}
```

### Integration Tests

```rust
#[test]
fn test_override_with_dither_flags_coexist() {
    // Material overrides (bits 2-7) and dither (bits 8-15) shouldn't conflict
    let mut state = PackedUnifiedShadingState::default();

    // Set material overrides
    state.set_override_flag(PackedUnifiedShadingState::FLAG_USE_UNIFORM_COLOR, true);
    state.set_override_flag(PackedUnifiedShadingState::FLAG_USE_UNIFORM_METALLIC, true);

    // Set dither values
    state.set_uniform_alpha(8);  // 50% alpha
    state.set_dither_offset(2, 1);

    // Verify both are preserved
    assert!(state.has_override_flag(PackedUnifiedShadingState::FLAG_USE_UNIFORM_COLOR));
    assert!(state.has_override_flag(PackedUnifiedShadingState::FLAG_USE_UNIFORM_METALLIC));
    assert_eq!(state.get_uniform_alpha(), 8);
    assert_eq!(state.get_dither_offset(), (2, 1));
}
```

---

## Example Games

### 1. Material Showcase Demo

**Purpose:** Demonstrate uniform vs texture material sourcing across all render modes

**Scene Layout:**
```
┌─────────────────────────────────────────────────┐
│                                                 │
│  Mode 0 (Unlit)    Mode 1 (Matcap)             │
│  ┌───┐  ┌───┐      ┌───┐  ┌───┐               │
│  │TEX│  │UNI│      │SKY│  │MAT│               │
│  └───┘  └───┘      └───┘  └───┘               │
│                                                 │
│  Mode 2 (PBR)      Mode 3 (Specular)          │
│  ┌───┐  ┌───┐      ┌───┐  ┌───┐               │
│  │TEX│  │UNI│      │TEX│  │UNI│               │
│  └───┘  └───┘      └───┘  └───┘               │
│                                                 │
└─────────────────────────────────────────────────┘
```

**Implementation:**
```c
void render() {
    // Mode 0: Unlit comparison
    render_mode(0);

    // Left: Texture color
    use_uniform_color(0);
    bind_texture(0, color_texture);
    draw_mesh(cube_mesh);

    // Right: Uniform color (no texture needed)
    use_uniform_color(1);
    set_color(0xFF6600FF);  // Orange
    draw_mesh(cube_mesh);

    // Mode 2: PBR comparison
    render_mode(2);

    // Left: Full texture workflow
    use_uniform_color(0);
    use_uniform_metallic(0);
    use_uniform_roughness(0);
    bind_texture(0, albedo_texture);
    bind_texture(1, mr_texture);
    draw_mesh(sphere_mesh);

    // Right: Uniform material (gold metal)
    use_uniform_color(1);
    use_uniform_metallic(1);
    use_uniform_roughness(1);
    set_color(0xFFD700FF);     // Gold color
    material_metallic(1.0);     // Full metal
    material_roughness(0.3);    // Slightly rough
    draw_mesh(sphere_mesh);
}
```

### 2. Procedural World Generator

**Purpose:** Demonstrate efficient rendering of procedural content using uniform materials

**Concept:** Generate a voxel-style world where each block type uses uniform color/material instead of textures

**Block Types:**
| Block | Color | Metallic | Roughness |
|-------|-------|----------|-----------|
| Grass | 0x4CAF50 | 0.0 | 0.9 |
| Dirt | 0x8B4513 | 0.0 | 0.95 |
| Stone | 0x808080 | 0.0 | 0.8 |
| Gold Ore | 0xFFD700 | 0.8 | 0.4 |
| Water | 0x2196F3 | 0.0 | 0.1 |

**Implementation:**
```c
typedef struct {
    u32 color;
    f32 metallic;
    f32 roughness;
} BlockMaterial;

const BlockMaterial BLOCKS[] = {
    { 0x4CAF50FF, 0.0f, 0.9f },   // Grass
    { 0x8B4513FF, 0.0f, 0.95f },  // Dirt
    { 0x808080FF, 0.0f, 0.8f },   // Stone
    { 0xFFD700FF, 0.8f, 0.4f },   // Gold
    { 0x2196F3FF, 0.0f, 0.1f },   // Water
};

void render_block(i32 x, i32 y, i32 z, u8 block_type) {
    BlockMaterial mat = BLOCKS[block_type];

    use_uniform_color(1);
    use_uniform_metallic(1);
    use_uniform_roughness(1);

    set_color(mat.color);
    material_metallic(mat.metallic);
    material_roughness(mat.roughness);

    transform_set(/* block transform at x,y,z */);
    draw_mesh(cube_mesh);
}
```

**Memory Savings:** Zero texture memory for blocks - entire material defined by 12 bytes (color + metallic + roughness)

### 3. Character Customization System

**Purpose:** Show runtime material changes for character customization

**Features:**
- Same character mesh with different skin tones (uniform color)
- Armor with different metal types (uniform metallic/roughness)
- Glowing effects (uniform emissive)

**Implementation:**
```c
typedef struct {
    u32 skin_color;
    u32 armor_color;
    f32 armor_metallic;
    f32 armor_roughness;
    u32 eye_glow_color;
} CharacterCustomization;

void render_character(CharacterCustomization* custom) {
    // Render skin (uniform color, no metallic)
    use_uniform_color(1);
    use_uniform_metallic(1);
    use_uniform_roughness(1);
    set_color(custom->skin_color);
    material_metallic(0.0);
    material_roughness(0.7);
    draw_mesh(skin_mesh);

    // Render armor (uniform material properties)
    set_color(custom->armor_color);
    material_metallic(custom->armor_metallic);
    material_roughness(custom->armor_roughness);
    draw_mesh(armor_mesh);

    // Render glowing eyes (uniform emissive)
    use_uniform_emissive(1);
    material_emissive(custom->eye_glow_color);
    draw_mesh(eyes_mesh);
}

void update() {
    // Animate eye glow
    f32 pulse = (sin(time * 3.0) + 1.0) * 0.5;
    u8 intensity = (u8)(pulse * 255);
    player.eye_glow_color = (intensity << 16) | (intensity << 8) | 0xFF;
}
```

### 4. Retro Racing Game

**Purpose:** 5th-gen console style with flat-shaded cars and environments

**Style:** Saturn/PS1 racing aesthetic with solid-colored polygons

**Implementation:**
```c
// Cars use uniform colors for retro flat-shaded look
void render_car(Car* car) {
    use_uniform_color(1);
    use_uniform_metallic(1);
    use_uniform_roughness(1);

    // Car body
    set_color(car->body_color);
    material_metallic(0.6);
    material_roughness(0.4);
    draw_mesh(car_body_mesh);

    // Windows (darker, more reflective)
    set_color(0x1A1A1AFF);
    material_metallic(0.0);
    material_roughness(0.1);
    draw_mesh(car_windows_mesh);

    // Chrome trim
    set_color(0xC0C0C0FF);
    material_metallic(1.0);
    material_roughness(0.2);
    draw_mesh(car_trim_mesh);
}

// Track uses uniform colors for retro look
void render_track() {
    use_uniform_color(1);
    use_uniform_metallic(1);
    use_uniform_roughness(1);

    // Asphalt
    set_color(0x333333FF);
    material_metallic(0.0);
    material_roughness(0.9);
    draw_mesh(track_mesh);

    // Grass
    set_color(0x228B22FF);
    material_metallic(0.0);
    material_roughness(0.95);
    draw_mesh(grass_mesh);

    // Rumble strips (alternating)
    for (int i = 0; i < NUM_STRIPS; i++) {
        set_color((i % 2) ? 0xFF0000FF : 0xFFFFFFFF);
        draw_mesh(strip_meshes[i]);
    }
}
```

---

## Performance Considerations

### Uniform Branching

Material override checks use **uniform branching**, which is SIMD-efficient:

```wgsl
// This is fast - all pixels in the draw call take the same branch
if has_flag(shading.flags, FLAG_USE_UNIFORM_COLOR) {
    color = uniform_color;  // All pixels go here
} else {
    color = texture_sample; // Or all pixels go here
}
```

**Why it's fast:**
- Flag value is **uniform** (same for all fragments in a draw call)
- GPU executes same branch for all pixels (no SIMD divergence)
- Branch prediction is perfect
- Essentially zero overhead compared to always sampling texture

### Texture Binding Optimization

When using uniform materials, texture binds can be skipped:

```c
// With uniform color, no need to bind albedo texture
use_uniform_color(1);
set_color(0xFF0000FF);
// skip: bind_texture(0, albedo);  // Not needed!
draw_mesh(red_cube);
```

The shader will read from the uniform value, so the texture binding (or lack thereof) doesn't matter.

---

## Files to Create/Modify

| File | Changes |
|------|---------|
| `emberware-z/src/graphics/unified_shading_state.rs` | Add `FLAG_USE_UNIFORM_*` constants (bits 2-7) |
| `emberware-z/src/state/ffi_state.rs` | Add `set_override_flag(&mut self, flag: u32, enabled: bool)` method |
| `emberware-z/src/ffi/material.rs` | **NEW:** FFI registration for all material functions (flag setters + value setters) |
| `emberware-z/src/ffi/mod.rs` | Register material module |
| `emberware-z/shaders/common.wgsl` | Add `FLAG_USE_UNIFORM_*` constants |
| `emberware-z/shaders/blinnphong_common.wgsl` | Add uniform vs texture branching for M/R/E and S/D/E + specular (shared by Mode 2 + 3) |
| `emberware-z/shaders/mode0_unlit.wgsl` | Add uniform color branching |
| `emberware-z/shaders/mode1_matcap.wgsl` | Add uniform color branching + matcap reflection flag |

---

## Future Work

- **Material presets:** Pre-defined material combinations (gold, silver, plastic, etc.)
- **Material interpolation:** Smooth transitions between material states
- **Per-vertex material flags:** Different parts of a mesh use different sources
- **Material palette:** Index into a table of predefined materials

---

## Related Documents

- [transparency-implementation-spec.md](transparency-implementation-spec.md) — Dither transparency system (flags bits 8-15)
- [ffi.md](../reference/ffi.md) — FFI API reference
- [emberware-z.md](../reference/emberware-z.md) — Z-specific API reference
