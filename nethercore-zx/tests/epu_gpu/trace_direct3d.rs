//! Fixed-axis spatial TRACE gate, not a graphics-backend test.
use super::*;

#[test]
fn trace_direct3d_boundaries_and_flat_reference() {
    // Frozen budget: final +/-1e-6 chart approach and same-side pairs <= .005.
    // Larger approaches characterize steep authored AA, not fabricated REDs.
    let source = include_str!("../../shaders/epu/features/04_trace.wgsl").replace("\r\n", "\n");
    let start = source.find("fn trace_segment_distance(").unwrap();
    let end = start + source[start..].find("\n}\n").unwrap() + 3;
    let mut reference = source.clone();
    reference.replace_range(start..end, "fn trace_segment_distance(p: vec2f, a: vec2f, b: vec2f, periodic: bool) -> f32 { return dist_to_segment(p,a,b); }\n");
    for line in source.lines() {
        if let Some(rest) = line
            .strip_prefix("fn ")
            .or_else(|| line.strip_prefix("const "))
        {
            let name = rest.split(['(', ':']).next().unwrap();
            reference = reference.replace(name, &format!("flat_{name}"));
        }
    }
    // Instrument only a test copy of the evaluator: actual production generators
    // still run; report the winning trace and distance before mask shaping.
    let mut diagnostic = source[source.find("fn eval_trace(").unwrap()..].to_string();
    for needle in [
        "var min_line_dist = 1000.0;",
        "min_line_dist = min(min_line_dist, trace_dists.x);",
        "return LayerSample(rgb, w);",
    ] {
        assert_eq!(diagnostic.matches(needle).count(), 1);
    }
    diagnostic = diagnostic.replace("fn eval_trace(", "fn diagnostic_trace(")
        .replace("var min_line_dist = 1000.0;", "var min_line_dist = 1000.0; var owner = 0u;")
        .replace("min_line_dist = min(min_line_dist, trace_dists.x);", "if trace_dists.x < min_line_dist { owner = i; } min_line_dist = min(min_line_dist, trace_dists.x);")
        .replace("return LayerSample(rgb, w);", "return LayerSample(vec3f(min_line_dist, f32(owner), f32(domain_id)), w);");
    let common = format!(
        "{reference}\n{diagnostic}\n{}",
        r#"
fn instruction(row:u32)->vec4u {
    let variant=row/6u;
    let high=(row/3u)%2u==1u;
    let pd=select(0u,127u,high); // seed0/7, shape0/15
    let jitter=select(0u,255u,high);
    let thick=select(0u,255u,high);
    // count byte51 -> four complete traces, red line / blue glow, default +Y.
    return vec4u((pd<<24u)|255u,(255u<<24u)|(51u<<16u)|(thick<<8u)|jitter,
        255u,(OP_TRACE<<27u)|(REGION_SKY<<24u)|(variant<<16u)|0xff00u);
}
fn direction(x:f32,row:u32)->vec3f {
    let y=array<f32,3>(-.3,0.,.3)[row%3u];
    let z=2.*x-1.;
    return vec3f(sqrt(1.-y*y-z*z),y,z);
}
fn diagnostic(x:f32,row:u32)->vec3f {
    return diagnostic_trace(direction(x,row),instruction(row),1.).rgb;
}
fn threshold(kind:u32,row:u32)->f32 {
    let t=select(.005,.1,(row/3u)%2u==1u);
    return array<f32,4>(0.,t-.002,t+.002,4.*t+.002)[kind];
}
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
"#
    );
    let grid = probe_image(
        &format!(
            "{common}\n{}",
            r#"
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let x=.1+.8*f32(p.x)/128.;
    let d=diagnostic(x,p.y);
    let s=evaluate_layer(direction(x,p.y),instruction(p.y),vec3f(0.,1.,0.),RegionWeights(1.,0.,0.));
    textureStore(result,p.xy,vec4f(d.xy,s.rgb.r*s.w,s.rgb.b*s.w));
}
"#
        ),
        129,
        24,
    );
    assert_eq!(grid.len(), 129 * 24);
    assert!(grid.iter().flatten().all(|v| v.is_finite()));
    let mut peaks = [[0.0f32; 2]; 4];
    let mut counts = [[0usize; 4]; 4];
    let mut witnesses = Vec::new();
    // Bounded discovery: first two brackets of each kind per scanline. No failed
    // refined witness is discarded. This is not exhaustive sub-grid coverage.
    for row in 0..24 {
        let line = &grid[row * 129..(row + 1) * 129];
        for p in line {
            for c in 0..2 {
                peaks[row / 6][c] = peaks[row / 6][c].max(p[2 + c]);
            }
        }
        let t = if (row / 3) % 2 == 1 { 0.1 } else { 0.005 };
        for kind in 0..4 {
            let target = [0., t - 0.002, t + 0.002, 4. * t + 0.002][kind];
            let mut selected = 0;
            for (i, pair) in line.windows(2).enumerate() {
                let crosses = if kind == 0 {
                    pair[0][1] != pair[1][1]
                } else {
                    (pair[0][0] < target) != (pair[1][0] < target)
                };
                if crosses && selected < 2 {
                    witnesses.push((
                        row,
                        kind,
                        0.1 + 0.8 * i as f32 / 128.,
                        0.1 + 0.8 * (i + 1) as f32 / 128.,
                    ));
                    counts[row / 6][kind] += 1;
                    selected += 1;
                }
            }
        }
    }
    println!("TRACE_DIRECT3D discovery counts={counts:?} line_glow_peaks={peaks:?}");
    assert!(peaks.iter().flatten().all(|v| *v > 0.01));
    assert!(
        peaks.iter().all(|v| v[0] > 0.99),
        "authored line cores lost"
    );
    for c in counts {
        assert!(
            c[0] > 0 && c[1] > 0 && c[2] > 0 && c[3] > 0,
            "missing boundary class: {c:?}"
        );
    }
    let cases = witnesses
        .iter()
        .map(|(r, k, a, b)| format!("vec4f({r}.,{k}.,{a:.9},{b:.9})"))
        .collect::<Vec<_>>()
        .join(",");
    let body = format!(
        "{common}\nconst CASES:array<vec4f,{}>=array<vec4f,{}>({cases});\n{}",
        witnesses.len(),
        witnesses.len(),
        r#"
fn predicate(x:f32,row:u32,kind:u32,owner:f32)->bool {
    let d=diagnostic(x,row);
    return select(d.x<threshold(kind,row),d.y==owner,kind==0u);
}
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let c=CASES[p.y];let row=u32(c.x);let kind=u32(c.y);
    var lo=c.z;var hi=c.w;let owner=diagnostic(lo,row).y;
    let side=predicate(lo,row,kind,owner);
    let valid=side!=predicate(hi,row,kind,owner);
    for(var i=0u;i<20u;i++) {
        let mid=(lo+hi)*.5;
        if predicate(mid,row,kind,owner)==side {lo=mid;} else {hi=mid;}
    }
    let center=(lo+hi)*.5;
    if p.x==40u {
        let a=diagnostic(center-.000001,row);let b=diagnostic(center+.000001,row);
        let crossed=predicate(center-.000001,row,kind,owner)!=predicate(center+.000001,row,kind,owner);
        textureStore(result,p.xy,vec4f(select(0.,1.,valid),select(0.,1.,crossed),a.x,b.x));return;
    }
    let eps=array<f32,4>(.001,.0001,.00001,.000001)[p.x/10u];
    let offset=f32((p.x%10u)/2u)-2.;
    let dir=direction(center+offset*eps,row);
    var s=evaluate_layer(dir,instruction(row),vec3f(0.,1.,0.),RegionWeights(1.,0.,0.));
    if p.x%2u==1u {s=flat_eval_trace(dir,instruction(row),1.);}
    textureStore(result,p.xy,vec4f(s.rgb*s.w,s.w));
}
"#
    );
    let pixels = probe_image(&body, 41, witnesses.len() as u32);
    assert_eq!(pixels.len(), 41 * witnesses.len());
    assert!(pixels.iter().flatten().all(|v| v.is_finite()));
    let mut approach = [0.0f32; 4];
    let mut same_side = [0.0f32; 4];
    let mut parity = 0.0f32;
    let mut csv = String::from("witness,row,kind,lo,hi,pixel,r,g,b,w\n");
    for (n, (samples, (row, kind, lo, hi))) in pixels
        .as_chunks::<41>()
        .0
        .iter()
        .zip(&witnesses)
        .enumerate()
    {
        assert_eq!(samples[40][0], 1., "GPU bracket invalid {n}");
        assert_eq!(samples[40][1], 1., "boundary not demonstrated {n}");
        assert!(
            (samples[40][2] - samples[40][3]).abs() <= 0.005,
            "raw distance discontinuity {n}"
        );
        let mut jumps = [0.0f32; 4];
        for scale in 0..4 {
            let s = &samples[scale * 10..scale * 10 + 10];
            for c in 0..4 {
                jumps[scale] = jumps[scale].max((s[2][c] - s[6][c]).abs());
                same_side[scale] = same_side[scale]
                    .max((s[0][c] - s[2][c]).abs())
                    .max((s[6][c] - s[8][c]).abs());
            }
            approach[scale] = approach[scale].max(jumps[scale]);
            if scale > 0 {
                assert!(
                    jumps[scale] <= jumps[scale - 1] * 0.35 + 0.0005,
                    "nonconverging approach {n}: {jumps:?}"
                );
            }
            for pair in s.as_chunks::<2>().0.iter() {
                for c in 0..4 {
                    parity = parity.max((pair[0][c] - pair[1][c]).abs());
                }
            }
        }
        assert!(jumps[3] <= 0.005, "continuity witness {n}: {jumps:?}");
        assert!(
            jumps[3] <= jumps[0] + 0.0005,
            "nonconverging witness {n}: {jumps:?}"
        );
        // Intended compact glow support remains zero, rather than blurred away.
        if *kind == 3 {
            let final_s = &samples[30..40];
            assert!(
                final_s[2] == [0.; 4] || final_s[6] == [0.; 4],
                "support extended {n}"
            );
        }
        println!(
            "TRACE_DIRECT3D witness={n} row={row} kind={kind} bracket=[{lo},{hi}] approach={jumps:?}"
        );
        for (i, p) in samples.iter().enumerate() {
            csv.push_str(&format!(
                "{n},{row},{kind},{lo},{hi},{i},{},{},{},{}\n",
                p[0], p[1], p[2], p[3]
            ));
        }
    }
    println!(
        "TRACE_DIRECT3D boundaries={} approach={approach:?} same_side={same_side:?} flat_reference_max={parity} tolerance=.005",
        witnesses.len()
    );
    assert!(same_side[3] <= 0.005);
    assert_eq!(parity, 0., "angular repair changed spatial domain");
    if let Ok(path) = std::env::var("TRACE_DIRECT3D_OUTPUT") {
        std::fs::write(path, csv).unwrap();
    }
}
