fn legacy_celestial_ringed(
    dir: vec3f,
    body_dir: vec3f,
    r: f32,
    angular_size_rad: f32,
    limb: f32,
    color_a: vec3f,
    color_b: vec3f,
    ring_tilt_deg: f32,
    alpha_b: f32
) -> LayerSample {
    // Ring geometry: rings are in a plane tilted from viewer
    // Ring tilt: 0 = edge-on, 90 = face-on
    let ring_tilt_rad = ring_tilt_deg * PI / 180.0;
    let tilt_factor = sin(ring_tilt_rad);

    // Build ring plane normal (tilted from body direction)
    let hint = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(body_dir.y) > 0.9);
    let ring_axis = normalize(cross(hint, body_dir));
    // Rotate body_dir around ring_axis by tilt angle
    let ring_normal = body_dir * cos(ring_tilt_rad) + cross(ring_axis, body_dir) * sin(ring_tilt_rad);

    // Compute angular distance from body center
    let body_dot = clamp(dot(dir, body_dir), -1.0, 1.0);
    let angle = acos(body_dot);

    // Ring radii in angular units (inner at 1.5x disk, outer at 2.5x disk)
    let inner_r = 1.5;
    let outer_r = 2.5;

    // Distance from ring plane
    let ring_plane_dist = abs(dot(dir - body_dir * body_dot, ring_normal));
    let in_ring_plane = ring_plane_dist < angular_size_rad * 0.3 * tilt_factor;

    // Ring mask: annular region
    let ring_mask = smoothstep(inner_r - 0.1, inner_r + 0.1, r) *
                    smoothstep(outer_r + 0.1, outer_r - 0.1, r);
    let ring_visible = select(0.0, ring_mask, in_ring_plane && r > 1.05);

    // Ring brightness varies with radius (Cassini division effect)
    let ring_r_norm = (r - inner_r) / (outer_r - inner_r);
    let ring_bands = sin(ring_r_norm * 8.0 * PI) * 0.3 + 0.7;
    // Gap at ~0.5 for Cassini-like division
    let cassini_gap = smoothstep(0.45, 0.5, ring_r_norm) * smoothstep(0.55, 0.5, ring_r_norm);
    let ring_brightness = ring_bands * (1.0 - cassini_gap * 0.8);

    // Planet disk (behind rings at some angles)
    let disk = smoothstep(1.05, 0.95, r);
    let planet_surface = color_a * limb * disk;

    // Ring color
    let ring_color = color_b * ring_visible * ring_brightness * tilt_factor;

    // Combine: rings can be in front of or behind the planet depending on geometry
    // Simplified: rings always rendered on top for visual clarity
    let rgb = planet_surface + ring_color * alpha_b;
    let w = disk + ring_visible * alpha_b;

    return LayerSample(rgb, w);
}

