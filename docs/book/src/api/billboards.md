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

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn draw_billboard(w: f32, h: f32, mode: u32, color: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void draw_billboard(float w, float h, uint32_t mode, uint32_t color);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn draw_billboard(w: f32, h: f32, mode: u32, color: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| w | `f32` | Width in world units |
| h | `f32` | Height in world units |
| mode | `u32` | Billboard mode (1-4) |
| color | `u32` | Tint color as `0xRRGGBBAA` |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render() {
    texture_bind(tree_sprite);

    // Trees with cylindrical Y billboards
    for (int i = 0; i < tree_count; i++) {
        push_identity();
        push_translate(trees[i].x, trees[i].y, trees[i].z);
        draw_billboard(2.0, 4.0, 2, 0xFFFFFFFF);
    }

    // Particles with spherical billboards
    texture_bind(particle_sprite);
    blend_mode(2); // Additive
    for (int i = 0; i < particle_count; i++) {
        push_identity();
        push_translate(particles[i].x, particles[i].y, particles[i].z);
        draw_billboard(0.5, 0.5, 1, particles[i].color);
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    texture_bind(tree_sprite);

    // Trees with cylindrical Y billboards
    for (trees) |tree| {
        push_identity();
        push_translate(tree.x, tree.y, tree.z);
        draw_billboard(2.0, 4.0, 2, 0xFFFFFFFF);
    }

    // Particles with spherical billboards
    texture_bind(particle_sprite);
    blend_mode(2); // Additive
    for (particles) |particle| {
        push_identity();
        push_translate(particle.x, particle.y, particle.z);
        draw_billboard(0.5, 0.5, 1, particle.color);
    }
}
```
{{#endtab}}

{{#endtabs}}

---

### draw_billboard_region

Draws a billboard using a texture region (sprite sheet).

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn draw_billboard_region(
    w: f32, h: f32,
    src_x: f32, src_y: f32, src_w: f32, src_h: f32,
    mode: u32,
    color: u32
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void draw_billboard_region(
    float w, float h,
    float src_x, float src_y, float src_w, float src_h,
    uint32_t mode,
    uint32_t color
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn draw_billboard_region(
    w: f32, h: f32,
    src_x: f32, src_y: f32, src_w: f32, src_h: f32,
    mode: u32,
    color: u32
) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| w, h | `f32` | Size in world units |
| src_x, src_y | `f32` | Source position in texture (pixels) |
| src_w, src_h | `f32` | Source size in texture (pixels) |
| mode | `u32` | Billboard mode (1-4) |
| color | `u32` | Tint color |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render() {
    texture_bind(enemy_sheet);

    // Animated enemy sprite
    uint32_t frame = ((uint32_t)(elapsed_time() * 8.0)) % 4;
    push_identity();
    push_translate(enemy.x, enemy.y + 1.0, enemy.z);
    draw_billboard_region(
        2.0, 2.0,                              // Size
        (float)(frame * 32), 0.0, 32.0, 32.0,  // Animation frame
        2,                                      // Cylindrical Y
        0xFFFFFFFF
    );
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    texture_bind(enemy_sheet);

    // Animated enemy sprite
    const frame = (@as(u32, @intFromFloat(elapsed_time() * 8.0))) % 4;
    push_identity();
    push_translate(enemy.x, enemy.y + 1.0, enemy.z);
    draw_billboard_region(
        2.0, 2.0,                              // Size
        @as(f32, @floatFromInt(frame * 32)), 0.0, 32.0, 32.0,  // Animation frame
        2,                                      // Cylindrical Y
        0xFFFFFFFF
    );
}
```
{{#endtab}}

{{#endtabs}}

---

## Use Cases

### Trees and Vegetation

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render() {
    texture_bind(vegetation_atlas);
    blend_mode(1); // Alpha blend for transparency
    cull_mode(0);  // Double-sided

    for (int i = 0; i < tree_count; i++) {
        push_identity();
        push_translate(trees[i].x, trees[i].height * 0.5, trees[i].z);

        // Different tree types from atlas
        float src_x = (float)(trees[i].type_id * 64);
        draw_billboard_region(
            trees[i].width, trees[i].height,
            src_x, 0.0, 64.0, 128.0,
            2, // Cylindrical Y - always upright
            0xFFFFFFFF
        );
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    texture_bind(vegetation_atlas);
    blend_mode(1); // Alpha blend for transparency
    cull_mode(0);  // Double-sided

    for (trees) |tree| {
        push_identity();
        push_translate(tree.x, tree.height * 0.5, tree.z);

        // Different tree types from atlas
        const src_x = @as(f32, @floatFromInt(tree.type_id * 64));
        draw_billboard_region(
            tree.width, tree.height,
            src_x, 0.0, 64.0, 128.0,
            2, // Cylindrical Y - always upright
            0xFFFFFFFF
        );
    }
}
```
{{#endtab}}

{{#endtabs}}

### Particle Effects

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render() {
    texture_bind(particle_texture);
    blend_mode(2); // Additive for glow
    depth_test(1);

    for (int i = 0; i < particle_count; i++) {
        push_identity();
        push_translate(particles[i].x, particles[i].y, particles[i].z);

        // Spherical billboard - faces camera completely
        uint32_t alpha = (uint32_t)(particles[i].life * 255.0);
        uint32_t color = (particles[i].color & 0xFFFFFF00) | alpha;
        draw_billboard(particles[i].size, particles[i].size, 1, color);
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    texture_bind(particle_texture);
    blend_mode(2); // Additive for glow
    depth_test(1);

    for (particles) |particle| {
        push_identity();
        push_translate(particle.x, particle.y, particle.z);

        // Spherical billboard - faces camera completely
        const alpha = @as(u32, @intFromFloat(particle.life * 255.0));
        const color = (particle.color & 0xFFFFFF00) | alpha;
        draw_billboard(particle.size, particle.size, 1, color);
    }
}
```
{{#endtab}}

{{#endtabs}}

### NPCs and Enemies

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render() {
    texture_bind(npc_sheet);
    blend_mode(1);

    for (int i = 0; i < npc_count; i++) {
        push_identity();
        push_translate(npcs[i].x, npcs[i].y + 1.0, npcs[i].z);

        // Select animation frame based on direction and state
        uint32_t frame = get_npc_frame(&npcs[i]);
        draw_billboard_region(
            2.0, 2.0,
            (float)(frame % 4 * 32),
            (float)(frame / 4 * 32),
            32.0, 32.0,
            2, // Cylindrical Y
            0xFFFFFFFF
        );
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    texture_bind(npc_sheet);
    blend_mode(1);

    for (npcs) |npc| {
        push_identity();
        push_translate(npc.x, npc.y + 1.0, npc.z);

        // Select animation frame based on direction and state
        const frame = get_npc_frame(npc);
        draw_billboard_region(
            2.0, 2.0,
            @as(f32, @floatFromInt(frame % 4 * 32)),
            @as(f32, @floatFromInt(frame / 4 * 32)),
            32.0, 32.0,
            2, // Cylindrical Y
            0xFFFFFFFF
        );
    }
}
```
{{#endtab}}

{{#endtabs}}

### Health Bars Above Enemies

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render() {
    // Draw enemies first
    for (int i = 0; i < enemy_count; i++) {
        draw_enemy(&enemies[i]);
    }

    // Then draw health bars as billboards
    depth_test(0); // On top of everything
    texture_bind(0); // No texture (solid color)

    for (int i = 0; i < enemy_count; i++) {
        if (enemies[i].health < enemies[i].max_health) {
            push_identity();
            push_translate(enemies[i].x, enemies[i].y + 2.5, enemies[i].z);

            // Background
            draw_billboard(1.0, 0.1, 1, 0x333333FF);

            // Health fill
            float ratio = enemies[i].health / enemies[i].max_health;
            push_scale(ratio, 1.0, 1.0);
            draw_billboard(1.0, 0.1, 1, 0x00FF00FF);
        }
    }

    depth_test(1);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Draw enemies first
    for (enemies) |enemy| {
        draw_enemy(enemy);
    }

    // Then draw health bars as billboards
    depth_test(0); // On top of everything
    texture_bind(0); // No texture (solid color)

    for (enemies) |enemy| {
        if (enemy.health < enemy.max_health) {
            push_identity();
            push_translate(enemy.x, enemy.y + 2.5, enemy.z);

            // Background
            draw_billboard(1.0, 0.1, 1, 0x333333FF);

            // Health fill
            const ratio = enemy.health / enemy.max_health;
            push_scale(ratio, 1.0, 1.0);
            draw_billboard(1.0, 0.1, 1, 0x00FF00FF);
        }
    }

    depth_test(1);
}
```
{{#endtab}}

{{#endtabs}}

---

## Complete Example

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
#include <math.h>

static uint32_t tree_tex = 0;
static uint32_t particle_tex = 0;

typedef struct {
    float x, y, z;
    float vx, vy, vz;
    float life;
    float size;
} Particle;

static Particle particles[100] = {0};

NCZX_EXPORT void init() {
    tree_tex = rom_texture("tree", 4);
    particle_tex = rom_texture("spark", 5);
}

NCZX_EXPORT void update() {
    float dt = delta_time();
    for (int i = 0; i < 100; i++) {
        if (particles[i].life > 0.0) {
            particles[i].x += particles[i].vx * dt;
            particles[i].y += particles[i].vy * dt;
            particles[i].z += particles[i].vz * dt;
            particles[i].vy -= 5.0 * dt; // Gravity
            particles[i].life -= dt;
        }
    }
}

NCZX_EXPORT void render() {
    // Trees - cylindrical billboards
    texture_bind(tree_tex);
    blend_mode(1);
    cull_mode(0);

    push_identity();
    push_translate(5.0, 2.0, -5.0);
    draw_billboard(2.0, 4.0, 2, 0xFFFFFFFF);

    push_identity();
    push_translate(-3.0, 1.5, -8.0);
    draw_billboard(1.5, 3.0, 2, 0xFFFFFFFF);

    // Particles - spherical billboards
    texture_bind(particle_tex);
    blend_mode(2); // Additive

    for (int i = 0; i < 100; i++) {
        if (particles[i].life > 0.0) {
            push_identity();
            push_translate(particles[i].x, particles[i].y, particles[i].z);
            uint32_t alpha = (uint32_t)(fminf(particles[i].life, 1.0) * 255.0);
            draw_billboard(particles[i].size, particles[i].size, 1, 0xFFAA00FF & (0xFFFFFF00 | alpha));
        }
    }

    blend_mode(0);
    cull_mode(1);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var tree_tex: u32 = 0;
var particle_tex: u32 = 0;

const Particle = struct {
    x: f32 = 0.0, y: f32 = 0.0, z: f32 = 0.0,
    vx: f32 = 0.0, vy: f32 = 0.0, vz: f32 = 0.0,
    life: f32 = 0.0,
    size: f32 = 0.0,
};

var particles: [100]Particle = [_]Particle{.{}} ** 100;

export fn init() void {
    tree_tex = rom_texture("tree", 4);
    particle_tex = rom_texture("spark", 5);
}

export fn update() void {
    const dt = delta_time();
    for (&particles) |*p| {
        if (p.life > 0.0) {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.z += p.vz * dt;
            p.vy -= 5.0 * dt; // Gravity
            p.life -= dt;
        }
    }
}

export fn render() void {
    // Trees - cylindrical billboards
    texture_bind(tree_tex);
    blend_mode(1);
    cull_mode(0);

    push_identity();
    push_translate(5.0, 2.0, -5.0);
    draw_billboard(2.0, 4.0, 2, 0xFFFFFFFF);

    push_identity();
    push_translate(-3.0, 1.5, -8.0);
    draw_billboard(1.5, 3.0, 2, 0xFFFFFFFF);

    // Particles - spherical billboards
    texture_bind(particle_tex);
    blend_mode(2); // Additive

    for (particles) |p| {
        if (p.life > 0.0) {
            push_identity();
            push_translate(p.x, p.y, p.z);
            const alpha = @as(u32, @intFromFloat(@min(p.life, 1.0) * 255.0));
            draw_billboard(p.size, p.size, 1, 0xFFAA00FF & (0xFFFFFF00 | alpha));
        }
    }

    blend_mode(0);
    cull_mode(1);
}
```
{{#endtab}}

{{#endtabs}}

**See Also:** [Textures](./textures.md), [Transforms](./transforms.md)
