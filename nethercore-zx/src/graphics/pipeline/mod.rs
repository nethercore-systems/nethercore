//! Shader pipeline management
//!
//! Handles shader compilation, pipeline caching, and bind group layout creation
//! for all render mode and vertex format combinations.

mod bind_groups;
mod cache;
mod pipeline_creation;
mod pipeline_key;

// Re-export public types
pub use cache::PipelineCache;

// Re-export internal types used by graphics module
pub(crate) use pipeline_creation::PipelineEntry;
pub(crate) use pipeline_key::PipelineKey;
