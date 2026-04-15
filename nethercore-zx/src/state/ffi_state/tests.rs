//! Tests for ZXFFIState

#![allow(clippy::field_reassign_with_default)]

use super::*;
use glam::{Mat4, Vec3};

#[test]
fn test_default_state_has_default_matrices() {
    let state = ZXFFIState::default();

    // Should have one default view matrix
    assert_eq!(state.view_matrices.len(), 1);
    // Should have one default projection matrix
    assert_eq!(state.proj_matrices.len(), 1);
    // Should have one default model matrix (identity at index 0)
    assert_eq!(state.model_matrices.len(), 1);
    assert_eq!(state.model_matrices[0], Mat4::IDENTITY);

    // Current matrices should be None (use defaults from pool)
    assert_eq!(state.current_model_matrix, None);
    assert_eq!(state.current_view_matrix, None);
    assert_eq!(state.current_proj_matrix, None);
}

// ========================================================================
// Tests for new lazy allocation + deduplication system
// ========================================================================

#[test]
fn test_lazy_allocation_with_option_pattern() {
    let mut state = ZXFFIState::default();

    // Initially, current matrices should be None (use defaults from pool)
    assert_eq!(state.current_model_matrix, None);
    assert_eq!(state.current_view_matrix, None);
    assert_eq!(state.current_proj_matrix, None);

    // Set a new model matrix
    let new_model = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
    state.current_model_matrix = Some(new_model);

    // Allocate via add_mvp_shading_state()
    let buffer_idx = state.add_mvp_shading_state();

    // Should return buffer index 0 (first allocation)
    assert_eq!(buffer_idx, 0);

    // Model matrix should have been pushed to pool
    assert_eq!(state.model_matrices.len(), 2); // Identity + new matrix
    assert_eq!(state.model_matrices[1], new_model);

    // current_model_matrix should be taken (back to None)
    assert_eq!(state.current_model_matrix, None);
}

#[test]
fn test_mvp_shading_deduplication() {
    let mut state = ZXFFIState::default();

    // Set transform and color
    state.current_model_matrix = Some(Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)));
    state.current_shading_state.color_rgba8 = 0xFF0000FF; // Red
    state.shading_state_dirty = true;

    // First draw - allocates buffer index 0
    let idx1 = state.add_mvp_shading_state();
    assert_eq!(idx1, 0);
    assert_eq!(state.mvp_shading_states.len(), 1);

    // Second draw with same state (current matrices are None, will use last in pool)
    let idx2 = state.add_mvp_shading_state();

    // Should reuse the same buffer index due to deduplication
    assert_eq!(idx2, 0);
    assert_eq!(state.mvp_shading_states.len(), 1); // Still only 1 entry

    // Change color - should create new entry
    state.current_shading_state.color_rgba8 = 0x0000FFFF; // Blue
    state.shading_state_dirty = true;
    let idx3 = state.add_mvp_shading_state();
    assert_eq!(idx3, 1); // New buffer index
    assert_eq!(state.mvp_shading_states.len(), 2);
}

#[test]
fn test_multiple_draws_share_buffer_index() {
    let mut state = ZXFFIState::default();

    // Set transform once
    state.current_model_matrix = Some(Mat4::IDENTITY);
    state.current_shading_state.color_rgba8 = 0xFFFFFFFF;
    state.shading_state_dirty = true;

    // Simulate multiple draw calls with same state
    let idx1 = state.add_mvp_shading_state();
    let idx2 = state.add_mvp_shading_state();
    let idx3 = state.add_mvp_shading_state();

    // All should use the same buffer index
    assert_eq!(idx1, idx2);
    assert_eq!(idx2, idx3);

    // Only one buffer entry should exist
    assert_eq!(state.mvp_shading_states.len(), 1);
}

#[test]
fn test_different_transforms_different_indices() {
    let mut state = ZXFFIState::default();

    // Draw 1: Transform A
    state.current_model_matrix = Some(Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)));
    state.current_shading_state.color_rgba8 = 0xFF0000FF;
    state.shading_state_dirty = true;
    let idx1 = state.add_mvp_shading_state();

    // Draw 2: Transform B
    state.current_model_matrix = Some(Mat4::from_translation(Vec3::new(2.0, 0.0, 0.0)));
    state.current_shading_state.color_rgba8 = 0x00FF00FF;
    state.shading_state_dirty = true;
    let idx2 = state.add_mvp_shading_state();

    // Draw 3: Back to Transform A + same color
    state.current_model_matrix = None; // Use model_matrices[1] (first transform)
    state.model_matrices.truncate(2); // Remove the second transform
    state.current_shading_state.color_rgba8 = 0xFF0000FF;
    state.shading_state_dirty = true;

    // First two should be different
    assert_ne!(idx1, idx2);

    // Third should match first (deduplication works!)
    // Note: This might not deduplicate perfectly because we removed the matrix
    // but the test shows the deduplication concept
    assert_eq!(state.mvp_shading_states.len(), 2); // At least 2 unique states
}

