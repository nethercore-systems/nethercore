//! Tests for buffer management

use super::*;
use crate::graphics::vertex::{FORMAT_COLOR, FORMAT_NORMAL, FORMAT_UV};
use std::borrow::Cow;

#[test]
fn test_retained_mesh_non_indexed() {
    let mesh = RetainedMesh {
        format: FORMAT_UV | FORMAT_NORMAL,
        vertex_count: 36,
        index_count: 0,
        vertex_offset: 1024,
        index_offset: 0,
    };
    assert_eq!(mesh.format, FORMAT_UV | FORMAT_NORMAL);
    assert_eq!(mesh.index_count, 0);
}

#[test]
fn test_retained_mesh_indexed() {
    let mesh = RetainedMesh {
        format: FORMAT_COLOR,
        vertex_count: 8,
        index_count: 36,
        vertex_offset: 0,
        index_offset: 512,
    };
    assert_eq!(mesh.vertex_count, 8);
    assert_eq!(mesh.index_count, 36);
}

/// Test that index data alignment padding works correctly for wgpu COPY_BUFFER_ALIGNMENT
#[test]
fn test_index_data_alignment_padding() {
    // Helper to compute padded index data (same logic as load_mesh_indexed*)
    fn pad_index_data(indices: &[u16]) -> Cow<'_, [u8]> {
        let index_byte_data: &[u8] = bytemuck::cast_slice(indices);
        if index_byte_data.len().is_multiple_of(4) {
            Cow::Borrowed(index_byte_data)
        } else {
            let padded_len = (index_byte_data.len() + 3) & !3;
            let mut padded = index_byte_data.to_vec();
            padded.resize(padded_len, 0);
            Cow::Owned(padded)
        }
    }

    // Even number of indices (e.g., 200) = 400 bytes = already 4-byte aligned
    let even_indices: Vec<u16> = (0..200).collect();
    let padded = pad_index_data(&even_indices);
    assert_eq!(padded.len(), 400);
    assert_eq!(padded.len() % 4, 0);
    assert!(
        matches!(padded, Cow::Borrowed(_)),
        "Should borrow when already aligned"
    );

    // Odd number of indices (e.g., 201) = 402 bytes = needs padding to 404
    let odd_indices: Vec<u16> = (0..201).collect();
    let padded = pad_index_data(&odd_indices);
    assert_eq!(padded.len(), 404);
    assert_eq!(padded.len() % 4, 0);
    assert!(
        matches!(padded, Cow::Owned(_)),
        "Should allocate when padding needed"
    );

    // Edge case: 1 index = 2 bytes = needs padding to 4
    let one_index: Vec<u16> = vec![42];
    let padded = pad_index_data(&one_index);
    assert_eq!(padded.len(), 4);
    assert_eq!(padded.len() % 4, 0);

    // Edge case: 3 indices = 6 bytes = needs padding to 8
    let three_indices: Vec<u16> = vec![1, 2, 3];
    let padded = pad_index_data(&three_indices);
    assert_eq!(padded.len(), 8);
    assert_eq!(padded.len() % 4, 0);

    // Edge case: empty = 0 bytes = already aligned
    let empty: Vec<u16> = vec![];
    let padded = pad_index_data(&empty);
    assert_eq!(padded.len(), 0);
    assert!(matches!(padded, Cow::Borrowed(_)));
}
