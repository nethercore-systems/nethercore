//! Bounded fixed-orientation structural-floor continuity, not all variants.
use super::*;

#[test]
fn aperture_plane_support_and_minimum_softness() {
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let eps = array<f32,9>(-0.001,0.001,-0.0001,0.0001,-0.00001,0.,0.00001,0.,0.)[p.x];
    let group = p.y / 16u;
    let angle = (f32(p.y%16u)+0.37)/16.*TAU;
    var instr = vec4u(0x00808000u,0x0089351cu,0xff181c20u,0x3fc1ffffu);
    if group >= 7u { instr = vec4u(0x000080f0u,0xb4301230u,0x80202830u,0x79077078u); }
    let n = decode_dir16(instr_dir16(instr));
    let hint=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(n.y)>.9);
    let r=normalize(cross(hint,n)); let t=cross(n,r);
    var d=array<f32,10>(0.,.06,.18,.22,.42,1.,1.,.05,.2,0.)[group]+eps;
    // At fade-end .2 the projected texture has a steep finite gradient.
    // Keep the larger approaches as diagnostics, test its limit at 1e-6.
    if group==8u && p.x>=4u && p.x<=6u {d=.2+eps*.1;}
    if p.x==7u { d=.5; } if p.x==8u { d=select(.03,.125,group>=7u); }
    var dir=normalize(n*d+(r*cos(angle)+t*sin(angle))*sqrt(max(0.,1.-d*d)));
    if group==5u || group==6u {
        // Exact reachable RECT SDF opening / outside-frame edge, min packed softness.
        let w=mix(0.1f,1.5f,137.0f/255.0f); let frame=mix(0.02f,0.5f,28.0f/255.0f);
        var u=w+select(0.,frame,group==6u)+eps;
        if p.x==7u {u=0.;} if p.x==8u {u=w+frame*.5;}
        dir=normalize(n+r*u+t*(f32(p.y%16u)-7.5)*.01);
    }
    var value:vec4f;
    if group<7u {
        let b=evaluate_bounds_layer(dir,instr,OP_APERTURE,n,RegionWeights(1.,0.,0.));
        value=vec4f(b.regions.sky,b.regions.wall,b.regions.floor,b.sample.w);
    } else {
        let s=evaluate_layer(dir,instr,n,RegionWeights(0.,0.,1.));
        value=vec4f(s.rgb*s.w,s.w);
    }
    textureStore(result,p.xy,value);
}
"#,
        9,
        160,
    );
    for g in 0..10 {
        let mut peak = 0f32;
        let mut core = 0f32;
        let mut interior = 0f32;
        let mut near = 0f32;
        let mut approaches = [0f32; 3];
        for row in pixels[g * 16 * 9..(g + 1) * 16 * 9]
            .as_chunks::<9>()
            .0
            .iter()
        {
            assert!(
                row.iter().flatten().all(|v| v.is_finite()),
                "group={g} {row:?}"
            );
            for c in 0..4 {
                for (i, (a, b)) in [(0, 1), (2, 3), (4, 6)].iter().enumerate() {
                    approaches[i] = approaches[i].max((row[*a][c] - row[*b][c]).abs());
                }
                near = near
                    .max((row[4][c] - row[5][c]).abs())
                    .max((row[6][c] - row[5][c]).abs());
            }
            peak = peak.max(row[5][3]);
            core = core.max(row[7][3]);
            interior = interior.max(row[8][3]);
            if g < 7 {
                for p in row {
                    assert!((p[0] + p[1] + p[2] - 1.).abs() < 0.001);
                }
            }
            if g == 5 {
                assert!(
                    row[7][0] > 0.999 && row[8][1] > 0.999,
                    "opening/frame controls {row:?}"
                );
            }
            if g == 6 {
                assert!((row[5][1] - 0.5).abs() < 0.002 && (row[5][2] - 0.5).abs() < 0.002);
            }
        }
        println!(
            "support group={g} approaches={approaches:?} exact_near={near} peak={peak} core={core} interior={interior} tolerance=.005 at eps=.00001"
        );
        assert!(approaches[2] < 0.005 && near < 0.005, "group={g}");
        assert!(core > 0.01 && interior > 0.01, "blank controls group={g}");
        if g == 0 || g == 7 || g == 9 {
            assert!(peak < 0.00001, "zero support group={g}");
        }
    }
}

