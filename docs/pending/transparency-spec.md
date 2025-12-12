# Emberware Z Transparency Specification

**Status:** Proposal / Draft  
**Author:** Zerve  
**Last Updated:** December 2025

---

## Overview

Emberware Z uses **stipple transparency** (screen-door / mesh transparency) instead of alpha blending. This is order-independent, requires no sorting, and produces an era-authentic Saturn/PS1 look.

**Key principles:**
- No alpha blending (ever)
- No depth sorting required
- Screen-space ordered dithering
- Configurable pattern size (2×2, 4×4, 8×8, 16×16)
- Alpha from texture OR uniform (packedShadingState)

---

## Why No Alpha Blending?

| Approach | Sorting | Overdraw | Artifacts | Complexity |
|----------|---------|----------|-----------|------------|
| Alpha blending | Required (back-to-front) | High | Order-dependent | Complex |
| **Stipple** | **None** | **Low** | **Shimmer (intentional)** | **Simple** |

Alpha blending requires sorting all transparent geometry every frame. This is:
- CPU intensive
- Error-prone (intersecting geometry)
- Not how 5th-gen consoles worked

Stipple transparency writes to depth buffer normally. No sorting. No overdraw. Just discard pixels below threshold.

---

## Ordered Dithering Patterns

Bayer matrices provide structured dithering with configurable level counts:

| Pattern | Dimensions | Alpha Levels | Look |
|---------|------------|--------------|------|
| 2×2 | 4 pixels | 4 | Extremely chunky, ultra-retro |
| **4×4** | 16 pixels | 16 | Classic Saturn stipple |
| 8×8 | 64 pixels | 64 | Smoother gradients |
| 16×16 | 256 pixels | 256 | Full alpha precision |

**Default: 4×4** (era-authentic, aligns with BC7 blocks)

---

## Bayer Matrices

### 2×2 (4 levels)

```wgsl
const BAYER_2X2: array<f32, 4> = array(
    0.0/4.0, 2.0/4.0,
    3.0/4.0, 1.0/4.0,
);
```

### 4×4 (16 levels) — Recommended Default

```wgsl
const BAYER_4X4: array<f32, 16> = array(
     0.0/16.0,  8.0/16.0,  2.0/16.0, 10.0/16.0,
    12.0/16.0,  4.0/16.0, 14.0/16.0,  6.0/16.0,
     3.0/16.0, 11.0/16.0,  1.0/16.0,  9.0/16.0,
    15.0/16.0,  7.0/16.0, 13.0/16.0,  5.0/16.0,
);
```

### 8×8 (64 levels)

```wgsl
const BAYER_8X8: array<f32, 64> = array(
     0.0/64.0, 32.0/64.0,  8.0/64.0, 40.0/64.0,  2.0/64.0, 34.0/64.0, 10.0/64.0, 42.0/64.0,
    48.0/64.0, 16.0/64.0, 56.0/64.0, 24.0/64.0, 50.0/64.0, 18.0/64.0, 58.0/64.0, 26.0/64.0,
    12.0/64.0, 44.0/64.0,  4.0/64.0, 36.0/64.0, 14.0/64.0, 46.0/64.0,  6.0/64.0, 38.0/64.0,
    60.0/64.0, 28.0/64.0, 52.0/64.0, 20.0/64.0, 62.0/64.0, 30.0/64.0, 54.0/64.0, 22.0/64.0,
     3.0/64.0, 35.0/64.0, 11.0/64.0, 43.0/64.0,  1.0/64.0, 33.0/64.0,  9.0/64.0, 41.0/64.0,
    51.0/64.0, 19.0/64.0, 59.0/64.0, 27.0/64.0, 49.0/64.0, 17.0/64.0, 57.0/64.0, 25.0/64.0,
    15.0/64.0, 47.0/64.0,  7.0/64.0, 39.0/64.0, 13.0/64.0, 45.0/64.0,  5.0/64.0, 37.0/64.0,
    63.0/64.0, 31.0/64.0, 55.0/64.0, 23.0/64.0, 61.0/64.0, 29.0/64.0, 53.0/64.0, 21.0/64.0,
);
```

