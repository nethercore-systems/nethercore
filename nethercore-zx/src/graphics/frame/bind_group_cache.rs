//! Bind group cache key for frame bind groups
//!
//! The frame bind group only needs recreation when buffer capacities change,
//! render mode changes, or EPU render-bound resources are recreated.

use std::hash::{Hash, Hasher};

/// Key for detecting when frame bind group needs recreation.
/// When any buffer capacity, render mode, or EPU resource version changes, the bind group must be recreated.
#[derive(Hash, PartialEq, Eq)]
pub(super) struct BindGroupKey {
    pub unified_transforms_capacity: usize,
    pub unified_animation_capacity: usize,
    pub shading_state_capacity: usize,
    pub mvp_indices_capacity: usize,
    pub render_mode: u8,
    pub quad_instance_capacity: u64,
    pub epu_resource_version: u64,
}

impl BindGroupKey {
    pub fn hash_value(&self) -> u64 {
        let mut hasher = xxhash_rust::xxh3::Xxh3::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}