#[test]
fn aperture_plane_lattice_limits() {
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let eps=array<f32,7>(-0.001,0.001,-.0001,.0001,-.00001,0.,.00001)[p.x];
    let group=p.y/192u; let k=p.y%192u;
    let corner=vec2f(f32(i32(k%8u)-4),f32(i32((k/8u)%8u)-4));
    let axis=k/64u;
    let travel=array<vec2f,3>(vec2f(1.,0.),vec2f(0.,1.),vec2f(1.,1.))[axis];
    let offset=array<vec2f,3>(vec2f(0.,.371),vec2f(.371,0.),vec2f(0.))[axis];
    let q=corner+offset+travel*eps;
    var value:vec4f;
    if group==0u {
        // Signed ordinary lattice plus exact IRREGULAR displacement lattice near its opening.
        let uv=(q+vec2f(336.))/16.;
        value=vec4f(aperture_value_noise(q,1.),aperture_value_noise(uv*2.,8.),aperture_sdf_irregular(q/16.,.1,.1,.3),1.);
    } else if group==1u {
        let vor=plane_voronoi(q); let mat=eval_plane_pavement(q,18./255.*.2,48./255.);
        let control=eval_plane_pavement(q,220./255.*.2,48./255.);
        let endpoint=eval_plane_pavement(q,.2,48./255.);
        // The raw second-nearest distance can change outside the 3x3 search;
        // only grout widths <=.4 (+.02 AA) consume it. Test actual materials.
        value=vec4f(vor.x,mat.x*(1.+mat.y*.5),control.x*(1.+control.y*.5),endpoint.x*(1.+endpoint.y*.5));
    } else if group==2u {
        let instr=vec4u(0xff808000u,0xff00001cu,0xff181c20u,0x3fc6ffffu);
        let n=decode_dir16(instr_dir16(instr));
        let r=normalize(cross(vec3f(0.,1.,0.),n));let t=cross(n,r);
        let dir=normalize(n+r*q.x/16.+t*q.y/16.);
        let b=evaluate_bounds_layer(dir,instr,OP_APERTURE,n,RegionWeights(1.,0.,0.));
        value=vec4f(b.regions.sky,b.regions.wall,b.regions.floor,b.sample.w);
    } else {
        let instr=vec4u(0x000080f0u,0xb4301230u,0x80202830u,0x79077078u);
        let n=decode_dir16(instr_dir16(instr));
        let r=normalize(cross(vec3f(1.,0.,0.),n));let t=cross(n,r);
        let scale=mix(.5f,16.f,48.f/255.f);
        let dir=normalize(n+r*q.x/scale+t*q.y/scale);
        let s=evaluate_layer(dir,instr,n,RegionWeights(0.,0.,1.));
        value=vec4f(s.rgb*s.w,s.w);
    }
    textureStore(result,p.xy,value);
}
"#,
        7,
        768,
    );
    let mut failures = Vec::new();
    for g in 0..4 {
        let mut max = [0f32; 3];
        let mut range = [f32::MAX, f32::MIN];
        let mut witness = 0;
        for (i, row) in pixels[g * 192 * 7..(g + 1) * 192 * 7]
            .as_chunks::<7>()
            .0
            .iter()
            .enumerate()
        {
            assert!(row.iter().flatten().all(|v| v.is_finite()));
            range[0] = range[0].min(row[5][0]);
            range[1] = range[1].max(row[5][0]);
            for (j, (a, b)) in [(0, 1), (2, 3), (4, 6)].iter().enumerate() {
                for c in 0..4 {
                    let delta = (row[*a][c] - row[*b][c]).abs();
                    if delta > max[j] {
                        max[j] = delta;
                        if j == 2 {
                            witness = i;
                        }
                    }
                }
            }
        }
        println!(
            "lattice group={g} signed XY/corners approaches={max:?} witness={witness} range={range:?} tolerance=.005 eps=.00001"
        );
        println!(
            "lattice witness group={g} row={witness} samples={:?}",
            &pixels[(g * 192 + witness) * 7..(g * 192 + witness + 1) * 7]
        );
        assert!(range[1] - range[0] > 0.01, "constant lattice group={g}");
        // IRREGULAR's +42 chart shift rounds away the smallest pair: also
        // require 1e-4 there. PLANE's steep grout gradients use the original
        // 1e-5 budget; larger steps are convergence diagnostics, not limits.
        if max[2] >= 0.005 || ((g == 0 || g == 2) && max[1] >= 0.005) {
            failures.push((g, witness, max));
        }
    }
    assert!(
        failures.is_empty(),
        "numeric lattice discontinuity {failures:?}"
    );
}