### 16×16 (256 levels)

```wgsl
// Generated recursively from smaller Bayer matrices
const BAYER_16X16: array<f32, 256> = array(
      0.0/256.0, 128.0/256.0,  32.0/256.0, 160.0/256.0,   8.0/256.0, 136.0/256.0,  40.0/256.0, 168.0/256.0,   2.0/256.0, 130.0/256.0,  34.0/256.0, 162.0/256.0,  10.0/256.0, 138.0/256.0,  42.0/256.0, 170.0/256.0,
    192.0/256.0,  64.0/256.0, 224.0/256.0,  96.0/256.0, 200.0/256.0,  72.0/256.0, 232.0/256.0, 104.0/256.0, 194.0/256.0,  66.0/256.0, 226.0/256.0,  98.0/256.0, 202.0/256.0,  74.0/256.0, 234.0/256.0, 106.0/256.0,
     48.0/256.0, 176.0/256.0,  16.0/256.0, 144.0/256.0,  56.0/256.0, 184.0/256.0,  24.0/256.0, 152.0/256.0,  50.0/256.0, 178.0/256.0,  18.0/256.0, 146.0/256.0,  58.0/256.0, 186.0/256.0,  26.0/256.0, 154.0/256.0,
    240.0/256.0, 112.0/256.0, 208.0/256.0,  80.0/256.0, 248.0/256.0, 120.0/256.0, 216.0/256.0,  88.0/256.0, 242.0/256.0, 114.0/256.0, 210.0/256.0,  82.0/256.0, 250.0/256.0, 122.0/256.0, 218.0/256.0,  90.0/256.0,
     12.0/256.0, 140.0/256.0,  44.0/256.0, 172.0/256.0,   4.0/256.0, 132.0/256.0,  36.0/256.0, 164.0/256.0,  14.0/256.0, 142.0/256.0,  46.0/256.0, 174.0/256.0,   6.0/256.0, 134.0/256.0,  38.0/256.0, 166.0/256.0,
    204.0/256.0,  76.0/256.0, 236.0/256.0, 108.0/256.0, 196.0/256.0,  68.0/256.0, 228.0/256.0, 100.0/256.0, 206.0/256.0,  78.0/256.0, 238.0/256.0, 110.0/256.0, 198.0/256.0,  70.0/256.0, 230.0/256.0, 102.0/256.0,
     60.0/256.0, 188.0/256.0,  28.0/256.0, 156.0/256.0,  52.0/256.0, 180.0/256.0,  20.0/256.0, 148.0/256.0,  62.0/256.0, 190.0/256.0,  30.0/256.0, 158.0/256.0,  54.0/256.0, 182.0/256.0,  22.0/256.0, 150.0/256.0,
    252.0/256.0, 124.0/256.0, 220.0/256.0,  92.0/256.0, 244.0/256.0, 116.0/256.0, 212.0/256.0,  84.0/256.0, 254.0/256.0, 126.0/256.0, 222.0/256.0,  94.0/256.0, 246.0/256.0, 118.0/256.0, 214.0/256.0,  86.0/256.0,
      3.0/256.0, 131.0/256.0,  35.0/256.0, 163.0/256.0,  11.0/256.0, 139.0/256.0,  43.0/256.0, 171.0/256.0,   1.0/256.0, 129.0/256.0,  33.0/256.0, 161.0/256.0,   9.0/256.0, 137.0/256.0,  41.0/256.0, 169.0/256.0,
    195.0/256.0,  67.0/256.0, 227.0/256.0,  99.0/256.0, 203.0/256.0,  75.0/256.0, 235.0/256.0, 107.0/256.0, 193.0/256.0,  65.0/256.0, 225.0/256.0,  97.0/256.0, 201.0/256.0,  73.0/256.0, 233.0/256.0, 105.0/256.0,
     51.0/256.0, 179.0/256.0,  19.0/256.0, 147.0/256.0,  59.0/256.0, 187.0/256.0,  27.0/256.0, 155.0/256.0,  49.0/256.0, 177.0/256.0,  17.0/256.0, 145.0/256.0,  57.0/256.0, 185.0/256.0,  25.0/256.0, 153.0/256.0,
    243.0/256.0, 115.0/256.0, 211.0/256.0,  83.0/256.0, 251.0/256.0, 123.0/256.0, 219.0/256.0,  91.0/256.0, 241.0/256.0, 113.0/256.0, 209.0/256.0,  81.0/256.0, 249.0/256.0, 121.0/256.0, 217.0/256.0,  89.0/256.0,
     15.0/256.0, 143.0/256.0,  47.0/256.0, 175.0/256.0,   7.0/256.0, 135.0/256.0,  39.0/256.0, 167.0/256.0,  13.0/256.0, 141.0/256.0,  45.0/256.0, 173.0/256.0,   5.0/256.0, 133.0/256.0,  37.0/256.0, 165.0/256.0,
    207.0/256.0,  79.0/256.0, 239.0/256.0, 111.0/256.0, 199.0/256.0,  71.0/256.0, 231.0/256.0, 103.0/256.0, 205.0/256.0,  77.0/256.0, 237.0/256.0, 109.0/256.0, 197.0/256.0,  69.0/256.0, 229.0/256.0, 101.0/256.0,
     63.0/256.0, 191.0/256.0,  31.0/256.0, 159.0/256.0,  55.0/256.0, 183.0/256.0,  23.0/256.0, 151.0/256.0,  61.0/256.0, 189.0/256.0,  29.0/256.0, 157.0/256.0,  53.0/256.0, 181.0/256.0,  21.0/256.0, 149.0/256.0,
    255.0/256.0, 127.0/256.0, 223.0/256.0,  95.0/256.0, 247.0/256.0, 119.0/256.0, 215.0/256.0,  87.0/256.0, 253.0/256.0, 125.0/256.0, 221.0/256.0,  93.0/256.0, 245.0/256.0, 117.0/256.0, 213.0/256.0,  85.0/256.0,
);
```

