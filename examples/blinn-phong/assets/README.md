# Blinn-Phong Texture Authoring Guide

This guide explains how to create textures for Mode 3 (Normalized Blinn-Phong) materials.

## Texture Slots

Mode 3 uses 3 texture slots:

| Slot | Purpose | Channels | Default Fallback |
|------|---------|----------|------------------|
| **0** | Albedo | RGB: Diffuse color<br>A: Unused (reserved for UI) | White (1.0, 1.0, 1.0) → uses material uniform color |
| **1** | RSE | R: Rim intensity<br>G: Shininess<br>B: Emissive | White (1.0, 1.0, 1.0) → uses uniform fallbacks |
| **2** | Specular | RGB: Specular highlight color<br>A: Unused | White (1.0, 1.0, 1.0) × light_color = light-colored specular |

## Slot 0: Albedo (albedo.png)

**Purpose:** Base diffuse color of the material

**Channel Layout:**
- **R, G, B:** RGB diffuse color (0-255)
- **A:** Unused for meshes (reserved for UI transparency)

**Tips:**
- Use standard texture painting tools (Substance Painter, Photoshop, etc.)
- This is the "base color" or "diffuse map" in most workflows
- Colors should be in sRGB colorspace (will be linearized by the GPU)
- Alpha channel is ignored for 3D meshes

**Example Values:**
- Gold: `(230, 153, 51)` - Warm orange-yellow
- Silver: `(230, 230, 230)` - Neutral light gray
- Leather: `(102, 64, 38)` - Dark brown
- Skin: `(217, 179, 166)` - Light pink-beige

---

## Slot 1: RSE (rse.png)

**Purpose:** Rim intensity, Shininess, and Emissive packed into RGB channels

**Channel Layout:**
- **R:** Rim intensity (0-255)
  - 0 (black) = no rim lighting
  - 255 (white) = maximum rim intensity
  - Use for edge highlights (paint edges bright, center dark)

- **G:** Shininess (0-255) → maps to 1-256 internally
  - 0 (black) = shininess 1 (very broad, soft highlights)
  - 128 (50% gray) = shininess 128 (medium)
  - 255 (white) = shininess 256 (very tight, sharp highlights like chrome)

- **B:** Emissive intensity (0-255)
  - 0 (black) = no self-illumination
  - 255 (white) = maximum emissive (albedo × 1.0)
  - Multiplies with albedo for final glow color

**Tips:**
- Create this as a separate texture, NOT a packed PBR map
- Use gradients for smooth shininess variation
- Paint rim intensity on edges for character silhouettes
- Emissive is additive - use sparingly for glowing details

**Shininess Reference Table:**

| Texture Value | Mapped Shininess | Visual | Use For |
|---------------|------------------|--------|---------|
| 0-51 (0.0-0.2) | 1-52 | Very broad, soft | Cloth, skin, rough stone |
| 51-102 (0.2-0.4) | 52-103 | Broad | Leather, wood, rubber |
| 102-154 (0.4-0.6) | 103-154 | Medium | Plastic, painted metal |
| 154-205 (0.6-0.8) | 154-205 | Tight | Polished metal, wet surfaces |
| 205-255 (0.8-1.0) | 205-256 | Very tight | Chrome, mirrors, glass |

**Example RSE Values (RGB):**
- Gold armor: `(51, 204, 0)` - Subtle rim, high shininess, no glow
- Silver metal: `(38, 217, 0)` - Minimal rim, very high shininess, no glow
- Leather: `(26, 77, 0)` - Very subtle rim, low shininess, no glow
- Wet skin: `(77, 179, 0)` - Moderate rim, medium-high shininess, no glow
- Glowing crystal: `(102, 192, 77)` - Strong rim, high shininess, moderate glow

---

## Slot 2: Specular (specular.png)

**Purpose:** Specular highlight color (tints the reflections)

**Channel Layout:**
- **R, G, B:** Specular tint color (0-255)
- **A:** Unused

**Tips:**
- White (255, 255, 255) = neutral specular (highlights match light color)
- Tinted values create colored highlights:
  - Gold: Warm orange `(230, 153, 51)`
  - Silver: Neutral white `(230, 230, 230)`
  - Copper: Orange-red `(230, 102, 51)`
  - Blue metal: Cool blue `(153, 179, 230)`
- Most non-metals should use neutral white or slight tint
- Metals can have strong color tints matching their albedo

**Default Behavior:**
- If Slot 2 is not bound, specular defaults to white (1.0, 1.0, 1.0)
- White × light_color = light-colored specular (natural behavior)

**Example Values:**
- Gold: `(230, 153, 51)` - Warm orange-yellow highlights
- Silver: `(230, 230, 230)` - Neutral white highlights
- Leather: `(77, 51, 26)` - Dark brown highlights
- Wet skin: `(204, 204, 204)` - Bright white highlights
- Matte plastic: `(102, 102, 102)` - Subtle gray highlights

---

## Workflow Examples

### Example 1: Gold Armor

**Albedo (Slot 0):**
```
RGB: (230, 153, 51) - Warm orange-yellow base
```

**RSE (Slot 1):**
```
R: 51   - Subtle rim on edges for definition
G: 204  - High shininess (tight highlights)
B: 0    - No emissive glow
```

**Specular (Slot 2):**
```
RGB: (230, 153, 51) - Warm orange highlights matching albedo
```

