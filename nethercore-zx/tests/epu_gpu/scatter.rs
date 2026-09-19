//! Angular SCATTER distributions must rotate with their authored axis.
use super::*;

#[test]
fn scatter_pole_coordinates_are_finite_and_preserve_nonpoles() {
    // Frozen pre-fix helpers: use only at nonzero radii, where normalize is defined.
    let legacy = r#"fn pole_baseline_cyl_uv(dir: vec3f, axis: vec3f) -> vec2f {
    let up = normalize(axis);
    let v = dot(dir, up);
    let proj = normalize(dir - up * v);
    // Reference for azimuth
    var right = cross(up, vec3f(0.0, 0.0, 1.0));
    if length(right) < 0.01 {
        right = cross(up, vec3f(1.0, 0.0, 0.0));
    }
    right = normalize(right);
    let fwd = cross(right, up);
    let u = atan2(dot(proj, fwd), dot(proj, right)) / TAU;
    return vec2f(u, v);
}

// Polar UV mapping for scatter domain
fn pole_baseline_polar_uv(dir: vec3f, axis: vec3f) -> vec2f {
    let up = normalize(axis);
    let v = dot(dir, up);
    let rad = sqrt(max(0.0, 1.0 - v * v));
    let proj = normalize(dir - up * v);
    var right = cross(up, vec3f(0.0, 0.0, 1.0));
    if length(right) < 0.01 {
        right = cross(up, vec3f(1.0, 0.0, 0.0));
    }
    right = normalize(right);
    let fwd = cross(right, up);
    let angle = atan2(dot(proj, fwd), dot(proj, right)) / TAU;
    return vec2f(angle, rad);
}"#;
    let body = r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p: vec3u) {
    let case_id = p.y / 16u;
    let axis = array<vec3f,6>(vec3f(1.,0.,0.),vec3f(-1.,0.,0.),
        vec3f(0.,1.,0.),vec3f(0.,-1.,0.),vec3f(0.,0.,1.),vec3f(0.,0.,-1.))[case_id%6u];
    let sign = select(1.,-1.,case_id>=6u);
    let radius = array<f32,6>(0.,0.00001,0.001,0.05,0.2,0.8)[p.x/2u];
    var right = cross(axis,vec3f(0.,0.,1.));
    if length(right)<0.01 { right=cross(axis,vec3f(1.,0.,0.)); }
    right=normalize(right);
    let forward=cross(right,axis);
    let angle=(f32(p.y%16u)+0.37)/16.*TAU;
    let dir=normalize(axis*(sign*sqrt(1.-radius*radius))
        +radius*(right*cos(angle)+forward*sin(angle)));
    let cyl=scatter_cyl_uv(dir,axis);
    let polar=scatter_polar_uv(dir,axis);
    var out=vec4f(cyl,polar);
    if p.x%2u==1u {
        if radius==0. {
            out=vec4f(select(0.,1.,cyl.x==0.),select(0.,1.,cyl.y==sign),
                select(0.,1.,polar.x==0.),select(0.,1.,polar.y==0.));
        } else {
            let old_cyl=pole_baseline_cyl_uv(dir,axis);
            let old_polar=pole_baseline_polar_uv(dir,axis);
            // Compare GPU f32 before half storage; shifted azimuths must reject.
            out=vec4f(select(0.,1.,all(cyl==old_cyl)),select(0.,1.,all(polar==old_polar)),
                select(0.,1.,any(cyl!=old_cyl+vec2f(0.125,0.))),
                select(0.,1.,any(polar!=old_polar+vec2f(0.125,0.))));
        }
    }
    textureStore(result,p.xy,out);
}
"#;
    let pixels = probe_image(&format!("{legacy}\n{body}"), 12, 12 * 16);
    assert_eq!(pixels.len(), 12 * 12 * 16);
    for (y, row) in pixels.as_chunks::<12>().0.iter().enumerate() {
        for radius in 0..6 {
            let actual = row[radius * 2];
            assert!(
                actual.iter().all(|v| v.is_finite()),
                "row={y} radius={radius} {actual:?}"
            );
            assert_eq!(
                row[radius * 2 + 1],
                [1.0; 4],
                "pole/unchanged/wrong-angle gate row={y} radius={radius}"
            );
        }
        let sign = if y / 16 < 6 { 1.0 } else { -1.0 };
        assert_eq!(row[0], [0.0, sign, 0.0, 0.0], "canonical pole row={y}");
    }
    println!(
        "SCATTER_POLE_COORDS exact_poles={} nonpole_pairs={} shifted_controls={} current_nonfinite=0 f32_nonpole_differences=0",
        12 * 16,
        12 * 16 * 5,
        12 * 16 * 5 * 2
    );
}

