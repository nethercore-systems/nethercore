//! Mesh generation and modification
//!
//! This module provides tools for creating and modifying meshes procedurally,
//! including primitives, modifiers, displacement, vertex coloring, and UV mapping.
//!
//! # Basic Example
//! ```no_run
//! use proc_gen::mesh::*;
//!
//! // Generate a basic sphere
//! let mut mesh: UnpackedMesh = generate_sphere_uv(1.0, 16, 8);
//!
//! // Apply noise displacement for organic look
//! mesh.apply(NoiseDisplace::weathered(42));
//!
//! // Bake ambient occlusion into vertex colors
//! mesh.apply(BakeVertexAO::default());
//!
//! // Snap UVs to pixel grid for clean texturing
//! mesh.apply(PixelSnapUVs { resolution: 256, half_pixel_offset: true });
//!
//! // Export
//! write_obj(&mesh, std::path::Path::new("output.obj"), "mesh").unwrap();
//! ```
//!
//! # PS1/PS2 Style Vertex Color Baking
//! ```no_run
//! use proc_gen::mesh::*;
//!
//! let mut mesh: UnpackedMesh = generate_cube_uv(1.0, 1.0, 1.0);
//!
//! // Bake ambient occlusion for classic PS1/N64 look
//! mesh.apply(BakeVertexAO::default());
//!
//! // Bake curvature for edge wear effect (stored in green channel)
//! mesh.apply(BakeVertexCurvature::default());
//!
//! // Add directional lighting bake
//! mesh.apply(BakeDirectionalLight {
//!     direction: [0.5, -0.8, 0.3],
//!     light_color: [255, 245, 230, 255],
//!     shadow_color: [60, 50, 70, 255],
//!     ..Default::default()
//! });
//! ```

// Re-export types from nethercore-zx
pub use nethercore_zx::procedural::{
    MeshBuilder, MeshBuilderUV, MeshData, MeshDataUV, UnpackedMesh,
};

// Re-export primitives
pub use nethercore_zx::procedural::{
    generate_cube, generate_sphere, generate_cylinder,
    generate_plane, generate_torus, generate_capsule,
    generate_cube_uv, generate_sphere_uv, generate_cylinder_uv,
    generate_plane_uv, generate_torus_uv, generate_capsule_uv,
};

// Re-export OBJ export
pub use nethercore_zx::procedural::write_obj;

// Local modules
pub mod modifiers;
pub mod combine;
pub mod displacement;
pub mod vertex_color;
pub mod uv;
pub mod details;

// Core modifiers
pub use modifiers::{
    MeshModifier, MeshApply, Transform, Mirror, SmoothNormals, FlatNormals,
    Subdivide, Chamfer, Axis,
};

// Mesh combining
pub use combine::{combine, combine_transformed};

// Noise displacement and deformers
pub use displacement::{
    NoiseDisplace, DirectionalDisplace, Bulge, BulgeAxis, Twist, Taper,
};

// Vertex color baking (PS1/PS2/N64 style)
pub use vertex_color::{
    BakeVertexAO, BakeVertexCurvature, VertexColorGradient, BakeDirectionalLight,
};

// UV manipulation
pub use uv::{
    PixelSnapUVs, NormalizeTexelDensity, ProjectUVs, UVProjection,
    FixCylindricalSeam, ScaleUVs, OffsetUVs, RotateUVs,
};

// Detail generators (rivets, panel lines, greebles)
pub use details::{
    AddRivets, AddPanelLines, PanelPattern, AddGreebles, GreebleVariety, AddBolts, BoltType,
};
