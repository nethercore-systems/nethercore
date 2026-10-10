//! Tests for 2D drawing functions

use crate::state::ZXFFIState;

/// Exercise the actual WASM imports and the quads consumed by quad_template.wgsl.
#[test]
fn screen_space_geometry_preserves_requested_origins() {
    use crate::{ffi::ZXGameContext, graphics::QuadInstance};
    use glam::{Mat2, Vec2};
    use wasmtime::{Engine, Linker, Module, Store};

    // Local pixel position transformed by the unchanged screen-space shader.
    fn screen(q: &QuadInstance, local: Vec2) -> Vec2 {
        let half = Vec2::from(q.size) * 0.5;
        Vec2::new(q.position[0], q.position[1])
            + half
            + Mat2::from_angle(q.rotation) * (local - half)
    }
    fn near(actual: Vec2, expected: Vec2) {
        assert!(
            (actual - expected).length() < 0.0001,
            "screen point {actual:?}, expected {expected:?}"
        );
    }
    fn quads(store: &Store<ZXGameContext>) -> Vec<QuadInstance> {
        store
            .data()
            .ffi
            .quad_batches()
            .iter()
            .flat_map(|batch| batch.instances.iter().copied())
            .collect()
    }

    let engine = Engine::default();
    let module = Module::new(&engine, r#"(module
      (import "env" "draw_line" (func $line (param f32 f32 f32 f32 f32)))
      (import "env" "draw_circle" (func $circle (param f32 f32 f32)))
      (import "env" "draw_circle_outline" (func $outline (param f32 f32 f32 f32)))
      (import "env" "draw_sprite_ex" (func $sprite (param f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32)))
      (memory (export "memory") 1)
      (export "line" (func $line)) (export "circle" (func $circle))
      (export "outline" (func $outline)) (export "sprite" (func $sprite)))"#).unwrap();
    let mut linker = Linker::new(&engine);
    crate::ffi::register_zx_ffi(&mut linker).unwrap();
    let mut store = Store::new(&engine, ZXGameContext::default());
    let instance = linker.instantiate(&mut store, &module).unwrap();
    let line = instance
        .get_typed_func::<(f32, f32, f32, f32, f32), ()>(&mut store, "line")
        .unwrap();
    let circle = instance
        .get_typed_func::<(f32, f32, f32), ()>(&mut store, "circle")
        .unwrap();
    let outline = instance
        .get_typed_func::<(f32, f32, f32, f32), ()>(&mut store, "outline")
        .unwrap();
    let sprite = instance
        .get_typed_func::<(f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32), ()>(
            &mut store, "sprite",
        )
        .unwrap();
    for (vx, vy) in [(0, 0), (320, 180)] {
        let offset = Vec2::new(vx as f32, vy as f32);
        for (x2, y2) in [(80.0, 35.0), (24.0, 90.0), (80.0, 90.0), (-8.0, -20.0)] {
            store.data_mut().ffi.clear_frame();
            store.data_mut().ffi.current_viewport.x = vx;
            store.data_mut().ffi.current_viewport.y = vy;
            line.call(&mut store, (24.0, 35.0, x2, y2, 4.0)).unwrap();
            let qs = quads(&store);
            assert_eq!(qs.len(), 1);
            near(
                screen(&qs[0], Vec2::new(0.0, 2.0)),
                offset + Vec2::new(24.0, 35.0),
            );
            near(
                screen(&qs[0], Vec2::new(qs[0].size[0], 2.0)),
                offset + Vec2::new(x2, y2),
            );
        }
        store.data_mut().ffi.clear_frame();
        store.data_mut().ffi.current_viewport.x = vx;
        store.data_mut().ffi.current_viewport.y = vy;
        line.call(&mut store, (24.0, 35.0, 24.0, 35.0, 4.0))
            .unwrap();
        circle.call(&mut store, (100.0, 90.0, 0.0)).unwrap();
        outline.call(&mut store, (100.0, 90.0, -1.0, 2.0)).unwrap();
        assert!(quads(&store).is_empty());
        circle.call(&mut store, (100.0, 90.0, 20.0)).unwrap();
        let qs = quads(&store);
        assert_eq!(qs.len(), 16);
        for q in qs {
            near(
                screen(&q, Vec2::new(q.size[0] * 0.5, 0.0)),
                offset + Vec2::new(100.0, 90.0),
            );
        }
        store.data_mut().ffi.clear_frame();
        store.data_mut().ffi.current_viewport.x = vx;
        store.data_mut().ffi.current_viewport.y = vy;
        outline.call(&mut store, (100.0, 90.0, 20.0, 2.0)).unwrap();
        let qs = quads(&store);
        assert_eq!(qs.len(), 16);
        for (i, q) in qs.iter().enumerate() {
            let angle = i as f32 * std::f32::consts::TAU / 16.0;
            near(
                screen(q, Vec2::new(0.0, 1.0)),
                offset + Vec2::new(100.0, 90.0) + Vec2::from_angle(angle) * 20.0,
            );
            near(
                screen(q, Vec2::new(q.size[0], 1.0)),
                screen(&qs[(i + 1) % qs.len()], Vec2::new(0.0, 1.0)),
            );
        }
        for (ox, oy) in [(0.0, 0.0), (20.0, 12.0), (10.0, 7.0), (50.0, -5.0)] {
            for angle in [-90.0, 0.0, 45.0, 135.0] {
                store.data_mut().ffi.clear_frame();
                store.data_mut().ffi.current_viewport.x = vx;
                store.data_mut().ffi.current_viewport.y = vy;
                sprite
                    .call(
                        &mut store,
                        (120.0, 80.0, 40.0, 24.0, 0.0, 0.0, 1.0, 1.0, ox, oy, angle),
                    )
                    .unwrap();
                let qs = quads(&store);
                assert_eq!(qs.len(), 1);
                near(
                    screen(&qs[0], Vec2::new(ox, oy)),
                    offset + Vec2::new(120.0, 80.0),
                );
            }
        }
    }
}

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
