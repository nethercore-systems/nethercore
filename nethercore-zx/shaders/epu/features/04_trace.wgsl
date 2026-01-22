// ============================================================================
// TRACE - Procedural Line/Crack Patterns (Lightning, Cracks, Lead Lines, Filaments)
// Opcode: 0x0C
// Role: Radiance (additive feature layer)
//
// Packed fields (v2):
//   color_a: Line/trace color (RGB24)
//   color_b: Glow/outline color (RGB24)
//   intensity: Brightness (0..255 -> 0..1)
//   param_a: Trace count (0..255 -> 1..16)
//   param_b: Line thickness (0..255 -> 0.005..0.1)
//   param_c: Jitter/roughness (0..255 -> 0..1)
//   param_d[7:4]: Seed value (0..15)
//   param_d[3:0]: Shape modifier (variant-specific)
//   direction: Axis (AXIS_CYL/AXIS_POLAR) or chart center (TANGENT_LOCAL)
//   alpha_a: Line alpha (0..15 -> 0..1)
//   alpha_b: Glow alpha (0..15 -> 0..1)
//
// Meta (via meta5):
//   domain_id: 1 AXIS_CYL, 2 AXIS_POLAR, 3 TANGENT_LOCAL
//   variant_id: 0 LIGHTNING, 1 CRACKS, 2 LEAD_LINES, 3 FILAMENTS
// ============================================================================

// Domain IDs for TRACE
const TRACE_DOMAIN_DIRECT3D: u32 = 0u;      // Legacy / fallback
const TRACE_DOMAIN_AXIS_CYL: u32 = 1u;      // Cylindrical (azimuth, height)
const TRACE_DOMAIN_AXIS_POLAR: u32 = 2u;    // Polar (angle, radius from axis)
const TRACE_DOMAIN_TANGENT_LOCAL: u32 = 3u; // Tangent plane at direction

// Variant IDs for TRACE
const TRACE_VARIANT_LIGHTNING: u32 = 0u;    // Jagged lightning bolts with branching
const TRACE_VARIANT_CRACKS: u32 = 1u;       // Random branching crack patterns
const TRACE_VARIANT_LEAD_LINES: u32 = 2u;   // Stained-glass polygonal cells
const TRACE_VARIANT_FILAMENTS: u32 = 3u;    // Organic spline-like curves

// Deterministic hash for trace generation (2D -> 3D)
fn trace_hash23(p: vec2f) -> vec3f {
    var p3 = fract(vec3f(p.xyx) * vec3f(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yxz + 33.33);
    return fract((p3.xxy + p3.yzz) * p3.zyx);
}

// Deterministic hash for trace generation (3D -> 1D)
fn trace_hash31(p: vec3f) -> f32 {
    let h = dot(p, vec3f(127.1, 311.7, 74.7));
    return fract(sin(h) * 43758.5453123);
}

// Deterministic hash for trace generation (3D -> 2D)
fn trace_hash32(p: vec3f) -> vec2f {
    let h1 = dot(p, vec3f(127.1, 311.7, 74.7));
    let h2 = dot(p, vec3f(269.5, 183.3, 246.1));
    return fract(sin(vec2f(h1, h2)) * 43758.5453123);
}

// Distance from point to line segment
fn dist_to_segment(p: vec2f, a: vec2f, b: vec2f) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / max(dot(ba, ba), 1e-6), 0.0, 1.0);
    return length(pa - ba * h);
}

// Map direction to cylindrical UV with axis
fn trace_cyl_uv(dir: vec3f, axis: vec3f) -> vec2f {
    // Build tangent basis perpendicular to axis
    let up = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(axis.y) > 0.9);
    let t = normalize(cross(up, axis));
    let b = normalize(cross(axis, t));

    // Project dir onto the plane perpendicular to axis
    let x = dot(dir, t);
    let z = dot(dir, b);
    let y = dot(dir, axis);

    // Azimuth angle [0, 1] and height [-1, 1] -> [0, 1]
    let u = atan2(x, z) / TAU + 0.5;
    let v = y * 0.5 + 0.5;
    return vec2f(u, v);
}

// Map direction to polar UV with axis
fn trace_polar_uv(dir: vec3f, axis: vec3f) -> vec2f {
    // Build tangent basis perpendicular to axis
    let up = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(axis.y) > 0.9);
    let t = normalize(cross(up, axis));
    let b = normalize(cross(axis, t));

    // Project dir onto the plane perpendicular to axis
    let x = dot(dir, t);
    let z = dot(dir, b);
    let y = dot(dir, axis);

    // Angle around axis [0, 1] and radial distance from axis [0, 1]
    let angle = atan2(x, z) / TAU + 0.5;
    let rad = acos(clamp(y, -1.0, 1.0)) / PI; // 0 at axis, 1 at opposite
    return vec2f(angle, rad);
}

