//! GLB/GLTF generation utilities for Nethercore asset tools
//!
//! This library provides builder-pattern APIs for constructing GLB files:
//! - BufferBuilder: Pack binary data with automatic alignment
//! - MeshBuilder: High-level mesh construction
//! - SkeletonBuilder: Skeleton and inverse bind matrices
//! - AnimationBuilder: Keyframe animation tracks
//! - GltfBuilder: Top-level GLTF document construction
//!
//! # Example
//!
//! ```no_run
//! use glb_builder::*;
//!
//! let mut buffer = BufferBuilder::new();
//! let mesh = MeshBuilder::new()
//!     .positions(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]])
//!     .normals(&[[0.0, 0.0, 1.0]; 3])
//!     .indices(&[0, 1, 2])
//!     .build(&mut buffer);
//!
//! let gltf = GltfBuilder::new()
//!     .buffer_byte_length(buffer.data().len() as u64)
//!     .add_mesh_from_accessors("Triangle", &mesh);
//!
//! // Build final document with accessors and views
//! let root = gltf.build(buffer.views(), buffer.accessors(), "glb-builder");
//! let glb_bytes = assemble_glb(&root, buffer.data());
//! ```

pub mod animation;
pub mod buffer;
pub mod document;
pub mod mesh;
pub mod skeleton;
pub mod utils;

pub use animation::{AnimationAccessors, AnimationBuilder};
pub use buffer::{AccessorIndex, BufferBuilder};
pub use document::GltfBuilder;
pub use mesh::{MeshAccessors, MeshBuilder};
pub use skeleton::{SkeletonAccessors, SkeletonBuilder};
pub use utils::{align_buffer, assemble_glb, compute_bounds};

// Re-export commonly used gltf-json types
pub use gltf_json as json;
pub use gltf_json::validation::Checked::Valid;
