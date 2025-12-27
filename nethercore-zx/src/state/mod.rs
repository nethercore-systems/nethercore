//! Nethercore ZX FFI state and types
//!
//! FFI staging state for Nethercore ZX console.
//! This state is rebuilt each frame from FFI calls and consumed by ZGraphics.
//! It is NOT part of rollback state - only GameState is rolled back.

mod config;
mod ffi_state;
mod resources;
mod rollback_state;

pub use config::ZXInitConfig;
pub use ffi_state::ZXFFIState;
pub use resources::{
    Font, KeyframeGpuInfo, KeyframeSource, PendingKeyframes, PendingMesh, PendingMeshPacked,
    PendingSkeleton, PendingTexture, SkeletonGpuInfo,
};
pub use rollback_state::{
    AudioPlaybackState, ChannelState, MAX_CHANNELS, TrackerState, ZRollbackState, tracker_flags,
};

/// Maximum number of bones for GPU skinning
pub const MAX_BONES: usize = 256;

/// Maximum number of skeletons that can be loaded
pub const MAX_SKELETONS: usize = 64;

/// Maximum number of keyframe collections that can be loaded
pub const MAX_KEYFRAME_COLLECTIONS: usize = 256;

// Re-export BoneMatrix3x4 from shared (the canonical POD type)
pub use nethercore_shared::math::BoneMatrix3x4;

/// Skeleton data containing inverse bind matrices
///
/// Stored on the CPU side for upload to GPU when bound.
/// The inverse bind matrices transform vertices from model space
/// to bone-local space at bind time.
#[derive(Clone, Debug)]
pub struct SkeletonData {
    /// Inverse bind matrices (one per bone, 3x4 row-major format)
    pub inverse_bind: Vec<BoneMatrix3x4>,
    /// Number of bones in this skeleton
    pub bone_count: u32,
}

/// Loaded keyframe collection (stored on host/ROM)
///
/// Contains keyframe data ready for decoding and use.
/// Data stays on host and is accessed via keyframe_read/keyframe_bind.
#[derive(Clone, Debug)]
pub struct LoadedKeyframeCollection {
    /// Number of bones per frame
    pub bone_count: u8,
    /// Number of frames in the collection
    pub frame_count: u16,
    /// Raw platform format data (frame_count × bone_count × 16 bytes)
    pub data: Vec<u8>,
}

/// A batch of quad instances that share the same texture bindings and viewport
#[derive(Debug, Clone)]
pub struct QuadBatch {
    /// True if this batch contains screen-space quads (2D), which forces depth_test on
    pub is_screen_space: bool,
    /// Texture handles for this batch (snapshot of bound_textures when batch was created)
    pub textures: [u32; 4],
    /// Quad instances in this batch
    pub instances: Vec<crate::graphics::QuadInstance>,
    /// Viewport for this batch (snapshot of current_viewport when batch was created)
    pub viewport: crate::graphics::Viewport,
    /// Stencil mode for this batch (snapshot of stencil_mode when batch was created)
    pub stencil_mode: crate::graphics::StencilMode,
    /// Layer for 2D ordering (higher layers render on top)
    pub layer: u32,
}