**Result:** Shiny gold armor with warm highlights and subtle edge definition

---

### Example 2: Glowing Crystal

**Albedo (Slot 0):**
```
RGB: (77, 179, 230) - Bright cyan base
```

**RSE (Slot 1):**
```
R: 102  - Strong rim for magical edge glow
G: 192  - High shininess (gem-like)
B: 77   - Moderate emissive (self-illumination)
```

**Specular (Slot 2):**
```
RGB: (204, 230, 255) - Cool white-blue highlights
```

**Result:** Glowing magical crystal with strong rim lighting and self-illumination

---

### Example 3: Leather Armor

**Albedo (Slot 0):**
```
RGB: (102, 64, 38) - Dark brown leather
```

**RSE (Slot 1):**
```
R: 26   - Very subtle rim
G: 77   - Low shininess (broad, soft highlights)
B: 0    - No glow
```

**Specular (Slot 2):**
```
RGB: (77, 51, 26) - Dark brown highlights
```

**Result:** Matte leather with soft, subtle highlights

---

## Uniform-Only Workflow

If you don't want to use textures, you can set material properties via uniform FFI functions:

```rust
// Set material color
set_color(0.9, 0.6, 0.2, 1.0);  // Gold color

// Set shininess (0.0-1.0 → maps to 1-256)
material_shininess(0.8);  // Maps to ~205 (tight highlights)

// Set rim lighting
material_rim(0.2, 0.15);  // intensity, power

// Set emissive
material_emissive(0.0);  // No glow
```

**Note:** Specular color cannot be set via uniform - it defaults to white (light-colored specular) when Slot 2 is not bound.

---

## Rim Power Parameter

**Rim power** controls the falloff curve of rim lighting:
- Low values (0.0-0.2): Broad, soft rim (maps to 0-6.4 internally)
- Medium values (0.2-0.4): Medium rim (maps to 6.4-12.8)
- High values (0.4-1.0): Tight, sharp rim (maps to 12.8-32)

**Rim power is uniform-only** - it cannot be set per-pixel from a texture. Use `material_rim(intensity, power)` to control it.

---

## Conversion from Other Workflows

### From PBR Workflow (Mode 2 MRE → Mode 3 RSE)

If you have existing Mode 2 (PBR) textures:

**Slot 1 Conversion:**
- **R channel (Metallic → Rim):** Delete metallic data, paint new rim intensity (edges bright)
- **G channel (Roughness → Shininess):** Invert roughness: `shininess = 1.0 - roughness` or `shininess_u8 = 255 - roughness_u8`
- **B channel (Emissive):** **No change** - emissive stays in same channel!

**Slot 2 Conversion:**
- Delete environment matcap
- Create new specular color texture:
  - Metals: Use albedo color
  - Non-metals: Use white or subtle tint

### From Hybrid Workflow (Old Mode 3)

If you have old Mode 3 (Hybrid) textures:
- Slot 0 (Albedo): No change
- Slot 1: Follow PBR conversion above (MRE → RSE)
- Slot 2: Replace matcap with specular color texture
- Slot 3: Delete (unused in new Mode 3)

---

## Tips and Best Practices

1. **Start with uniforms:** Test materials with `material_shininess()` and `material_rim()` before creating textures
2. **Shininess variation:** Use gradients in Slot 1.G for smooth transitions (e.g., worn metal edges vs polished center)
3. **Rim for silhouettes:** Paint Slot 1.R bright on edges for character definition against backgrounds
4. **Emissive sparingly:** High emissive values can wash out lighting - use subtly for magical effects
5. **Specular tinting:** Most materials should use neutral white or subtle tints - strong tints are for metals
6. **Test in-engine:** Preview all materials with actual lighting before finalizing textures

---

## Material Preset Reference

These are the materials demonstrated in the example:

| Material | Albedo RGB | Shininess | Rim (I, P) | Emissive | Specular RGB |
|----------|------------|-----------|------------|----------|--------------|
| Gold Armor | (230, 153, 51) | 0.8 (205) | 0.2, 0.15 | 0.0 | (230, 153, 51) |
| Silver Metal | (230, 230, 230) | 0.85 (217) | 0.15, 0.12 | 0.0 | (230, 230, 230) |
| Leather | (102, 64, 38) | 0.3 (77) | 0.1, 0.2 | 0.0 | (77, 51, 26) |
| Wet Skin | (217, 179, 166) | 0.7 (179) | 0.3, 0.25 | 0.0 | (204, 204, 204) |
| Matte Plastic | (128, 128, 140) | 0.5 (128) | 0.0, 0.0 | 0.0 | (102, 102, 102) |
| Glowing Crystal | (77, 179, 230) | 0.75 (192) | 0.4, 0.18 | 0.3 | (204, 230, 255) |

---

## Gotanda Normalization

Mode 3 uses **Gotanda 2010 energy-conserving normalization** for Blinn-Phong:

```
normalization = shininess × 0.0397436 + 0.0856832
```

This ensures consistent brightness across all shininess values (1-256):
- Low shininess (1-52): Broad highlights, same total energy
- Medium shininess (103-154): Medium highlights, same total energy
- High shininess (205-256): Tight highlights, same total energy

**You don't need to do anything special** - the shader handles this automatically. Just paint shininess values naturally in Slot 1.G.
