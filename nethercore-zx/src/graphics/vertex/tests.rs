//! Tests for vertex format functionality

use super::*;

#[test]
fn test_vertex_stride_pos_only() {
    // POS: 3 floats = 12 bytes
    assert_eq!(vertex_stride(0), 12);
}

#[test]
fn test_vertex_stride_pos_uv() {
    // POS + UV: 3 + 2 floats = 20 bytes
    assert_eq!(vertex_stride(FORMAT_UV), 20);
}

#[test]
fn test_vertex_stride_pos_color() {
    // POS + COLOR: 3 + 3 floats = 24 bytes
    assert_eq!(vertex_stride(FORMAT_COLOR), 24);
}

#[test]
fn test_vertex_stride_pos_uv_color() {
    // POS + UV + COLOR: 3 + 2 + 3 floats = 32 bytes
    assert_eq!(vertex_stride(FORMAT_UV | FORMAT_COLOR), 32);
}

#[test]
fn test_vertex_stride_pos_normal() {
    // POS + NORMAL: 3 + 3 floats = 24 bytes
    assert_eq!(vertex_stride(FORMAT_NORMAL), 24);
}

#[test]
fn test_vertex_stride_pos_uv_normal() {
    // POS + UV + NORMAL: 3 + 2 + 3 floats = 32 bytes
    assert_eq!(vertex_stride(FORMAT_UV | FORMAT_NORMAL), 32);
}

#[test]
fn test_vertex_stride_pos_color_normal() {
    // POS + COLOR + NORMAL: 3 + 3 + 3 floats = 36 bytes
    assert_eq!(vertex_stride(FORMAT_COLOR | FORMAT_NORMAL), 36);
}

#[test]
fn test_vertex_stride_pos_uv_color_normal() {
    // POS + UV + COLOR + NORMAL: 3 + 2 + 3 + 3 floats = 44 bytes
    assert_eq!(vertex_stride(FORMAT_UV | FORMAT_COLOR | FORMAT_NORMAL), 44);
}

#[test]
fn test_vertex_stride_pos_skinned() {
    // POS + SKINNED: 3 floats + 4 u8 + 4 floats = 12 + 4 + 16 = 32 bytes
    assert_eq!(vertex_stride(FORMAT_SKINNED), 32);
}

#[test]
fn test_vertex_stride_full() {
    // All flags: POS + UV + COLOR + NORMAL + SKINNED
    // 12 + 8 + 12 + 12 + 20 = 64 bytes
    assert_eq!(vertex_stride(FORMAT_ALL), 64);
}

#[test]
fn test_vertex_format_info_names() {
    assert_eq!(VertexFormatInfo::for_format(0).name, "POS");
    assert_eq!(VertexFormatInfo::for_format(1).name, "POS_UV");
    assert_eq!(VertexFormatInfo::for_format(2).name, "POS_COLOR");
    assert_eq!(VertexFormatInfo::for_format(3).name, "POS_UV_COLOR");
    assert_eq!(VertexFormatInfo::for_format(4).name, "POS_NORMAL");
    assert_eq!(VertexFormatInfo::for_format(5).name, "POS_UV_NORMAL");
    assert_eq!(VertexFormatInfo::for_format(6).name, "POS_COLOR_NORMAL");
    assert_eq!(VertexFormatInfo::for_format(7).name, "POS_UV_COLOR_NORMAL");
    assert_eq!(VertexFormatInfo::for_format(8).name, "POS_SKINNED");
    assert_eq!(
        VertexFormatInfo::for_format(15).name,
        "POS_UV_COLOR_NORMAL_SKINNED"
    );
}

#[test]
fn test_vertex_format_info_flags() {
    let format = VertexFormatInfo::for_format(FORMAT_UV | FORMAT_NORMAL);
    assert!(format.has_uv());
    assert!(!format.has_color());
    assert!(format.has_normal());
    assert!(!format.has_skinned());
}

#[test]
fn test_all_32_vertex_formats() {
    // Verify all 32 formats have valid packed strides
    for i in 0..VERTEX_FORMAT_COUNT {
        let info = VertexFormatInfo::for_format(i as u8);
        assert!(
            info.stride >= 8, // Minimum: position only (f16x4) = 8 bytes
            "Format {} has stride {} < 8",
            i,
            info.stride
        );
        assert!(
            info.stride <= 32, // Maximum: full format with tangent packed = 32 bytes
            "Format {} has stride {} > 32",
            i,
            info.stride
        );
    }
}