---

## Shader Implementation

### Configurable Pattern Size

```wgsl
// ============================================
// DITHER CONFIGURATION - Change this to test!
// ============================================
const DITHER_SIZE: u32 = 4u;  // 2, 4, 8, or 16

// Include ONE of these based on DITHER_SIZE:
// (In practice, use #ifdef or just include all and select at runtime)

const BAYER_4X4: array<f32, 16> = array(
     0.0/16.0,  8.0/16.0,  2.0/16.0, 10.0/16.0,
    12.0/16.0,  4.0/16.0, 14.0/16.0,  6.0/16.0,
     3.0/16.0, 11.0/16.0,  1.0/16.0,  9.0/16.0,
    15.0/16.0,  7.0/16.0, 13.0/16.0,  5.0/16.0,
);

fn get_bayer_threshold(frag_coord: vec2<f32>) -> f32 {
    let x = u32(frag_coord.x) % DITHER_SIZE;
    let y = u32(frag_coord.y) % DITHER_SIZE;
    
    // For DITHER_SIZE = 4
    return BAYER_4X4[y * DITHER_SIZE + x];
}

fn should_discard(alpha: f32, frag_coord: vec2<f32>) -> bool {
    // Alpha 0.0 = always discard
    // Alpha 1.0 = never discard
    // Alpha 0.5 = 50% stipple pattern
    return alpha < get_bayer_threshold(frag_coord);
}
```

### Multi-Pattern Support (For Testing)

```wgsl
// All patterns available, select via uniform or compile-time const
const DITHER_PATTERN: u32 = 1u;  // 0=2x2, 1=4x4, 2=8x8, 3=16x16

fn get_threshold(frag_coord: vec2<f32>) -> f32 {
    let x = u32(frag_coord.x);
    let y = u32(frag_coord.y);
    
    switch DITHER_PATTERN {
        case 0u: {  // 2×2
            let idx = (y % 2u) * 2u + (x % 2u);
            return BAYER_2X2[idx];
        }
        case 1u: {  // 4×4
            let idx = (y % 4u) * 4u + (x % 4u);
            return BAYER_4X4[idx];
        }
        case 2u: {  // 8×8
            let idx = (y % 8u) * 8u + (x % 8u);
            return BAYER_8X8[idx];
        }
        case 3u: {  // 16×16
            let idx = (y % 16u) * 16u + (x % 16u);
            return BAYER_16X16[idx];
        }
        default: {
            return 0.5;
        }
    }
}
```