#[test]
fn scatter_exact_point_centers_keep_their_core() {
    let source = include_str!("../../shaders/epu/features/02_scatter.wgsl");
    let evaluator = &source[source.find("fn eval_scatter(").unwrap()..];
    let current = "atan2(length(cross(dir_s, point_dir)), dot(dir_s, point_dir))";
    assert!(evaluator.contains(current));
    let legacy = evaluator
        .replace("fn eval_scatter(", "fn legacy_eval_scatter(")
        .replace(current, "acos(epu_saturate(dot(dir_s, point_dir)))");
    let body = r#"
fn guest_brightness(phase:u32) -> u32 {
 let p=phase%256u; let x=min(p,256u-p);
 return 48u+(x*x*(384u-2u*x)*192u+1048576u)/2097152u;
}
fn fixture(seed:u32,a:u32,b:u32) -> vec4u {
 let rgb_a=a*65793u; let rgb_b=b*65793u;
 return vec4u((seed<<24u)|(0x8080u<<8u)|0xf0u,
  (255u<<24u)|(15u<<16u)|(24u<<8u),
  ((rgb_a&255u)<<24u)|rgb_b,(OP_SCATTER<<27u)|(7u<<24u)|(rgb_a>>8u));
}
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let x=f32(i32(p.y%8u)-4); let y=f32(i32(p.y/8u)-4);
 let cell=vec3f(x,y,floor(sqrt(256.-x*x-y*y)));
 let h=hash3(cell+vec3f(53.)); let dir=normalize(cell+h.xyz-.5);
 var fixed=fixture(53u,255u,255u);fixed.y=fixed.y&0xffff00ffu;
let value=eval_scatter(dir,fixed,1.);
let legacy=legacy_eval_scatter(dir,fixed,1.);
textureStore(result,p.xy,vec4f(value.w,legacy.w,0.,1.));}
"#;
    let pixels = probe_image(&format!("{legacy}\n{body}"), 1, 64);
    assert_eq!(pixels.len(), 64);
    let failures = pixels.iter().filter(|p| p[0] < 0.999).count();
    let legacy_failures = pixels.iter().filter(|p| p[1] < 0.999).count();
    assert!(pixels.iter().flatten().all(|v| v.is_finite()));
    println!(
        "SCATTER_POINT_CORES cases={} failures={} legacy_failures={}",
        pixels.len(),
        failures,
        legacy_failures
    );
    assert_eq!(
        failures, 0,
        "an exact seeded point center must retain its full core"
    );
    assert!(
        legacy_failures > 0,
        "legacy acos control must expose center attenuation"
    );
}