// Map direction to tangent-local UV
fn trace_tangent_uv(dir: vec3f, center: vec3f) -> vec3f {
    // Returns (u, v, visibility_weight)
    let d = dot(dir, center);
    if d <= 0.0 {
        return vec3f(0.0, 0.0, 0.0); // Behind the hemisphere
    }
    // Project onto tangent plane
    let up = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(center.y) > 0.9);
    let t = normalize(cross(up, center));
    let b = normalize(cross(center, t));

    let proj = dir - center * d;
    let u = dot(proj, t) / d;
    let v = dot(proj, b) / d;

    // Grazing angle fade
    let grazing_w = smoothstep(0.1, 0.3, d);
    return vec3f(u, v, grazing_w);
}

// Generate a lightning bolt segment chain
fn lightning_trace(uv: vec2f, seed: f32, trace_idx: f32, branch_count: u32, jitter: f32, thickness: f32) -> vec2f {
    // Returns (line_dist, glow_dist)
    var min_dist = 1000.0;

    let base_hash = trace_hash23(vec2f(seed * 17.3, trace_idx * 31.7));

    // Origin point for this trace
    var origin = base_hash.xy;

    // Main bolt direction (generally downward with variation)
    let main_dir = normalize(vec2f(
        (base_hash.z - 0.5) * 0.4,
        -0.8 - base_hash.x * 0.2
    ));

    // Generate main bolt segments
    let seg_count = 5u + u32(base_hash.y * 4.0);
    var prev = origin;

    for (var s = 0u; s < seg_count; s++) {
        let seg_hash = trace_hash23(vec2f(seed + f32(s) * 7.1, trace_idx + f32(s) * 13.3));

        // Segment length with variation
        let seg_len = 0.08 + seg_hash.x * 0.12;

        // Jitter the direction
        let jitter_angle = (seg_hash.y - 0.5) * jitter * PI * 0.5;
        let c = cos(jitter_angle);
        let sn = sin(jitter_angle);
        let jittered_dir = vec2f(
            main_dir.x * c - main_dir.y * sn,
            main_dir.x * sn + main_dir.y * c
        );

        let next = prev + jittered_dir * seg_len;

        // Distance to this segment
        let d = dist_to_segment(uv, prev, next);
        min_dist = min(min_dist, d);

        // Branching at certain segments
        if branch_count > 0u && s > 0u && s < seg_count - 1u {
            let branch_prob = seg_hash.z;
            if branch_prob < f32(branch_count) / 8.0 {
                // Create a branch
                let branch_hash = trace_hash23(vec2f(seed + f32(s) * 23.7, trace_idx * 41.3 + f32(s)));
                let branch_angle = (branch_hash.x - 0.5) * PI * 0.6;
                let bc = cos(branch_angle);
                let bs = sin(branch_angle);
                let branch_dir = vec2f(
                    jittered_dir.x * bc - jittered_dir.y * bs,
                    jittered_dir.x * bs + jittered_dir.y * bc
                );

                let branch_len = seg_len * (0.5 + branch_hash.y * 0.5);
                let branch_end = prev + branch_dir * branch_len;
                let bd = dist_to_segment(uv, prev, branch_end);
                min_dist = min(min_dist, bd);

                // Secondary branch
                if branch_count > 3u && branch_hash.z > 0.5 {
                    let sub_hash = trace_hash23(vec2f(seed + f32(s) * 37.1, trace_idx * 53.7));
                    let sub_angle = (sub_hash.x - 0.5) * PI * 0.4;
                    let sc = cos(sub_angle);
                    let ss = sin(sub_angle);
                    let sub_dir = vec2f(
                        branch_dir.x * sc - branch_dir.y * ss,
                        branch_dir.x * ss + branch_dir.y * sc
                    );
                    let sub_end = branch_end + sub_dir * branch_len * 0.5;
                    let sd = dist_to_segment(uv, branch_end, sub_end);
                    min_dist = min(min_dist, sd);
                }
            }
        }

        prev = next;
    }

    return vec2f(min_dist, min_dist);
}