#[test]
fn aperture_plane_pavement_nearest_material_boundary() {
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    // Locate real nearest-cell transition by bisection on production cell identity.
    let start=vec2f(f32(i32(p.y%16u)-8),f32(i32(p.y/16u)-2)+.371);
    var lo=start;var hi=start+vec2f(1.,0.);let id=plane_voronoi(lo).z;
    for(var i=0u;i<22u;i++){let mid=(lo+hi)*.5;if plane_voronoi(mid).z==id {lo=mid;} else {hi=mid;}}
    let q=(lo+hi)*.5+vec2f(array<f32,9>(-0.001,0.001,-.0001,.0001,-.00001,0.,.00001,-.03,.03)[p.x],0.);
    let v=plane_voronoi(q);
    let a=eval_plane_pavement(q,18./255.*.2,48./255.);
    let control=eval_plane_pavement(q,220./255.*.2,48./255.);
    let hard=eval_plane_pavement(q,0.,0.);
    // Varied material is deliberately flat per cell; finite grout must hide the ID switch.
    textureStore(result,p.xy,vec4f(a.x*(1.+a.y*.5),control.x*(1.+control.y*.5),hard.x*(1.+hard.y*.5),v.y));
}
"#,
        9,
        64,
    );
    let mut max = 0f32;
    let mut hard = 0f32;
    let mut interior = 0f32;
    let mut ties = 0;
    for row in pixels.as_chunks::<9>().0.iter() {
        assert!(row.iter().flatten().all(|v| v.is_finite()));
        if row[5][3] > 0.001 {
            continue;
        } // No nearest-cell switch in this bracket.
        ties += 1;
        for c in 0..2 {
            max = max.max((row[4][c] - row[6][c]).abs());
        }
        hard = hard.max((row[4][2] - row[6][2]).abs());
        interior = interior.max(row[7][0]).max(row[8][0]);
    }
    println!(
        "PAVEMENT real nearest transitions={ties} candidate/control jump={max} deliberate zero-gap material step={hard} interior={interior} tolerance=.005 eps=.00001"
    );
    assert!(ties > 8 && interior > 0.1);
    assert!(max < 0.005);
    assert!(hard > 0.001, "lost authored per-cell variation");
}