#[test]
fn scatter_color_endpoints_admit_stable_guest_twinkle() {
    // Existing color interpolation is sufficient: no seed/shape/ABI changes.
    let pixels = probe_image(
        r#"
fn guest_brightness(phase:u32) -> u32 {
 let p=phase%256u; let x=min(p,256u-p);
 return 48u+(x*x*(384u-2u*x)*192u+1048576u)/2097152u;
}
fn fixture(seed:u32,a:u32,b:u32) -> vec4u {
 let rgb_a=a*65793u; let rgb_b=b*65793u;
 return vec4u((seed<<24u)|(0x8080u<<8u)|0xf0u,
  (255u<<24u)|(15u<<16u)|(24u<<8u),
  ((rgb_a&255u)<<24u)|rgb_b,(OP_SCATTER<<27u)|(7u<<24u)|(rgb_a>>8u));
}
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let x=f32(i32(p.y%8u)-4); let y=f32(i32(p.y/8u)-4);
 let cell=vec3f(x,y,floor(sqrt(256.-x*x-y*y)));
 let h=hash3(cell+vec3f(53.)); let dir=normalize(cell+h.xyz-.5);
 let a=guest_brightness(p.x); let b=guest_brightness(p.x+85u);
 let value=eval_scatter(dir,fixture(53u,a,b),1.);
 let moved=eval_scatter(dir,fixture(p.x,a,b),1.);
 let global=eval_scatter(dir,fixture(53u,a,a),1.);
 textureStore(result,p.xy,vec4f(value.rgb.x,value.w,moved.w,global.rgb.x));
}
"#,
        256,
        64,
    );
    let mut peaks = Vec::new();
    let mut global_peaks = Vec::new();
    let mut wrong_seed = 0;
    let mut max_step = 0.0f32;
    for row in pixels.as_chunks::<256>().0.iter() {
        assert!(row.iter().flatten().all(|x| x.is_finite()));
        let mut peak = 0;
        let mut global_peak = 0;
        let mut minimum = row[0][0];
        for phase in 0..256 {
            let p = row[phase];
            assert!(p[1] > 0.01);
            assert_eq!(p[1], row[0][1], "color driver changed point geometry");
            if p[0] > row[peak][0] {
                peak = phase;
            }
            if p[3] > row[global_peak][3] {
                global_peak = phase;
            }
            minimum = minimum.min(p[0]);
            wrong_seed += usize::from((p[2] - row[0][1]).abs() > 0.001);
            max_step = max_step.max((row[(phase + 1) % 256][0] - p[0]).abs());
        }
        assert!(row[peak][0] - minimum > 0.1, "static output is not twinkle");
        peaks.push(peak);
        global_peaks.push(global_peak);
    }
    let peak_span = peaks.iter().max().unwrap() - peaks.iter().min().unwrap();
    assert!(
        peak_span >= 32,
        "synchronized global flashing is not per-point variation"
    );
    assert!(
        max_step <= 3.0 / 255.0 + 0.001,
        "including the wrap-adjacent step"
    );
    assert!(
        wrong_seed > 0,
        "reseeding control must move the point field"
    );
    assert!(global_peaks.iter().all(|p| *p == global_peaks[0]));
    println!(
        "SCATTER existing endpoint-color driver: 64 points x256 phases; geometry exact; peak_span={peak_span}; max_step={max_step}; rejected_seed_samples={wrong_seed}"
    );
}

#[test]
fn scatter_variant_finite_support() {
    // Shape-level support, not neighborhood completeness. Keep raw unsaturated
    // profiles: a white/clamped field must not conceal a discontinuous edge.
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let variant = p.y % 7u;
    let size = array<f32,3>(0.001,0.05,0.5)[p.y / 7u] * scatter_size_mult(variant);
    var support = size;
    if variant == 5u { support *= 3.0; }
    if variant == 4u { support *= 2.0; }
    let offset = array<f32,4>(-0.00001,0.00001,0.0,1.0)[p.x];
    var dist = support * (1.0 + offset);
    if p.x == 2u { dist = 0.0; }
    let h = vec4f(0.27,0.43,0.61,0.79);
    textureStore(result,p.xy,vec4f(scatter_point_shape(variant,dist,size,h),
        scatter_twinkle_mod(variant,0.0,h.w),scatter_twinkle_mod(variant,1.0,h.w),1.0));
}
"#,
        4,
        21,
    );
    let mut max_jump = 0.0f32;
    for (case, row) in pixels.as_chunks::<4>().0.iter().enumerate() {
        assert!(row.iter().flatten().all(|v| v.is_finite()));
        max_jump = max_jump.max((row[0][0] - row[1][0]).abs());
        assert_eq!(row[1][0], 0.0, "outside support case {case}");
        assert_eq!(row[3][0], 0.0, "far outside case {case}");
        assert!(row[2][0] >= 1.0, "nonzero core case {case}");
        assert_eq!(row[2][1], 1.0, "twinkle disabled identity");
        assert!(row[2][2] >= 0.0 && row[2][2] <= 1.0);
        if case % 7 == 2 {
            assert_eq!(row[2][2], 1.0, "WINDOWS no twinkle");
        }
    }
    println!(
        "SCATTER 21 variant/size support cases max_jump={max_jump} tolerance=.001; nonzero raw cores preserved"
    );
    assert!(max_jump < 0.001);
}