#[test]
fn test_tangent_format_strides() {
    // Test tangent format strides (packed)
    // Format 20: POS_NORMAL_TANGENT = 8 + 4 + 4 = 16
    assert_eq!(vertex_stride_packed(FORMAT_NORMAL | FORMAT_TANGENT), 16);
    // Format 21: POS_UV_NORMAL_TANGENT = 8 + 4 + 4 + 4 = 20
    assert_eq!(
        vertex_stride_packed(FORMAT_UV | FORMAT_NORMAL | FORMAT_TANGENT),
        20
    );
    // Format 31: Full with tangent = 8 + 4 + 4 + 4 + 4 + 8 = 32
    assert_eq!(vertex_stride_packed(FORMAT_ALL_WITH_TANGENT), 32);
}

#[test]
fn test_tangent_format_names() {
    assert_eq!(VertexFormatInfo::for_format(20).name, "POS_NORMAL_TANGENT");
    assert_eq!(
        VertexFormatInfo::for_format(21).name,
        "POS_UV_NORMAL_TANGENT"
    );
    assert_eq!(
        VertexFormatInfo::for_format(31).name,
        "POS_UV_COLOR_NORMAL_TANGENT_SKINNED"
    );
}

#[test]
fn test_tangent_requires_normal_validation() {
    // Formats with tangent but without normal should be invalid
    assert!(!VertexFormatInfo::for_format(16).is_valid()); // POS_TANGENT
    assert!(!VertexFormatInfo::for_format(17).is_valid()); // POS_UV_TANGENT
    assert!(!VertexFormatInfo::for_format(24).is_valid()); // POS_TANGENT_SKINNED

    // Formats with tangent AND normal should be valid
    assert!(VertexFormatInfo::for_format(20).is_valid()); // POS_NORMAL_TANGENT
    assert!(VertexFormatInfo::for_format(21).is_valid()); // POS_UV_NORMAL_TANGENT
    assert!(VertexFormatInfo::for_format(31).is_valid()); // Full with tangent
}

#[test]
fn test_vertex_stride_pos_uv_skinned() {
    // POS + UV + SKINNED: 12 + 8 + 20 = 40 bytes
    assert_eq!(vertex_stride(FORMAT_UV | FORMAT_SKINNED), 40);
}

#[test]
fn test_vertex_stride_pos_color_skinned() {
    // POS + COLOR + SKINNED: 12 + 12 + 20 = 44 bytes
    assert_eq!(vertex_stride(FORMAT_COLOR | FORMAT_SKINNED), 44);
}

#[test]
fn test_vertex_stride_pos_normal_skinned() {
    // POS + NORMAL + SKINNED: 12 + 12 + 20 = 44 bytes
    assert_eq!(vertex_stride(FORMAT_NORMAL | FORMAT_SKINNED), 44);
}

#[test]
fn test_vertex_stride_pos_uv_color_skinned() {
    // POS + UV + COLOR + SKINNED: 12 + 8 + 12 + 20 = 52 bytes
    assert_eq!(vertex_stride(FORMAT_UV | FORMAT_COLOR | FORMAT_SKINNED), 52);
}

#[test]
fn test_vertex_stride_pos_uv_normal_skinned() {
    // POS + UV + NORMAL + SKINNED: 12 + 8 + 12 + 20 = 52 bytes
    assert_eq!(
        vertex_stride(FORMAT_UV | FORMAT_NORMAL | FORMAT_SKINNED),
        52
    );
}

#[test]
fn test_vertex_stride_pos_color_normal_skinned() {
    // POS + COLOR + NORMAL + SKINNED: 12 + 12 + 12 + 20 = 56 bytes
    assert_eq!(
        vertex_stride(FORMAT_COLOR | FORMAT_NORMAL | FORMAT_SKINNED),
        56
    );
}

#[test]
fn test_skinned_vertex_format_info() {
    let format = VertexFormatInfo::for_format(FORMAT_SKINNED);
    assert!(!format.has_uv());
    assert!(!format.has_color());
    assert!(!format.has_normal());
    assert!(format.has_skinned());
    assert_eq!(format.name, "POS_SKINNED");
    assert_eq!(format.stride, 16); // Packed: pos(8) + skinned(8) = 16
}

