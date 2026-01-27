//! Procedural Mesh Generation (init-only)
//!
//! All procedural mesh functions must be called during init().
//! They queue meshes for GPU upload which must happen before the game loop.

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Generate a cube mesh. **Init-only.**
    ///
    /// # Arguments
    /// * `size_x`, `size_y`, `size_z` — Half-extents along each axis
    pub fn cube(size_x: f32, size_y: f32, size_z: f32) -> u32;

    /// Generate a UV sphere mesh. **Init-only.**
    ///
    /// # Arguments
    /// * `radius` — Sphere radius
    /// * `segments` — Longitudinal divisions (3-256)
    /// * `rings` — Latitudinal divisions (2-256)
    pub fn sphere(radius: f32, segments: u32, rings: u32) -> u32;

    /// Generate a cylinder or cone mesh. **Init-only.**
    ///
    /// # Arguments
    /// * `radius_bottom`, `radius_top` — Radii (>= 0.0, use 0 for cone tip)
    /// * `height` — Cylinder height
    /// * `segments` — Radial divisions (3-256)
    pub fn cylinder(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) -> u32;

    /// Generate a plane mesh on the XZ plane. **Init-only.**
    ///
    /// # Arguments
    /// * `size_x`, `size_z` — Dimensions
    /// * `subdivisions_x`, `subdivisions_z` — Subdivisions (1-256)
    pub fn plane(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) -> u32;

    /// Generate a torus (donut) mesh. **Init-only.**
    ///
    /// # Arguments
    /// * `major_radius` — Distance from center to tube center
    /// * `minor_radius` — Tube radius
    /// * `major_segments`, `minor_segments` — Segment counts (3-256)
    pub fn torus(
        major_radius: f32,
        minor_radius: f32,
        major_segments: u32,
        minor_segments: u32,
    ) -> u32;

    /// Generate a capsule (pill shape) mesh. **Init-only.**
    ///
    /// # Arguments
    /// * `radius` — Capsule radius
    /// * `height` — Height of cylindrical section (total = height + 2*radius)
    /// * `segments` — Radial divisions (3-256)
    /// * `rings` — Divisions per hemisphere (1-128)
    pub fn capsule(radius: f32, height: f32, segments: u32, rings: u32) -> u32;

    // UV-enabled variants (Format 5: POS_UV_NORMAL) — also init-only

    /// Generate a UV sphere mesh with equirectangular texture mapping. **Init-only.**
    pub fn sphere_uv(radius: f32, segments: u32, rings: u32) -> u32;

    /// Generate a plane mesh with UV mapping. **Init-only.**
    pub fn plane_uv(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) -> u32;

    /// Generate a cube mesh with box-unwrapped UV mapping. **Init-only.**
    pub fn cube_uv(size_x: f32, size_y: f32, size_z: f32) -> u32;

    /// Generate a cylinder mesh with cylindrical UV mapping. **Init-only.**
    pub fn cylinder_uv(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) -> u32;

    /// Generate a torus mesh with wrapped UV mapping. **Init-only.**
    pub fn torus_uv(
        major_radius: f32,
        minor_radius: f32,
        major_segments: u32,
        minor_segments: u32,
    ) -> u32;

    /// Generate a capsule mesh with hybrid UV mapping. **Init-only.**
    pub fn capsule_uv(radius: f32, height: f32, segments: u32, rings: u32) -> u32;

    // Tangent-enabled variants (Format 21: POS_UV_NORMAL_TANGENT) — for normal mapping

    /// Generate a sphere mesh with tangent data for normal mapping. **Init-only.**
    ///
    /// Tangent follows direction of increasing U (longitude).
    /// Use with material_normal() for normal-mapped rendering.
    pub fn sphere_tangent(radius: f32, segments: u32, rings: u32) -> u32;

    /// Generate a plane mesh with tangent data for normal mapping. **Init-only.**
    ///
    /// Tangent points along +X, bitangent along +Z, normal along +Y.
    pub fn plane_tangent(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32)
        -> u32;

    /// Generate a cube mesh with tangent data for normal mapping. **Init-only.**
    ///
    /// Each face has correct tangent space for normal map sampling.
    pub fn cube_tangent(size_x: f32, size_y: f32, size_z: f32) -> u32;

    /// Generate a torus mesh with tangent data for normal mapping. **Init-only.**
    ///
    /// Tangent follows the major circle direction.
    pub fn torus_tangent(
        major_radius: f32,
        minor_radius: f32,
        major_segments: u32,
        minor_segments: u32,
    ) -> u32;
}