// Generate crack pattern (radial stress lines)
fn crack_trace(uv: vec2f, seed: f32, trace_idx: f32, seg_count: u32, jitter: f32, thickness: f32) -> vec2f {
    var min_dist = 1000.0;

    let base_hash = trace_hash23(vec2f(seed * 19.3, trace_idx * 37.7));

    // Origin point (crack source)
    let origin = base_hash.xy;

    // Cracks radiate outward from origin
    let start_angle = base_hash.z * TAU;

    var prev = origin;
    var current_angle = start_angle;

    for (var s = 0u; s < seg_count; s++) {
        let seg_hash = trace_hash23(vec2f(seed + f32(s) * 11.3, trace_idx + f32(s) * 17.7));

        // Segment length (shorter near origin, longer as it spreads)
        let seg_len = 0.03 + f32(s) * 0.015 + seg_hash.x * 0.02;

        // Angle deviation (cracks follow stress patterns)
        let angle_dev = (seg_hash.y - 0.5) * jitter * PI * 0.3;
        current_angle += angle_dev;

        let dir = vec2f(cos(current_angle), sin(current_angle));
        let next = prev + dir * seg_len;

        let d = dist_to_segment(uv, prev, next);
        min_dist = min(min_dist, d);

        // Occasionally branch
        if seg_hash.z > 0.7 && s > 0u {
            let branch_hash = trace_hash23(vec2f(seed + f32(s) * 29.1, trace_idx * 43.3));
            let branch_angle = current_angle + (select(-1.0, 1.0, branch_hash.x > 0.5)) * PI * 0.3;
            let branch_dir = vec2f(cos(branch_angle), sin(branch_angle));
            let branch_len = seg_len * (0.5 + branch_hash.y * 0.5);
            let branch_end = prev + branch_dir * branch_len;
            let bd = dist_to_segment(uv, prev, branch_end);
            min_dist = min(min_dist, bd);
        }

        prev = next;
    }

    return vec2f(min_dist, min_dist);
}

// Generate lead lines (stained glass polygons)
fn lead_lines_trace(uv: vec2f, seed: f32, trace_idx: f32, vertex_count: u32, jitter: f32, thickness: f32) -> vec2f {
    var min_dist = 1000.0;

    let base_hash = trace_hash23(vec2f(seed * 23.1, trace_idx * 41.3));

    // Polygon center
    let center = base_hash.xy;
    let radius = 0.1 + base_hash.z * 0.15;

    // Generate polygon vertices
    let n = max(vertex_count, 3u);
    let angle_step = TAU / f32(n);
    let start_angle = base_hash.z * TAU;

    var prev_vertex = vec2f(0.0);

    for (var v = 0u; v <= n; v++) {
        let vidx = v % n;
        let vert_hash = trace_hash23(vec2f(seed + f32(vidx) * 7.3, trace_idx + f32(vidx) * 11.7));

        // Vertex position with jitter
        let angle = start_angle + f32(vidx) * angle_step;
        let r = radius * (1.0 + (vert_hash.x - 0.5) * jitter * 0.5);
        let vertex = center + vec2f(cos(angle), sin(angle)) * r;

        if v > 0u {
            let d = dist_to_segment(uv, prev_vertex, vertex);
            min_dist = min(min_dist, d);
        }

        prev_vertex = vertex;
    }

    return vec2f(min_dist, min_dist);
}

// Generate filament (organic spline curve)
fn filament_trace(uv: vec2f, seed: f32, trace_idx: f32, point_count: u32, jitter: f32, thickness: f32) -> vec2f {
    var min_dist = 1000.0;

    let base_hash = trace_hash23(vec2f(seed * 29.7, trace_idx * 47.3));

    // Generate control points for the filament
    let n = max(point_count, 2u);

    var prev = base_hash.xy;
    let flow_dir = normalize(vec2f(base_hash.z - 0.5, base_hash.x - 0.5));

    for (var p = 1u; p < n; p++) {
        let pt_hash = trace_hash23(vec2f(seed + f32(p) * 13.7, trace_idx + f32(p) * 19.3));

        // Smooth flowing motion with jitter
        let step_len = 0.08 + pt_hash.x * 0.06;
        let curve = (pt_hash.y - 0.5) * jitter * PI * 0.4;
        let c = cos(curve);
        let s = sin(curve);
        let dir = vec2f(
            flow_dir.x * c - flow_dir.y * s,
            flow_dir.x * s + flow_dir.y * c
        );

        let next = prev + dir * step_len;

        // For smooth filaments, we approximate with line segments
        // A proper implementation would use Catmull-Rom or Bezier
        let d = dist_to_segment(uv, prev, next);
        min_dist = min(min_dist, d);

        prev = next;
    }

    return vec2f(min_dist, min_dist);
}

