// ============================================================================
// COMMON BINDINGS, STRUCTURES, AND UTILITIES
// Shared across all rendering modes
// ============================================================================

// ============================================================================
// UNIFIED BUFFER ARCHITECTURE (6 bindings, grouped by purpose)
// ============================================================================
// Reduced from 9 storage buffers to 6 storage for WebGPU compatibility.
// All mat4x4 matrices merged into unified_transforms.
// All mat3x4 animation matrices merged into unified_animation.
// CPU pre-computes absolute indices into unified_transforms (no frame_offsets needed).
// Screen dimensions eliminated - resolution_index packed into QuadInstance.mode.
//
// Layout (grouped by purpose):
// - Binding 0-1: Transforms (unified_transforms, mvp_shading_indices)
// - Binding 2: Shading (shading_states)
// - Binding 3: Animation (unified_animation)
// - Binding 4: Environment (environment_states) - Multi-Environment v3
// - Binding 5: Quad rendering (quad_instances)

// Binding 0: unified_transforms - all mat4x4 matrices [models | views | projs]
// Indices are pre-computed on CPU to be absolute offsets into this array
@group(0) @binding(0) var<storage, read> unified_transforms: array<mat4x4<f32>>;

// Binding 1: mvp_shading_indices - absolute indices into unified_transforms + shading_states
// Each entry is 4 × u32: [model_idx, view_idx, proj_idx, shading_idx]
// view_idx and proj_idx are PRE-OFFSET by CPU (already absolute indices)
@group(0) @binding(1) var<storage, read> mvp_shading_indices: array<vec4<u32>>;

// Packed light data (12 bytes) - supports directional and point lights
struct PackedLight {
    data0: u32,  // Directional: octahedral direction (snorm16x2), Point: position XY (f16x2)
    data1: u32,  // RGB8 (bits 31-8) + type (bit 7) + intensity (bits 6-0)
    data2: u32,  // Directional: unused (0), Point: position Z + range (f16x2)
}

// Unified per-draw shading state (80 bytes)
struct PackedUnifiedShadingState {
    color_rgba8: u32,                // Material color (RGBA8 packed)
    uniform_set_0: u32,              // Mode-specific: [b0, b1, b2, rim_intensity]
    uniform_set_1: u32,              // Mode-specific: [b0, b1, b2, rim_power]
    flags: u32,                      // Bit 0: skinning_mode (0=raw, 1=inverse bind)
    lights: array<PackedLight, 4>,   // 48 bytes (4 × 12-byte lights)
    // Animation System v2 fields (8 bytes) + padding (4 bytes) + environment index (4 bytes)
    keyframe_base: u32,              // Base offset into all_keyframes buffer
    inverse_bind_base: u32,          // Base offset into inverse_bind buffer
    _pad: u32,                       // Unused padding for struct alignment
    environment_index: u32,          // Index into environment_states buffer
}

// Binding 2: shading_states - per-draw shading state array
@group(0) @binding(2) var<storage, read> shading_states: array<PackedUnifiedShadingState>;

// 3x4 bone matrix struct (row-major storage, 48 bytes per bone)
// Implicit 4th row is [0, 0, 0, 1] (affine transform)
struct BoneMatrix3x4 {
    row0: vec4<f32>,  // [m00, m01, m02, tx]
    row1: vec4<f32>,  // [m10, m11, m12, ty]
    row2: vec4<f32>,  // [m20, m21, m22, tz]
}

// Binding 3: unified_animation - all mat3x4 animation data [inverse_bind | keyframes | immediate]
// CPU pre-computes keyframe_base to point to the correct section
@group(0) @binding(3) var<storage, read> unified_animation: array<BoneMatrix3x4>;

// ============================================================================
// ENVIRONMENT SYSTEM (Multi-Environment v3)
// ============================================================================
// Procedural environment rendering with layering and blend modes.
// 8 modes: Gradient, Scatter, Lines, Silhouette, Rectangles, Room, Curtains, Rings

// Packed environment state (48 bytes)
// Header: base_mode(3) + overlay_mode(3) + blend_mode(2) + reserved(24)
// Data: base[0..5], overlay[5..10], shared[10]
struct PackedEnvironmentState {
    header: u32,
    data: array<u32, 11>,
}

// Environment mode constants
const ENV_MODE_GRADIENT: u32 = 0u;
const ENV_MODE_SCATTER: u32 = 1u;
const ENV_MODE_LINES: u32 = 2u;
const ENV_MODE_SILHOUETTE: u32 = 3u;
const ENV_MODE_RECTANGLES: u32 = 4u;
const ENV_MODE_ROOM: u32 = 5u;
const ENV_MODE_CURTAINS: u32 = 6u;
const ENV_MODE_RINGS: u32 = 7u;

// Blend mode constants
const ENV_BLEND_ALPHA: u32 = 0u;     // lerp(base, overlay, overlay.a)
const ENV_BLEND_ADD: u32 = 1u;       // base + overlay
const ENV_BLEND_MULTIPLY: u32 = 2u;  // base * overlay
const ENV_BLEND_SCREEN: u32 = 3u;    // 1 - (1-base) * (1-overlay)

// Binding 4: environment_states - per-frame array of PackedEnvironmentState
@group(0) @binding(4) var<storage, read> environment_states: array<PackedEnvironmentState>;

// Binding 5: quad_instances - for GPU-instanced quad rendering (declared in quad_template.wgsl)
// (not declared here - only used by quad shader)

// Helper to expand 3x4 bone matrix → 4x4 for skinning calculations
// Input is row-major, output is column-major (WGSL mat4x4 convention)
fn bone_to_mat4(bone: BoneMatrix3x4) -> mat4x4<f32> {
    return mat4x4<f32>(
        vec4<f32>(bone.row0.x, bone.row1.x, bone.row2.x, 0.0), // column 0
        vec4<f32>(bone.row0.y, bone.row1.y, bone.row2.y, 0.0), // column 1
        vec4<f32>(bone.row0.z, bone.row1.z, bone.row2.z, 0.0), // column 2
        vec4<f32>(bone.row0.w, bone.row1.w, bone.row2.w, 1.0)  // column 3 (translation)
    );
}

// Skinning mode flag constant (bit 0 of flags field)
const FLAG_SKINNING_MODE: u32 = 1u;

// Texture filter mode flag constant (bit 1 of flags field)
// 0 = nearest (pixelated), 1 = linear (smooth)
const FLAG_TEXTURE_FILTER_LINEAR: u32 = 2u;

// Animation System v2: CPU pre-computes absolute keyframe_base offsets
// (no flags needed - shader just reads from unified_animation[keyframe_base + bone_idx])

// ============================================================================
// Material Override Flags (bits 2-7)
// See material-overrides-spec.md for details
// ============================================================================

// Uniform color override (bit 2): 0 = texture/vertex, 1 = uniform color_rgba8
const FLAG_USE_UNIFORM_COLOR: u32 = 4u;

// Uniform metallic override (bit 3): Mode 2 = metallic, Mode 3 = spec_damping
const FLAG_USE_UNIFORM_METALLIC: u32 = 8u;

