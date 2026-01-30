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

// Sample background from EPU EnvRadiance texture (LOD 0)
fn epu_eval_hi(env_index: u32, direction: vec3<f32>) -> vec3f {
    let dir = normalize(direction);
    let st = epu_states[env_index];

    // Procedural evaluation path used by:
    // - the sky/background draw (never limited by EnvRadiance resolution)
    // - the high-frequency residual used in specular reflections
    //
    // Must match the compute shader's evaluation semantics (multi-bounds).

    // Start with default enclosure.
    var enc = EnclosureConfig(vec3f(0.0, 1.0, 0.0), 0.5, -0.5, 0.1);
    // Default regions should be direction-dependent, so presets can start with any bounds
    // opcode (SECTOR / SPLIT / SILHOUETTE / APERTURE) without requiring a leading RAMP.
    var regions = compute_region_weights(dir, enc);

    var radiance = vec3f(0.0);
    for (var i = 0u; i < 8u; i++) {
        let instr = st.layers[i];
        let opcode = instr_opcode(instr);
        if opcode == OP_NOP { continue; }

        let is_bounds = opcode < OP_FEATURE_MIN;
        let blend = instr_blend(instr);

        if is_bounds {
            // Bounds opcode: update enclosure, evaluate bounds, and feed its output regions
            // into subsequent feature layers.
            enc = enclosure_from_layer(instr, opcode, enc);
            let bounds_result = evaluate_bounds_layer(dir, instr, opcode, enc, regions);
            regions = bounds_result.regions;
            radiance = apply_blend(radiance, bounds_result.sample, blend);
        } else {
            // Feature opcode: evaluate using the current enclosure + region weights.
            let sample = evaluate_layer(dir, instr, enc, regions);
            radiance = apply_blend(radiance, sample, blend);
        }
    }

    return radiance;
}

fn sample_epu_background(env_index: u32, direction: vec3<f32>) -> vec4<f32> {
    return vec4f(epu_eval_hi(env_index, direction), 1.0);
}

// ============================================================================
// EPU REFLECTION SAMPLING (Continuous Roughness -> LOD)
// ============================================================================
// Sample from the mip-mapped EnvRadiance texture for roughness-based reflections.
// Roughness is mapped continuously across the available mip levels.

fn sample_epu_reflection(env_id: u32, refl_dir: vec3f, roughness: f32) -> vec3f {
    let uv = epu_octahedral_encode(normalize(refl_dir)) * 0.5 + 0.5;

    // Use roughness^2 for a more perceptually linear blur ramp.
    let r = epu_saturate(roughness);
    let max_lod = max(0.0, f32(textureNumLevels(epu_env_radiance) - 1));
    let lod = (r * r) * max_lod;

    // Manual mip lerp (keeps results smooth even if sampler mipmap_filter is Nearest).
    let lod0 = floor(lod);
    let lod1 = min(lod0 + 1.0, max_lod);
    let t = lod - lod0;

    let c0 = textureSampleLevel(epu_env_radiance, epu_sampler, uv, i32(env_id), lod0).rgb;
    let c1 = textureSampleLevel(epu_env_radiance, epu_sampler, uv, i32(env_id), lod1).rgb;
    let l_lp = mix(c0, c1, t);

    // Residual blend: add back high-frequency energy that the low-pass cache cannot represent,
    // fading out continuously with roughness (no thresholds).
    let l0 = textureSampleLevel(epu_env_radiance, epu_sampler, uv, i32(env_id), 0.0).rgb;
    let l_hi = epu_eval_hi(env_id, refl_dir);
    let alpha = r * r;

    // NOTE: In practice `l0` can slightly overshoot `l_hi` due to finite EnvRadiance resolution
    // and sampler filtering, producing negative residuals. On fully metallic materials this can
    // manifest as "peppered" black pixels at mid roughness when the residual is clamped.
    // Treat the residual as additive energy only.
    let residual = max(l_hi - l0, vec3f(0.0));
    let l_spec = l_lp + (1.0 - alpha) * residual;

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
