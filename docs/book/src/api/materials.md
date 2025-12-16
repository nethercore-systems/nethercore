# Material Functions

Material properties for PBR (Mode 2) and Blinn-Phong (Mode 3) rendering.

## Mode 2: Metallic-Roughness (PBR)

### material_metallic

Sets the metallic value for PBR rendering.

**Signature:**
```rust
fn material_metallic(value: f32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| value | `f32` | Metallic value (0.0 = dielectric, 1.0 = metal) |

**Example:**
```rust
fn render() {
    // Non-metallic plastic
    material_metallic(0.0);
    draw_mesh(plastic_toy);

    // Full metal
    material_metallic(1.0);
    draw_mesh(sword);

    // Partially metallic (worn paint on metal)
    material_metallic(0.3);
    draw_mesh(rusty_barrel);
}
```

---

### material_roughness

Sets the roughness value for PBR rendering.

**Signature:**
```rust
fn material_roughness(value: f32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| value | `f32` | Roughness value (0.0 = smooth/mirror, 1.0 = rough/matte) |

**Example:**
```rust
fn render() {
    // Mirror-like chrome
    material_roughness(0.1);
    draw_mesh(chrome_bumper);

    // Rough stone
    material_roughness(0.9);
    draw_mesh(stone_wall);

    // Smooth plastic
    material_roughness(0.4);
    draw_mesh(toy);
}
```

---

### material_emissive

Sets the emissive (self-illumination) intensity.

**Signature:**
```rust
fn material_emissive(value: f32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| value | `f32` | Emissive intensity (0.0 = none, 1.0+ = glowing) |

**Example:**
```rust
fn render() {
    // Glowing lava
    set_color(0xFF4400FF);
    material_emissive(2.0);
    draw_mesh(lava);

    // Neon sign
    set_color(0x00FFFFFF);
    material_emissive(1.5);
    draw_mesh(neon_tube);

    // Normal object (no glow)
    material_emissive(0.0);
    draw_mesh(normal_object);
}
```

---

### material_rim

Sets rim lighting parameters.

**Signature:**
```rust
fn material_rim(intensity: f32, power: f32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| intensity | `f32` | Rim light intensity (0.0-1.0) |
| power | `f32` | Rim light falloff power (0.0-1.0, maps to 0-32 internally) |

**Example:**
```rust
fn render() {
    // Subtle rim for characters
    material_rim(0.2, 0.15);
    draw_mesh(character);

    // Strong backlighting effect
    material_rim(0.5, 0.3);
    draw_mesh(silhouette_enemy);

    // No rim lighting
    material_rim(0.0, 0.0);
    draw_mesh(ground);
}
```

---

## Mode 3: Specular-Shininess (Blinn-Phong)

### material_shininess

Sets the shininess for specular highlights (Mode 3).

**Signature:**
```rust
fn material_shininess(value: f32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| value | `f32` | Shininess (0.0-1.0, maps to 1-256 internally) |

**Shininess Guide:**

| Value | Internal | Visual | Use For |
|-------|----------|--------|---------|
| 0.0-0.2 | 1-52 | Very soft, broad | Cloth, skin, rough stone |
| 0.2-0.4 | 52-103 | Broad | Leather, wood, rubber |
| 0.4-0.6 | 103-154 | Medium | Plastic, painted metal |
| 0.6-0.8 | 154-205 | Tight | Polished metal, wet surfaces |
| 0.8-1.0 | 205-256 | Very tight | Chrome, mirrors, glass |

**Example:**
```rust
fn render() {
    // Matte cloth
    material_shininess(0.1);
    draw_mesh(cloth);

    // Polished armor
    material_shininess(0.8);
    draw_mesh(armor);

    // Chrome
    material_shininess(0.95);
    draw_mesh(chrome_sphere);
}
```

---

### material_specular

Sets the specular highlight color (Mode 3).

**Signature:**
```rust
fn material_specular(color: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| color | `u32` | Specular color as `0xRRGGBBAA` |

**Example:**
```rust
fn render() {
    // White specular (default, most materials)
    material_specular(0xFFFFFFFF);
    draw_mesh(plastic);

    // Gold specular
    material_specular(0xFFD700FF);
    draw_mesh(gold_ring);

    // Copper specular
    material_specular(0xB87333FF);
    draw_mesh(copper_pot);
}
```

---

### material_specular_color

Sets the specular highlight color as RGB floats (Mode 3).

**Signature:**
```rust
fn material_specular_color(r: f32, g: f32, b: f32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| r, g, b | `f32` | Specular color components (0.0-1.0) |

**Example:**
```rust
fn render() {
    // Gold specular
    material_specular_color(1.0, 0.84, 0.0);
    draw_mesh(gold);

    // Tinted specular
    material_specular_color(0.8, 0.9, 1.0);
    draw_mesh(ice);
}
```

---

### material_specular_damping

Sets specular damping (Mode 3, alias for metallic behavior).

**Signature:**
```rust
fn material_specular_damping(value: f32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| value | `f32` | Damping value (0.0-1.0) |

---

## Texture Slots

### material_albedo

Binds an albedo (diffuse) texture to slot 0.

**Signature:**
```rust
fn material_albedo(texture: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| texture | `u32` | Texture handle |

**Note:** Equivalent to `texture_bind_slot(texture, 0)`.

---

### material_mre

Binds an MRE (Metallic/Roughness/Emissive) texture to slot 1 (Mode 2).

**Signature:**
```rust
fn material_mre(texture: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| texture | `u32` | Texture handle for MRE map |

**MRE Texture Channels:**
- **R:** Metallic (0-255 maps to 0.0-1.0)
- **G:** Roughness (0-255 maps to 0.0-1.0)
- **B:** Emissive (0-255 maps to emissive intensity)

**Example:**
```rust
fn render() {
    material_albedo(character_albedo);
    material_mre(character_mre);
    draw_mesh(character);
}
```

---

## Override Flags

These functions enable uniform values instead of texture sampling.

### use_uniform_color

Use uniform color instead of albedo texture.

**Signature:**
```rust
fn use_uniform_color(enabled: u32)
```

**Example:**
```rust
fn render() {
    // Use texture
    use_uniform_color(0);
    texture_bind(wood_tex);
    draw_mesh(table);

    // Use uniform color
    use_uniform_color(1);
    set_color(0xFF0000FF);
    draw_mesh(red_cube);
}
```

---

### use_uniform_metallic

Use uniform metallic value instead of MRE texture.

**Signature:**
```rust
fn use_uniform_metallic(enabled: u32)
```

---

### use_uniform_roughness

Use uniform roughness value instead of MRE texture.

**Signature:**
```rust
fn use_uniform_roughness(enabled: u32)
```

---

### use_uniform_emissive

Use uniform emissive value instead of MRE texture.

**Signature:**
```rust
fn use_uniform_emissive(enabled: u32)
```

---

### use_uniform_specular

Use uniform specular color instead of specular texture (Mode 3).

**Signature:**
```rust
fn use_uniform_specular(enabled: u32)
```

---

### use_matcap_reflection

Use matcap for environmental reflection (Mode 1).

**Signature:**
```rust
fn use_matcap_reflection(enabled: u32)
```

---

## Complete Examples

### PBR Material (Mode 2)

```rust
fn init() {
    render_mode(2); // Metallic-Roughness
}

fn render() {
    // Shiny metal sword
    material_albedo(sword_albedo);
    material_mre(sword_mre);
    material_rim(0.15, 0.2);
    push_identity();
    push_translate(player.x, player.y, player.z);
    draw_mesh(sword);

    // Simple colored object (no textures)
    use_uniform_color(1);
    use_uniform_metallic(1);
    use_uniform_roughness(1);

    set_color(0x4080FFFF);
    material_metallic(0.0);
    material_roughness(0.3);
    push_identity();
    draw_mesh(magic_orb);
}
```

### Blinn-Phong Material (Mode 3)

```rust
fn init() {
    render_mode(3); // Specular-Shininess
}

fn render() {
    // Gold armor
    set_color(0xE6B84DFF);  // Gold base color
    material_shininess(0.8);
    material_specular(0xFFD700FF);  // Gold specular
    material_rim(0.2, 0.15);
    material_emissive(0.0);
    draw_mesh(armor);

    // Glowing crystal
    set_color(0x4D99E6FF);  // Blue crystal
    material_shininess(0.75);
    material_specular(0xFFFFFFFF);
    material_rim(0.4, 0.18);
    material_emissive(0.3);  // Self-illumination
    draw_mesh(crystal);

    // Wet skin
    set_color(0xD9B399FF);
    material_shininess(0.7);
    material_specular(0xFFFFFFFF);
    material_rim(0.3, 0.25);
    material_emissive(0.0);
    draw_mesh(character_skin);
}
```

**See Also:** [Render Modes Guide](../guides/render-modes.md), [Textures](./textures.md), [Lighting](./lighting.md)