// Uniform roughness override (bit 4): Mode 2 = roughness, Mode 3 = shininess
const FLAG_USE_UNIFORM_ROUGHNESS: u32 = 16u;

// Uniform emissive override (bit 5): 0 = texture, 1 = uniform intensity
const FLAG_USE_UNIFORM_EMISSIVE: u32 = 32u;

// Uniform specular override (bit 6, Mode 3 only): 0 = texture, 1 = uniform RGB
const FLAG_USE_UNIFORM_SPECULAR: u32 = 64u;

// Matcap reflection override (bit 7, Mode 1 only): 0 = sky, 1 = matcap texture
const FLAG_USE_MATCAP_REFLECTION: u32 = 128u;

// ============================================================================
// Normal Mapping Flag (Bit 16)
// ============================================================================

// Skip normal map flag (opt-out): when set, normal map sampling is disabled
// Default behavior (flag NOT set): sample normal map from slot 3 when tangent data exists
// This is developer-friendly: tangent data → normal mapping works automatically
const FLAG_SKIP_NORMAL_MAP: u32 = 0x10000u;

// Helper function to check if a flag is set
fn has_flag(flags: u32, flag: u32) -> bool {
    return (flags & flag) != 0u;
}

// ============================================================================
// Dither Transparency Constants and Helpers (bits 8-15)
// ============================================================================

// Dither field masks in PackedUnifiedShadingState.flags
const FLAG_UNIFORM_ALPHA_MASK: u32 = 0xF00u;      // Bits 8-11
const FLAG_UNIFORM_ALPHA_SHIFT: u32 = 8u;
const FLAG_DITHER_OFFSET_X_MASK: u32 = 0x3000u;   // Bits 12-13
const FLAG_DITHER_OFFSET_X_SHIFT: u32 = 12u;
const FLAG_DITHER_OFFSET_Y_MASK: u32 = 0xC000u;   // Bits 14-15
const FLAG_DITHER_OFFSET_Y_SHIFT: u32 = 14u;

// 4x4 Bayer matrix - classic Saturn/PS1 dither (16 alpha levels)
const BAYER_4X4: array<f32, 16> = array(
     0.0/16.0,  8.0/16.0,  2.0/16.0, 10.0/16.0,
    12.0/16.0,  4.0/16.0, 14.0/16.0,  6.0/16.0,
     3.0/16.0, 11.0/16.0,  1.0/16.0,  9.0/16.0,
    15.0/16.0,  7.0/16.0, 13.0/16.0,  5.0/16.0,
);

// Extract uniform alpha (0-15 → 0.0-1.0)
fn get_uniform_alpha(flags: u32) -> f32 {
    let level = (flags & FLAG_UNIFORM_ALPHA_MASK) >> FLAG_UNIFORM_ALPHA_SHIFT;
    return f32(level) / 15.0;
}

// Extract dither offset
fn get_dither_offset(flags: u32) -> vec2<u32> {
    let x = (flags & FLAG_DITHER_OFFSET_X_MASK) >> FLAG_DITHER_OFFSET_X_SHIFT;
    let y = (flags & FLAG_DITHER_OFFSET_Y_MASK) >> FLAG_DITHER_OFFSET_Y_SHIFT;
    return vec2<u32>(x, y);
}

// Dither transparency check with two-layer alpha system
// - base_alpha: per-pixel alpha from texture/material (0.0-1.0)
// - effect_alpha: global multiplier from uniform_alpha (0-15 → 0.0-1.0)
// Returns true if the fragment should be discarded
fn should_discard_dither(frag_coord: vec2<f32>, flags: u32, base_alpha: f32) -> bool {
    let effect_alpha = get_uniform_alpha(flags);
    let final_alpha = base_alpha * effect_alpha;

    let offset = get_dither_offset(flags);

    // Apply offset to break pattern alignment for stacked meshes
    let x = (u32(frag_coord.x) + offset.x) % 4u;
    let y = (u32(frag_coord.y) + offset.y) % 4u;

    let threshold = BAYER_4X4[y * 4u + x];
    return final_alpha <= threshold;
}

// Texture bindings (group 1)
@group(1) @binding(0) var slot0: texture_2d<f32>;
@group(1) @binding(1) var slot1: texture_2d<f32>;
@group(1) @binding(2) var slot2: texture_2d<f32>;
@group(1) @binding(3) var slot3: texture_2d<f32>;
@group(1) @binding(4) var sampler_nearest: sampler;
@group(1) @binding(5) var sampler_linear: sampler;

// Sample texture with per-draw filter selection via shading state flags
// This reduces code bloat - all texture sampling goes through this helper
fn sample_filtered(tex: texture_2d<f32>, flags: u32, uv: vec2<f32>) -> vec4<f32> {
    if ((flags & FLAG_TEXTURE_FILTER_LINEAR) != 0u) {
        return textureSample(tex, sampler_linear, uv);
    }
    return textureSample(tex, sampler_nearest, uv);
}

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

// ============================================================================
// Environment Sampling (Multi-Environment v3)
// ============================================================================

// Sample gradient environment (Mode 0)
// data[0]: zenith color (RGBA8)
// data[1]: sky_horizon color (RGBA8)
// data[2]: ground_horizon color (RGBA8)
// data[3]: nadir color (RGBA8)
// data[4]: rotation (f16) + shift (f16) packed
fn sample_gradient(data: array<u32, 11>, offset: u32, direction: vec3<f32>) -> vec4<f32> {
    let zenith = unpack_rgba8(data[offset]);
    let sky_horizon = unpack_rgba8(data[offset + 1u]);
    let ground_horizon = unpack_rgba8(data[offset + 2u]);
    let nadir = unpack_rgba8(data[offset + 3u]);
    let rotation_shift = unpack2x16float(data[offset + 4u]);
    let rotation = rotation_shift.x;
    let shift = rotation_shift.y;

    // Apply Y-axis rotation to direction
    let cos_r = cos(rotation);
    let sin_r = sin(rotation);
    let rotated_dir = vec3<f32>(
        direction.x * cos_r + direction.z * sin_r,
        direction.y,
        -direction.x * sin_r + direction.z * cos_r
    );

    // Calculate blend factor with shift applied
    let y = clamp(rotated_dir.y + shift, -1.0, 1.0);

    // Smooth 4-color gradient: nadir (-1) -> ground_horizon (-0.1) -> sky_horizon (0.1) -> zenith (1)
    // Small horizon band for smooth ground/sky transition
    var result: vec4<f32>;

    if (y < -0.1) {
        // Below horizon: nadir to ground_horizon
        // Map [-1, -0.1] → [0, 1]
        let t = (y + 1.0) / 0.9;
        result = mix(nadir, ground_horizon, t);
    } else if (y < 0.1) {
        // Horizon band: ground_horizon to sky_horizon
        // Map [-0.1, 0.1] → [0, 1]
        let t = (y + 0.1) / 0.2;
        result = mix(ground_horizon, sky_horizon, t);
    } else {
        // Above horizon: sky_horizon to zenith
        // Map [0.1, 1] → [0, 1]
        let t = (y - 0.1) / 0.9;
        result = mix(sky_horizon, zenith, t);
    }

    return result;
}

