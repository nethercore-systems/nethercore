// ============================================================================
// EPU COMPUTE: BLUR PYRAMID GENERATION (Octahedral-Aware)
// Generates synthetic mipmaps by blurring EnvLight0 -> EnvLight1 -> EnvLight2...
// Uses Kawase blur (5-tap) with direction-space sampling to avoid octahedral
// seam artifacts at axis boundaries.
// ============================================================================

const PI: f32 = 3.141592653589793;

struct BlurUniforms {
    active_count: u32,
    map_size: u32,
    blur_offset: f32,
    _pad0: u32,
}

@group(0) @binding(2) var<storage, read> epu_active_env_ids: array<u32>;
@group(0) @binding(4) var epu_in: texture_2d_array<f32>;
@group(0) @binding(5) var epu_out: texture_storage_2d_array<rgba16float, write>;
@group(0) @binding(6) var epu_samp: sampler;
@group(0) @binding(7) var<uniform> epu_blur: BlurUniforms;

// ============================================================================
// OCTAHEDRAL ENCODING/DECODING
// These must match the functions in epu_common.wgsl
// ============================================================================

// Encode unit direction to octahedral [-1, 1]^2 coordinates.
fn oct_encode(dir: vec3f) -> vec2f {
    let n = dir / (abs(dir.x) + abs(dir.y) + abs(dir.z));
    if n.z < 0.0 {
        return (1.0 - abs(n.yx)) * sign(n.xy);
    }
    return n.xy;
}

// Decode octahedral [-1, 1]^2 coordinates to unit direction.
fn oct_decode(oct: vec2f) -> vec3f {
    var n = vec3f(oct.xy, 1.0 - abs(oct.x) - abs(oct.y));
    if n.z < 0.0 {
        n = vec3f((1.0 - abs(n.yx)) * sign(n.xy), n.z);
    }
    return normalize(n);
}

// Convert UV [0,1]^2 to octahedral [-1,1]^2
fn uv_to_oct(uv: vec2f) -> vec2f {
    return uv * 2.0 - 1.0;
}

// Convert octahedral [-1,1]^2 to UV [0,1]^2
fn oct_to_uv(oct: vec2f) -> vec2f {
    return oct * 0.5 + 0.5;
}

// ============================================================================
// OCTAHEDRAL-AWARE KAWASE BLUR
//
// Instead of sampling in UV space (which crosses discontinuous seams at the
// octahedral fold where z < 0), we:
// 1. Convert the center UV to a 3D direction
// 2. Build a tangent frame around that direction
// 3. Apply angular offsets in direction space
// 4. Convert offset directions back to UV for sampling
//
// This ensures blur samples always follow the sphere surface and never
// "jump" across the octahedral seam, eliminating the visible "plus" artifacts.
// ============================================================================

@compute @workgroup_size(8, 8, 1)
fn epu_kawase_blur(@builtin(global_invocation_id) gid: vec3u) {
    let env_slot = gid.z;
    if env_slot >= epu_blur.active_count { return; }

    let env_id = epu_active_env_ids[env_slot];
    let map_size = epu_blur.map_size;
    if gid.x >= map_size || gid.y >= map_size { return; }

    let resolution = vec2f(f32(map_size));
    let uv = (vec2f(gid.xy) + 0.5) / resolution;

    // Convert center UV to direction
    let oct = uv_to_oct(uv);
    let center_dir = oct_decode(oct);

    // Build tangent frame for angular offsets
    // Choose an up vector that isn't parallel to center_dir
    let up = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(center_dir.y) > 0.9);
    let tangent = normalize(cross(up, center_dir));
    let bitangent = cross(center_dir, tangent);

    // Angular offset based on blur kernel size
    // blur_offset is typically 1.0 or 2.0 for Kawase passes
    let angular_offset = epu_blur.blur_offset / resolution.x * PI;

    // Sample center
    var c = textureSampleLevel(epu_in, epu_samp, uv, i32(env_id), 0.0).rgb;

    // Sample 4 corners in direction space (45°, 135°, 225°, 315°)
    // This is equivalent to the Kawase blur's diagonal sampling pattern
    for (var i = 0; i < 4; i++) {
        let angle = f32(i) * 1.5708 + 0.7854;  // PI/2 * i + PI/4
        let cos_a = cos(angle);
        let sin_a = sin(angle);

        // Offset direction on the tangent plane, then normalize to sphere
        let offset_dir = normalize(
            center_dir +
            (cos_a * tangent + sin_a * bitangent) * sin(angular_offset)
        );

        // Convert back to UV space for sampling
        let sample_uv = oct_to_uv(oct_encode(offset_dir));
        c += textureSampleLevel(epu_in, epu_samp, sample_uv, i32(env_id), 0.0).rgb;
    }

    // Average 5 samples (center + 4 corners)
    c /= 5.0;

    textureStore(epu_out, vec2u(gid.xy), i32(env_id), vec4f(c, 1.0));
}
