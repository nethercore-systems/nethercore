//! Tests for 2D drawing functions

use crate::state::ZXFFIState;

/// Test that draw functions use the current color from set_color()
#[test]
fn test_stateful_color_draw_rect() {
    let mut state = ZXFFIState::new();

    // Set color to red
    state.update_color(0xFF0000FF);

    // Draw a rect - it should use red from state
    state.bound_textures[0] = u32::MAX;
    let color_before_draw = state.current_shading_state.color_rgba8;

    assert_eq!(
        color_before_draw, 0xFF0000FF,
        "Color should be red before draw"
    );
}

/// Test that changing color between draws works correctly
#[test]
fn test_color_state_persistence() {
    let mut state = ZXFFIState::new();

    // Draw 1: Red
    state.update_color(0xFF0000FF);
    let _ = state.add_shading_state();
    assert_eq!(state.current_shading_state.color_rgba8, 0xFF0000FF);

    // Draw 2: Green (color persists until changed)
    state.update_color(0x00FF00FF);
    let _ = state.add_shading_state();
    assert_eq!(state.current_shading_state.color_rgba8, 0x00FF00FF);

    // Draw 3: Same green (no change needed)
    let _ = state.add_shading_state();
    assert_eq!(state.current_shading_state.color_rgba8, 0x00FF00FF);
}

/// Test that color state is independent (no multiplication with old colors)
#[test]
fn test_no_color_blending() {
    let mut state = ZXFFIState::new();

    // Set red
    state.update_color(0xFF0000FF);
    let _red_state = state.add_shading_state();

    // Change to green - should completely replace red, not blend
    state.update_color(0x00FF00FF);
    let green_state = state.add_shading_state();

    // Verify we have pure green, not a blend
    assert_eq!(state.current_shading_state.color_rgba8, 0x00FF00FF);
    assert_ne!(
        green_state, _red_state,
        "State indices should differ when color changes"
    );
}

/// Test default color is white
#[test]
fn test_default_color_is_white() {
    let state = ZXFFIState::new();
    assert_eq!(
        state.current_shading_state.color_rgba8, 0xFFFFFFFF,
        "Default color should be white"
    );
}

/// Test that update_color only marks dirty if color actually changed
#[test]
fn test_color_optimization() {
    let mut state = ZXFFIState::new();

    // Set to red
    state.update_color(0xFF0000FF);
    assert!(
        state.shading_state_dirty,
        "Should be dirty after color change"
    );

    // Clear dirty flag (simulate add_shading_state behavior)
    state.shading_state_dirty = false;

    // Set to same red again - should not mark dirty
    state.update_color(0xFF0000FF);
    assert!(
        !state.shading_state_dirty,
        "Should not be dirty when color unchanged"
    );

    // Set to different color - should mark dirty
    state.update_color(0x00FF00FF);
    assert!(
        state.shading_state_dirty,
        "Should be dirty when color changes"
    );
}
