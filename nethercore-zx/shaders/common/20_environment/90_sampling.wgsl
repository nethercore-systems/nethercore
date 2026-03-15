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

// Sample background from the procedural EPU state.
fn epu_eval_hi_raw(env_index: u32, direction: vec3<f32>) -> vec3f {
    let dir = normalize(direction);
    let st = epu_states[env_index];

    // Procedural evaluation path used by:
    // - the sky/background draw (never limited by EnvRadiance resolution)
    // - the high-frequency residual used in specular reflections
    //
    // Must match the compute shader's evaluation semantics (multi-bounds).

    // Start with default bounds direction (will be updated by bounds layers)
    var bounds_dir = vec3f(0.0, 1.0, 0.0);
    // Default regions: all-sky (bounds layers will compute their own regions)
    var regions = RegionWeights(1.0, 0.0, 0.0);
    var bounds_count = 0u;
    var shared_bounds_dir_set = false;

    var radiance = vec3f(0.0);
    for (var i = 0u; i < 8u; i++) {
        let instr = st.layers[i];
        let opcode = instr_opcode(instr);
        if opcode == OP_NOP { continue; }

        let is_bounds = opcode < OP_FEATURE_MIN;
        let blend = instr_blend(instr);

        if is_bounds {
            // Keep the direct background path in lockstep with the compute
            // evaluation semantics so background, reflection, and debug views
            // do not drift apart when bounds composition changes.
            let eval_bounds_dir = bounds_dir_from_layer(instr, opcode, bounds_dir);
            if !shared_bounds_dir_set {
                bounds_dir = eval_bounds_dir;
                shared_bounds_dir_set = true;
            }
            let bounds_result = evaluate_bounds_layer(dir, instr, opcode, eval_bounds_dir, regions);
            var region_mix = bounds_result.region_mix;
            if bounds_count > 0u {
                let organizer_bounds =
                    opcode == OP_SECTOR
                    || opcode == OP_SPLIT
                    || opcode == OP_CELL
                    || opcode == OP_APERTURE;
                let retag_scale = select(0.72, 0.38, organizer_bounds);
                region_mix *= retag_scale;
            }
            regions = compose_bounds_regions(regions, bounds_result.regions, region_mix);
            radiance = apply_blend(radiance, bounds_result.sample, blend);
            bounds_count += 1u;
        } else {
            // Feature opcode: evaluate using the current bounds_dir + region weights.
            let sample = evaluate_layer(dir, instr, bounds_dir, regions);
            radiance = apply_blend(radiance, sample, blend);
        }
    }

    return radiance;
}

fn epu_eval_hi(env_index: u32, direction: vec3<f32>) -> vec3f {
    let dir = normalize(direction);
    let hint = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(dir.y) > 0.9);
    let tangent = normalize(cross(hint, dir));
    let filter_radius = 0.02;
    let dir_a = normalize(dir + tangent * filter_radius);
    let dir_b = normalize(dir - tangent * filter_radius);

    return (
        epu_eval_hi_raw(env_index, dir)
        + epu_eval_hi_raw(env_index, dir_a)
        + epu_eval_hi_raw(env_index, dir_b)
    ) / 3.0;
}

fn sample_epu_background(env_index: u32, direction: vec3<f32>) -> vec4<f32> {
    return vec4f(epu_eval_hi_raw(env_index, direction), 1.0);
}

// ============================================================================
// EPU REFLECTION SAMPLING (Continuous Roughness -> LOD)
// ============================================================================
// Sample from the mip-mapped EnvRadiance texture for roughness-based reflections.
// Roughness is mapped continuously across the available mip levels.

fn sample_epu_reflection(env_id: u32, refl_dir: vec3f, roughness: f32) -> vec3f {
    let uv = epu_octahedral_encode(normalize(refl_dir)) * 0.5 + 0.5;

    // Use roughness^2 for a perceptually linear blur ramp, then add a modest
    // mid/high-roughness bias so metallic probes stop reprojecting such crisp
    // shell structure in the middle of the roughness range.
    let r = epu_saturate(roughness);
    let max_lod = max(0.0, f32(textureNumLevels(epu_env_radiance) - 1));
    let lod_bias = 0.75 * smoothstep(0.28, 0.75, r);
    let lod = min((r * r) * max_lod + lod_bias, max_lod);

    // Manual mip lerp (keeps results smooth even if sampler mipmap_filter is Nearest).
    let lod0 = floor(lod);
    let lod1 = min(lod0 + 1.0, max_lod);
    let t = lod - lod0;

    let c0 = textureSampleLevel(epu_env_radiance, epu_sampler, uv, i32(env_id), lod0).rgb;
    let c1 = textureSampleLevel(epu_env_radiance, epu_sampler, uv, i32(env_id), lod1).rgb;
    let l_lp = mix(c0, c1, t);

    // Residual blend: add back a small amount of high-frequency energy that the
    // low-pass cache cannot represent. Keep this correction confined to the
    // very-smooth band so it does not rebuild broad probe-shell structure.
    let l0 = textureSampleLevel(epu_env_radiance, epu_sampler, uv, i32(env_id), 0.0).rgb;
    let l_hi = epu_eval_hi(env_id, refl_dir);
    let alpha = r * r;
    let residual_fade = 1.0 - smoothstep(0.04, 0.18, r);

    // NOTE: In practice `l0` can slightly overshoot `l_hi` due to finite EnvRadiance resolution
    // and sampler filtering, producing negative residuals. On fully metallic materials this can
    // manifest as "peppered" black pixels at mid roughness when the residual is clamped.
    // Treat the residual as additive energy only.
    let residual_raw = max(l_hi - l0, vec3f(0.0));
    let residual_cap = l_lp * 0.35;
    let residual = min(residual_raw, residual_cap);
    let l_spec = l_lp + (1.0 - alpha) * residual_fade * residual;

    return max(l_spec, vec3f(0.0));
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