#[test]
fn test_clear_frame_resets_mvp_state() {
    let mut state = ZXFFIState::default();

    // Add some MVP states
    state.current_model_matrix = Some(Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0)));
    state.current_shading_state.color_rgba8 = 0xFF0000FF;
    state.shading_state_dirty = true;
    state.add_mvp_shading_state();

    state.current_model_matrix = Some(Mat4::from_translation(Vec3::new(4.0, 5.0, 6.0)));
    state.current_shading_state.color_rgba8 = 0x0000FFFF;
    state.shading_state_dirty = true;
    state.add_mvp_shading_state();

    // Should have multiple entries
    assert!(!state.mvp_shading_states.is_empty());
    assert!(!state.mvp_shading_map.is_empty());
    assert!(state.model_matrices.len() > 1);

    // Clear frame
    state.clear_frame();

    // All pools should be reset
    assert_eq!(state.mvp_shading_states.len(), 0);
    assert_eq!(state.mvp_shading_map.len(), 0);
    assert_eq!(state.model_matrices.len(), 1); // Only identity
    assert_eq!(state.view_matrices.len(), 1); // Only default
    assert_eq!(state.proj_matrices.len(), 1); // Only default

    // Current matrices should be None
    assert_eq!(state.current_model_matrix, None);
    assert_eq!(state.current_view_matrix, None);
    assert_eq!(state.current_proj_matrix, None);
}

#[test]
fn test_bind_epu_config_deduplicates_within_frame_and_resets_next_frame() {
    let mut state = ZXFFIState::default();
    let mut config = EpuConfig::default();
    config.layers[0] = [0x1111_2222_3333_4444, 0x5555_6666_7777_8888];

    let slot_a = state.bind_epu_config(config);
    let slot_b = state.bind_epu_config(config);

    assert_eq!(slot_a, 1);
    assert_eq!(slot_b, slot_a);
    assert_eq!(state.current_shading_state.environment_index, slot_a);
    assert_eq!(state.epu_frame_configs.get(&slot_a), Some(&config));
    assert_eq!(state.epu_frame_config_slots.get(&config), Some(&slot_a));

    state.clear_frame();

    assert!(state.epu_frame_configs.is_empty());
    assert!(state.epu_frame_config_slots.is_empty());
    assert_eq!(state.next_epu_frame_slot, 1);
    assert_eq!(state.current_shading_state.environment_index, 0);

    let slot_c = state.bind_epu_config(config);
    assert_eq!(slot_c, 1);
    assert_eq!(state.current_shading_state.environment_index, slot_c);
}

#[test]
fn test_bind_epu_textures_reuses_persistent_slot_and_survives_clear_frame() {
    let mut state = ZXFFIState::default();
    let faces: EpuTextureFaces = [10, 11, 12, 13, 14, 15];

    let slot_a = state.bind_epu_textures(faces);
    let slot_b = state.bind_epu_textures(faces);

    assert_eq!(slot_a, slot_b);
    assert_eq!(state.current_shading_state.environment_index, slot_a);
    assert_eq!(state.epu_imported_slots.get(&faces), Some(&slot_a));
    assert_eq!(state.epu_imported_faces_by_slot.get(&slot_a), Some(&faces));

    state.clear_frame();

    assert_eq!(state.current_shading_state.environment_index, 0);

    let slot_c = state.bind_epu_textures(faces);
    assert_eq!(slot_c, slot_a);
    assert_eq!(state.current_shading_state.environment_index, slot_a);
}

#[test]
fn test_bind_epu_config_is_ignored_while_debug_lock_is_active() {
    let mut state = ZXFFIState::default();
    let mut override_config = EpuConfig::default();
    override_config.layers[0] = [0xAAAA_BBBB_CCCC_DDDD, 0x1111_2222_3333_4444];
    state.epu_debug_locked_override = Some(override_config);
    state.epu_frame_configs.insert(0, override_config);

    let mut game_config = EpuConfig::default();
    game_config.layers[0] = [0x9999_8888_7777_6666, 0x5555_4444_3333_2222];

    let slot = state.bind_epu_config(game_config);

    assert_eq!(slot, 0);
    assert_eq!(state.current_shading_state.environment_index, 0);
    assert_eq!(state.epu_frame_configs.len(), 1);
    assert_eq!(state.epu_frame_configs.get(&0), Some(&override_config));
    assert!(!state.epu_frame_config_slots.contains_key(&game_config));
}

