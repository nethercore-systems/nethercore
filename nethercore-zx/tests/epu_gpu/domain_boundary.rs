//! Finite angular singularity / hard-threshold checks; no whole-domain acceptance.
use super::*;

#[test]
fn domain_boundary_scatter_angular_limits() {
    // DUST, density=1, size=255, no twinkle: large nonzero footprints.
    // Each group sweeps 16 azimuths and four seeds, both pole signs.
    // Negative/exact/positive limits, unfaded core and nonzero fade interior.
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let group = p.y / 128u;
    let case_id = p.y % 128u;
    let domain = select(1u,2u,group >= 3u);
    let axis_bits = 0xffffu; // exactly -Z after oct decode: exact degeneracy is reachable
    let axis = decode_dir16(axis_bits);
    let right = vec3f(0.,-1.,0.);
    let forward = cross(right,axis);
    let angle = (f32(case_id % 16u)+0.37)/16.0*TAU;
    let sign = select(-1.0,1.0,case_id >= 64u);
    let eps = array<f32,5>(-0.00001,0.,0.00001,0.,0.)[p.x];
    var h = 0.0;
    var radius = 0.0;
    if group == 0u || group == 3u {
        radius = eps;
        h = sign*sqrt(1.0-radius*radius);
    } else if group < 3u {
        h = sign*(select(0.8,0.95,group == 2u)+eps);
        radius = sqrt(max(0.,1.0-h*h));
    } else if group < 6u {
        radius = select(0.05,0.2,group == 5u)+eps;
        h = sign*sqrt(max(0.,1.0-radius*radius));
    } else {
        // Also approach outside the f32 radius==1 rounding plateau.
        h = eps * array<f32,5>(1.0,100.0,50.0,25.0,10.0)[group-6u];
        radius = sqrt(1.0-h*h);
    }
    if p.x == 3u { h = sign*0.6; radius = 0.8; }
    if p.x == 4u {
        h = sign*0.875; radius = sqrt(1.0-h*h);
        if domain == 2u { radius = 0.125; h = sign*sqrt(1.0-radius*radius); }
    }
    let dir = normalize(axis*h + radius*(right*cos(angle)+forward*sin(angle)));
    let seed = (case_id / 16u) % 4u;
    let instr = vec4u((seed<<24u)|(axis_bits<<8u)|255u,
        (255u<<24u)|(255u<<8u),0xffffffffu,
        (OP_SCATTER<<27u)|(7u<<24u)|(((domain<<3u)|1u)<<16u)|65535u);
    let s = evaluate_layer(dir,instr,axis,RegionWeights(1.,0.,0.));
    // Raw RGB and weight, no saturation hiding nonfinite or large values.
    textureStore(result,p.xy,vec4f(s.rgb,s.w));
}
"#,
        5,
        11 * 128,
    );
    let names = [
        "CYL exact poles",
        "CYL fade .8",
        "CYL fade .95",
        "POLAR exact centers",
        "POLAR fade .05",
        "POLAR fade .2",
        "POLAR equator switch",
        "POLAR equator approach .001",
        "POLAR equator approach .0005",
        "POLAR equator approach .00025",
        "POLAR equator approach .0001",
    ];
    assert_eq!(pixels.len(), names.len() * 128 * 5);
    println!(
        "SCATTER angular boundary groups={} rows={} raw samples={}",
        names.len(),
        names.len() * 128,
        pixels.len()
    );
    let mut residuals = [0.0f32; 11];
    for (g, name) in names.iter().enumerate() {
        let mut max_jump = 0.0f32;
        let mut paired_jump = 0.0f32;
        let mut peak = 0.0f32;
        let mut core = 0.0f32;
        let mut fade = 0.0f32;
        for row in pixels[g * 128 * 5..(g + 1) * 128 * 5].chunks_exact(5) {
            assert!(
                row.iter().flatten().all(|v| v.is_finite()),
                "{name}: {row:?}"
            );
            core = core.max(row[3][3]);
            fade = fade.max(row[4][3]);
            peak = peak.max(row[..3].iter().map(|p| p[3]).fold(0., f32::max));
            for c in 0..3 {
                paired_jump =
                    paired_jump.max((row[0][c] * row[0][3] - row[2][c] * row[2][3]).abs());
            }
            for (a, b) in [(0, 1), (1, 2), (0, 2)] {
                for c in 0..3 {
                    max_jump = max_jump.max((row[a][c] * row[a][3] - row[b][c] * row[b][3]).abs());
                }
            }
            if [0, 2, 3, 4].contains(&g) {
                assert!(row[1][3] <= 0.00001, "support exact {name}: {row:?}");
            }
        }
        println!(
            "{name}: max_raw_radiance_jump={max_jump} paired_jump={paired_jump} limit=.005 peak={peak} interior_peak={core} fade_peak={fade}"
        );
        assert!(core > 0.01, "blank control {name}");
        assert!(fade > 0.01, "blank fade control {name}");
        if g == 1 || g >= 5 {
            assert!(peak > 0.01, "blank boundary {name}");
        }
        assert!(paired_jump < 0.005, "{name} paired limit");
        // .001 is a convergence diagnostic, not a zero-distance limit: the
        // finite slope may move farther than .005. Exact/side limit budget is
        // unchanged at .00001; also require it at the three closer approaches.
        if g != 7 {
            assert!(max_jump < 0.005, "{name} exact limit");
        }
        residuals[g] = max_jump;
    }
    assert!(
        residuals[6] < residuals[7] * 0.5,
        "equator approach must converge: {residuals:?}"
    );
}