#[test]
fn scatter_angular_distribution_follows_axis() {
    // Columns: cylindrical Y/X, polar Y/X; rows sweep local azimuth/elevation.
    // Both axes receive identical local directions, seed, density and point size.
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let domain = 1u + p.x / 2u;
    let axis_bits = select(0xff80u, 0x80ffu, (p.x % 2u) == 1u);
    let axis = decode_dir16(axis_bits);
    let right = normalize(cross(axis, vec3f(0.,0.,1.)));
    let forward = cross(right, axis);
    let angle = (f32(p.y % 32u)+0.37)/32.0 * TAU - PI;
    let h = mix(-0.7,0.7,f32(p.y / 32u)/7.0);
    let dir = normalize(axis*h + sqrt(1.0-h*h)*(right*cos(angle)+forward*sin(angle)));
    let instr = vec4u((axis_bits<<8u)|255u, (255u<<24u)|(2u<<16u)|(255u<<8u),
        0xffffffffu, (OP_SCATTER<<27u)|(7u<<24u)|(((domain<<3u)|1u)<<16)|65535u);
    let sample = eval_scatter(dir,instr,1.0);
    textureStore(result,p.xy,vec4f(sample.rgb * epu_saturate(sample.w),1.0));
}
"#,
        4,
        256,
    );
    for domain in 0..2 {
        let mut max_error = 0.0f32;
        let mut peak = 0.0f32;
        for row in pixels.as_chunks::<4>().0.iter() {
            for c in 0..3 {
                let a = row[domain * 2][c];
                let b = row[domain * 2 + 1][c];
                assert!(a.is_finite() && b.is_finite());
                max_error = max_error.max((a - b).abs());
                peak = peak.max(a.max(b));
            }
        }
        println!(
            "SCATTER angular domain={} rotation error={max_error} peak={peak}",
            domain + 1
        );
        assert!(peak > 0.01, "empty output is not rotational correctness");
        assert!(
            max_error < 0.005,
            "axis-relative distribution depends on world orientation: domain={} error={max_error}",
            domain + 1
        );
    }
}

