# Billboard Functions

Camera-facing quads for sprites in 3D space.

## Billboard Modes

| Mode | Name | Description |
|------|------|-------------|
| 1 | Spherical | Always faces camera (all axes) |
| 2 | Cylindrical Y | Rotates around Y axis only (trees, NPCs) |
| 3 | Cylindrical X | Rotates around X axis only |
| 4 | Cylindrical Z | Rotates around Z axis only |

---

## Functions

### draw_billboard

Draws a camera-facing quad using the bound texture.

**Signature:**
```rust
fn draw_billboard(w: f32, h: f32, mode: u32, color: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| w | `f32` | Width in world units |
| h | `f32` | Height in world units |
| mode | `u32` | Billboard mode (1-4) |
| color | `u32` | Tint color as `0xRRGGBBAA` |

**Example:**
```rust
fn render() {
    texture_bind(tree_sprite);

    // Trees with cylindrical Y billboards
    for tree in &trees {
        push_identity();
        push_translate(tree.x, tree.y, tree.z);
        draw_billboard(2.0, 4.0, 2, 0xFFFFFFFF);
    }

    // Particles with spherical billboards
    texture_bind(particle_sprite);
    blend_mode(2); // Additive
    for particle in &particles {
        push_identity();
        push_translate(particle.x, particle.y, particle.z);
        draw_billboard(0.5, 0.5, 1, particle.color);
    }
}
```

---

### draw_billboard_region

Draws a billboard using a texture region (sprite sheet).

**Signature:**
```rust
fn draw_billboard_region(
    w: f32, h: f32,
    src_x: f32, src_y: f32, src_w: f32, src_h: f32,
    mode: u32,
    color: u32
)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| w, h | `f32` | Size in world units |
| src_x, src_y | `f32` | Source position in texture (pixels) |
| src_w, src_h | `f32` | Source size in texture (pixels) |
| mode | `u32` | Billboard mode (1-4) |
| color | `u32` | Tint color |

**Example:**
```rust
fn render() {
    texture_bind(enemy_sheet);

    // Animated enemy sprite
    let frame = ((elapsed_time() * 8.0) as u32) % 4;
    push_identity();
    push_translate(enemy.x, enemy.y + 1.0, enemy.z);
    draw_billboard_region(
        2.0, 2.0,                              // Size
        (frame * 32) as f32, 0.0, 32.0, 32.0,  // Animation frame
        2,                                      // Cylindrical Y
        0xFFFFFFFF
    );
}
```

---

## Use Cases

### Trees and Vegetation

```rust
fn render() {
    texture_bind(vegetation_atlas);
    blend_mode(1); // Alpha blend for transparency
    cull_mode(0);  // Double-sided

    for tree in &trees {
        push_identity();
        push_translate(tree.x, tree.height * 0.5, tree.z);

        // Different tree types from atlas
        let src_x = (tree.type_id * 64) as f32;
        draw_billboard_region(
            tree.width, tree.height,
            src_x, 0.0, 64.0, 128.0,
            2, // Cylindrical Y - always upright
            0xFFFFFFFF
        );
    }
}
```

### Particle Effects

```rust
fn render() {
    texture_bind(particle_texture);
    blend_mode(2); // Additive for glow
    depth_test(1);

    for particle in &particles {
        push_identity();
        push_translate(particle.x, particle.y, particle.z);

        // Spherical billboard - faces camera completely
        let alpha = (particle.life * 255.0) as u32;
        let color = (particle.color & 0xFFFFFF00) | alpha;
        draw_billboard(particle.size, particle.size, 1, color);
    }
}
```

### NPCs and Enemies

```rust
fn render() {
    texture_bind(npc_sheet);
    blend_mode(1);

    for npc in &npcs {
        push_identity();
        push_translate(npc.x, npc.y + 1.0, npc.z);

        // Select animation frame based on direction and state
        let frame = get_npc_frame(npc);
        draw_billboard_region(
            2.0, 2.0,
            (frame % 4 * 32) as f32,
            (frame / 4 * 32) as f32,
            32.0, 32.0,
            2, // Cylindrical Y
            0xFFFFFFFF
        );
    }
}
```

### Health Bars Above Enemies

```rust
fn render() {
    // Draw enemies first
    for enemy in &enemies {
        draw_enemy(enemy);
    }

    // Then draw health bars as billboards
    depth_test(0); // On top of everything
    texture_bind(0); // No texture (solid color)

    for enemy in &enemies {
        if enemy.health < enemy.max_health {
            push_identity();
            push_translate(enemy.x, enemy.y + 2.5, enemy.z);

            // Background
            draw_billboard(1.0, 0.1, 1, 0x333333FF);

            // Health fill
            let ratio = enemy.health / enemy.max_health;
            push_scale(ratio, 1.0, 1.0);
            draw_billboard(1.0, 0.1, 1, 0x00FF00FF);
        }
    }

    depth_test(1);
}
```

---

## Complete Example

```rust
static mut TREE_TEX: u32 = 0;
static mut PARTICLE_TEX: u32 = 0;

struct Particle {
    x: f32, y: f32, z: f32,
    vx: f32, vy: f32, vz: f32,
    life: f32,
    size: f32,
}

static mut PARTICLES: [Particle; 100] = [Particle {
    x: 0.0, y: 0.0, z: 0.0,
    vx: 0.0, vy: 0.0, vz: 0.0,
    life: 0.0, size: 0.0,
}; 100];

fn init() {
    unsafe {
        TREE_TEX = rom_texture(b"tree".as_ptr(), 4);
        PARTICLE_TEX = rom_texture(b"spark".as_ptr(), 5);
    }
}

fn update() {
    unsafe {
        let dt = delta_time();
        for p in &mut PARTICLES {
            if p.life > 0.0 {
                p.x += p.vx * dt;
                p.y += p.vy * dt;
                p.z += p.vz * dt;
                p.vy -= 5.0 * dt; // Gravity
                p.life -= dt;
            }
        }
    }
}

fn render() {
    unsafe {
        // Trees - cylindrical billboards
        texture_bind(TREE_TEX);
        blend_mode(1);
        cull_mode(0);

        push_identity();
        push_translate(5.0, 2.0, -5.0);
        draw_billboard(2.0, 4.0, 2, 0xFFFFFFFF);

        push_identity();
        push_translate(-3.0, 1.5, -8.0);
        draw_billboard(1.5, 3.0, 2, 0xFFFFFFFF);

        // Particles - spherical billboards
        texture_bind(PARTICLE_TEX);
        blend_mode(2); // Additive

        for p in &PARTICLES {
            if p.life > 0.0 {
                push_identity();
                push_translate(p.x, p.y, p.z);
                let alpha = (p.life.min(1.0) * 255.0) as u32;
                draw_billboard(p.size, p.size, 1, 0xFFAA00FF & (0xFFFFFF00 | alpha));
            }
        }

        blend_mode(0);
        cull_mode(1);
    }
}
```

**See Also:** [Textures](./textures.md), [Transforms](./transforms.md)