---

## Alpha Sources

Stipple alpha can come from multiple sources:

### 1. Texture Alpha

From albedo texture's alpha channel (BC7 or RGBA8):

```wgsl
let albedo = textureSample(t_albedo, s_albedo, in.uv);
let alpha = albedo.a;

if should_discard(alpha, in.position.xy) {
    discard;
}
```

### 2. Uniform Alpha (packedShadingState)

Per-draw-call alpha for entire mesh:

```wgsl
// packedShadingState bit layout (example):
// Bits 0-7:   uniform_alpha (0-255 → 0.0-1.0)
// Bits 8-15:  other shading params...

@group(0) @binding(0) var<uniform> packed_shading_state: u32;

fn get_uniform_alpha() -> f32 {
    let alpha_bits = packed_shading_state & 0xFFu;
    return f32(alpha_bits) / 255.0;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uniform_alpha = get_uniform_alpha();
    
    if should_discard(uniform_alpha, in.position.xy) {
        discard;
    }
    
    // Continue with shading...
}
```

### 3. Combined (Texture × Uniform)

Multiply texture alpha with uniform alpha for fadeout effects:

```wgsl
let tex_alpha = textureSample(t_albedo, s_albedo, in.uv).a;
let uniform_alpha = get_uniform_alpha();
let final_alpha = tex_alpha * uniform_alpha;

if should_discard(final_alpha, in.position.xy) {
    discard;
}
```

Use cases:
- Texture alpha: Per-pixel transparency (windows, fences, decals)
- Uniform alpha: Whole-mesh fadeout (death animation, teleport)
- Combined: Fade out a textured transparent object

---

## packedShadingState Integration

### Proposed Bit Layout

```
packedShadingState (32 bits):

Bits 0-7:    uniform_alpha (0-255)
Bits 8-9:    transparency_mode (0=opaque, 1=stipple, 2=alpha_test, 3=reserved)
Bits 10-11:  dither_pattern (0=2x2, 1=4x4, 2=8x8, 3=16x16)
Bits 12-31:  other shading parameters...
```

### Transparency Modes

| Mode | Value | Behavior |
|------|-------|----------|
| Opaque | 0 | No alpha processing, no discard |
| Stipple | 1 | Ordered dithering (Bayer pattern) |
| Alpha Test | 2 | Binary cutout (discard if alpha < 0.5) |
| Reserved | 3 | Future use |

### Shader Unpacking

```wgsl
struct ShadingParams {
    uniform_alpha: f32,
    transparency_mode: u32,
    dither_pattern: u32,
}

fn unpack_shading_state(packed: u32) -> ShadingParams {
    return ShadingParams(
        f32(packed & 0xFFu) / 255.0,           // Bits 0-7
        (packed >> 8u) & 0x3u,                  // Bits 8-9
        (packed >> 10u) & 0x3u,                 // Bits 10-11
    );
}
```

### Fragment Shader Integration

```wgsl
@group(0) @binding(0) var<uniform> packed_shading_state: u32;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let params = unpack_shading_state(packed_shading_state);
    let tex_color = textureSample(t_albedo, s_albedo, in.uv);
    
    // Combine texture and uniform alpha
    let final_alpha = tex_color.a * params.uniform_alpha;
    
    // Apply transparency mode
    switch params.transparency_mode {
        case 0u: {
            // Opaque - no discard
        }
        case 1u: {
            // Stipple
            let threshold = get_threshold_for_pattern(params.dither_pattern, in.position.xy);
            if final_alpha < threshold {
                discard;
            }
        }
        case 2u: {
            // Alpha test (binary cutout)
            if final_alpha < 0.5 {
                discard;
            }
        }
        default: {}
    }
    
    // Continue with lighting...
    return calculate_lighting(tex_color.rgb, in);
}
```

