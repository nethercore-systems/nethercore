//! Utility functions for GLB construction

use gltf_json as json;

/// Compute bounding box for positions
pub fn compute_bounds(positions: &[[f32; 3]]) -> (Vec<f32>, Vec<f32>) {
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];

    for pos in positions {
        for i in 0..3 {
            min[i] = min[i].min(pos[i]);
            max[i] = max[i].max(pos[i]);
        }
    }

    (min.to_vec(), max.to_vec())
}

/// Align buffer to 4-byte boundary
pub fn align_buffer(buffer: &mut Vec<u8>) {
    while buffer.len() % 4 != 0 {
        buffer.push(0);
    }
}

/// Assemble GLB binary from JSON and buffer data
pub fn assemble_glb(root: &json::Root, buffer_data: &[u8]) -> Vec<u8> {
    let json_string = json::serialize::to_string(root).expect("Failed to serialize GLTF JSON");
    let json_bytes = json_string.as_bytes();

    // Pad JSON to 4-byte alignment
    let json_padding = (4 - (json_bytes.len() % 4)) % 4;
    let json_chunk_length = json_bytes.len() + json_padding;

    // Pad buffer to 4-byte alignment
    let buffer_padding = (4 - (buffer_data.len() % 4)) % 4;
    let buffer_chunk_length = buffer_data.len() + buffer_padding;

    // Total file length
    let total_length = 12 + 8 + json_chunk_length + 8 + buffer_chunk_length;

    let mut glb = Vec::with_capacity(total_length);

    // GLB header
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2u32.to_le_bytes()); // version
    glb.extend_from_slice(&(total_length as u32).to_le_bytes());

    // JSON chunk
    glb.extend_from_slice(&(json_chunk_length as u32).to_le_bytes());
    glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // "JSON"
    glb.extend_from_slice(json_bytes);
    for _ in 0..json_padding {
        glb.push(0x20); // Space for JSON padding
    }

    // Binary chunk
    glb.extend_from_slice(&(buffer_chunk_length as u32).to_le_bytes());
    glb.extend_from_slice(&0x004E4942u32.to_le_bytes()); // "BIN\0"
    glb.extend_from_slice(buffer_data);
    for _ in 0..buffer_padding {
        glb.push(0);
    }

    glb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_bounds_simple() {
        let positions = [[0.0, 0.0, 0.0], [1.0, 2.0, 3.0], [-1.0, -2.0, -3.0]];
        let (min, max) = compute_bounds(&positions);
        assert_eq!(min, vec![-1.0, -2.0, -3.0]);
        assert_eq!(max, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_align_buffer() {
        let mut buffer = vec![1, 2, 3];
        align_buffer(&mut buffer);
        assert_eq!(buffer.len(), 4);
        assert_eq!(buffer, vec![1, 2, 3, 0]);

        let mut buffer2 = vec![1, 2, 3, 4];
        align_buffer(&mut buffer2);
        assert_eq!(buffer2.len(), 4); // Already aligned
    }
}
