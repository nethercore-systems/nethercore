// ============================================================================
// APERTURE - Shaped Opening Enclosure Modifier (0x07)
// Creates a view-centered viewport/frame. The wall region becomes the frame,
// and sky becomes the opening through it, creating a strong "interior" cue.
// 128-bit packed fields:
//   color_a: Opening/sky color (RGB24)
//   color_b: Frame/wall color (RGB24)
//   intensity: Frame edge softness (0..255 -> 0.005..min(0.1, frame_thickness*0.45))
//   param_a: Opening half-width (0..255 -> 0.1..1.5 tangent units)
//   param_b: Opening half-height (0..255 -> 0.1..1.5 tangent units)
//   param_c: Frame thickness (0..255 -> 0.02..0.5 tangent units)
//   param_d: Variant-specific (corner radius, bar count, or cell count)
//   direction: Aperture center direction (oct-u16)
//   alpha_a: Unused (set to 0)
//   alpha_b: Unused (set to 0)
//   variant_id: 0 CIRCLE, 1 RECT, 2 ROUNDED_RECT, 3 ARCH, 4 BARS, 5 MULTI, 6 IRREGULAR
// ============================================================================

// Noise hash for IRREGULAR variant boundary displacement
fn aperture_hash21(p: vec2f) -> f32 {
    return fract(sin(dot(p, vec2f(127.1, 311.7))) * 43758.5453123);
}

// Value noise for IRREGULAR variant
fn aperture_value_noise(uv: vec2f, freq: f32) -> f32 {
    let p = uv * freq;
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let a = aperture_hash21(i);
    let b = aperture_hash21(i + vec2f(1.0, 0.0));
    let c = aperture_hash21(i + vec2f(0.0, 1.0));
    let d = aperture_hash21(i + vec2f(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y) * 2.0 - 1.0;
}

// SDF for circle aperture
fn aperture_sdf_circle(uv: vec2f, half_w: f32, half_h: f32) -> f32 {
    // Ellipse: scale uv by aspect ratio and use circular SDF
    let scaled = vec2f(uv.x / half_w, uv.y / half_h);
    return (length(scaled) - 1.0) * min(half_w, half_h);
}

// SDF for rectangle aperture
fn aperture_sdf_rect(uv: vec2f, half_w: f32, half_h: f32) -> f32 {
    let d = abs(uv) - vec2f(half_w, half_h);
    return max(d.x, d.y);
}

// SDF for rounded rectangle aperture
fn aperture_sdf_rounded_rect(uv: vec2f, half_w: f32, half_h: f32, radius: f32) -> f32 {
    let r = min(radius, min(half_w, half_h));
    let q = abs(uv) - vec2f(half_w - r, half_h - r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2f(0.0))) - r;
}

// SDF for arch aperture (rectangle bottom + semicircle top)
fn aperture_sdf_arch(uv: vec2f, half_w: f32, half_h: f32, rise: f32) -> f32 {
    // rise: 0 = flat top, 1 = full semicircle
    let arch_height = half_h * rise;
    let rect_height = half_h * (1.0 - rise * 0.5);

    // Rectangle portion (lower part)
    let rect_sdf = max(abs(uv.x) - half_w, max(-uv.y - rect_height, uv.y - rect_height));

    // Arch portion (upper semicircle)
    let arch_center = vec2f(0.0, rect_height - arch_height);
    let arch_radius = half_w;
    let to_arch = uv - arch_center;

    // Only apply arch SDF above the rectangle
    if uv.y > rect_height - arch_height && length(to_arch) > 0.001 {
        let arch_sdf = length(to_arch) - arch_radius;
        // Union: min of rect and negated arch constraint
        if uv.y > 0.0 {
            return min(rect_sdf, arch_sdf);
        }
    }

    return rect_sdf;
}

