//! Tests for debug registry

#![cfg(test)]

use super::super::types::{Constraints, DebugValue, ValueType};
use super::{DebugRegistry, TreeNode};

#[test]
fn test_registry_basic() {
    let mut registry = DebugRegistry::new();
    assert!(registry.is_empty());

    registry.register("test_value", 0x100, ValueType::F32, None);
    assert_eq!(registry.len(), 1);
    assert_eq!(registry.values[0].name, "test_value");
    assert_eq!(registry.values[0].full_path, "test_value");
}

#[test]
fn test_registry_groups() {
    let mut registry = DebugRegistry::new();

    registry.group_begin("player");
    registry.register("speed", 0x100, ValueType::F32, None);
    registry.register("health", 0x104, ValueType::I32, None);

    registry.group_begin("attacks");
    registry.register("damage", 0x108, ValueType::U8, None);
    registry.group_end();

    registry.group_end();

    assert_eq!(registry.len(), 3);
    assert_eq!(registry.values[0].full_path, "player/speed");
    assert_eq!(registry.values[1].full_path, "player/health");
    assert_eq!(registry.values[2].full_path, "player/attacks/damage");
}

#[test]
fn test_registry_finalize() {
    let mut registry = DebugRegistry::new();
    registry.group_begin("unclosed");
    registry.register("value", 0x100, ValueType::I32, None);

    // Finalize should auto-close groups
    registry.finalize_registration();
    assert!(registry.finalized);
    assert!(registry.group_stack.is_empty());

    // Further registrations should be ignored
    registry.register("ignored", 0x200, ValueType::I32, None);
    assert_eq!(registry.len(), 1);
}

#[test]
fn test_build_tree() {
    let mut registry = DebugRegistry::new();

    registry.group_begin("player");
    registry.register("speed", 0x100, ValueType::F32, None);
    registry.group_begin("attacks");
    registry.register("damage", 0x104, ValueType::U8, None);
    registry.group_end();
    registry.group_end();

    registry.register("global_value", 0x200, ValueType::I32, None);

    let tree = registry.build_tree();
    assert_eq!(tree.len(), 2); // player group + global_value

    // Check structure
    match &tree[0] {
        TreeNode::Group { name, children } => {
            assert_eq!(name, "player");
            assert_eq!(children.len(), 2); // speed + attacks group
        }
        _ => panic!("Expected group"),
    }
    match &tree[1] {
        TreeNode::Value(idx) => {
            assert_eq!(*idx, 2); // global_value is at index 2
        }
        _ => panic!("Expected value"),
    }
}

#[test]
fn test_constraints() {
    let mut registry = DebugRegistry::new();
    let constraints = Some(Constraints::new(0.0, 100.0));
    registry.register("clamped", 0x100, ValueType::F32, constraints);

    assert!(registry.values[0].constraints.is_some());
    let c = registry.values[0].constraints.unwrap();
    assert_eq!(c.min, 0.0);
    assert_eq!(c.max, 100.0);
}

#[test]
fn test_read_write_value_slice() {
    let registry = DebugRegistry::new();

    // Test f32
    let mut data = [0u8; 4];
    registry.write_value_to_slice(&mut data, &DebugValue::F32(3.125));
    let read = registry.read_value_from_slice(&data, ValueType::F32);
    assert_eq!(read, DebugValue::F32(3.125));

    // Test i32
    let mut data = [0u8; 4];
    registry.write_value_to_slice(&mut data, &DebugValue::I32(-12345));
    let read = registry.read_value_from_slice(&data, ValueType::I32);
    assert_eq!(read, DebugValue::I32(-12345));

    // Test Vec2
    let mut data = [0u8; 8];
    registry.write_value_to_slice(&mut data, &DebugValue::Vec2 { x: 1.5, y: 2.5 });
    let read = registry.read_value_from_slice(&data, ValueType::Vec2);
    assert_eq!(read, DebugValue::Vec2 { x: 1.5, y: 2.5 });

    // Test Rect
    let mut data = [0u8; 8];
    registry.write_value_to_slice(
        &mut data,
        &DebugValue::Rect {
            x: 10,
            y: 20,
            w: 30,
            h: 40,
        },
    );
    let read = registry.read_value_from_slice(&data, ValueType::Rect);
    assert_eq!(
        read,
        DebugValue::Rect {
            x: 10,
            y: 20,
            w: 30,
            h: 40
        }
    );

    // Test Color - verify byte layout matches u32 0xRRGGBBAA format
    // A game with `static COLOR: u32 = 0xFF8040FF` (R=255, G=128, B=64, A=255)
    // On little-endian, this is stored as bytes [0xFF, 0x40, 0x80, 0xFF]
    let game_bytes: [u8; 4] = 0xFF8040FFu32.to_le_bytes();
    assert_eq!(
        game_bytes,
        [0xFF, 0x40, 0x80, 0xFF],
        "Sanity check: u32 LE byte order"
    );

    // Reading from game memory should give correct RGBA
    let read = registry.read_value_from_slice(&game_bytes, ValueType::Color);
    assert_eq!(
        read,
        DebugValue::Color(0xFF8040FF),
        "Reading u32 0xFF8040FF should give Color(0xFF8040FF)"
    );

    // Writing should produce bytes that match the u32 format
    let mut data = [0u8; 4];
    registry.write_value_to_slice(&mut data, &DebugValue::Color(0xFF8040FF));
    assert_eq!(
        data, game_bytes,
        "Written bytes should match u32 0xFF8040FF layout"
    );

    // Test with different alpha to catch byte-swap bugs
    // R=255, G=0, B=0, A=1 should produce u32 = 0xFF000001
    let red_low_alpha: [u8; 4] = 0xFF000001u32.to_le_bytes();
    let mut data = [0u8; 4];
    registry.write_value_to_slice(&mut data, &DebugValue::Color(0xFF000001));
    assert_eq!(
        data, red_low_alpha,
        "Color(0xFF000001) should produce bytes for u32 0xFF000001"
    );

    // Verify round-trip
    let read = registry.read_value_from_slice(&data, ValueType::Color);
    assert_eq!(read, DebugValue::Color(0xFF000001));

    // Test Bool
    let mut data = [0u8; 1];
    registry.write_value_to_slice(&mut data, &DebugValue::Bool(true));
    let read = registry.read_value_from_slice(&data, ValueType::Bool);
    assert_eq!(read, DebugValue::Bool(true));
}
