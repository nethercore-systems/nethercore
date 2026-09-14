fn bands_original(dir: vec3f, n0: vec3f, basis: mat3x3f, band_count: f32, band_offset: f32, bw: f32) -> RegionWeights {
    let u = dot(dir, n0) * 0.5 + 0.5 + band_offset;
    let cross_phase = vec2f(dot(dir, basis[0]), dot(dir, basis[1]));
    let sweep = dot(cross_phase, vec2f(0.63, -0.77));
    let relief = epu_relief_wave(cross_phase * vec2f(1.35, 1.05), band_offset + band_count * 0.031);
    let secondary = epu_relief_wave(cross_phase.yx * vec2f(0.82, 1.41), band_offset + 0.37);
    let warp = (relief * 0.075 + secondary * 0.04 + sweep * 0.085) / max(band_count, 1.0);
    let local_bw = bw * band_count * mix(0.84, 1.2, relief * 0.5 + 0.5);
    let d = epu_periodic_centered(u * band_count + warp);
    return regions_from_signed_distance(d, local_bw);
}