// ============================================================================
// EPU BACKGROUND SAMPLING
// ============================================================================
// Sky/background uses procedural evaluation (L_hi) so it is never limited by
// the EnvRadiance texture resolution.

// Octahedral encode for EPU texture sampling (direction -> UV)
// WGSL `sign()` returns 0 for 0 inputs, which breaks octahedral fold math on the
// axes (producing visible "plus" seams). Use a non-zero sign instead.
fn sign_not_zero(v: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        select(-1.0, 1.0, v.x >= 0.0),
        select(-1.0, 1.0, v.y >= 0.0)
    );
}

fn epu_octahedral_encode(dir: vec3<f32>) -> vec2<f32> {
    let n = dir / (abs(dir.x) + abs(dir.y) + abs(dir.z));
    if n.z < 0.0 {
        return (1.0 - abs(n.yx)) * sign_not_zero(n.xy);
    }
    return n.xy;
}

fn epu_source_kind(env_index: u32) -> u32 {
    return epu_source_kinds[env_index];
}

fn epu_imported_face_base(env_index: u32) -> u32 {
    return epu_imported_face_base_layers[env_index];
}

fn sample_epu_imported_face_layer(face_layer: u32, uv: vec2f) -> vec3f {
    let uv_clamped = clamp(uv, vec2f(0.0), vec2f(1.0));
    return textureSampleLevel(epu_imported_faces, epu_sampler, uv_clamped, i32(face_layer), 0.0).rgb;
}

fn sample_epu_imported_cube(env_index: u32, direction: vec3f) -> vec3f {
    let base_layer = epu_imported_face_base(env_index);
    if base_layer == EPU_IMPORTED_FACE_BASE_INVALID {
        let uv = epu_octahedral_encode(normalize(direction)) * 0.5 + 0.5;
        return textureSampleLevel(epu_env_radiance, epu_sampler, uv, i32(env_index), 0.0).rgb;
    }

    let dir = normalize(direction);
    let abs_dir = abs(dir);

    if abs_dir.x >= abs_dir.y && abs_dir.x >= abs_dir.z {
        let inv = 1.0 / max(abs_dir.x, 1e-6);
        if dir.x > 0.0 {
            let uv = vec2f(dir.z, -dir.y) * inv * 0.5 + 0.5;
            return sample_epu_imported_face_layer(base_layer + 0u, uv);
        }

        let uv = vec2f(-dir.z, -dir.y) * inv * 0.5 + 0.5;
        return sample_epu_imported_face_layer(base_layer + 1u, uv);
    }

    if abs_dir.y >= abs_dir.z {
        let inv = 1.0 / max(abs_dir.y, 1e-6);
        if dir.y > 0.0 {
            let uv = vec2f(dir.x, -dir.z) * inv * 0.5 + 0.5;
            return sample_epu_imported_face_layer(base_layer + 2u, uv);
        }

        let uv = vec2f(dir.x, dir.z) * inv * 0.5 + 0.5;
        return sample_epu_imported_face_layer(base_layer + 3u, uv);
    }

    let inv = 1.0 / max(abs_dir.z, 1e-6);
    if dir.z > 0.0 {
        let uv = vec2f(-dir.x, -dir.y) * inv * 0.5 + 0.5;
        return sample_epu_imported_face_layer(base_layer + 4u, uv);
    }

    let uv = vec2f(dir.x, -dir.y) * inv * 0.5 + 0.5;
    return sample_epu_imported_face_layer(base_layer + 5u, uv);
}

// Sample background from the procedural EPU state.
fn epu_eval_hi(env_index: u32, direction: vec3<f32>) -> vec3f {
    return evaluate_epu_layers(normalize(direction), epu_states[env_index].layers);
}
fn sample_epu_background(env_index: u32, direction: vec3<f32>) -> vec4<f32> {
    if epu_source_kind(env_index) == EPU_SOURCE_IMPORTED {
        return vec4f(sample_epu_imported_cube(env_index, direction), 1.0);
    }
    return vec4f(epu_eval_hi(env_index, direction), 1.0);
}