---

## FFI Functions

```rust
/// Set transparency mode for subsequent draw calls
/// mode: 0=opaque, 1=stipple, 2=alpha_test
fn set_transparency_mode(mode: u32);

/// Set uniform alpha for subsequent draw calls
/// alpha: 0-255 (maps to 0.0-1.0)
fn set_uniform_alpha(alpha: u8);

/// Set dither pattern for stipple mode
/// pattern: 0=2x2, 1=4x4, 2=8x8, 3=16x16
fn set_dither_pattern(pattern: u32);
```

Or, if using packed state:

```rust
/// Set packed shading state (includes alpha, mode, pattern, etc.)
fn set_shading_state(packed: u32);
```

---

## Visual Reference

### Stipple Levels (4×4 pattern)

```
α=0.00 (0/16):    α=0.25 (4/16):    α=0.50 (8/16):    α=0.75 (12/16):   α=1.00 (16/16):
░░░░              █░░░              █░█░              █░█░              ████
░░░░              ░░░░              ░█░█              ██░█              ████
░░░░              ░░█░              █░█░              █░██              ████
░░░░              ░░░░              ░█░█              ░███              ████
```

### Pattern Comparison at α=0.5

```
2×2:              4×4:              8×8:              16×16:
█░█░█░█░          █░█░              █░█░░█░█          (more varied,
░█░█░█░█          ░█░█              ░░█░█░░█           less obvious
█░█░█░█░          █░█░              ░█░░█░█░           pattern)
░█░█░█░█          ░█░█              █░░█░░█░
(obvious)         (classic)         (smoother)
```

---

## Shimmer Behavior

Screen-space stippling produces shimmer when objects/camera move. **This is intentional and era-authentic.**

As objects move, screen pixels cross threshold boundaries:
```
Frame 1: pixel at screen(100,100) → threshold 0.5 → α=0.4 HIDDEN
Frame 2: pixel at screen(100,100) → threshold 0.25 → α=0.4 VISIBLE
                                                      ↑ shimmer!
```

This "mesh transparency shimmer" is iconic to Saturn/PS1 games:
- Silent Hill fog
- Saturn transparencies (Panzer Dragoon, NiGHTS)
- PS1 water effects

**Do not try to fix the shimmer.** It's the aesthetic.

---

## Use Cases

| Effect | Alpha Source | Mode | Pattern |
|--------|--------------|------|---------|
| Glass window | Texture (constant α=0.5) | Stipple | 4×4 |
| Chain link fence | Texture (binary mask) | Alpha Test | — |
| Character fadeout | Uniform (animate 255→0) | Stipple | 4×4 |
| Fog volume | Uniform | Stipple | 8×8 |
| Particle smoke | Texture (soft edges) | Stipple | 4×4 |
| Tree leaves | Texture (cutout) | Alpha Test | — |
| Ghost enemy | Uniform (pulsing 128-200) | Stipple | 4×4 |

---

## Performance Considerations

| Aspect | Impact |
|--------|--------|
| Discard | Early-Z benefits preserved for opaque, broken for stipple |
| Overdraw | Minimal (stipple writes depth) |
| Pattern lookup | Trivial (array index + compare) |
| Memory | Pattern arrays are tiny (max 256 floats for 16×16) |

Stipple is essentially free compared to alpha blending.

---

## Open Questions

1. **Default pattern** — Ship with 4×4 default, or let developer choose at init?

2. **Per-draw pattern override** — Allow changing pattern mid-frame, or set once at init?

3. **Vertex alpha** — Should vertex color alpha also contribute to stipple?

4. **Depth write** — Always write depth for stippled pixels, or make configurable?

---

## References

- [Ordered Dithering (Wikipedia)](https://en.wikipedia.org/wiki/Ordered_dithering)
- [Bayer Matrix Generation](https://www.anisopteragames.com/how-to-fix-color-banding-with-dithering/)
- [Return of the Obra Dinn (1-bit dithering showcase)](https://obradinn.com/)
- [Saturn Mesh Transparency](https://segaretro.org/Sega_Saturn/Hardware_features)