// SDF for bars aperture (rect with vertical bars subtracted)
fn aperture_sdf_bars(uv: vec2f, half_w: f32, half_h: f32, bar_count: u32) -> f32 {
    let base_sdf = aperture_sdf_rect(uv, half_w, half_h);

    // Inside the opening: check if we hit a bar
    if base_sdf < 0.0 {
        let count = max(bar_count, 1u);
        let bar_spacing = (half_w * 2.0) / f32(count + 1u);
        let bar_width = bar_spacing * 0.15;

        // Find distance to nearest bar center
        let x_offset = uv.x + half_w;
        let bar_index = floor(x_offset / bar_spacing + 0.5);
        let bar_center_x = bar_index * bar_spacing - half_w;
        let bar_dist = abs(uv.x - bar_center_x) - bar_width;

        // Bar occludes the opening (makes it wall)
        if bar_dist < 0.0 && bar_index >= 1.0 && bar_index <= f32(count) {
            return -bar_dist; // Inside bar = positive SDF (outside opening)
        }
    }

    return base_sdf;
}

// SDF for multi aperture (grid of smaller openings)
fn aperture_sdf_multi(uv: vec2f, half_w: f32, half_h: f32, cell_count: u32) -> f32 {
    let count = max(cell_count, 1u);
    let cell_w = (half_w * 2.0) / f32(count);
    let cell_h = (half_h * 2.0) / f32(count);
    let gap = min(cell_w, cell_h) * 0.1;

    // Transform uv into cell-local coordinates
    let cell_uv = vec2f(
        ((uv.x + half_w) % cell_w) - cell_w * 0.5,
        ((uv.y + half_h) % cell_h) - cell_h * 0.5
    );

    // Cell-local opening SDF
    let cell_half_w = (cell_w - gap) * 0.5;
    let cell_half_h = (cell_h - gap) * 0.5;
    let cell_sdf = aperture_sdf_rect(cell_uv, cell_half_w, cell_half_h);

    // Combine with outer boundary
    let outer_sdf = aperture_sdf_rect(uv, half_w, half_h);

    // Inside outer boundary: use cell SDF; outside: use outer
    if outer_sdf < 0.0 {
        return cell_sdf;
    }
    return outer_sdf;
}

// SDF for irregular aperture (rect/circle with noise-displaced boundary)
fn aperture_sdf_irregular(uv: vec2f, half_w: f32, half_h: f32, amplitude: f32) -> f32 {
    // Get base distance
    let base_sdf = aperture_sdf_rect(uv, half_w, half_h);

    // Apply noise displacement to boundary
    let noise_freq = 8.0;
    let noise_val = aperture_value_noise(uv * 2.0 + vec2f(42.0), noise_freq);

    // Displace the SDF by noise
    return base_sdf - noise_val * amplitude;
}

