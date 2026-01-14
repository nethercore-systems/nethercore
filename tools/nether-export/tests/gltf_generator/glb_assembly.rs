//! GLB file assembly.

use gltf_json as json;

/// Assemble the final GLB binary
pub(crate) fn assemble_glb(root: &json::Root, buffer_data: &[u8]) -> Vec<u8> {
    // Update buffer byte length in root
    let mut root = root.clone();
    root.buffers[0].byte_length = buffer_data.len().into();

    // Serialize JSON
    let json_string = json::serialize::to_string(&root).expect("Failed to serialize JSON");
    let json_bytes = json_string.as_bytes();

    // Pad JSON to 4-byte alignment
    let json_padding = (4 - (json_bytes.len() % 4)) % 4;
    let json_chunk_length = json_bytes.len() + json_padding;

    // Pad buffer to 4-byte alignment
    let buffer_padding = (4 - (buffer_data.len() % 4)) % 4;
    let buffer_chunk_length = buffer_data.len() + buffer_padding;

    // Calculate total length
    let total_length = 12 + 8 + json_chunk_length + 8 + buffer_chunk_length;

    // Build GLB
    let mut glb = Vec::with_capacity(total_length);

    // Header
    glb.extend_from_slice(b"glTF"); // magic
    glb.extend_from_slice(&2u32.to_le_bytes()); // version
    glb.extend_from_slice(&(total_length as u32).to_le_bytes()); // length

    // JSON chunk
    glb.extend_from_slice(&(json_chunk_length as u32).to_le_bytes()); // chunk length
    glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // chunk type "JSON"
    glb.extend_from_slice(json_bytes);
    glb.extend(std::iter::repeat_n(0x20u8, json_padding)); // pad with spaces

    // BIN chunk
    glb.extend_from_slice(&(buffer_chunk_length as u32).to_le_bytes()); // chunk length
    glb.extend_from_slice(&0x004E4942u32.to_le_bytes()); // chunk type "BIN\0"
    glb.extend_from_slice(buffer_data);
    glb.extend(std::iter::repeat_n(0u8, buffer_padding)); // pad with zeros

    glb
}