#[test]
fn scatter_angular_wrap_continuity() {
    const TOLERANCE: f32 = 0.005;
    const CASES_PER_DOMAIN: usize = 64;
    const DOMAIN_COUNT: usize = 2;
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let domain_index = p.y / 64u;
    let domain = 1u + domain_index;
    let case_id = p.y % 64u;
    let elevation_id = case_id % 4u;
    let seed = case_id / 4u;
    let density_code = 4u + (case_id % 5u);
    let size_code = 96u + ((case_id * 29u) % 96u);
    let axis_bits = select(0xff80u, 0x80ffu, domain_index == 1u);
    let axis = decode_dir16(axis_bits);
    var right = cross(axis, vec3f(0.,0.,1.));
    if length(right) < 0.01 { right = cross(axis, vec3f(1.,0.,0.)); }
    right = normalize(right);
    let forward = cross(right, axis);
    let h = mix(-0.75, 0.75, f32(elevation_id) / 3.0);
    let eps = 0.00001;
    let phi = select(-PI + eps, PI - eps, p.x == 1u);
    let dir = normalize(axis*h + sqrt(1.0-h*h)*(right*cos(phi)+forward*sin(phi)));
    let instr = vec4u((seed<<24u)|(axis_bits<<8u)|255u,
        (255u<<24u)|(density_code<<16u)|(size_code<<8u)|255u,
        0xffffffffu, (OP_SCATTER<<27u)|(7u<<24u)|(((domain<<3u)|1u)<<16u)|65535u);
    let sample = eval_scatter(dir,instr,1.0);
    textureStore(result,p.xy,vec4f(sample.rgb * epu_saturate(sample.w),1.0));
}
"#,
        2,
        (DOMAIN_COUNT * CASES_PER_DOMAIN) as u32,
    );

    let mut max_errors = [0.0f32; DOMAIN_COUNT];
    let mut worst_inputs = [(0usize, 0usize, 0usize, 0usize, 0usize); DOMAIN_COUNT];
    for domain_index in 0..DOMAIN_COUNT {
        let mut max_error = 0.0f32;
        let mut worst_input = (0usize, 0usize, 0usize, 0usize, 0usize);
        let mut aggregate = 0.0f32;
        for case_id in 0..CASES_PER_DOMAIN {
            let row = domain_index * CASES_PER_DOMAIN + case_id;
            let left = pixels[row * 2];
            let right = pixels[row * 2 + 1];
            for sample in [left, right] {
                assert!(sample.iter().all(|channel| channel.is_finite()));
                aggregate += sample[..3].iter().map(|channel| channel.abs()).sum::<f32>();
            }
            let error = left[..3]
                .iter()
                .zip(right[..3].iter())
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f32, f32::max);
            if error > max_error {
                max_error = error;
                let elevation = case_id % 4;
                let seed = case_id / 4;
                let density_code = 4 + (case_id % 5);
                let size_code = 96 + ((case_id * 29) % 96);
                worst_input = (case_id, elevation, seed, density_code, size_code);
            }
        }
        max_errors[domain_index] = max_error;
        worst_inputs[domain_index] = worst_input;
        println!(
            "SCATTER angular wrap domain={} max_error={} worst_input=(case={}, elevation={}, seed={}, density_u8={}, size_u8={}) aggregate={}",
            domain_index + 1,
            max_error,
            worst_input.0,
            worst_input.1,
            worst_input.2,
            worst_input.3,
            worst_input.4,
            aggregate
        );
        assert!(
            aggregate > 0.01,
            "domain {} produced empty aggregate output",
            domain_index + 1
        );
    }
    for domain_index in 0..DOMAIN_COUNT {
        assert!(
            max_errors[domain_index] < TOLERANCE,
            "angular wrap discontinuity: domain={} max_error={} worst_input={:?} tolerance={TOLERANCE}",
            domain_index + 1,
            max_errors[domain_index],
            worst_inputs[domain_index]
        );
    }
}

