//! SADL (Semantic Asset Description Language) Execution System
//!
//! SADL bridges creative intent and procedural generation by providing:
//! - Style tokens for consistent visual styles
//! - Color palettes for harmonious color schemes
//! - Material database for PBR parameters
//! - Generation recipes combining these into reusable presets
//!
//! # Example
//! ```no_run
//! use proc_gen::sadl::*;
//!
//! // Create a recipe from semantic description
//! let recipe = GenerationRecipe::from_description(
//!     "weathered medieval barrel",
//!     RecipeConstraints::default(),
//! );
//!
//! // Generate texture
//! let texture = recipe.generate_texture(256, 256, 42);
//!
//! // Generate mesh with vertex colors
//! let mesh = recipe.generate_mesh_with_colors(42);
//! ```

mod style_tokens;
mod palettes;
mod materials;
mod recipes;

pub use style_tokens::*;
pub use palettes::*;
pub use materials::*;
pub use recipes::*;