// ============================================================================
// EPU REFLECTION SAMPLING (View-dependent Blinn-Phong integral)
// ============================================================================
// Unit-specular-color integral of the existing direct BP contract. The caller
// multiplies by material specular_color once; there is no angular Fresnel term.
fn sample_epu_reflection(env_id: u32, world_n: vec3f, view_dir: vec3f, roughness: f32) -> vec3f {
    let N = normalize(world_n);
    let V = normalize(view_dir);
    let s = 1.0 + 255.0 * (1.0 - clamp(roughness, 0.0, 1.0));
    var integral = vec3f(0.0);
    // p(H)=(s+1)/(2*pi)*(N.H)^s; dL=4*(V.H)*dH.
    // Blend two overlapping frames instead of abruptly rotating finite quadrature.
    // Each frame's weight vanishes at its own singular axis; their sum is >= 1.
    // ponytail: up to 256 source fetches/pixel; performance gate DEFERRED.
    var weight_sum = 0.0;
    for (var frame = 0u; frame < 2u; frame++) {
        let up = select(vec3f(0.0, 1.0, 0.0), vec3f(0.0, 0.0, 1.0), frame == 1u);
        let tangent = cross(up, N);
        let weight = dot(tangent, tangent);
        weight_sum += weight;
        if (weight == 0.0) { continue; }
        let T = tangent / sqrt(weight);
        let B = cross(N, T);
        for (var i = 0u; i < 128u; i++) {
            let mass = (f32(i) + 0.5) / 128.0;
            let z = pow(mass, 1.0 / (s + 1.0));
            let phi = 2.0 * PI * fract((f32(i) + 0.5) * 0.6180339887498949);
            let radius = sqrt(max(1.0 - z * z, 0.0));
            let H = T * (radius * cos(phi)) + B * (radius * sin(phi)) + N * z;
            let VoH = dot(V, H);
            let L = 2.0 * VoH * H - V;
            let uv = epu_octahedral_encode(normalize(L)) * 0.5 + 0.5;
            let radiance = textureSampleLevel(epu_env_radiance, epu_sampler, uv, i32(env_id), 0.0).rgb;
            integral += weight * radiance * max(VoH, 0.0) * max(dot(N, L), 0.0);
        }
    }
    let normalization = s * 0.0397436 + 0.0856832;
    return integral * (normalization * 8.0 * PI / (s + 1.0) / 128.0 / weight_sum);
}

// ============================================================================
// EPU SH9 DIFFUSE IRRADIANCE (L2)
// ============================================================================
// Sample from pre-computed SH9 coefficients for diffuse ambient lighting.
// SH9 is much smoother on curved surfaces than a 6-direction ambient cube.

fn sample_epu_ambient(env_id: u32, n: vec3f) -> vec3f {
    let c = epu_sh9[env_id];

    let nn = normalize(n);
    let x = nn.x;
    let y = nn.y;
    let z = nn.z;

    // Real SH basis functions (L2), evaluated at the surface normal.
    // Order: [Y00, Y1-1, Y10, Y11, Y2-2, Y2-1, Y20, Y21, Y22]
    let sh0 = 0.282095;
    let sh1 = 0.488603 * y;
    let sh2 = 0.488603 * z;
    let sh3 = 0.488603 * x;
    let sh4 = 1.092548 * x * y;
    let sh5 = 1.092548 * y * z;
    let sh6 = 0.315392 * (3.0 * z * z - 1.0);
    let sh7 = 1.092548 * x * z;
    let sh8 = 0.546274 * (x * x - y * y);

    let e = c.c0 * sh0
        + c.c1 * sh1
        + c.c2 * sh2
        + c.c3 * sh3
        + c.c4 * sh4
        + c.c5 * sh5
        + c.c6 * sh6
        + c.c7 * sh7
        + c.c8 * sh8;

    // SH reconstruction can go slightly negative; clamp to prevent artifacts.
    //
    // NOTE: The SH coefficients represent diffuse irradiance (Lambertian-convolved).
    // Convert to Lambertian diffuse radiance for albedo=1 by dividing by PI.
    return max(e / PI, vec3f(0.0));
}