#[test]
fn test_skinned_full_vertex_format_info() {
    let format = VertexFormatInfo::for_format(FORMAT_ALL);
    assert!(format.has_uv());
    assert!(format.has_color());
    assert!(format.has_normal());
    assert!(format.has_skinned());
    assert_eq!(format.name, "POS_UV_COLOR_NORMAL_SKINNED");
    assert_eq!(format.stride, 28); // Packed: pos(8) + uv(4) + color(4) + normal(4) + skinned(8) = 28
}

#[test]
fn test_all_skinned_vertex_format_strides() {
    // Verify all 8 skinned variants have correct strides
    assert_eq!(vertex_stride(FORMAT_SKINNED), 12 + 20);
    assert_eq!(vertex_stride(FORMAT_UV | FORMAT_SKINNED), 20 + 20);
    assert_eq!(vertex_stride(FORMAT_COLOR | FORMAT_SKINNED), 24 + 20);
    assert_eq!(
        vertex_stride(FORMAT_UV | FORMAT_COLOR | FORMAT_SKINNED),
        32 + 20
    );
    assert_eq!(vertex_stride(FORMAT_NORMAL | FORMAT_SKINNED), 24 + 20);
    assert_eq!(
        vertex_stride(FORMAT_UV | FORMAT_NORMAL | FORMAT_SKINNED),
        32 + 20
    );
    assert_eq!(
        vertex_stride(FORMAT_COLOR | FORMAT_NORMAL | FORMAT_SKINNED),
        36 + 20
    );
    assert_eq!(vertex_stride(FORMAT_ALL), 44 + 20);
}

#[test]
fn test_skinned_format_flags_isolation() {
    // Verify FORMAT_SKINNED doesn't interfere with other flags
    for base_format in 0..8u8 {
        let skinned_format = base_format | FORMAT_SKINNED;
        let base_info = VertexFormatInfo::for_format(base_format);
        let skinned_info = VertexFormatInfo::for_format(skinned_format);

        assert_eq!(base_info.has_uv(), skinned_info.has_uv());
        assert_eq!(base_info.has_color(), skinned_info.has_color());
        assert_eq!(base_info.has_normal(), skinned_info.has_normal());

        assert!(!base_info.has_skinned());
        assert!(skinned_info.has_skinned());

        assert_eq!(skinned_info.stride, base_info.stride + 8); // skinned = u8x4 indices + unorm8x4 weights = 8 bytes
    }
}

#[test]
fn test_format_all_includes_skinned() {
    assert_eq!(
        FORMAT_ALL,
        FORMAT_UV | FORMAT_COLOR | FORMAT_NORMAL | FORMAT_SKINNED
    );
    assert_eq!(FORMAT_ALL, 15);
}

#[test]
fn test_vertex_buffer_layout_pos_only() {
    let info = VertexFormatInfo::for_format(0);
    let layout = info.vertex_buffer_layout();
    assert_eq!(layout.array_stride, 8); // f16x4 (8 bytes) - PACKED format
    assert_eq!(layout.attributes.len(), 1);
    assert_eq!(layout.attributes[0].shader_location, 0);
}

#[test]
fn test_vertex_buffer_layout_full() {
    let info = VertexFormatInfo::for_format(FORMAT_ALL);
    let layout = info.vertex_buffer_layout();
    assert_eq!(layout.array_stride, 28); // Packed: pos(8) + uv(4) + color(4) + normal(4) + skinned(8) = 28
    assert_eq!(layout.attributes.len(), 6);
}

#[test]
fn test_vertex_attribute_offsets_pos_uv_color_normal() {
    let info = VertexFormatInfo::for_format(FORMAT_UV | FORMAT_COLOR | FORMAT_NORMAL);
    let layout = info.vertex_buffer_layout();
    // Packed offsets: pos(0-7), uv(8-11), color(12-15), normal(16-19)
    assert_eq!(layout.attributes[0].offset, 0); // Position at 0
    assert_eq!(layout.attributes[1].offset, 8); // UV at 8
    assert_eq!(layout.attributes[2].offset, 12); // Color at 12
    assert_eq!(layout.attributes[3].offset, 16); // Normal at 16
}

#[test]
fn test_vertex_attribute_shader_locations() {
    let info = VertexFormatInfo::for_format(FORMAT_UV | FORMAT_NORMAL);
    let layout = info.vertex_buffer_layout();
    assert_eq!(layout.attributes[0].shader_location, 0);
    assert_eq!(layout.attributes[1].shader_location, 1);
    assert_eq!(layout.attributes[2].shader_location, 3);
}