// ============================================================================
// Hash Functions (for procedural randomness)
// ============================================================================

// Fast integer hash → float in [0,1] (for discrete randomness)
fn hash21(p: vec2<u32>) -> f32 {
    var n = p.x * 1597u + p.y * 2549u;
    n = n ^ (n >> 13u);
    n = n * 1013904223u;
    return f32(n) * (1.0 / 4294967295.0);
}

fn hash11(p: u32) -> f32 {
    var n = p * 1597u;
    n = n ^ (n >> 13u);
    n = n * 1013904223u;
    return f32(n) * (1.0 / 4294967295.0);
}

// Hash vec3 to float
fn hash31(p: vec3<f32>) -> f32 {
    var p3 = fract(p * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// ============================================================================
// Mode 1: Scatter - Cellular noise particle field
// ============================================================================
// data[0]: variant(2) + density(8) + size(8) + glow(8) + streak_len(6)
// data[1]: color_primary RGB (bits 31-8) + parallax_rate (bits 7-0)
// data[2]: color_secondary RGB (bits 31-8) + parallax_size (bits 7-0)
// data[3]: phase(16) + layer_count(2) + reserved(14)
fn sample_scatter(data: array<u32, 11>, offset: u32, direction: vec3<f32>) -> vec4<f32> {
    // Unpack parameters
    let d0 = data[offset];
    let variant = d0 & 0x3u;
    let density = f32((d0 >> 2u) & 0xFFu) / 255.0;
    let size = f32((d0 >> 10u) & 0xFFu) / 255.0 * 0.1 + 0.001;
    let glow = f32((d0 >> 18u) & 0xFFu) / 255.0;
    let streak_len = f32((d0 >> 26u) & 0x3Fu) / 63.0;

    let d1 = data[offset + 1u];
    let color_primary = vec3<f32>(
        f32((d1 >> 24u) & 0xFFu) / 255.0,
        f32((d1 >> 16u) & 0xFFu) / 255.0,
        f32((d1 >> 8u) & 0xFFu) / 255.0
    );

    let d2 = data[offset + 2u];
    let color_secondary = vec3<f32>(
        f32((d2 >> 24u) & 0xFFu) / 255.0,
        f32((d2 >> 16u) & 0xFFu) / 255.0,
        f32((d2 >> 8u) & 0xFFu) / 255.0
    );

    let d3 = data[offset + 3u];
    let phase = f32(d3 & 0xFFFFu) / 65535.0;
    let layer_count = ((d3 >> 16u) & 0x3u) + 1u;

    // Normalize direction
    let dir = normalize(direction);

    // Project direction to spherical coordinates for cell lookup
    let theta = atan2(dir.z, dir.x);
    let phi = asin(clamp(dir.y, -1.0, 1.0));

    // Grid cell size based on density
    let cell_size = mix(0.5, 0.05, density);

    // UV coordinates on sphere
    var uv = vec2<f32>(theta / 6.28318 + 0.5, phi / 3.14159 + 0.5);

    // Apply animation based on variant
    switch (variant) {
        case 1u: { // Vertical (rain)
            uv.y = uv.y + phase;
        }
        case 2u: { // Horizontal (speed lines)
            uv.x = uv.x + phase;
        }
        case 3u: { // Warp (radial)
            // Move outward from center
            let center = vec2<f32>(0.5, 0.5);
            let to_center = uv - center;
            uv = center + to_center * (1.0 + phase * 0.5);
        }
        default: { // Stars (0) - twinkle
            // Twinkle handled below
        }
    }

    // Cell coordinates
    let cell = floor(uv / cell_size);
    let cell_uv = fract(uv / cell_size);

    // Check current and neighboring cells for particles
    var brightness = 0.0;
    var final_color = vec3<f32>(0.0);

    for (var dy: i32 = -1; dy <= 1; dy = dy + 1) {
        for (var dx: i32 = -1; dx <= 1; dx = dx + 1) {
            let neighbor = cell + vec2<f32>(f32(dx), f32(dy));
            let cell_hash = hash21(vec2<u32>(u32(neighbor.x + 1000.0) & 0xFFFFu, u32(neighbor.y + 1000.0) & 0xFFFFu));

            // Only spawn particle if hash is below density threshold
            if (cell_hash > density) {
                continue;
            }

            // Random position within cell
            let px = hash21(vec2<u32>(u32(neighbor.x + 2000.0) & 0xFFFFu, u32(neighbor.y + 2000.0) & 0xFFFFu));
            let py = hash21(vec2<u32>(u32(neighbor.x + 3000.0) & 0xFFFFu, u32(neighbor.y + 3000.0) & 0xFFFFu));
            let particle_pos = neighbor * cell_size + vec2<f32>(px, py) * cell_size;

            // Distance to particle
            var dist: f32;
            if (variant == 1u && streak_len > 0.0) {
                // Vertical streak
                let streak_dist = abs(uv.x - particle_pos.x);
                let y_dist = abs(fract(uv.y - particle_pos.y + 0.5) - 0.5);
                dist = max(streak_dist, y_dist / (1.0 + streak_len * 10.0));
            } else if (variant == 2u && streak_len > 0.0) {
                // Horizontal streak
                let streak_dist = abs(uv.y - particle_pos.y);
                let x_dist = abs(fract(uv.x - particle_pos.x + 0.5) - 0.5);
                dist = max(streak_dist, x_dist / (1.0 + streak_len * 10.0));
            } else {
                dist = length(uv - particle_pos);
            }

            // Particle brightness with size and glow
            let particle_size = size * (0.5 + 0.5 * hash11(u32(neighbor.x + neighbor.y * 1000.0) & 0xFFFFu));
            let particle_brightness = smoothstep(particle_size * (1.0 + glow), 0.0, dist);

            // Twinkle for stars variant
            var twinkle = 1.0;
            if (variant == 0u) {
                let twinkle_hash = hash11(u32(neighbor.x * 7.0 + neighbor.y * 13.0) & 0xFFFFu);
                twinkle = 0.5 + 0.5 * sin(phase * 6.28318 * 4.0 + twinkle_hash * 6.28318);
            }

            // Color variation
            let color_mix = hash11(u32(neighbor.x * 11.0 + neighbor.y * 17.0) & 0xFFFFu);
            let particle_color = mix(color_primary, color_secondary, color_mix);

            brightness = max(brightness, particle_brightness * twinkle);
            final_color = max(final_color, particle_color * particle_brightness * twinkle);
        }
    }

    return vec4<f32>(final_color, brightness);
}

// ============================================================================
// Mode 2: Lines - Infinite grid lines projected onto a plane
// ============================================================================
// data[0]: variant(2) + line_type(2) + thickness(8) + accent_every(8) + reserved(12)
// data[1]: spacing(f16) + fade_distance(f16)
// data[2]: color_primary (RGBA8)
// data[3]: color_accent (RGBA8)
// data[4]: phase(u16) + reserved(16)
fn sample_lines(data: array<u32, 11>, offset: u32, direction: vec3<f32>) -> vec4<f32> {
    // Unpack parameters
    let d0 = data[offset];
    let variant = d0 & 0x3u;           // 0=Floor, 1=Ceiling, 2=Sphere
    let line_type = (d0 >> 2u) & 0x3u; // 0=Horizontal, 1=Vertical, 2=Grid
    let thickness = f32((d0 >> 4u) & 0xFFu) / 255.0 * 0.1 + 0.005;
    let accent_every = max(1u, (d0 >> 12u) & 0xFFu);

    let spacing_fade = unpack2x16float(data[offset + 1u]);
    let spacing = max(0.1, spacing_fade.x);
    let fade_distance = max(1.0, spacing_fade.y);

    let color_primary = unpack_rgba8(data[offset + 2u]);
    let color_accent = unpack_rgba8(data[offset + 3u]);

    let phase = f32(data[offset + 4u] & 0xFFFFu) / 65535.0;

    let dir = normalize(direction);

    // Calculate UV based on variant
    var uv: vec2<f32>;
    var fade: f32 = 1.0;

    switch (variant) {
        case 0u: { // Floor - project onto Y=0 plane
            if (dir.y >= -0.001) {
                return vec4<f32>(0.0); // Looking up, no floor visible
            }
            let t = -1.0 / dir.y; // Intersection distance
            uv = vec2<f32>(dir.x * t, dir.z * t);
            fade = 1.0 - smoothstep(0.0, fade_distance, length(uv));
        }
        case 1u: { // Ceiling - project onto Y=1 plane
            if (dir.y <= 0.001) {
                return vec4<f32>(0.0); // Looking down, no ceiling visible
            }
            let t = 1.0 / dir.y;
            uv = vec2<f32>(dir.x * t, dir.z * t);
            fade = 1.0 - smoothstep(0.0, fade_distance, length(uv));
        }
        default: { // Sphere - use spherical coordinates
            let theta = atan2(dir.z, dir.x);
            let phi = asin(clamp(dir.y, -1.0, 1.0));
            uv = vec2<f32>(theta / 3.14159, phi / 1.5708) * spacing * 5.0;
            fade = 1.0;
        }
    }

    // Apply phase offset (scroll animation)
    uv.y = uv.y + phase * spacing;

    // Calculate grid lines
    var line_intensity = 0.0;
    var is_accent = false;

    // Scale UV by spacing
    let scaled_uv = uv / spacing;

    if (line_type == 0u || line_type == 2u) {
        // Horizontal lines
        let line_y = fract(scaled_uv.y);
        let dist_y = min(line_y, 1.0 - line_y);
        let h_line = smoothstep(thickness, 0.0, dist_y);
        if (h_line > line_intensity) {
            line_intensity = h_line;
            let line_index = u32(floor(scaled_uv.y));
            is_accent = (line_index % accent_every) == 0u;
        }
    }

    if (line_type == 1u || line_type == 2u) {
        // Vertical lines
        let line_x = fract(scaled_uv.x);
        let dist_x = min(line_x, 1.0 - line_x);
        let v_line = smoothstep(thickness, 0.0, dist_x);
        if (v_line > line_intensity) {
            line_intensity = v_line;
            let line_index = u32(floor(scaled_uv.x));
            is_accent = (line_index % accent_every) == 0u;
        }
    }

    // Choose color based on accent
    let line_color = select(color_primary, color_accent, is_accent);

    // Apply fade and return
    let final_intensity = line_intensity * fade;
    return vec4<f32>(line_color.rgb * final_intensity, line_color.a * final_intensity);
}

// ============================================================================
// Mode 3: Silhouette - Layered terrain silhouettes with parallax
// ============================================================================
// data[0]: jaggedness(8) + layer_count(2) + parallax_rate(8) + reserved(14)
// data[1]: color_near (RGBA8)
// data[2]: color_far (RGBA8)
// data[3]: sky_zenith (RGBA8)
// data[4]: sky_horizon (RGBA8)
// data[10]: seed stored in upper 16 bits (base) or lower 16 bits (overlay)

// Looping value noise for smooth terrain generation
fn looping_value_noise(t: f32, period: u32, seed: u32) -> f32 {
    let scaled = t * f32(period);
    let i = u32(floor(scaled)) % period;
    let i_next = (i + 1u) % period;
    let f = fract(scaled);

    let seed_offset = seed * 7919u;
    let a = hash11(i + seed_offset);
    let b = hash11(i_next + seed_offset);
    let smooth_t = f * f * (3.0 - 2.0 * f);  // smoothstep interpolation
    return mix(a, b, smooth_t);
}

fn sample_silhouette(data: array<u32, 11>, offset: u32, direction: vec3<f32>) -> vec4<f32> {
    // Unpack parameters
    let d0 = data[offset];
    let jaggedness = f32(d0 & 0xFFu) / 255.0;
    let layer_count = ((d0 >> 8u) & 0x3u) + 1u;
    let parallax_rate = f32((d0 >> 10u) & 0xFFu) / 255.0;

    let color_near = unpack_rgba8(data[offset + 1u]);
    let color_far = unpack_rgba8(data[offset + 2u]);
    let sky_zenith = unpack_rgba8(data[offset + 3u]);
    let sky_horizon = unpack_rgba8(data[offset + 4u]);

    // Extract seed from shared data[10]
    var seed: u32;
    if (offset == 0u) {
        seed = (data[10] >> 16u) & 0xFFFFu;
    } else {
        seed = data[10] & 0xFFFFu;
    }

    let dir = normalize(direction);

    // Only render silhouettes below horizon
    if (dir.y > 0.3) {
        // Sky gradient above silhouettes
        let sky_t = clamp((dir.y - 0.3) / 0.7, 0.0, 1.0);
        return mix(sky_horizon, sky_zenith, sky_t);
    }

    // Calculate horizontal angle (0-1 wrapping)
    let theta = atan2(dir.z, dir.x);
    let angle = (theta / 6.28318) + 0.5;  // 0 to 1

    // Noise period based on jaggedness (more peaks = more jagged)
    let base_period = u32(mix(4.0, 32.0, jaggedness));

    // Check each layer from back to front
    var result_color = mix(sky_horizon, sky_zenith, clamp(dir.y + 0.3, 0.0, 1.0));

    for (var layer: u32 = 0u; layer < layer_count; layer = layer + 1u) {
        let layer_f = f32(layer) / f32(max(1u, layer_count - 1u));

        // Layer depth affects base height and parallax offset
        let layer_depth = 1.0 - layer_f;  // 1.0 = far, 0.0 = near
        let base_height = -0.1 - layer_depth * 0.3 * parallax_rate;

        // Generate terrain height using looping noise
        let layer_seed = seed + layer * 12345u;
        let noise_period = base_period + layer * 4u;

        // Multi-octave noise for more interesting terrain
        var terrain_height = 0.0;
        terrain_height += looping_value_noise(angle, noise_period, layer_seed) * 0.5;
        terrain_height += looping_value_noise(angle * 2.0, noise_period * 2u, layer_seed + 1000u) * 0.25;
        terrain_height += looping_value_noise(angle * 4.0, noise_period * 4u, layer_seed + 2000u) * 0.125;

        // Scale height based on jaggedness
        let height_scale = mix(0.1, 0.4, jaggedness);
        let silhouette_height = base_height + terrain_height * height_scale;

        // Check if direction is below silhouette line
        if (dir.y < silhouette_height) {
            // Interpolate color based on layer depth
            let layer_color = mix(color_near, color_far, layer_depth);
            result_color = layer_color;
        }
    }

    return result_color;
}

// ============================================================================
// Mode 4: Rectangles - Rectangular light sources (windows, screens, panels)
// ============================================================================
// data[0]: variant(2) + density(8) + lit_ratio(8) + size_min(6) + size_max(6) + aspect(2)
// data[1]: color_primary (RGBA8)
// data[2]: color_variation (RGBA8)
// data[3]: parallax_rate(8) + reserved(8) + phase(16)
fn sample_rectangles(data: array<u32, 11>, offset: u32, direction: vec3<f32>) -> vec4<f32> {
    // Unpack parameters
    let d0 = data[offset];
    let variant = d0 & 0x3u;
    let density = f32((d0 >> 2u) & 0xFFu) / 255.0;
    let lit_ratio = f32((d0 >> 10u) & 0xFFu) / 255.0;
    let size_min = f32((d0 >> 18u) & 0x3Fu) / 63.0 * 0.1 + 0.01;
    let size_max = f32((d0 >> 24u) & 0x3Fu) / 63.0 * 0.2 + 0.02;
    let aspect = f32((d0 >> 30u) & 0x3u);

    let color_primary = unpack_rgba8(data[offset + 1u]);
    let color_variation = unpack_rgba8(data[offset + 2u]);

    let d3 = data[offset + 3u];
    let phase = f32((d3 >> 16u) & 0xFFFFu) / 65535.0;

    let dir = normalize(direction);

    // Project to spherical coordinates
    let theta = atan2(dir.z, dir.x);
    let phi = asin(clamp(dir.y, -1.0, 1.0));
    var uv = vec2<f32>((theta / 6.28318) + 0.5, (phi / 3.14159) + 0.5);

    // Apply variant-specific UV modifications
    // Variant 0: Scatter - random placement (default)
    // Variant 1: Buildings - only in lower half, grid-aligned columns
    // Variant 2: Bands - horizontal bands with uniform spacing
    // Variant 3: Panels - uniform grid, centered rectangles

    var effective_density = density;
    var cell_size = mix(0.2, 0.03, density);
    var force_grid_center = false;

    switch (variant) {
        case 1u: { // Buildings - lower half only, vertical columns
            if (uv.y > 0.55) {
                return vec4<f32>(0.0);  // Nothing above horizon
            }
            cell_size = mix(0.15, 0.04, density);  // Taller cells
            effective_density = density * 1.2;  // More dense
        }
        case 2u: { // Bands - horizontal stripes
            cell_size = vec2<f32>(mix(0.3, 0.05, density), mix(0.08, 0.02, density)).y;
            // Modulate density by horizontal bands
            let band = floor(uv.y / 0.1);
            if (u32(band) % 2u == 0u) {
                effective_density = density * 0.3;
            }
        }
        case 3u: { // Panels - uniform grid
            cell_size = mix(0.12, 0.04, density);
            force_grid_center = true;  // Center rectangles in cells
            effective_density = 1.0;  // All cells have rectangles
        }
        default: { // Scatter
            // Keep defaults
        }
    }

    let cell = floor(uv / cell_size);
    let cell_uv = fract(uv / cell_size);

    // Hash for this cell
    let cell_hash = hash21(vec2<u32>(u32(cell.x + 500.0) & 0xFFFFu, u32(cell.y + 500.0) & 0xFFFFu));

    // Only create rectangle if hash below density
    if (cell_hash > effective_density) {
        return vec4<f32>(0.0);
    }

    // Check if lit (using phase for flicker)
    let lit_hash = hash21(vec2<u32>(u32(cell.x * 7.0 + 100.0) & 0xFFFFu, u32(cell.y * 11.0 + 100.0) & 0xFFFFu));
    let flicker = sin(phase * 6.28318 * 8.0 + lit_hash * 6.28318) * 0.5 + 0.5;
    let is_lit = lit_hash < lit_ratio * (0.5 + 0.5 * flicker);

    if (!is_lit) {
        return vec4<f32>(0.0);
    }

    // Rectangle size and position within cell
    let size_hash = hash21(vec2<u32>(u32(cell.x * 13.0 + 200.0) & 0xFFFFu, u32(cell.y * 17.0 + 200.0) & 0xFFFFu));
    var rect_size = mix(size_min, size_max, size_hash) / cell_size;

    // Panels variant uses uniform size
    if (variant == 3u) {
        rect_size = size_max / cell_size * 0.7;
    }

    // Aspect ratio - buildings are taller
    var aspect_mult = 1.0 + aspect * 0.5;
    if (variant == 1u) {
        aspect_mult = 2.0 + aspect;  // Tall windows for buildings
    }
    let rect_w = rect_size;
    let rect_h = rect_size * aspect_mult;

    // Center position
    var rect_center: vec2<f32>;
    if (force_grid_center) {
        rect_center = vec2<f32>(0.5, 0.5);  // Centered in cell
    } else {
        let pos_hash_x = hash21(vec2<u32>(u32(cell.x * 19.0 + 300.0) & 0xFFFFu, u32(cell.y * 23.0 + 300.0) & 0xFFFFu));
        let pos_hash_y = hash21(vec2<u32>(u32(cell.x * 29.0 + 400.0) & 0xFFFFu, u32(cell.y * 31.0 + 400.0) & 0xFFFFu));
        rect_center = vec2<f32>(
            0.5 + (pos_hash_x - 0.5) * (1.0 - rect_w),
            0.5 + (pos_hash_y - 0.5) * (1.0 - rect_h)
        );
    }

    // Check if inside rectangle
    let dist_x = abs(cell_uv.x - rect_center.x);
    let dist_y = abs(cell_uv.y - rect_center.y);

    if (dist_x < rect_w * 0.5 && dist_y < rect_h * 0.5) {
        // Color variation
        let color_hash = hash21(vec2<u32>(u32(cell.x * 37.0) & 0xFFFFu, u32(cell.y * 41.0) & 0xFFFFu));
        let rect_color = mix(color_primary, color_variation, color_hash);

        // Slight edge glow
        let edge_x = smoothstep(0.0, rect_w * 0.1, rect_w * 0.5 - dist_x);
        let edge_y = smoothstep(0.0, rect_h * 0.1, rect_h * 0.5 - dist_y);
        let brightness = edge_x * edge_y;

        return vec4<f32>(rect_color.rgb * brightness, rect_color.a * brightness);
    }

    return vec4<f32>(0.0);
}

// ============================================================================
// Mode 5: Room - Interior of a 3D box with directional lighting
// ============================================================================
// data[0]: color_ceiling_RGB(24) + viewer_x_snorm8(8)
// data[1]: color_floor_RGB(24) + viewer_y_snorm8(8)
// data[2]: color_walls_RGB(24) + viewer_z_snorm8(8)
// data[3]: panel_size(f16) + panel_gap(8) + corner_darken(8)
// data[4]: light_dir_oct(16) + light_intensity(8) + room_scale(8)
// Note: Does NOT use data[10] - can safely layer with other modes
fn sample_room(data: array<u32, 11>, offset: u32, direction: vec3<f32>) -> vec4<f32> {
    // Unpack colors (RGB only, alpha byte contains viewer position)
    let color_ceiling = vec4<f32>(unpack_rgb8(data[offset]), 1.0);
    let color_floor = vec4<f32>(unpack_rgb8(data[offset + 1u]), 1.0);
    let color_walls = vec4<f32>(unpack_rgb8(data[offset + 2u]), 1.0);

    // Unpack viewer position from color alpha bytes (snorm8: -128..127 -> -1.0..1.0)
    let viewer_x = f32(bitcast<i32>((data[offset] & 0xFFu) << 24u) >> 24) / 127.0;
    let viewer_y = f32(bitcast<i32>((data[offset + 1u] & 0xFFu) << 24u) >> 24) / 127.0;
    let viewer_z = f32(bitcast<i32>((data[offset + 2u] & 0xFFu) << 24u) >> 24) / 127.0;
    let viewer = vec3<f32>(viewer_x, viewer_y, viewer_z);

    let d3 = data[offset + 3u];
    let panel_size = unpack2x16float(d3).x;
    let panel_gap = f32((d3 >> 16u) & 0xFFu) / 255.0;
    let corner_darken = f32((d3 >> 24u) & 0xFFu) / 255.0;

    let d4 = data[offset + 4u];
    let light_intensity = f32((d4 >> 16u) & 0xFFu) / 255.0;
    let room_scale = f32((d4 >> 24u) & 0xFFu) / 10.0 + 0.1;

    let dir = normalize(direction);

    // Ray-box intersection to find which surface we hit
    // Box is centered at origin, half-extents = room_scale
    let inv_dir = 1.0 / dir;
    let t_min = (-room_scale - viewer) * inv_dir;
    let t_max = (room_scale - viewer) * inv_dir;

    let t1 = min(t_min, t_max);
    let t2 = max(t_min, t_max);

    let t_near = max(max(t1.x, t1.y), t1.z);
    let t_far = min(min(t2.x, t2.y), t2.z);

    if (t_far < 0.0) {
        return vec4<f32>(0.0);
    }

    let t = select(t_far, t_near, t_near > 0.0);
    let hit_point = viewer + dir * t;

    // Determine which face we hit
    let abs_hit = abs(hit_point);
    var surface_color: vec4<f32>;
    var normal: vec3<f32>;
    var uv: vec2<f32>;

    if (abs_hit.y > abs_hit.x && abs_hit.y > abs_hit.z) {
        // Floor or ceiling
        if (hit_point.y < 0.0) {
            surface_color = color_floor;
            normal = vec3<f32>(0.0, 1.0, 0.0);
        } else {
            surface_color = color_ceiling;
            normal = vec3<f32>(0.0, -1.0, 0.0);
        }
        uv = hit_point.xz / room_scale;
    } else if (abs_hit.x > abs_hit.z) {
        // Left or right wall
        surface_color = color_walls;
        normal = vec3<f32>(select(1.0, -1.0, hit_point.x > 0.0), 0.0, 0.0);
        uv = hit_point.yz / room_scale;
    } else {
        // Front or back wall
        surface_color = color_walls;
        normal = vec3<f32>(0.0, 0.0, select(1.0, -1.0, hit_point.z > 0.0));
        uv = hit_point.xy / room_scale;
    }

    // Panel pattern
    if (panel_size > 0.01) {
        let panel_uv = fract(uv / panel_size);
        let panel_edge = step(panel_gap, panel_uv.x) * step(panel_gap, panel_uv.y) *
                         step(panel_uv.x, 1.0 - panel_gap) * step(panel_uv.y, 1.0 - panel_gap);
        surface_color = surface_color * (0.8 + 0.2 * panel_edge);
    }

    // Corner darkening (distance from center of each face)
    let corner_dist = length(uv) / 1.414;  // Normalize by diagonal
    let corner_factor = 1.0 - corner_darken * corner_dist * corner_dist;

    // Simple directional lighting
    let light_dir = vec3<f32>(0.3, -0.8, 0.5);  // Default light direction
    let n_dot_l = max(dot(normal, -normalize(light_dir)), 0.0);
    let lighting = 0.3 + 0.7 * n_dot_l * light_intensity;

    return vec4<f32>(surface_color.rgb * lighting * corner_factor, surface_color.a);
}

// ============================================================================
// Mode 6: Curtains - Vertical structures (pillars, trees) around viewer
// ============================================================================
// data[0]: layer_count(2) + density(8) + height_min(6) + height_max(6) + width(5) + spacing(5)
// data[1]: waviness(8) + glow(8) + parallax_rate(8) + reserved(8)
// data[2]: color_near (RGBA8)
// data[3]: color_far (RGBA8)
// data[4]: phase(u16) + reserved(16)
fn sample_curtains(data: array<u32, 11>, offset: u32, direction: vec3<f32>) -> vec4<f32> {
    // Unpack parameters
    let d0 = data[offset];
    let layer_count = (d0 & 0x3u) + 1u;
    let density = f32((d0 >> 2u) & 0xFFu) / 255.0;
    let height_min = f32((d0 >> 10u) & 0x3Fu) / 63.0;
    let height_max = f32((d0 >> 16u) & 0x3Fu) / 63.0;
    let width = f32((d0 >> 22u) & 0x1Fu) / 31.0 * 0.1 + 0.01;
    let spacing = f32((d0 >> 27u) & 0x1Fu) / 31.0 * 0.2 + 0.05;

    let d1 = data[offset + 1u];
    let waviness = f32(d1 & 0xFFu) / 255.0;
    let glow = f32((d1 >> 8u) & 0xFFu) / 255.0;
    let parallax_rate = f32((d1 >> 16u) & 0xFFu) / 255.0;

    let color_near = unpack_rgba8(data[offset + 2u]);
    let color_far = unpack_rgba8(data[offset + 3u]);

    let phase = f32(data[offset + 4u] & 0xFFFFu) / 65535.0;

    let dir = normalize(direction);

    // Horizontal angle for curtain placement
    let theta = atan2(dir.z, dir.x);
    let angle = (theta / 6.28318) + 0.5 + phase;  // 0 to 1, with scroll

    var result = vec4<f32>(0.0);

    // Check each layer from back to front
    for (var layer: u32 = 0u; layer < layer_count; layer = layer + 1u) {
        let layer_f = f32(layer) / f32(max(1u, layer_count - 1u));
        let layer_depth = 1.0 - layer_f;  // 1.0 = far, 0.0 = near

        // Adjust cell size based on layer (farther = smaller apparent spacing)
        let layer_spacing = spacing * (1.0 + layer_depth * parallax_rate);
        let cell = floor(angle / layer_spacing);
        let cell_fract = fract(angle / layer_spacing);

        // Hash for this curtain
        let cell_hash = hash21(vec2<u32>(u32(cell + 1000.0 + f32(layer) * 100.0) & 0xFFFFu, layer));

        // Only spawn curtain if hash below density
        if (cell_hash > density) {
            continue;
        }

        // Curtain position within cell
        let pos_hash = hash21(vec2<u32>(u32(cell * 7.0 + 200.0) & 0xFFFFu, layer + 1u));
        let curtain_pos = pos_hash * (1.0 - width / layer_spacing);

        // Height of this curtain
        let height_hash = hash21(vec2<u32>(u32(cell * 11.0 + 300.0) & 0xFFFFu, layer + 2u));
        let curtain_height = mix(height_min, height_max, height_hash);

        // Vertical position check
        let curtain_base = -0.3 - layer_depth * 0.2;
        let curtain_top = curtain_base + curtain_height;

        // Apply waviness - offset curtain position based on vertical position
        var wave_offset = 0.0;
        if (waviness > 0.0) {
            let wave_hash = hash21(vec2<u32>(u32(cell * 13.0) & 0xFFFFu, layer + 3u));
            // Multiple frequencies for organic look
            wave_offset = sin((dir.y * 8.0 + wave_hash * 6.28318) + phase * 6.28318 * 2.0) * waviness * 0.03
                        + sin((dir.y * 15.0 + wave_hash * 3.14159) + phase * 6.28318 * 3.0) * waviness * 0.015;
        }

        // Distance to curtain center (with waviness applied)
        let dist_to_curtain = abs(cell_fract - curtain_pos - wave_offset);
        let curtain_width = width / layer_spacing;

        if (dist_to_curtain < curtain_width * 0.5) {
            if (dir.y >= curtain_base && dir.y <= curtain_top) {
                // Color interpolation by depth
                let curtain_color = mix(color_near, color_far, layer_depth);

                // Edge softness
                let edge_dist = dist_to_curtain / (curtain_width * 0.5);
                let alpha = (1.0 - edge_dist) * curtain_color.a;

                // Glow effect
                let glow_factor = 1.0 + glow * (1.0 - edge_dist);

                // Take frontmost visible curtain
                if (alpha > result.a) {
                    result = vec4<f32>(curtain_color.rgb * glow_factor, alpha);
                }
            }
        }
    }

    return result;
}

// ============================================================================
// Mode 7: Rings - Concentric rings around focal direction
// ============================================================================
// data[0]: ring_count(8) + thickness(8) + center_falloff(8) + reserved(8)
// data[1]: color_a (RGBA8)
// data[2]: color_b (RGBA8)
// data[3]: center_color (RGBA8)
// data[4]: spiral_twist(f16, bits 0-15) + axis_oct16(16, bits 16-31)
// data[10]: phase stored in upper 16 bits (base) or lower 16 bits (overlay)

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

fn sample_rings(data: array<u32, 11>, offset: u32, direction: vec3<f32>) -> vec4<f32> {
    // Unpack parameters
    let d0 = data[offset];
    let ring_count = max(1u, d0 & 0xFFu);
    let thickness = f32((d0 >> 8u) & 0xFFu) / 255.0;
    let center_falloff = f32((d0 >> 16u) & 0xFFu) / 255.0;

    let color_a = unpack_rgba8(data[offset + 1u]);
    let color_b = unpack_rgba8(data[offset + 2u]);
    let center_color = unpack_rgba8(data[offset + 3u]);

    let d4 = data[offset + 4u];
    let spiral_twist = unpack2x16float(d4).x; // degrees

    // Extract phase from shared data[10]
    var phase: f32;
    if (offset == 0u) {
        phase = f32((data[10] >> 16u) & 0xFFFFu) / 65535.0;
    } else {
        phase = f32(data[10] & 0xFFFFu) / 65535.0;
    }

    // Unpack axis from 16-bit octahedral (2x snorm8) in upper 16 bits of d4
    let axis_oct16 = (d4 >> 16u) & 0xFFFFu;
    let axis = unpack_octahedral_u16(axis_oct16);

    let dir = normalize(direction);

    // Calculate angle from axis
    let dot_axis = dot(dir, axis);
    let angle_from_axis = acos(clamp(dot_axis, -1.0, 1.0));

    // Normalized distance from center (0 at axis, 1 at perpendicular)
    let dist = angle_from_axis / 3.14159;

    // Apply spiral twist
    var ring_pos = dist * f32(ring_count);
    if (spiral_twist != 0.0) {
        // Calculate angular position around axis
        let perp = dir - axis * dot_axis;
        let perp_len = length(perp);
        if (perp_len > 0.001) {
            let perp_norm = perp / perp_len;
            let angle = atan2(
                dot(perp_norm, vec3<f32>(1.0, 0.0, 0.0)),
                dot(perp_norm, vec3<f32>(0.0, 1.0, 0.0))
            );
            ring_pos = ring_pos + angle * spiral_twist / 360.0;
        }
    }

    // Apply phase animation (rotation)
    ring_pos = ring_pos + phase * f32(ring_count);

    // Calculate ring pattern
    let ring_fract = fract(ring_pos);
    let ring_index = u32(floor(ring_pos));

    // Alternating colors
    let is_color_a = (ring_index % 2u) == 0u;
    let ring_color = select(color_b, color_a, is_color_a);

    // Ring edge softness based on thickness
    let edge_soft = (1.0 - thickness) * 0.5;
    let ring_intensity = smoothstep(0.0, edge_soft, ring_fract) * smoothstep(1.0, 1.0 - edge_soft, ring_fract);

    // Center glow
    let center_intensity = pow(1.0 - dist, 1.0 / max(0.01, center_falloff));

    // Blend ring color with center color
    let final_color = mix(ring_color.rgb, center_color.rgb, center_intensity * center_color.a);
    let final_alpha = max(ring_color.a * ring_intensity, center_color.a * center_intensity);

    return vec4<f32>(final_color, final_alpha);
}

// Sample a single environment mode
fn sample_mode(mode: u32, data: array<u32, 11>, offset: u32, direction: vec3<f32>) -> vec4<f32> {
    switch (mode) {
        case 0u: { return sample_gradient(data, offset, direction); }
        case 1u: { return sample_scatter(data, offset, direction); }
        case 2u: { return sample_lines(data, offset, direction); }
        case 3u: { return sample_silhouette(data, offset, direction); }
        case 4u: { return sample_rectangles(data, offset, direction); }
        case 5u: { return sample_room(data, offset, direction); }
        case 6u: { return sample_curtains(data, offset, direction); }
        case 7u: { return sample_rings(data, offset, direction); }
        default: { return sample_gradient(data, offset, direction); }
    }
}

// Blend two layers together
fn blend_layers(base: vec4<f32>, overlay: vec4<f32>, mode: u32) -> vec4<f32> {
    switch (mode) {
        case 0u: { return mix(base, overlay, overlay.a); }  // Alpha blend
        case 1u: { return base + overlay; }                  // Add
        case 2u: { return base * overlay; }                  // Multiply
        case 3u: {
            // Screen: 1 - (1-base) * (1-overlay)
            return vec4<f32>(1.0) - (vec4<f32>(1.0) - base) * (vec4<f32>(1.0) - overlay);
        }
        default: { return base; }
    }
}

// Sample complete environment (base + overlay with blend)
fn sample_environment(env_index: u32, direction: vec3<f32>) -> vec4<f32> {
    let env = environment_states[env_index];
    let base_mode = env.header & 0x7u;
    let overlay_mode = (env.header >> 3u) & 0x7u;
    let blend_mode = (env.header >> 6u) & 0x3u;

    let base_color = sample_mode(base_mode, env.data, 0u, direction);

    // If same mode for both layers, skip overlay
    if (overlay_mode == base_mode) {
        return base_color;
    }

    let overlay_color = sample_mode(overlay_mode, env.data, 5u, direction);
    return blend_layers(base_color, overlay_color, blend_mode);
}

// Sample environment ambient (average of zenith and horizon)
// Used for material lighting when sky data is needed
fn sample_environment_ambient(env_index: u32, direction: vec3<f32>) -> vec3<f32> {
    let env_color = sample_environment(env_index, direction);
    return env_color.rgb;
}

// Lambert diffuse lighting (unified for all modes)
// light_dir convention: direction rays travel (negated to get L in lighting calculations)
fn lambert_diffuse(
    normal: vec3<f32>,
    light_dir: vec3<f32>,
    albedo: vec3<f32>,
    light_color: vec3<f32>,
) -> vec3<f32> {
    // Re-normalize after interpolation (interpolated normals have length < 1.0)
    let N = normalize(normal);
    let L = -light_dir;
    let n_dot_l = max(dot(N, L), 0.0);
    return albedo * light_color * n_dot_l;
}

// Smooth distance attenuation for point lights
// Returns 1.0 at distance=0, 0.0 at distance>=range
fn point_light_attenuation(distance: f32, range: f32) -> f32 {
    if (range <= 0.0) {
        return 0.0;
    }
    let t = clamp(distance / range, 0.0, 1.0);
    // Smooth falloff: quadratic ease-out (feels more natural than linear)
    let inv_t = 1.0 - t;
    return inv_t * inv_t;
}

// ============================================================================
// Light Utilities
// ============================================================================

// Unpacked light data (extended for point lights)
struct LightData {
    direction: vec3<f32>,    // For directional lights
    position: vec3<f32>,     // For point lights
    color: vec3<f32>,
    intensity: f32,
    range: f32,              // For point lights
    light_type: u32,         // 0 = directional, 1 = point
    enabled: bool,
}

// Unpack PackedLight to usable values
// Supports both directional and point lights
fn unpack_light(packed: PackedLight) -> LightData {
    var light: LightData;

    // Extract type from bit 7 of data1
    light.light_type = (packed.data1 >> 7u) & 1u;

    // Extract intensity from bits 0-6, map 0-127 -> 0.0-8.0
    let intensity_7bit = f32(packed.data1 & 0x7Fu);
    light.intensity = intensity_7bit / 127.0 * 8.0;
    light.enabled = intensity_7bit > 0.0;

    // Extract color (bits 31-8)
    light.color = vec3<f32>(
        f32((packed.data1 >> 24u) & 0xFFu) / 255.0,
        f32((packed.data1 >> 16u) & 0xFFu) / 255.0,
        f32((packed.data1 >> 8u) & 0xFFu) / 255.0
    );

    if (light.light_type == 0u) {
        // Directional light: unpack octahedral direction
        light.direction = unpack_octahedral(packed.data0);
        light.position = vec3<f32>(0.0);
        light.range = 0.0;
    } else {
        // Point light: unpack position (f16x2 + f16) and range (f16)
        let xy = unpack2x16float(packed.data0);
        let z_range = unpack2x16float(packed.data2);
        light.position = vec3<f32>(xy.x, xy.y, z_range.x);
        light.range = z_range.y;
        light.direction = vec3<f32>(0.0, -1.0, 0.0);  // Default, unused
    }

    return light;
}

// Computed light data after resolving direction and attenuation
struct ComputedLight {
    direction: vec3<f32>,  // Light direction (ray direction convention)
    color: vec3<f32>,      // Light color × intensity × attenuation
}

// Compute light direction and effective color for both directional and point lights
// Resolves the directional vs point light branching in one place
fn compute_light(light: LightData, world_position: vec3<f32>) -> ComputedLight {
    var result: ComputedLight;

    if (light.light_type == 0u) {
        // Directional light: use stored direction
        result.direction = light.direction;
        result.color = light.color * light.intensity;
    } else {
        // Point light: compute direction and attenuation
        let to_light = light.position - world_position;
        let distance = length(to_light);
        result.direction = -normalize(to_light);  // Negate: convention is "ray direction"
        let attenuation = point_light_attenuation(distance, light.range);
        result.color = light.color * light.intensity * attenuation;
    }

    return result;
}

// ============================================================================
// Unified Vertex Input/Output (all modes)
// ============================================================================

struct VertexIn {
    @location(0) position: vec3<f32>,
    //VIN_UV
    //VIN_COLOR
    //VIN_NORMAL
    //VIN_SKINNED
    //VIN_TANGENT
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    //VOUT_WORLD_NORMAL
    //VOUT_VIEW_NORMAL
    @location(3) @interpolate(flat) shading_state_index: u32,
    //VOUT_VIEW_POS
    //VOUT_CAMERA_POS
    //VOUT_TANGENT
    //VOUT_VIEW_TANGENT
    //VOUT_UV
    //VOUT_COLOR
}

// ============================================================================
// Unified Vertex Shader (all modes)
// ============================================================================

fn extract_camera_position(view_matrix: mat4x4<f32>) -> vec3<f32> {
    let r00 = view_matrix[0][0]; let r10 = view_matrix[0][1]; let r20 = view_matrix[0][2];
    let r01 = view_matrix[1][0]; let r11 = view_matrix[1][1]; let r21 = view_matrix[1][2];
    let r02 = view_matrix[2][0]; let r12 = view_matrix[2][1]; let r22 = view_matrix[2][2];
    let tx = view_matrix[3][0];
    let ty = view_matrix[3][1];
    let tz = view_matrix[3][2];
    return -vec3<f32>(
        r00 * tx + r10 * ty + r20 * tz,
        r01 * tx + r11 * ty + r21 * tz,
        r02 * tx + r12 * ty + r22 * tz
    );
}

@vertex
fn vs(in: VertexIn, @builtin(instance_index) instance_index: u32) -> VertexOut {
    var out: VertexOut;

    // Extract indices first (needed by skinning code)
    let indices = mvp_shading_indices[instance_index];
    let model_idx = indices.x;
    let view_idx = indices.y;
    let proj_idx = indices.z;
    let shading_state_idx = indices.w;

    //VS_SKINNED

    // Access unified_transforms directly - indices are pre-offset by CPU
    let model_matrix = unified_transforms[model_idx];
    let view_matrix = unified_transforms[view_idx];
    let projection_matrix = unified_transforms[proj_idx];

    //VS_POSITION
    let model_pos = model_matrix * world_pos;
    out.world_position = model_pos.xyz;

    //VS_WORLD_NORMAL
    //VS_VIEW_NORMAL
    //VS_VIEW_POS
    //VS_TANGENT
    //VS_VIEW_TANGENT

    out.clip_position = projection_matrix * view_matrix * model_pos;
    out.shading_state_index = shading_state_idx;

    //VS_CAMERA_POS

    //VS_UV
    //VS_COLOR

    return out;
}