#[test]
fn plane_pattern_gap_controls() {
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let family=p.y/(5u*64u);let gap_index=(p.y/64u)%5u;
    let variant=array<u32,3>(0u,1u,5u)[family];
    let byte=array<u32,5>(0u,64u,128u,192u,255u)[gap_index];
    let gap=f32(byte)/255.0*0.2;
    let uv=vec2f((f32(p.x)+0.5)/128.0,(f32(p.y%64u)+0.5)/64.0*select(1.0,sqrt(3.0),family==1u));
    var pattern=vec2f(0.0);
    switch variant {
        case 0u: {pattern=eval_plane_tiles(uv,gap);}
        case 1u: {pattern=eval_plane_hex(uv,gap);}
        default: {pattern=eval_plane_grating(uv,gap);}
    }
    let instr=vec4u(0x000080f7u,(255u<<24u)|(128u<<16u)|(byte<<8u),0x00000080u,
                   (15u<<27u)|(7u<<24u)|(3u<<21u)|(variant<<16u)|0x8000u);
    let n=decode_dir16(instr_dir16(instr));
    let hint=select(vec3f(0,1,0),vec3f(1,0,0),abs(n.y)>0.9);
    let r=normalize(cross(hint,n));let t=cross(n,r);
    let scale=mix(0.5f,16.0f,128.0f/255.0f);
    let direction=normalize(n+(r*uv.x+t*uv.y)/scale);
    let sample=evaluate_layer(direction,instr,n,RegionWeights(0,0,1));
    textureStore(result,p.xy,vec4f(pattern.x,sample.rgb.r,sample.rgb.b,sample.w));
}
"#,
        128,
        3 * 5 * 64,
    );
    let mut means = [[0f64; 5]; 3];
    for (group, chunk) in pixels.as_chunks::<{ 128 * 64 }>().0.iter().enumerate() {
        for p in chunk {
            assert!(p.iter().all(|x| x.is_finite()) && p[3] > 0.99);
            assert!(p[0] >= 0.0 && p[0] <= 1.0);
            if p[0] > 0.999 {
                assert!(p[1] > 0.49 && p[2] < 0.002, "surface colour {p:?}");
            }
            if p[0] < 0.001 {
                assert!(p[1] < 0.002 && p[2] > 0.49, "gap colour {p:?}");
            }
        }
        means[group / 5][group % 5] =
            chunk.iter().map(|p| f64::from(p[0])).sum::<f64>() / chunk.len() as f64;
    }
    let mut failures = 0;
    for (family, row) in means.iter().enumerate() {
        failures += usize::from(row[0] < 0.95);
        failures += row.windows(2).filter(|p| p[1] > p[0] + 0.005).count();
        if family == 2 {
            failures += usize::from(row[4] > 0.005);
        }
    }
    println!(
        "PLANE_GAP samples={} means={means:?} failures={failures} monotonic_limit=.005",
        pixels.len()
    );
    assert_eq!(
        failures, 0,
        "increasing gap must remove surface, not add it"
    );
}

#[test]
fn plane_hex_nearest_site_metric() {
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let q=(vec2f(f32(p.x),f32(p.y%64u))+0.37)/64.0*4.0-2.0;
    let gap=f32(array<u32,3>(16u,128u,255u)[p.y/64u])/255.0*0.2;
    // Independent Voronoi bisectors on an enumerated triangular lattice.
    var best=1e9;var owner=vec2f(0.0);
    for(var y=-4;y<=4;y++) {for(var x=-4;x<=4;x++) {
        let site=vec2f(f32(x)+f32(y&1)*0.5,f32(y)*sqrt(3.0)*0.5);
        let d=dot(q-site,q-site);if d<best {best=d;owner=site;}
    }}
    var edge=1e9;
    for(var y=-4;y<=4;y++) {for(var x=-4;x<=4;x++) {
        let site=vec2f(f32(x)+f32(y&1)*0.5,f32(y)*sqrt(3.0)*0.5);
        let separation=length(site-owner);
        if separation>0.1 {edge=min(edge,(dot(q-site,q-site)-best)/(2.0*separation));}
    }}
    let expected=smoothstep(gap-0.01,gap+0.01,edge);
    let actual=eval_plane_hex(q,gap).x;
    textureStore(result,p.xy,vec4f(actual,expected,edge,select(0.0,1.0,abs(actual-expected)>0.002)));
}
"#,
        64,
        64 * 3,
    );
    let failures = pixels.iter().filter(|p| p[3] != 0.0).count();
    let max = pixels
        .iter()
        .map(|p| (p[0] - p[1]).abs())
        .fold(0f32, f32::max);
    assert!(pixels.iter().flatten().all(|v| v.is_finite()));
    assert!(pixels.iter().any(|p| p[1] > 0.99) && pixels.iter().any(|p| p[1] < 0.01));
    println!(
        "PLANE_HEX_METRIC samples={} failures={failures} max_error={max} f32_limit=.002",
        pixels.len()
    );
    assert_eq!(
        failures, 0,
        "HEX must follow regular nearest-site cell edges"
    );
}
