//! Finite TRACE angular-cut regression. GPU evaluates the production shader.
use super::*;

#[test]
fn trace_wrap_nonperiodic_reference_and_reserved() {
    // Reuse production geometry/packing but replace the distance adapter with
    // the pre-repair flat primitive. No tmp shader copy or CPU hash oracle.
    let source = include_str!("../../shaders/epu/features/04_trace.wgsl").replace("\r\n", "\n");
    let start = source.find("fn trace_segment_distance(").unwrap();
    let end = start + source[start..].find("\n}\n").unwrap() + 3;
    let mut reference = source.to_string();
    reference.replace_range(start..end, "fn trace_segment_distance(p: vec2f, a: vec2f, b: vec2f, periodic: bool) -> f32 { return dist_to_segment(p,a,b); }\n");
    // Namespace every shader-local declaration, retaining canonical ABI helpers.
    for line in source.lines() {
        if let Some(rest) = line
            .strip_prefix("fn ")
            .or_else(|| line.strip_prefix("const "))
        {
            let name = rest.split(['(', ':']).next().unwrap();
            reference = reference.replace(name, &format!("reference_{name}"));
        }
    }
    let pixels = probe_image(
        &format!(
            "{reference}\n{}",
            r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    var domain = select(0u,3u,p.y >= 64u);
    let variant = (p.y/8u)%8u;
    if variant>=4u { domain=p.y%4u; }
    let seed = array<u32,4>(0u,1u,7u,15u)[(p.y/2u)%4u];
    let thick = select(0u,255u,p.y%2u==1u);
    let k = f32(p.x/2u);
    let dir = normalize(vec3f(cos(k*.71),sin(k*.37),cos(k*.17)));
    let instr = vec4u(((seed*16u+15u)<<24u)|255u,
        (255u<<24u)|(255u<<16u)|(thick<<8u)|255u,255u,
        (OP_TRACE<<27u)|(REGION_SKY<<24u)|(((domain<<3u)|variant)<<16u)|0xff00u);
    var s = eval_trace(dir,instr,1.);
    if p.x%2u==1u { s = reference_eval_trace(dir,instr,1.); }
    textureStore(result,p.xy,vec4f(s.rgb*s.w,s.w));
}
"#
        ),
        128,
        128,
    );
    let mut max_error = 0.0f32;
    let mut peaks = [[0.0f32; 2]; 8];
    for (i, pair) in pixels.as_chunks::<2>().0.iter().enumerate() {
        assert!(pair.iter().flatten().all(|v| v.is_finite()));
        let row = i / 64;
        let variant = (row / 8) % 8;
        for c in 0..4 {
            max_error = max_error.max((pair[0][c] - pair[1][c]).abs());
        }
        if variant >= 4 {
            assert_eq!(pair[0], [0.; 4], "reserved variant");
        } else {
            let group = (row / 64) * 4 + variant;
            peaks[group][0] = peaks[group][0].max(pair[0][0]);
            peaks[group][1] = peaks[group][1].max(pair[0][2]);
        }
    }
    println!(
        "TRACE nonperiodic source/flat reference max_error={max_error} line/glow peaks={peaks:?}; reserved variants zero"
    );
    assert!(max_error < 0.005);
    assert!(peaks.iter().flatten().all(|v| *v > 0.01));
}

