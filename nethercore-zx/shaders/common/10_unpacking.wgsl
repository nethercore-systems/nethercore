// ============================================================================
// Data Unpacking Utilities
// ============================================================================

// Unpack u8 from low byte of u32 to f32 [0.0, 1.0]
fn unpack_unorm8_from_u32(packed: u32) -> f32 {
    return f32(packed & 0xFFu) / 255.0;
}

// Unpack RGBA8 from u32 to vec4<f32>
// Format: 0xRRGGBBAA (R in highest byte, A in lowest)
fn unpack_rgba8(packed: u32) -> vec4<f32> {
    let r = f32((packed >> 24u) & 0xFFu) / 255.0;
    let g = f32((packed >> 16u) & 0xFFu) / 255.0;
    let b = f32((packed >> 8u) & 0xFFu) / 255.0;
    let a = f32(packed & 0xFFu) / 255.0;
    return vec4<f32>(r, g, b, a);
}

// Unpack RGB8 from u32 to vec3<f32> (ignore alpha)
fn unpack_rgb8(packed: u32) -> vec3<f32> {
    return unpack_rgba8(packed).rgb;
}

// Decode octahedral encoding to normalized direction
// Uses signed octahedral mapping for uniform precision distribution
fn unpack_octahedral(packed: u32) -> vec3<f32> {
    // Extract i16 components with sign extension
    // NOTE: We use bitcast<i32>() instead of i32() because i32(u32) for values >= 2^31
    // is implementation-defined in WGSL, while bitcast always preserves the bit pattern.
    let u_i16 = bitcast<i32>((packed & 0xFFFFu) << 16u) >> 16;  // Sign-extend low 16 bits
    let v_i16 = bitcast<i32>(packed) >> 16;                      // Arithmetic shift sign-extends

    // Convert snorm16 to float [-1, 1]
    let u = f32(u_i16) / 32767.0;
    let v = f32(v_i16) / 32767.0;

    // Reconstruct 3D direction
    var dir: vec3<f32>;
    dir.x = u;
    dir.y = v;
    dir.z = 1.0 - abs(u) - abs(v);

    // Unfold lower hemisphere (z < 0 case)
    if (dir.z < 0.0) {
        let old_x = dir.x;
        dir.x = (1.0 - abs(dir.y)) * select(-1.0, 1.0, old_x >= 0.0);
        dir.y = (1.0 - abs(old_x)) * select(-1.0, 1.0, dir.y >= 0.0);
    }

    return normalize(dir);
}

// Unpack 16-bit octahedral (2x snorm8) to direction vector
fn unpack_octahedral_u16(packed: u32) -> vec3<f32> {
    // Extract snorm8 components with sign extension
    let u_i8 = bitcast<i32>((packed & 0xFFu) << 24u) >> 24;
    let v_i8 = bitcast<i32>(((packed >> 8u) & 0xFFu) << 24u) >> 24;

    let u = f32(u_i8) / 127.0;
    let v = f32(v_i8) / 127.0;

    // Reconstruct 3D direction (same as unpack_octahedral but lower precision input)
    var dir: vec3<f32>;
    dir.x = u;
    dir.y = v;
    dir.z = 1.0 - abs(u) - abs(v);

    if (dir.z < 0.0) {
        let old_x = dir.x;
        dir.x = (1.0 - abs(dir.y)) * select(-1.0, 1.0, old_x >= 0.0);
        dir.y = (1.0 - abs(old_x)) * select(-1.0, 1.0, dir.y >= 0.0);
    }

    return normalize(dir);
}

// ============================================================================
// Tangent Unpacking and TBN Matrix Construction (Normal Mapping)
// ============================================================================

// Unpack tangent from u32 (octahedral encoding with sign bit for handedness)
// Format: bits 0-15 = octahedral UV (snorm16x2), bit 16 = handedness sign (0=+1, 1=-1)
// Returns (tangent_direction, handedness_sign)
fn unpack_tangent(packed: u32) -> vec4<f32> {
    // Extract handedness from bit 16
    let handedness = select(1.0, -1.0, (packed & 0x10000u) != 0u);

    // Mask out bit 16 to get clean octahedral encoding
    let oct = packed & 0xFFFEFFFFu;

    // Extract i16 components with sign extension (same as unpack_octahedral)
    let u_i16 = bitcast<i32>((oct & 0xFFFFu) << 16u) >> 16;
    let v_i16 = bitcast<i32>(oct) >> 16;

    // Convert snorm16 to float [-1, 1]
    let u = f32(u_i16) / 32767.0;
    let v = f32(v_i16) / 32767.0;

    // Reconstruct 3D direction
    var dir: vec3<f32>;
    dir.x = u;
    dir.y = v;
    dir.z = 1.0 - abs(u) - abs(v);

    // Unfold lower hemisphere (z < 0 case)
    if (dir.z < 0.0) {
        let old_x = dir.x;
        dir.x = (1.0 - abs(dir.y)) * select(-1.0, 1.0, old_x >= 0.0);
        dir.y = (1.0 - abs(old_x)) * select(-1.0, 1.0, dir.y >= 0.0);
    }

    return vec4<f32>(normalize(dir), handedness);
}

// Build TBN (Tangent-Bitangent-Normal) matrix for normal mapping
// tangent: world-space tangent vector
// normal: world-space normal vector
// handedness: sign for bitangent direction (+1 or -1)
// Returns 3x3 matrix where columns are [T, B, N]
fn build_tbn(tangent: vec3<f32>, normal: vec3<f32>, handedness: f32) -> mat3x3<f32> {
    let T = normalize(tangent);
    let N = normalize(normal);
    let B = cross(N, T) * handedness;
    // Column-major: mat3x3 columns are T, B, N
    return mat3x3<f32>(T, B, N);
}

// Sample BC5 normal map and reconstruct world-space normal
// BC5 stores only RG channels; Z is reconstructed: z = sqrt(1 - x² - y²)
// tex: BC5 normal map texture (slot 3)
// uv: texture coordinates
// tbn: tangent-bitangent-normal matrix
// flags: shading flags to check FLAG_SKIP_NORMAL_MAP
//
// Default behavior: sample normal map (when tangent data exists)
// When FLAG_SKIP_NORMAL_MAP is set: return vertex normal instead
fn sample_normal_map(tex: texture_2d<f32>, uv: vec2<f32>, tbn: mat3x3<f32>, flags: u32) -> vec3<f32> {
    // If skip flag is set, return vertex normal (column 2 of TBN = N)
    if ((flags & FLAG_SKIP_NORMAL_MAP) != 0u) {
        return tbn[2];
    }

    // Sample BC5 texture (RG channels contain XY of normal)
    let normal_sample = textureSample(tex, sampler_linear, uv).rg;

    // Convert from [0,1] to [-1,1] range
    let xy = normal_sample * 2.0 - 1.0;

    // Reconstruct Z component (always positive for tangent-space normals)
    let z = sqrt(max(0.0, 1.0 - dot(xy, xy)));

    // Transform from tangent space to world space
    let tangent_normal = vec3<f32>(xy, z);
    return normalize(tbn * tangent_normal);
}