#[test]
fn scatter_tangent_distribution_follows_axis() {
    const TOLERANCE: f32 = 0.005;
    const CASES: usize = 64;
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let axis_bits = select(0xff80u, 0x80ffu, p.x == 1u);
    let axis = decode_dir16(axis_bits);
    var right = cross(axis, vec3f(0.,0.,1.));
    if length(right) < 0.01 { right = cross(axis, vec3f(1.,0.,0.)); }
    right = normalize(right);
    let forward = cross(right, axis);
    let case_id = p.y;
    let seed = case_id % 8u;
    let density_code = 1u + ((case_id * 11u) % 8u);
    let axis_dot = mix(0.96, 0.995, f32(case_id % 8u) / 7.0);
    let tangent = sqrt(1.0 - axis_dot * axis_dot);
    let angle = (f32(case_id) * 0.731) * TAU;
    let dir = normalize(axis * axis_dot
        + tangent * (right * cos(angle) + forward * sin(angle)));
    let instr = vec4u((seed<<24u)|(axis_bits<<8u)|255u,
        (255u<<24u)|(density_code<<16u)|(255u<<8u)|255u,
        0xffffffffu, (OP_SCATTER<<27u)|(7u<<24u)|(((3u<<3u)|1u)<<16)|65535u);
    let sample = eval_scatter(dir,instr,1.0);
    textureStore(result,p.xy,vec4f(sample.rgb * epu_saturate(sample.w),1.0));
}
"#,
        2,
        CASES as u32,
    );

    let mut max_errors = [0.0f32; 2];
    let mut peaks = [0.0f32; 2];
    let mut aggregates = [0.0f32; 2];
    let mut worst_inputs = [(0usize, 0usize, 0usize, 0usize, 0usize); 2];
    for axis_index in 0..2 {
        let mut max_error = 0.0f32;
        let mut peak = 0.0f32;
        let mut aggregate = 0.0f32;
        let mut worst_input = (0usize, 0usize, 0usize, 0usize, 0usize);
        for case_id in 0..CASES {
            let sample = pixels[case_id * 2 + axis_index];
            assert!(sample.iter().all(|channel| channel.is_finite()));
            aggregate += sample[..3].iter().map(|channel| channel.abs()).sum::<f32>();
            peak = peak.max(sample[..3].iter().copied().fold(0.0f32, f32::max));

            let other = pixels[case_id * 2 + (1 - axis_index)];
            let error = sample[..3]
                .iter()
                .zip(other[..3].iter())
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f32, f32::max);
            if error > max_error {
                max_error = error;
                let seed = case_id % 8;
                let density_code = 1 + ((case_id * 11) % 8);
                let angle_index = case_id;
                let axis_dot_milli = 960 + ((case_id % 8) * 5);
                worst_input = (case_id, seed, density_code, angle_index, axis_dot_milli);
            }
        }
        max_errors[axis_index] = max_error;
        peaks[axis_index] = peak;
        aggregates[axis_index] = aggregate;
        worst_inputs[axis_index] = worst_input;
        println!(
            "SCATTER tangent-local axis={} peak={} aggregate={} max_error={} worst_input=(case={}, seed={}, density_u8={}, angle_case={}, axis_dot_milli={})",
            if axis_index == 0 { "Y" } else { "X" },
            peak,
            aggregate,
            max_error,
            worst_input.0,
            worst_input.1,
            worst_input.2,
            worst_input.3,
            worst_input.4
        );
        assert!(
            aggregate > 0.01,
            "axis {} produced empty aggregate output",
            if axis_index == 0 { "Y" } else { "X" }
        );
    }
    for axis_index in 0..2 {
        assert!(
            max_errors[axis_index] < TOLERANCE,
            "tangent-local distribution depends on world orientation: axis={} peak={} aggregate={} max_error={} worst_input={:?} tolerance={TOLERANCE}",
            if axis_index == 0 { "Y" } else { "X" },
            peaks[axis_index],
            aggregates[axis_index],
            max_errors[axis_index],
            worst_inputs[axis_index]
        );
    }
}