#[test]
fn domain_boundary_patches_hard_ownership() {
    // Existing positive-width convention: d<0 sky, d=0 wall, d>0 floor.
    // Exercise exactly +/-0 and either side through the production helper.
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let d = array<f32,6>(-0.00001,-0.0,0.0,0.00001,-1.0,1.0)[p.x];
    let bw = select(0.0,0.5,p.y == 1u);
    let r = regions_from_signed_distance(d,bw);
    textureStore(result,p.xy,vec4f(r.sky,r.wall,r.floor,1.));
}
"#,
        6,
        2,
    );
    println!("PATCHES threshold helper samples={pixels:?}");
    for (i, p) in pixels.iter().enumerate() {
        assert!(
            p.iter().all(|v| v.is_finite()),
            "nonfinite threshold {i}: {p:?}"
        );
        assert!(
            (p[0] + p[1] + p[2] - 1.).abs() < 0.001,
            "partition {i}: {p:?}"
        );
        if i < 6 {
            let expected = if i == 0 || i == 4 {
                [1., 0., 0.]
            } else if i == 1 || i == 2 {
                [0., 1., 0.]
            } else {
                [0., 0., 1.]
            };
            assert_eq!(&p[..3], &expected, "hard-edge ownership {i}");
        }
    }
    // Full production dispatch, normalized directions, all six variants, three
    // domains, three coverages. STATIC hard lattice edges remain intentional.
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let v = p.y % 6u;
    let domain = (p.y / 6u) % 3u;
    let coverage = array<u32,3>(0u,128u,255u)[p.y / 18u];
    let dir = normalize(vec3f(cos(f32(p.x)*0.71),sin(f32(p.x)*0.37),cos(f32(p.x)*0.17)));
    let instr = vec4u((3u<<24u)|(0xffffu<<8u)|255u,
        (255u<<16u)|(coverage<<8u)|255u,0xff0000ffu,
        (OP_PATCHES<<27u)|(7u<<24u)|(((domain<<3u)|v)<<16u)|65535u);
    let b = evaluate_bounds_layer(dir,instr,OP_PATCHES,vec3f(0.,0.,-1.),RegionWeights(0.2,0.3,0.5));
    textureStore(result,p.xy,vec4f(b.regions.sky,b.regions.wall,b.regions.floor,b.sample.w));
}
"#,
        64,
        54,
    );
    let mut counts = [0usize; 3];
    for p in &pixels {
        assert!(
            p.iter().all(|v| v.is_finite() && *v >= 0. && *v <= 1.),
            "full eval {p:?}"
        );
        assert_eq!(p[0] + p[1] + p[2], 1., "full eval partition {p:?}");
        for c in 0..3 {
            if p[c] == 1. {
                counts[c] += 1;
            }
        }
    }
    println!(
        "PATCHES full-eval hard ownership counts sky/wall/floor={counts:?}, samples={}",
        pixels.len()
    );
    assert!(counts[0] > 0 && counts[2] > 0, "hard sides preserved");
    // Reach exact threshold in full eval without replacing/injecting noise.
    // STATIC's actual integer hash can equal zero: coverage=255 then gives d=0.
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let k = f32(p.x + p.y*128u);
    let dir = normalize(vec3f(cos(k*0.71),sin(k*0.37),cos(k*0.17)));
    let instr = vec4u((3u<<24u)|(0xffffu<<8u)|255u,
        (255u<<16u)|(255u<<8u)|255u,0xff0000ffu,
        (OP_PATCHES<<27u)|(7u<<24u)|(4u<<16u)|65535u);
    let noise = patches_static(dir*16.0+vec3f(0.3,0.51,0.93))*0.5+0.5;
    let b = evaluate_bounds_layer(dir,instr,OP_PATCHES,vec3f(0.,0.,-1.),RegionWeights(1.,0.,0.));
    textureStore(result,p.xy,vec4f(b.regions.sky,b.regions.wall,b.regions.floor,select(0.,1.,noise==0.)));
}
"#,
        128,
        64,
    );
    let mut ties = 0;
    for (i, p) in pixels.iter().enumerate() {
        assert!(p.iter().all(|v| v.is_finite()));
        assert_eq!(p[0] + p[1] + p[2], 1.);
        if p[3] == 1. {
            ties += 1;
            println!("PATCHES full-eval exact STATIC threshold sample={i} ownership={p:?}");
            assert_eq!(&p[..3], &[0., 1., 0.]);
        }
    }
    assert!(ties > 0, "no actual full-eval exact threshold exercised");
    println!("PATCHES actual full-eval exact threshold ties={ties}");
}
