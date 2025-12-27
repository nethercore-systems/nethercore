# Procedural Shapes Example

Demonstrates all procedural mesh generation functions with optional texture mapping.

## Features

- **7 procedural shapes** in plain mode (cube, sphere, cylinder, cone, plane, torus, capsule)
- **6 procedural shapes** in textured mode (cone has no UV variant)
- **B button toggle** to switch between plain and UV-mapped rendering
- **Interactive shape cycling** with A button
- **Manual and automatic rotation** for visual inspection
- **UV debug texture** with checker pattern (textured mode only)

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

Note: Cone is only available in plain mode as there is no `cone_uv()` FFI function.

## Controls

- **A button**: Cycle through shapes
- **B button**: Toggle texture mode (plain/textured)
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

## Building

```bash
# From project root
cargo run -- procedural-shapes

# Or build directly
cd examples/procedural-shapes
cargo build --target wasm32-unknown-unknown --release
```

## Learning Notes

This example demonstrates:

1. **Procedural mesh generation** — Creating geometry at runtime without asset files
2. **UV mapping** — Understanding how 2D texture coordinates wrap onto 3D shapes
3. **Dual rendering paths** — Toggling between textured and untextured modes
4. **Input handling** — Button press edge detection for mode switching
5. **State management** — Maintaining separate mesh arrays for each mode

Use this example as a reference for incorporating procedural shapes into your own games!