#[test]
fn scatter_tangent_cell_boundary_continuity() {
    const TOLERANCE: f32 = 0.005;
    const OTHER_COORDS: [f32; 4] = [-0.2, -0.1, 0.1, 0.2];
    const BOUNDARIES: [f32; 3] = [-0.5, 0.0, 0.5];
    const CASES_PER_AXIS: usize = 2 * BOUNDARIES.len() * OTHER_COORDS.len();
    const AXES: usize = 2;
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let axis_index = p.y / 24u;
    let local_case = p.y % 24u;
    let boundary_axis = local_case / 12u;
    let boundary_index = (local_case / 4u) % 3u;
    let other_index = local_case % 4u;
    let axis_bits = select(0xff80u, 0x80ffu, axis_index == 1u);
    let axis = decode_dir16(axis_bits);
    var right = cross(axis, vec3f(0.0, 0.0, 1.0));
    if length(right) < 0.01 { right = cross(axis, vec3f(1.0, 0.0, 0.0)); }
    right = normalize(right);
    let forward = cross(right, axis);
    let eps = select(-0.00001, 0.00001, p.x == 1u);
    let boundary = array<f32, 3>(-0.5, 0.0, 0.5)[boundary_index] + eps;
    let other = array<f32, 4>(-0.2, -0.1, 0.1, 0.2)[other_index];
    let local = select(vec2f(boundary, other), vec2f(other, boundary), boundary_axis == 1u);
    let dir = normalize(axis * sqrt(max(0.0, 1.0 - dot(local, local)))
        + right * local.x + forward * local.y);
    let instr = vec4u((axis_bits << 8u) | 255u,
        (255u << 24u) | (1u << 16u) | (255u << 8u) | 255u,
        0xffffffffu,
        (OP_SCATTER << 27u) | (7u << 24u) | (((3u << 3u) | 1u) << 16u) | 65535u);
    let sample = eval_scatter(dir, instr, 1.0);
    let domain_w = smoothstep(0.8, 0.95, dot(dir, axis));
    textureStore(result, p.xy, vec4f(sample.rgb * epu_saturate(sample.w), domain_w));
}
"#,
        2,
        (AXES * CASES_PER_AXIS) as u32,
    );

    let mut max_error = 0.0f32;
    let mut aggregate = 0.0f32;
    let mut worst = (0usize, 0usize, 0usize, 0usize, 0.0f32, [0.0f32; 2]);
    for row in 0..(AXES * CASES_PER_AXIS) {
        let pair = &pixels[row * 2..row * 2 + 2];
        for sample in pair {
            assert!(sample.iter().all(|channel| channel.is_finite()));
            aggregate += sample[..3].iter().map(|channel| channel.abs()).sum::<f32>();
        }
        let error = pair[0][..3]
            .iter()
            .zip(pair[1][..3].iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        if error > max_error {
            let axis_index = row / CASES_PER_AXIS;
            let local_case = row % CASES_PER_AXIS;
            let boundary_axis = local_case / 12;
            let boundary_index = (local_case / 4) % 3;
            let other_index = local_case % 4;
            max_error = error;
            worst = (
                axis_index,
                boundary_axis,
                boundary_index,
                other_index,
                error,
                [pair[0][3], pair[1][3]],
            );
        }
    }
    println!(
        "SCATTER tangent cell boundary continuity max_error={max_error} worst=(axis_bits=0x{:04x}, boundary_axis={}, boundary={}, other={}, local_minus=({:.8},{:.8}), local_plus=({:.8},{:.8}), boundary_index={}, other_index={}, domain_w=({:.8},{:.8}), inputs=(density_u8=1, size_u8=255, intensity_u8=255, param_c_u8=255, variant=1, seed=0, color_a=0xffffff, color_b=0xffffff)) aggregate={aggregate}",
        if worst.0 == 0 { 0xff80 } else { 0x80ff },
        worst.1,
        BOUNDARIES[worst.2],
        OTHER_COORDS[worst.3],
        if worst.1 == 0 {
            BOUNDARIES[worst.2] - 0.00001
        } else {
            OTHER_COORDS[worst.3]
        },
        if worst.1 == 0 {
            OTHER_COORDS[worst.3]
        } else {
            BOUNDARIES[worst.2] - 0.00001
        },
        if worst.1 == 0 {
            BOUNDARIES[worst.2] + 0.00001
        } else {
            OTHER_COORDS[worst.3]
        },
        if worst.1 == 0 {
            OTHER_COORDS[worst.3]
        } else {
            BOUNDARIES[worst.2] + 0.00001
        },
        worst.2,
        worst.3,
        worst.5[0],
        worst.5[1],
    );
    assert!(
        aggregate > 0.01,
        "tangent boundary probe produced empty aggregate"
    );
    assert!(
        max_error < TOLERANCE,
        "tangent cell boundary discontinuity: max_error={max_error} worst={worst:?} tolerance={TOLERANCE}"
    );
}
