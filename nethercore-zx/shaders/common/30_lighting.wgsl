
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