fn eval_aperture(
    dir: vec3f,
    instr: vec4u,
    base_regions: RegionWeights,
) -> BoundsResult {
    // Decode aperture center direction
    let center_dir = decode_dir16(instr_dir16(instr));

    // Extract parameters
    let half_w = mix(0.1, 1.5, u8_to_01(instr_a(instr)));
    let half_h = mix(0.1, 1.5, u8_to_01(instr_b(instr)));
    let frame_thickness = mix(0.02, 0.5, u8_to_01(instr_c(instr)));
    let raw_softness = mix(0.005, 0.1, u8_to_01(instr_intensity(instr)));
    let softness = min(raw_softness, frame_thickness * 0.45);
    let param_d_raw = instr_d(instr);
    let variant = instr_variant_id(instr);

    // Use baseline region weights passed in
    let baseline = base_regions;

    // Avoid a hard hemisphere cutoff at dot(dir, center_dir) == 0.
    // Fade the aperture influence out near the horizon, and keep it strictly
    // zero on the back hemisphere to prevent great-circle seams.
    let d_raw = dot(dir, center_dir);
    let horizon_fade = 0.06;
    let front_w = smoothstep(0.0, horizon_fade, d_raw);
    if front_w <= 0.0 {
        return BoundsResult(LayerSample(vec3f(0.0), 0.0), baseline);
    }

    // Clamp for projection stability (prevents division blow-ups near the horizon).
    let d_proj = max(d_raw, horizon_fade);

    // Build view-centered tangent chart projection
    // Project direction onto the tangent plane at center_dir
    // We need to extract 2D coordinates (tangent plane basis)
    let up = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(center_dir.y) > 0.9);
    let right = normalize(cross(up, center_dir));
    let tangent_up = normalize(cross(center_dir, right));
    let proj = (dir - center_dir * d_raw) / d_proj;
    let uv = vec2f(dot(proj, right), dot(proj, tangent_up));

    // Evaluate SDF based on variant
    var sdf: f32;
    switch variant {
        case 0u: {
            // CIRCLE: Circular/elliptical aperture
            sdf = aperture_sdf_circle(uv, half_w, half_h);
        }
        case 1u: {
            // RECT: Rectangular aperture
            sdf = aperture_sdf_rect(uv, half_w, half_h);
        }
        case 2u: {
            // ROUNDED_RECT: Rounded rectangle
            // param_d: 0..255 -> 0.0..0.5 corner radius
            let corner_radius = u8_to_01(param_d_raw) * 0.5;
            sdf = aperture_sdf_rounded_rect(uv, half_w, half_h, corner_radius);
        }
        case 3u: {
            // ARCH: Rectangle with semicircular top
            // param_d: 0..255 -> 0.0..1.0 arch rise ratio
            let rise = u8_to_01(param_d_raw);
            sdf = aperture_sdf_arch(uv, half_w, half_h, rise);
        }
        case 4u: {
            // BARS: Rectangle with vertical bars
            // param_d: 1..16 bar count (clamped)
            let bar_count = clamp(param_d_raw, 1u, 16u);
            sdf = aperture_sdf_bars(uv, half_w, half_h, bar_count);
        }
        case 5u: {
            // MULTI: Grid of smaller openings
            // param_d: 1..8 cells per axis (clamped)
            let cell_count = clamp(param_d_raw, 1u, 8u);
            sdf = aperture_sdf_multi(uv, half_w, half_h, cell_count);
        }
        case 6u: {
            // IRREGULAR: Rectangle with noise-displaced boundary
            // param_d: 0..255 -> 0.0..0.3 noise amplitude
            let amplitude = u8_to_01(param_d_raw) * 0.3;
            sdf = aperture_sdf_irregular(uv, half_w, half_h, amplitude);
        }
        default: {
            // Default to circle
            sdf = aperture_sdf_circle(uv, half_w, half_h);
        }
    }

    // Compute zone weights using frame_thickness and softness
    // opening_w: 1 inside aperture opening, 0 outside
    let opening_w0 = smoothstep(softness, -softness, sdf);

    // frame_w: 1 in the frame band between opening edge and frame outer edge
    let frame_inner = smoothstep(-softness, softness, sdf);
    let frame_outer = smoothstep(softness, -softness, sdf - frame_thickness);
    let frame_w0 = frame_inner * frame_outer;

    // Keep zone weights normalized.
    let outside_w0 = clamp(1.0 - opening_w0 - frame_w0, 0.0, 1.0);

    // Region outputs (bounds): inside the opening becomes SKY; the frame becomes WALLS.
    // Outside the aperture keeps the baseline region weights.
    let w_sky_front = baseline.sky * outside_w0 + opening_w0;
    let w_wall_front = baseline.wall * outside_w0 + frame_w0;
    let w_floor_front = baseline.floor * outside_w0;

    let w_sky = mix(baseline.sky, w_sky_front, front_w);
    let w_wall = mix(baseline.wall, w_wall_front, front_w);
    let w_floor = mix(baseline.floor, w_floor_front, front_w);

    // Get colors
    let opening_color = instr_color_a(instr);  // Sky through opening
    let frame_color = instr_color_b(instr);    // Frame/wall color

    // IMPORTANT: Do not tint the full sphere outside the aperture.
    // Only draw the opening/frame itself; outside stays whatever prior bounds already produced.
    let zone_w = opening_w0 + frame_w0;
    let zone_rgb = select(
        vec3f(0.0),
        (opening_color * opening_w0 + frame_color * frame_w0) / zone_w,
        zone_w > 1e-5
    );
    let rgb = zone_rgb;
    let a = epu_saturate(zone_w * front_w);

    // Output modified regions
    let output_regions = RegionWeights(w_sky, w_wall, w_floor);
    return BoundsResult(LayerSample(rgb, a), output_regions);
}