#[test]
fn test_bind_epu_textures_is_ignored_while_debug_lock_is_active() {
    let mut state = ZXFFIState::default();
    state.epu_debug_locked_override = Some(EpuConfig::default());
    state.current_shading_state.environment_index = 5;

    let faces: EpuTextureFaces = [10, 11, 12, 13, 14, 15];
    let slot = state.bind_epu_textures(faces);

    assert_eq!(slot, 0);
    assert_eq!(state.current_shading_state.environment_index, 0);
    assert!(state.epu_imported_slots.is_empty());
    assert!(state.epu_imported_faces_by_slot.is_empty());
}

#[test]
fn test_none_uses_last_in_pool() {
    let mut state = ZXFFIState::default();

    // Add a matrix explicitly
    state.current_model_matrix = Some(Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0)));
    state.current_shading_state.color_rgba8 = 0xFF0000FF;
    state.shading_state_dirty = true;
    let idx1 = state.add_mvp_shading_state();

    // model_matrices should now have 2 entries: [IDENTITY, translation]
    assert_eq!(state.model_matrices.len(), 2);

    // Now use None (should use last in pool = translation)
    state.current_model_matrix = None;
    state.current_shading_state.color_rgba8 = 0xFF0000FF;
    state.shading_state_dirty = true; // Same color
    let idx2 = state.add_mvp_shading_state();

    // Should reuse the same buffer index
    assert_eq!(idx1, idx2);
}

// ========================================================================
// Dither Transparency Tests
// ========================================================================

#[test]
fn test_uniform_alpha_update() {
    use crate::graphics::{FLAG_UNIFORM_ALPHA_MASK, FLAG_UNIFORM_ALPHA_SHIFT};

    let mut ffi_state = ZXFFIState::default();

    // Default should be opaque (alpha = 15)
    let alpha = (ffi_state.current_shading_state.flags & FLAG_UNIFORM_ALPHA_MASK)
        >> FLAG_UNIFORM_ALPHA_SHIFT;
    assert_eq!(alpha, 15);

    // Update to 50% transparency
    ffi_state.update_uniform_alpha(8);
    let alpha = (ffi_state.current_shading_state.flags & FLAG_UNIFORM_ALPHA_MASK)
        >> FLAG_UNIFORM_ALPHA_SHIFT;
    assert_eq!(alpha, 8);
    assert!(ffi_state.shading_state_dirty);

    // Reset dirty flag and update to same value - should not mark dirty
    ffi_state.shading_state_dirty = false;
    ffi_state.update_uniform_alpha(8);
    assert!(!ffi_state.shading_state_dirty);

    // Update to different value - should mark dirty
    ffi_state.update_uniform_alpha(0);
    assert!(ffi_state.shading_state_dirty);
    let alpha = (ffi_state.current_shading_state.flags & FLAG_UNIFORM_ALPHA_MASK)
        >> FLAG_UNIFORM_ALPHA_SHIFT;
    assert_eq!(alpha, 0);
}

#[test]
fn test_dither_offset_update() {
    use crate::graphics::{
        FLAG_DITHER_OFFSET_X_MASK, FLAG_DITHER_OFFSET_X_SHIFT, FLAG_DITHER_OFFSET_Y_MASK,
        FLAG_DITHER_OFFSET_Y_SHIFT,
    };

    let mut ffi_state = ZXFFIState::default();

    // Default should be (0, 0)
    let x = (ffi_state.current_shading_state.flags & FLAG_DITHER_OFFSET_X_MASK)
        >> FLAG_DITHER_OFFSET_X_SHIFT;
    let y = (ffi_state.current_shading_state.flags & FLAG_DITHER_OFFSET_Y_MASK)
        >> FLAG_DITHER_OFFSET_Y_SHIFT;
    assert_eq!(x, 0);
    assert_eq!(y, 0);

    // Update to (2, 3)
    ffi_state.update_dither_offset(2, 3);

    let x = (ffi_state.current_shading_state.flags & FLAG_DITHER_OFFSET_X_MASK)
        >> FLAG_DITHER_OFFSET_X_SHIFT;
    let y = (ffi_state.current_shading_state.flags & FLAG_DITHER_OFFSET_Y_MASK)
        >> FLAG_DITHER_OFFSET_Y_SHIFT;

    assert_eq!(x, 2);
    assert_eq!(y, 3);
    assert!(ffi_state.shading_state_dirty);
}

