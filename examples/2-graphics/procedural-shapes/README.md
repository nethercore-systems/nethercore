# Procedural Shapes Example

Demonstrates procedural mesh generation with optional UV textures and normal mapping.

## Features

- **7 procedural shapes** in plain mode (cube, sphere, cylinder, cone, plane, torus, capsule)
- **6 procedural shapes** in textured mode (UV debug texture)
- **4 procedural shapes** in normal-mapped mode (tangent-enabled variants)
- **Material mode cycling** (plain → textured → normal mapped)
- **Interactive shape cycling** with A button + manual/auto rotation

## Shapes

### Plain Mode (7 shapes)
1. **Cube** — Box with flat normals (1×1×1)
2. **Sphere** — UV sphere with smooth normals (r=1.5, 32×16 segments)
3. **Cylinder** — Cylinder with caps (r=1, h=2, 24 segments)
4. **Cone** — Tapered cylinder (r=1.5→0, h=2, 24 segments) *Plain mode only*
5. **Plane** — Subdivided ground plane (3×3, 8×8 subdivisions)
6. **Torus** — Donut shape (R=1.5, r=0.5, 32×16 segments)
7. **Capsule** — Pill shape (r=0.8, h=2, 24×8 segments)

### Textured Mode (6 shapes)
1. **Cube** — UV box unwrap
2. **Sphere** — UV equirectangular mapping
3. **Cylinder** — UV cylindrical mapping
4. **Plane** — UV grid mapping
5. **Torus** — UV wrapped mapping
6. **Capsule** — UV hybrid mapping

Note: The cone is generated via `cylinder(radius_bottom, 0.0, height, segments)`. This example omits the UV cone variant.

### Normal Mapped Mode (4 shapes)
1. **Cube** — Tangent-enabled mesh + procedural normal map
2. **Sphere** — Tangent-enabled mesh + procedural normal map
3. **Plane** — Tangent-enabled mesh + procedural normal map
4. **Torus** — Tangent-enabled mesh + procedural normal map

## Controls

- **A button**: Cycle through shapes
- **B button**: Cycle material mode (plain/textured/normal mapped)
- **X button**: Cycle normal map type (waves/bricks/ripples)
- **Left stick**: Rotate shape manually
- **Auto-rotates**: When stick is idle

## UV Debug Texture

When in textured mode, a procedurally generated 64×64 UV debug texture is used:

- **Red channel**: Increases left to right (U axis)
- **Green channel**: Increases bottom to top (V axis)
- **Blue channel**: Checker pattern for orientation

This helps visualize how UVs are mapped onto each procedural shape.

## FFI Functions Demonstrated

### Plain Mesh Generation
- `cube(size_x, size_y, size_z)` → mesh handle
- `sphere(radius, segments, rings)` → mesh handle
- `cylinder(radius_bottom, radius_top, height, segments)` → mesh handle
- `plane(size_x, size_z, subdivisions_x, subdivisions_z)` → mesh handle
- `torus(major_radius, minor_radius, major_segments, minor_segments)` → mesh handle
- `capsule(radius, height, segments, rings)` → mesh handle

### UV-Enabled Mesh Generation
- `cube_uv(size_x, size_y, size_z)` → mesh handle
- `sphere_uv(radius, segments, rings)` → mesh handle
- `cylinder_uv(radius_bottom, radius_top, height, segments)` → mesh handle
- `plane_uv(size_x, size_z, subdivisions_x, subdivisions_z)` → mesh handle
- `torus_uv(major_radius, minor_radius, major_segments, minor_segments)` → mesh handle
- `capsule_uv(radius, height, segments, rings)` → mesh handle

### Tangent-Enabled Mesh Generation (Normal Mapping)
- `cube_tangent(size_x, size_y, size_z)` → mesh handle
- `sphere_tangent(radius, segments, rings)` → mesh handle
- `plane_tangent(size_x, size_z, subdivisions_x, subdivisions_z)` → mesh handle
- `torus_tangent(major_radius, minor_radius, major_segments, minor_segments)` → mesh handle

## Building

```bash
# From the nethercore repo root
cargo run -- procedural-shapes

# Or build the WASM directly
cd examples/2-graphics/procedural-shapes
cargo build --target wasm32-unknown-unknown --release
```

## Learning Notes

This example demonstrates:

1. **Procedural mesh generation** — Creating geometry at runtime without asset files
2. **UV mapping** — Understanding how 2D texture coordinates wrap onto 3D shapes
3. **Tangent data** — Preparing meshes for normal-mapped rendering
4. **Procedural textures** — UV debug + albedo + normal maps
5. **Input handling** — Button edge detection for mode switching

Use this example as a reference for incorporating procedural shapes into your own games!