#[test]
fn trace_wrap_segments_and_far_chains() {
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let points=array<vec2f,8>(vec2f(0.,.5),vec2f(.5,.5),vec2f(0.,1.25),vec2f(.25,.4),vec2f(.75,-.2),vec2f(1.,.5),vec2f(-1.,.5),vec2f(0.,2.));
    let starts=array<vec2f,8>(vec2f(.9,0.),vec2f(5.9,0.),vec2f(-6.1,0.),vec2f(-3.2,-.2),vec2f(.9,0.),vec2f(6.25,.4),vec2f(4.,.5),vec2f(0.,0.));
    let ends=array<vec2f,8>(vec2f(1.1,1.),vec2f(6.1,1.),vec2f(-5.9,1.),vec2f(4.3,1.4),vec2f(.9,1.),vec2f(6.25,.4),vec2f(-4.,.5),vec2f(0.,1.));
    let point=points[p.x];let a=starts[p.y];let b=ends[p.y];
    var oracle=1000.;
    for(var k=-16;k<=16;k++) { oracle=min(oracle,dist_to_segment(point+vec2f(f32(k),0.),a,b)); }
    textureStore(result,p.xy,vec4f(trace_segment_distance(point,a,b,true),oracle,
        trace_segment_distance(point,a,b,false),dist_to_segment(point,a,b)));
}
"#,
        8,
        8,
    );
    let mut error = 0.0f32;
    for p in &pixels {
        assert!(p.iter().all(|v| v.is_finite()));
        error = error.max((p[0] - p[1]).abs()).max((p[2] - p[3]).abs());
    }
    assert!(error < 0.005, "segment reference {error}");
    assert_eq!(
        pixels[0][0], 0.,
        "intact diagonal crosses seam, not wrapped vertices"
    );
    assert!(pixels[1][0] > 0.3, "no artificial long chord");
    assert_eq!(
        pixels[7 * 8 + 7][0],
        1.,
        "finite endpoint, not infinite line or y wrap"
    );
    println!(
        "TRACE long/translated/reversed/horizontal/vertical/degenerate segments: max_error={error}; finite endpoints preserved"
    );
    let pixels = probe_image(
        r#"
fn generated(uv:vec2f,v:u32,seed:f32,idx:f32,j:f32,periodic:bool)->f32 {
    switch v {
        case 0u: { return lightning_trace(uv,seed,idx,8u,j,periodic).x; }
        case 1u: { return crack_trace(uv,seed,idx,18u,j,periodic).x; }
        case 2u: { return lead_lines_trace(uv,seed,idx,18u,j,periodic).x; }
        default: { return filament_trace(uv,seed,idx,17u,j,periodic).x; }
    }
}
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let variant=p.y/32u;
    let seed=f32(array<u32,4>(0u,1u,7u,15u)[(p.y/8u)%4u]);
    let idx=f32(array<u32,4>(0u,1u,7u,15u)[(p.y/2u)%4u]);
    let jitter=f32(p.y%2u);
    let uv=vec2f(f32(p.x%8u)/7.,f32(p.x/8u)/3.);
    var oracle=1000.;
    for(var k=-8;k<=8;k++) { oracle=min(oracle,generated(uv+vec2f(f32(k),0.),variant,seed,idx,jitter,false)); }
    textureStore(result,p.xy,vec4f(generated(uv,variant,seed,idx,jitter,true),oracle,
        generated(uv+vec2f(5.,0.),variant,seed,idx,jitter,true),1.));
}
"#,
        32,
        128,
    );
    let mut error = 0.0f32;
    for p in pixels {
        assert!(p.iter().all(|v| v.is_finite()));
        error = error.max((p[0] - p[1]).abs()).max((p[0] - p[2]).abs());
    }
    println!(
        "TRACE maximum-shape generated chains and branches vs exhaustive +/-8 flat images, +5 query translation: max_error={error}"
    );
    assert!(error < 0.005);
}

#[test]
fn trace_wrap_angular_seams_and_controls() {
    // Frozen before RED: absolute weighted RGB .005, paired phi +/-PI at 1e-6.
    // Red line / blue glow separate nonzero controls; no saturated readback.
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let domain = 1u + p.y / 512u;
    let variant = (p.y / 128u) % 4u;
    let seed = array<u32,4>(0u,1u,7u,15u)[(p.y / 32u) % 4u];
    let shape = select(0u,15u,(p.y / 16u) % 2u == 1u);
    let jitter = select(0u,255u,(p.y / 8u) % 2u == 1u);
    let h = array<f32,8>(-0.9,-0.7,-0.4,0.,0.3,0.6,0.8,0.95)[p.y % 8u];
    var phi = select(-PI+0.000001,PI-0.000001,p.x == 1u);
    if p.x >= 2u { phi = (f32(p.x-2u)+0.37)/16.0*TAU-PI; }
    // Default Y basis: t=+Z, b=+X. atan2(dot(dir,t),dot(dir,b)).
    let dir = vec3f(sqrt(1.-h*h)*cos(phi),h,sqrt(1.-h*h)*sin(phi));
    let instr = vec4u(((seed*16u+shape)<<24u)|255u,
        (255u<<24u)|(255u<<8u)|jitter, 255u,
        (OP_TRACE<<27u)|(REGION_SKY<<24u)|(((domain<<3u)|variant)<<16u)|0xff00u);
    let s = evaluate_layer(dir,instr,vec3f(0.,1.,0.),RegionWeights(1.,0.,0.));
    textureStore(result,p.xy,vec4f(s.rgb*s.w,s.w));
}
"#,
        18,
        1024,
    );
    assert_eq!(pixels.len(), 18 * 1024);
    let mut failures = Vec::new();
    for group in 0..8 {
        let mut error = 0.0f32;
        let mut line = 0.0f32;
        let mut glow = 0.0f32;
        let mut witness = 0;
        for (row, p) in pixels[group * 128 * 18..(group + 1) * 128 * 18]
            .as_chunks::<18>()
            .0
            .iter()
            .enumerate()
        {
            assert!(p.iter().flatten().all(|v| v.is_finite()));
            for c in 0..3 {
                let d = (p[0][c] - p[1][c]).abs();
                if d > error {
                    error = d;
                    witness = row;
                }
            }
            for q in p {
                line = line.max(q[0]);
                glow = glow.max(q[2]);
            }
        }
        println!(
            "TRACE domain={} variant={} max_weighted_rgb_jump={error} witness_row={witness} line_peak={line} glow_peak={glow} tolerance=.005",
            1 + group / 4,
            group % 4
        );
        assert!(line > 0.01 && glow > 0.01, "blank line/glow group {group}");
        if error >= 0.005 {
            failures.push((group, error, witness));
        }
    }
    println!(
        "TRACE proposed CYL LEAD_LINES seed0 shape0 jitter0 h=-.9 pair={:?}",
        &pixels[256 * 18..256 * 18 + 2]
    );
    assert!(failures.is_empty(), "TRACE angular cut: {failures:?}");
}