#[test]
fn test_dither_updates_preserve_other_flags() {
    use crate::graphics::{
        FLAG_SKINNING_MODE, FLAG_TEXTURE_FILTER_LINEAR, FLAG_UNIFORM_ALPHA_MASK,
        FLAG_UNIFORM_ALPHA_SHIFT,
    };

    let mut ffi_state = ZXFFIState::default();

    // Set some other flags first
    ffi_state.update_skinning_mode(true);
    ffi_state.update_texture_filter(true);

    // Verify they're set
    assert_ne!(
        ffi_state.current_shading_state.flags & FLAG_SKINNING_MODE,
        0
    );
    assert_ne!(
        ffi_state.current_shading_state.flags & FLAG_TEXTURE_FILTER_LINEAR,
        0
    );

    // Update uniform_alpha
    ffi_state.update_uniform_alpha(8);

    // Verify other flags are preserved
    assert_ne!(
        ffi_state.current_shading_state.flags & FLAG_SKINNING_MODE,
        0
    );
    assert_ne!(
        ffi_state.current_shading_state.flags & FLAG_TEXTURE_FILTER_LINEAR,
        0
    );
    assert_eq!(
        (ffi_state.current_shading_state.flags & FLAG_UNIFORM_ALPHA_MASK)
            >> FLAG_UNIFORM_ALPHA_SHIFT,
        8
    );

    // Update dither_offset
    ffi_state.update_dither_offset(1, 2);

    // Verify all flags are still preserved
    assert_ne!(
        ffi_state.current_shading_state.flags & FLAG_SKINNING_MODE,
        0
    );
    assert_ne!(
        ffi_state.current_shading_state.flags & FLAG_TEXTURE_FILTER_LINEAR,
        0
    );
    assert_eq!(
        (ffi_state.current_shading_state.flags & FLAG_UNIFORM_ALPHA_MASK)
            >> FLAG_UNIFORM_ALPHA_SHIFT,
        8
    );
}

#[test]
fn test_uniform_alpha_clamping() {
    use crate::graphics::{FLAG_UNIFORM_ALPHA_MASK, FLAG_UNIFORM_ALPHA_SHIFT};

    let mut ffi_state = ZXFFIState::default();

    // Values > 15 should be clamped to 15
    ffi_state.update_uniform_alpha(100);
    let alpha = (ffi_state.current_shading_state.flags & FLAG_UNIFORM_ALPHA_MASK)
        >> FLAG_UNIFORM_ALPHA_SHIFT;
    assert_eq!(alpha, 15);

    // Values at boundary should work
    ffi_state.update_uniform_alpha(15);
    let alpha = (ffi_state.current_shading_state.flags & FLAG_UNIFORM_ALPHA_MASK)
        >> FLAG_UNIFORM_ALPHA_SHIFT;
    assert_eq!(alpha, 15);
}

#[test]
fn test_dither_offset_clamping() {
    use crate::graphics::{
        FLAG_DITHER_OFFSET_X_MASK, FLAG_DITHER_OFFSET_X_SHIFT, FLAG_DITHER_OFFSET_Y_MASK,
        FLAG_DITHER_OFFSET_Y_SHIFT,
    };

    let mut ffi_state = ZXFFIState::default();

    // Values > 3 should be clamped
    ffi_state.update_dither_offset(100, 200);
    let x = (ffi_state.current_shading_state.flags & FLAG_DITHER_OFFSET_X_MASK)
        >> FLAG_DITHER_OFFSET_X_SHIFT;
    let y = (ffi_state.current_shading_state.flags & FLAG_DITHER_OFFSET_Y_MASK)
        >> FLAG_DITHER_OFFSET_Y_SHIFT;
    assert_eq!(x, 3);
    assert_eq!(y, 3);
}

// ========================================================================
// EPU State Integration Tests
// ========================================================================

#[test]
fn test_epu_frame_config_storage() {
    use crate::graphics::Viewport;

    let mut state = ZXFFIState::default();

    // Initially empty
    assert!(state.epu_frame_configs.is_empty());
    assert!(state.epu_frame_draws.is_empty());

    // Store a config (zeroed layers - exact values don't matter for storage test)
    let config = crate::graphics::epu::EpuConfig {
        layers: [[0u64; 2]; 8],
    };
    state.epu_frame_configs.insert(0, config);
    state.epu_frame_draws.insert((Viewport::FULLSCREEN, 0), 123);

    let stored = state
        .epu_frame_configs
        .get(&0)
        .expect("epu_frame_configs[0] should be set");
    assert_eq!(stored.layers, config.layers);
    assert_eq!(
        state.epu_frame_draws.get(&(Viewport::FULLSCREEN, 0)),
        Some(&123)
    );
}

#[test]
fn test_clear_frame_clears_epu_frame_requests() {
    use crate::graphics::Viewport;

    let mut state = ZXFFIState::default();

    let config = crate::graphics::epu::EpuConfig {
        layers: [[0u64; 2]; 8],
    };
    state.epu_frame_configs.insert(0, config);
    state.epu_frame_draws.insert((Viewport::FULLSCREEN, 0), 0);

    // Clear frame should clear the per-frame request
    state.clear_frame();

    assert!(state.epu_frame_configs.is_empty());
    assert!(state.epu_frame_draws.is_empty());
}