fn eval_trace(
    dir: vec3f,
    instr: vec4u,
    region_w: f32,
    time: f32
) -> LayerSample {
    if region_w < 0.001 { return LayerSample(vec3f(0.0), 0.0); }

    // Extract domain and variant from meta5
    let domain_id = instr_domain_id(instr);
    let variant_id = instr_variant_id(instr);

    // Extract parameters
    let trace_count = 1u + (instr_a(instr) * 15u) / 255u; // 1..16 traces
    let thickness = mix(0.005, 0.1, u8_to_01(instr_b(instr)));
    let jitter = u8_to_01(instr_c(instr));

    let pd = instr_d(instr);
    let seed = f32((pd >> 4u) & 0xFu);
    let shape_q = pd & 0xFu;

    // Decode axis/center direction
    let axis_or_center = decode_dir16(instr_dir16(instr));

    // Map to 2D chart based on domain
    var uv = vec2f(0.0);
    var domain_w = 1.0;

    switch domain_id {
        case TRACE_DOMAIN_AXIS_CYL: {
            uv = trace_cyl_uv(dir, axis_or_center);
            // Pole fade at v near 0 or 1
            let pole_dist = min(uv.y, 1.0 - uv.y);
            domain_w = smoothstep(0.0, 0.15, pole_dist);
        }
        case TRACE_DOMAIN_AXIS_POLAR: {
            uv = trace_polar_uv(dir, axis_or_center);
            // Axis fade near center (rad near 0)
            domain_w = smoothstep(0.05, 0.2, uv.y);
        }
        case TRACE_DOMAIN_TANGENT_LOCAL: {
            let result = trace_tangent_uv(dir, axis_or_center);
            uv = result.xy;
            domain_w = result.z;
            if domain_w < 0.001 {
                return LayerSample(vec3f(0.0), 0.0);
            }
            // Remap tangent UV to [0,1] range (it's unbounded otherwise)
            uv = uv * 0.5 + 0.5;
        }
        default: {
            // DIRECT3D fallback - use cylindrical with Y-up
            uv = trace_cyl_uv(dir, vec3f(0.0, 1.0, 0.0));
            let pole_dist = min(uv.y, 1.0 - uv.y);
            domain_w = smoothstep(0.0, 0.15, pole_dist);
        }
    }

    // Accumulate minimum distance across all traces
    var min_line_dist = 1000.0;

    for (var i = 0u; i < trace_count; i++) {
        var trace_dists = vec2f(1000.0);

        switch variant_id {
            case TRACE_VARIANT_LIGHTNING: {
                // shape_q controls branch count (0..15 -> 0..8)
                let branch_count = (shape_q * 8u) / 15u;
                trace_dists = lightning_trace(uv, seed, f32(i), branch_count, jitter, thickness);
            }
            case TRACE_VARIANT_CRACKS: {
                // shape_q controls segment count (0..15 -> 3..18)
                let seg_count = 3u + shape_q;
                trace_dists = crack_trace(uv, seed, f32(i), seg_count, jitter, thickness);
            }
            case TRACE_VARIANT_LEAD_LINES: {
                // shape_q controls vertex count (0..15 -> 3..18)
                let vertex_count = 3u + shape_q;
                trace_dists = lead_lines_trace(uv, seed, f32(i), vertex_count, jitter, thickness);
            }
            case TRACE_VARIANT_FILAMENTS: {
                // shape_q controls control point count (0..15 -> 2..17)
                let point_count = 2u + shape_q;
                trace_dists = filament_trace(uv, seed, f32(i), point_count, jitter, thickness);
            }
            default: {
                // Fallback to lightning
                trace_dists = lightning_trace(uv, seed, f32(i), 2u, jitter, thickness);
            }
        }

        min_line_dist = min(min_line_dist, trace_dists.x);
    }

    // Compute line and glow masks with anti-aliasing
    // Use a simple smoothstep since fwidth is not available in all contexts
    let aa_width = 0.002;
    let line_mask = 1.0 - smoothstep(thickness - aa_width, thickness + aa_width, min_line_dist);
    let glow_mask = smoothstep(thickness * 4.0 + aa_width, thickness, min_line_dist) * (1.0 - line_mask);

    // Extract colors and alphas
    let line_rgb = instr_color_a(instr);
    let glow_rgb = instr_color_b(instr);
    let alpha_a = instr_alpha_a_f32(instr);
    let alpha_b = instr_alpha_b_f32(instr);

    // Blend colors
    let rgb = line_rgb * line_mask + glow_rgb * glow_mask;

    // Compute final weight
    let intensity = u8_to_01(instr_intensity(instr));
    let w = (line_mask * alpha_a + glow_mask * alpha_b) * intensity * domain_w * region_w;

    return LayerSample(rgb, w);
}
