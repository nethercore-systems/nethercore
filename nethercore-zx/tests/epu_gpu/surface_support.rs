//! Frozen LOW diagnostic: numerical continuity RED does not authorize contour repair.
use super::*;
const HEADER: &str = r#"
fn instruction(control:u32)->vec4u {
 return vec4u(select(240u,0u,control==1u),0x80000080u,0x404060a0u,
   select(0xb703c080u,0xb003c080u,control==2u));
}
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
"#;
#[test]
fn dusted_front_cutoff() {
    let out =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../tmp/epu-review/surface-support");
    std::fs::create_dir_all(&out).unwrap();
    let save = |name: &str, pixels: &Vec<[f32; 4]>| {
        if let Ok(stage) = std::env::var("SURFACE_SUPPORT_STAGE") {
            let path = out.join(format!("{stage}-{name}.f32"));
            assert!(!path.exists(), "preserve evidence");
            std::fs::write(path, bytemuck::cast_slice(pixels)).unwrap();
        }
    };
    // Decode on GPU before executing radiance; NONE=0, ADD=0, SURFACE fallback != decode_dir16(0).
    let fixture = probe_image(
        &format!(
            "{HEADER}{}",
            r#"
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let i=instruction(p.x);let a=instr_dir16(i);
 let axis=normalize(select(vec3f(0.,1.,0.),decode_dir16(a),a!=0u));
 var v=vec4f(f32(instr_opcode(i)),f32(instr_variant_id(i)),f32(instr_domain_id(i)),f32(a));
 if p.y==1u {v=vec4f(f32(instr_intensity(i)),f32(instr_a(i)),f32(instr_b(i)),f32(instr_c(i)));}
 if p.y==2u {v=vec4f(f32(instr_d(i)),f32(instr_alpha_a(i)),f32(instr_alpha_b(i)),f32(instr_blend(i)));}
 if p.y==3u {v=vec4f(axis,region_weight(RegionWeights(1.,0.,0.),instr_region(i)));}
 if p.y==4u {v=vec4f(instr_color_a(i)*255.,f32(instr_region(i)));}
 if p.y==5u {v=vec4f(instr_color_b(i)*255.,0.);}
 // Byte readback retains all packed bits exactly in half float.
 if p.y>=6u {let w=i[p.y-6u];v=vec4f(f32(w&255u),f32((w>>8u)&255u),f32((w>>16u)&255u),f32(w>>24u));}
 textureStore(result,p.xy,v);
}"#
        ),
        3,
        10,
    );
    save("fixture", &fixture);
    for control in 0..3 {
        let at = |y: usize| fixture[y * 3 + control];
        println!(
            "FIXTURE control={control} decoded={:?}",
            (0..10).map(at).collect::<Vec<_>>()
        );
        assert_eq!(
            at(0),
            [22., 3., 0., 0.],
            "fixture opcode/variant/domain/axis"
        );
        assert_eq!(at(1), [128., 0., 0., 128.]);
        assert_eq!(at(2), [0., if control == 1 { 0. } else { 15. }, 0., 0.]);
        assert_eq!(at(3), [0., 1., 0., if control == 2 { 0. } else { 1. }]);
        assert_eq!(at(4), [192., 128., 64., if control == 2 { 0. } else { 7. }]);
        assert_eq!(at(5), [64., 96., 160., 0.]);
        let words = [
            if control == 1 { 0u32 } else { 240 },
            0x80000080,
            0x404060a0,
            if control == 2 { 0xb003c080 } else { 0xb703c080 },
        ];
        for w in 0..4 {
            assert_eq!(at(6 + w), words[w].to_le_bytes().map(f32::from));
        }
        println!("PACKED control={control} words={words:08x?}");
    }
    println!("FIXTURE_PASS: decoded frozen inputs, +Y fallback, NONE, alpha, ADD");
    let mut dirs = Vec::new();
    for phi in [0f32, 0.7, 1.7, 2.8] {
        for s in 0..20 {
            let h = if s < 15 {
                0.05 + ((s % 5) as f32 - 2.) * [0.0001, 0.00001, 0.000001][s / 5]
            } else {
                [0.04, 0.1, 0.5, 0.99, 1.][s - 15]
            };
            let r = (1f32 - h * h).sqrt();
            let bits = [r * phi.cos(), h, r * phi.sin()].map(f32::to_bits);
            println!(
                "DIRECTION index={} phi={phi} h={h:.9} bits={bits:08x?}",
                dirs.len()
            );
            dirs.push(format!("vec3u({}u,{}u,{}u)", bits[0], bits[1], bits[2]));
        }
    }
    let body = format!(
        "{HEADER}\nconst DIRS:array<vec3u,80>=array<vec3u,80>({});{}",
        dirs.join(","),
        r#"
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 if p.y!=0u {return;}
 let dir=bitcast<vec3f>(DIRS[p.x]);
 for(var c=0u;c<3u;c++) {
  let i=instruction(c);var layers:array<vec4u,8>;layers[0]=i;
  let raw=eval_surface(dir,i,region_weight(RegionWeights(1.,0.,0.),instr_region(i)));
  let full=evaluate_epu_layers(dir,layers);
  let axis=normalize(select(vec3f(0.,1.,0.),decode_dir16(instr_dir16(i)),instr_dir16(i)!=0u));
  let h=dot(dir,axis);
  textureStore(result,vec2u(p.x,c*3u),vec4f(raw.rgb*raw.w,raw.w));
  textureStore(result,vec2u(p.x,c*3u+1u),vec4f(full,1.));
  textureStore(result,vec2u(p.x,c*3u+2u),vec4f(select((h-.05)*select(1.,1000000.,p.x%20u<15u),0.,h==.05),select(0.,1.,h>.05),h,select(0.,1.,h==.05)));
 }
}"#
    );
    if let Ok(stage) = std::env::var("SURFACE_SUPPORT_STAGE") {
        std::fs::write(out.join(format!("{stage}-probe.wgsl")), &body).unwrap();
    }
    let pixels = probe_image(&body, 80, 9);
    save("samples", &pixels);
    assert_eq!(pixels.len(), 720);
    assert!(
        pixels.iter().flatten().all(|x| x.is_finite()),
        "finite gate"
    );
    let at = |c: usize, p: usize, phi: usize, s: usize| pixels[(c * 3 + p) * 80 + phi * 20 + s];
    let mut failures = Vec::new();
    let mut parity = 0f32;
    for phi in 0..4 {
        for s in 0..20 {
            let raw = at(0, 0, phi, s);
            let full = at(0, 1, phi, s);
            let diag = at(0, 2, phi, s);
            println!("SAMPLE phi={phi} s={s} raw={raw:?} full={full:?} diagnostic={diag:?}");
            for ch in 0..3 {
                parity = parity.max((raw[ch] - full[ch]).abs());
            }
            for c in 1..3 {
                assert_eq!(at(c, 0, phi, s), [0.; 4], "negative raw");
                assert_eq!(at(c, 1, phi, s), [0., 0., 0., 1.], "negative full");
            }
            if s < 15 {
                let sign = (s % 5) as i32 - 2;
                assert_eq!(
                    diag[1],
                    if sign > 0 { 1. } else { 0. },
                    "fixture support predicate"
                );
                assert!(
                    if sign == 0 {
                        diag[0] == 0.
                    } else {
                        diag[0] * sign as f32 > 0.
                    },
                    "fixture scaled side"
                );
            }
            if s == 15 || s == 19 {
                assert_eq!(raw[3], 0., "outside/center control");
            }
            if s == 16 || s == 17 {
                assert!(
                    raw[3] >= 0.34 && raw[..3].iter().copied().fold(0., f32::max) > 0.01,
                    "interior control"
                );
            }
        }
        for (kind, p, channels, tol) in [
            ("weight", 0, 3..4, 0.005),
            ("weighted_rgb", 0, 0..3, 0.01),
            ("dispatch", 1, 0..3, 0.01),
        ] {
            for (pair, (a, b)) in [(1, 3), (1, 2), (2, 3), (0, 1), (3, 4)]
                .into_iter()
                .enumerate()
            {
                let mut j = [0f32; 3];
                for k in 0..3 {
                    for ch in channels.clone() {
                        j[k] = j[k].max(
                            (at(0, p, phi, k * 5 + a)[ch] - at(0, p, phi, k * 5 + b)[ch]).abs(),
                        );
                    }
                }
                println!("PAIR phi={phi} kind={kind} pair={pair} jumps={j:?}");
                if j[2] > tol || (1..3).any(|k| j[k] > 0.35 * j[k - 1] + 0.001) {
                    failures.push(format!("phi={phi} {kind} pair={pair} {j:?}"));
                }
            }
        }
    }
    println!(
        "CONTROLS_PASS parity={parity} failures={} details={failures:?}",
        failures.len()
    );
    assert!(parity <= 0.001, "raw/full parity");
    assert!(
        failures.is_empty(),
        "SURFACE numerical support continuity RED; no contour repair authorized"
    );
}
// Authorized fixed .05..1 support fade; all encoded variants include fallback 4..7.
#[test]
fn authorized_support_all_variants() {
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../tmp/epu-review/surface-support-authorized");
    let original = include_str!("fixtures/surface-support/original.wgsl");
    let reference = original[original.find("fn eval_surface(").unwrap()..]
        .replace("eval_surface", "reference_surface");
    let dispatch = include_str!("../../shaders/epu/epu_dispatch.wgsl")
        .replace("eval_surface", "reference_surface")
        .replace("evaluate_bounds_layer", "reference_bounds_layer")
        .replace("evaluate_layer", "reference_layer")
        .replace("evaluate_epu_layers", "reference_epu_layers");
    let mut dirs = Vec::new();
    for phi in [0f32, 0.7, 1.7, 2.8] {
        for boundary in [0.05f32, 0.1] {
            for e in [0.0001f32, 0.00001, 0.000001] {
                for side in [-2f32, -1., 0., 1., 2.] {
                    let h = boundary + side * e;
                    let r = (1. - h * h).sqrt();
                    dirs.push([r * phi.cos(), h, r * phi.sin()]);
                }
            }
        }
        for h in [
            -1f32, 0., 0.04, 0.05, 0.1, 0.125, 0.25, 0.5, 0.75, 0.9, 0.99, 1.,
        ] {
            let r = (1. - h * h).sqrt();
            dirs.push([r * phi.cos(), h, r * phi.sin()]);
        }
    }
    let literals = dirs
        .iter()
        .map(|d| {
            let b = d.map(f32::to_bits);
            format!("vec3u({}u,{}u,{}u)", b[0], b[1], b[2])
        })
        .collect::<Vec<_>>()
        .join(",");
    let body = format!(
        "{HEADER}\n{reference}\n{dispatch}\nconst DIRS:array<vec3u,168>=array<vec3u,168>({literals});{}",
        r#"
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let dir=bitcast<vec3f>(DIRS[p.x]);let variant=p.y/3u;let control=p.y%3u;
 var i=instruction(control);i.w=(i.w&~0x70000u)|(variant<<16u);
 let rw=region_weight(RegionWeights(1.,0.,0.),instr_region(i));
 let a=eval_surface(dir,i,rw);let b=reference_surface(dir,i,rw);
 var layers:array<vec4u,8>;layers[0]=i;
 let f=evaluate_epu_layers(dir,layers);let g=reference_epu_layers(dir,layers);
 let y=p.y*7u;
 textureStore(result,vec2u(p.x,y),vec4f(a.rgb*a.w,a.w));
 textureStore(result,vec2u(p.x,y+1u),vec4f(f,1.));
 textureStore(result,vec2u(p.x,y+2u),vec4f(b.rgb*b.w,b.w));
 textureStore(result,vec2u(p.x,y+3u),vec4f(g,1.));
 textureStore(result,vec2u(p.x,y+4u),vec4f(a.rgb,a.w));
 textureStore(result,vec2u(p.x,y+5u),vec4f(b.rgb,b.w));
 textureStore(result,vec2u(p.x,y+6u),vec4f(
 select(0.,1.,all(bitcast<vec4u>(vec4f(a.rgb,a.w))==bitcast<vec4u>(vec4f(b.rgb,b.w)))),
 select(0.,1.,all(bitcast<vec3u>(f)==bitcast<vec3u>(g))),f32(instr_variant_id(i)),f32(instr_domain_id(i))));
}"#
    );
    // probe_image dispatches height workgroups; the shader must only run 24 logical rows.
    let body = body.replace("let dir=bitcast", "if p.y>=24u {return;}\n let dir=bitcast");
    let pixels = probe_image(&body, 168, 168);
    if let Ok(stage) = std::env::var("SURFACE_SUPPORT_STAGE") {
        let capture = out.join(format!("{stage}-all.f32"));
        assert!(!capture.exists(), "preserve evidence");
        std::fs::write(out.join(format!("{stage}-all.wgsl")), &body).unwrap();
        std::fs::write(capture, bytemuck::cast_slice(&pixels)).unwrap();
    }
    assert!(pixels.iter().flatten().all(|x| x.is_finite()));
    let at = |v: usize, c: usize, row: usize, s: usize| pixels[((v * 3 + c) * 7 + row) * 168 + s];
    let mut max_parity = 0f32;
    let mut maxima = [0f32; 3];
    let mut off = 0;
    for v in 0..8 {
        for (s, dir) in dirs.iter().enumerate() {
            for c in 0..3 {
                let a = at(v, c, 0, s);
                let f = at(v, c, 1, s);
                let flags = at(v, c, 6, s);
                assert_eq!(&flags[2..], &[v as f32, 0.]);
                for ch in 0..3 {
                    max_parity = max_parity.max((a[ch] - f[ch]).abs());
                }
                assert_eq!(
                    &at(v, c, 4, s)[..3],
                    &at(v, c, 5, s)[..3],
                    "material RGB unchanged including band"
                );
                if dir[1] <= 0.05 || dir[1] >= 0.1 {
                    assert_eq!(
                        &flags[..2],
                        &[1., 1.],
                        "off-band f32 bit parity v={v} c={c} s={s}"
                    );
                    assert_eq!(a, at(v, c, 2, s));
                    assert_eq!(f, at(v, c, 3, s));
                    off += 1;
                }
                if c > 0 {
                    assert_eq!(a, [0.; 4]);
                    assert_eq!(f, [0., 0., 0., 1.]);
                }
                if dir[1] <= 0.05 || dir[1] >= 0.99 {
                    assert_eq!(a[3], 0.);
                }
                if c == 0 && (dir[1] == 0.1 || dir[1] == 0.5) {
                    assert!(a[3] >= 0.34 && a[..3].iter().copied().fold(0., f32::max) > 0.01);
                }
            }
        }
        for phi in 0..4 {
            for boundary in 0..2 {
                for (kind, row, channels, tol) in [
                    (0, 0, 3..4, 0.005f32),
                    (1, 0, 0..3, 0.01),
                    (2, 1, 0..3, 0.01),
                ] {
                    for (a, b) in [(1, 3), (1, 2), (2, 3), (0, 1), (3, 4)] {
                        let mut j = [0f32; 3];
                        for k in 0..3 {
                            for ch in channels.clone() {
                                let base = phi * 42 + boundary * 15 + k * 5;
                                j[k] = j[k].max(
                                    (at(v, 0, row, base + a)[ch] - at(v, 0, row, base + b)[ch])
                                        .abs(),
                                );
                            }
                        }
                        maxima[kind] = maxima[kind].max(j[2]);
                        assert!(
                            j[2] <= tol && (1..3).all(|k| j[k] <= 0.35 * j[k - 1] + 0.001),
                            "v={v} phi={phi} boundary={boundary} kind={kind} pair={a},{b} jumps={j:?}"
                        );
                    }
                }
            }
        }
    }
    assert!(max_parity <= 0.001);
    for v in 0..4 {
        for w in v + 1..4 {
            assert!(
                (0..168).any(|s| dirs[s][1] >= 0.1 && at(v, 0, 4, s) != at(w, 0, 4, s)),
                "variant identities must remain distinct"
            );
        }
    }
    println!(
        "ALL_VARIANTS_GREEN variants=8 boundaries=2 off_band_cases={off} max_weight_rgb_dispatch={maxima:?} parity={max_parity}; f32 bit-exact raw/full preservation, RGB structure and controls PASS"
    );
}
