# Render Modes Guide

Emberware ZX supports 4 rendering modes, each with different lighting and material features.

## Overview

| Mode | Name | Lighting | Best For |
|------|------|----------|----------|
| 0 | Unlit | None | Flat colors, UI, retro 2D |
| 1 | Matcap | Pre-baked | Stylized, toon, sculpted look |
| 2 | Metallic-Roughness | PBR-style Blinn-Phong | Realistic materials |
| 3 | Specular-Shininess | Traditional Blinn-Phong | Classic 3D, arcade |

Set the mode once in `init()`:
```rust
fn init() {
    render_mode(2); // PBR-style lighting
}
```

---

## Mode 0: Unlit

No lighting calculations. Colors come directly from textures and `set_color()`.

**Features:**
- Fastest rendering
- Flat, solid colors
- No shadows or highlights
- Perfect for 2D sprites, UI, or intentionally flat aesthetics

**Example:**
```rust
fn init() {
    render_mode(0);
}

fn render() {
    // Color comes purely from texture + set_color tint
    texture_bind(sprite_tex);
    set_color(0xFFFFFFFF);
    draw_mesh(quad);
}
```

**Use cases:**
- 2D games with sprite-based graphics
- UI elements
- Retro flat-shaded PS1 style
- Unlit portions of scenes (skyboxes, emissive objects)

---

## Mode 1: Matcap

Uses matcap textures for pre-baked lighting. Fast and stylized.

**Features:**
- Lighting baked into matcap textures
- No dynamic lights
- Great for stylized/toon looks
- Multiple matcaps can be layered

**Texture Slots:**

| Slot | Purpose | Blend Mode |
|------|---------|------------|
| 0 | Albedo (UV-mapped) | Base color |
| 1-3 | Matcap (normal-mapped) | Configurable |

**Matcap Blend Modes:**
- **0 (Multiply):** Darkens (shadows, AO)
- **1 (Add):** Brightens (highlights, rim)
- **2 (HSV Modulate):** Hue/saturation shift

**Example:**
```rust
fn init() {
    render_mode(1);
    SHADOW_MATCAP = rom_texture(b"matcap_shadow".as_ptr(), 13);
    HIGHLIGHT_MATCAP = rom_texture(b"matcap_highlight".as_ptr(), 16);
}

fn render() {
    texture_bind(character_albedo);
    matcap_set(1, SHADOW_MATCAP);
    matcap_blend_mode(1, 0); // Multiply
    matcap_set(2, HIGHLIGHT_MATCAP);
    matcap_blend_mode(2, 1); // Add
    draw_mesh(character);
}
```

**Use cases:**
- Stylized/cartoon characters
- Sculpt-like rendering
- Fast mobile-friendly lighting
- Consistent lighting regardless of scene

---

## Mode 2: Metallic-Roughness

PBR-inspired Blinn-Phong with metallic/roughness workflow.

**Features:**
- Up to 4 dynamic lights
- Metallic/roughness material properties
- MRE texture support (Metallic/Roughness/Emissive)
- Rim lighting
- Procedural sky ambient
- Energy-conserving Gotanda normalization

**Texture Slots:**

| Slot | Purpose | Channels |
|------|---------|----------|
| 0 | Albedo | RGB: Diffuse color |
| 1 | MRE | R: Metallic, G: Roughness, B: Emissive |

**Material Functions:**
```rust
material_metallic(0.0);    // 0 = dielectric, 1 = metal
material_roughness(0.5);   // 0 = mirror, 1 = rough
material_emissive(0.0);    // Self-illumination
material_rim(0.2, 0.15);   // Rim light intensity and power
```

**Example:**
```rust
fn init() {
    render_mode(2);
}

fn render() {
    // Set up lighting
    light_set(0, 0.5, -0.7, 0.5);
    light_color(0, 0xFFF2E6FF);
    light_enable(0);

    // Shiny metal
    material_metallic(1.0);
    material_roughness(0.2);
    material_rim(0.1, 0.2);
    texture_bind(sword_tex);
    draw_mesh(sword);

    // Rough stone
    material_metallic(0.0);
    material_roughness(0.9);
    material_rim(0.0, 0.0);
    texture_bind(stone_tex);
    draw_mesh(wall);
}
```

**Use cases:**
- Realistic materials (metal, plastic, wood)
- PBR asset pipelines
- Games requiring material variety
- Modern 3D aesthetics

---

## Mode 3: Specular-Shininess

Traditional Blinn-Phong with direct specular color control.

**Features:**
- Up to 4 dynamic lights
- Shininess-based specular
- Direct specular color control
- Rim lighting
- Energy-conserving Gotanda normalization

**Texture Slots:**

| Slot | Purpose | Channels |
|------|---------|----------|
| 0 | Albedo | RGB: Diffuse color |
| 1 | SSE | R: Specular intensity, G: Shininess, B: Emissive |
| 2 | Specular | RGB: Specular highlight color |

**Material Functions:**
```rust
material_shininess(0.7);           // 0-1 → maps to 1-256
material_specular(0xFFD700FF);     // Specular highlight color
material_emissive(0.0);            // Self-illumination
material_rim(0.2, 0.15);           // Rim light
```

**Shininess Values:**

| Value | Shininess | Appearance |
|-------|-----------|------------|
| 0.0-0.2 | 1-52 | Very soft (cloth, skin) |
| 0.2-0.4 | 52-103 | Broad (leather, wood) |
| 0.4-0.6 | 103-154 | Medium (plastic) |
| 0.6-0.8 | 154-205 | Tight (polished metal) |
| 0.8-1.0 | 205-256 | Mirror (chrome, glass) |

**Example:**
```rust
fn init() {
    render_mode(3);
}

fn render() {
    // Gold armor
    set_color(0xE6B84DFF);
    material_shininess(0.8);
    material_specular(0xFFD700FF);
    material_rim(0.2, 0.15);
    draw_mesh(armor);

    // Wet skin
    set_color(0xD9B399FF);
    material_shininess(0.7);
    material_specular(0xFFFFFFFF);
    material_rim(0.3, 0.25);
    draw_mesh(character);
}
```

**Use cases:**
- Classic 3D game aesthetics
- Colored specular highlights (metals)
- Artist-friendly workflow
- Fighting games, action games

---

## Choosing a Mode

| If you need... | Use Mode |
|----------------|----------|
| Fastest rendering, no lighting | 0 (Unlit) |
| Stylized, consistent lighting | 1 (Matcap) |
| PBR workflow with MRE textures | 2 (Metallic-Roughness) |
| Colored specular, artist control | 3 (Specular-Shininess) |

**Performance:** All lit modes (1-3) have similar performance. Mode 0 is fastest.

**Compatibility:** All modes work with procedural meshes and skeletal animation.

---

## Common Setup

All lit modes benefit from proper sky and light setup:

```rust
fn init() {
    render_mode(2); // or 1 or 3

    // Sky provides ambient light
    sky_set_colors(0xB2D8F2FF, 0x3366B2FF);
    sky_set_sun(0.5, 0.7, 0.5, 0xFFF2E6FF, 0.95);
}

fn render() {
    draw_sky();

    // Main light (match sun direction)
    light_set(0, 0.5, -0.7, 0.5);
    light_color(0, 0xFFF2E6FF);
    light_intensity(0, 1.0);
    light_enable(0);

    // Fill light
    light_set(1, -0.8, -0.3, 0.0);
    light_color(1, 0x8899BBFF);
    light_intensity(1, 0.3);
    light_enable(1);

    // Draw scene...
}
```
