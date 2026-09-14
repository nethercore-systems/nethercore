//! Diagnostic only: no production substitution. Frozen .005 region/.01 RGB budgets.
use super::*;
const BODY: &str = r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let pc=array<u32,4>(0u,128u,129u,255u)[p.y%4u];
 let pa=array<u32,3>(0u,64u,255u)[(p.y/4u)%3u];
 let pd=array<u32,3>(0u,64u,255u)[(p.y/12u)%3u];
 let h=array<f32,7>(-1.,-0.98,-0.95,0.,0.95,0.98,1.)[(p.y/36u)%7u];
 let cut=p.y/252u; let sample=p.x%33u; let path=p.x/33u;
 let n=decode_dir16(0x8080u); let basis=split_build_basis(n);
 let rotation=u8_to_01(pd); let sectors=clamp(mix(2.,16.,u8_to_01(pc)),3.,16.);
 let bw=max(u8_to_01(pa)*0.2,0.001);
 let center=select(PI-TAU*rotation,PI,cut==1u);
 var delta=0.;
 if sample<25u { delta=f32(i32(sample%5u)-2)*array<f32,5>(0.001,0.0001,0.00001,0.000003,0.000001)[sample/5u]; }
 else { delta=(f32(sample-25u)+0.37)*TAU/8.; }
 // Rotate small offsets directly, not PI +/- epsilon (which collapses in f32).
 let tc=cos(center); let bc=sin(center);
 let t=tc*cos(delta)-bc*sin(delta); let b=bc*cos(delta)+tc*sin(delta);
 let dir=normalize(n*h+sqrt(max(0.,1.-h*h))*(basis[0]*t+basis[1]*b));
 let instr=vec4u((pd<<24u)|(0x8080u<<8u)|240u,(pa<<16u)|pc,0x4060c080u,(4u<<27u)|(7u<<24u)|(3u<<21u)|(5u<<16u)|0xe080u);
 let weights=split_prism(dir,n,basis,mix(2.,16.,u8_to_01(pc)),rotation,bw);
 let v=eval_split(dir,instr,RegionWeights(1.,0.,0.));
 var layers:array<vec4u,8>; layers[0]=instr;
 var out=vec4f(weights,1.);
 if path==1u { out=vec4f(v.regions.sky,v.regions.wall,v.regions.floor,v.region_mix); }
 if path==2u { out=vec4f(v.sample.rgb,v.sample.w); }
 if path==3u { out=vec4f(evaluate_epu_layers(dir,layers),1.); }
 if path>=4u && path<=9u {
   let masks=array<u32,6>(0u,1u,2u,4u,7u,2u); let mask=masks[path-4u];
   layers[0].x=layers[0].x & 0xffffff0fu; // zero paint still owns regions
   let axes=array<u32,6>(0x80ffu,0x8000u,0xff80u,0x0080u,0x8080u,0xffffu);
   for(var j=0u;j<6u;j++) {
     layers[j+1u]=vec4u((axes[j]<<8u)|240u,32u<<24u,0x80808080u,(18u<<27u)|(mask<<24u)|0x8080u);
   }
   if path==9u { let last=layers[0]; layers[0]=layers[6]; layers[6]=vec4u(0u); layers[7]=last; }
   out=vec4f(evaluate_epu_layers(dir,layers),1.);
 }
 if path==10u { let z=dot(dir,n); out=vec4f(smoothstep(-0.95+bw,-0.95-bw,z),1.-smoothstep(-0.95-bw,-0.95+bw,z),z,bw); }
 if path==11u { out=vec4f(f32(instr_c(instr)),f32(instr_a(instr)),f32(instr_d(instr)),f32(instr_variant_id(instr))); }
 textureStore(result,p.xy,out);
}
"#;
fn run(gate: bool) {
    let pixels = probe_image(BODY, 396, 504);
    // Keep unaffected region values exact, using the same GPU/backend for the base.
    if gate {
        let legacy_body = BODY.replace(
            "let weights=split_prism(",
            "let weights=split_prism_legacy(",
        );
        assert_ne!(legacy_body, BODY);
        let legacy = probe_image(&legacy_body, 396, 504);
        let mut checked = 0;
        for row in 0..504 {
            for sample in 0..33 {
                if row % 4 == 0 || row % 4 == 3 || sample >= 25 {
                    let expected = legacy[row * 396 + sample];
                    for path in 0..2 {
                        assert_eq!(
                            pixels[row * 396 + path * 33 + sample],
                            expected,
                            "unaffected PRISM regions row={row} path={path} sample={sample}"
                        );
                        checked += 1;
                    }
                }
            }
        }
        println!("PRISM_UNAFFECTED_REGION_CHECKS={checked}");
    }

    let label = if gate { "gate" } else { "controls" };

    let mut csv = String::from("row,path,sample,r,g,b,a\n");
    let mut max_jump = [0f32; 10];
    let mut same = [0f32; 10];
    let mut cap_error = 0f32;
    let mut peak = [[0f32; 3]; 10];
    let mut failures = Vec::new();
    for row in 0..504 {
        for path in 0..12 {
            for s in 0..33 {
                let v = pixels[row * 396 + path * 33 + s];
                assert!(
                    v.iter().all(|x| x.is_finite()),
                    "nonfinite {row}/{path}/{s}: {v:?}"
                );
                csv += &format!("{row},{path},{s},{},{},{},{}\n", v[0], v[1], v[2], v[3]);
                if path < 10 {
                    for c in 0..3 {
                        peak[path][c] = peak[path][c].max(v[c]);
                    }
                }
                if path == 0 {
                    assert_eq!(v, pixels[row * 396 + 33 + s], "helper/eval weights");
                }
                if path == 2 && v != pixels[row * 396 + 99 + s] {
                    println!(
                        "dispatch parity row={row} sample={s} raw={v:?} dispatch={:?}",
                        pixels[row * 396 + 99 + s]
                    );
                }
                if path == 4 || path == 9 {
                    assert_eq!(
                        &v[..3],
                        &[0.; 3],
                        "zero mask / feature before wall ownership"
                    );
                }
                if path == 10 {
                    cap_error = cap_error.max((v[0] - v[1]).abs());
                }
                if path == 11 {
                    assert_eq!(
                        v,
                        [
                            [0., 128., 129., 255.][row % 4],
                            [0., 64., 255.][(row / 4) % 3],
                            [0., 64., 255.][(row / 12) % 3],
                            5.
                        ]
                    );
                }
            }
        }
    }
    for row in 0..504 {
        for path in 0..10 {
            let base = row * 396 + path * 33;
            let eps = 4 * 5;
            let mut jump = 0f32;
            let mut ss = 0f32;
            for c in 0..3 {
                let at = |s: usize| pixels[base + eps + s][c];
                jump = jump
                    .max((at(1) - at(3)).abs())
                    .max((at(1) - at(2)).abs())
                    .max((at(2) - at(3)).abs());
                ss = ss.max((at(0) - at(1)).abs()).max((at(3) - at(4)).abs());
            }
            max_jump[path] = max_jump[path].max(jump);
            same[path] = same[path].max(ss);
            let tol = if path < 2 { 0.005 } else { 0.01 };
            if jump > tol {
                failures.push((row, path, jump, ss));
            }
        }
    }
    if let Some(out) = std::env::var_os("EPU_DIAGNOSTIC_OUTPUT_DIR") {
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        let artifact = out.join(format!("split-prism-{label}.csv"));
        assert!(!artifact.exists(), "preserve evidence");
        std::fs::write(artifact, csv).unwrap();
    }
    println!(
        "PRISM max bilateral/on-cut={max_jump:?} same-side={same:?} peaks={peak:?} floor reversed/complement max={cap_error}"
    );
    println!("PRISM failures={failures:?}");
    assert!(
        cap_error <= 0.001,
        "backend reversed smoothstep characterization"
    );
    for c in 0..3 {
        assert!(peak[0][c] > 0.1, "nonzero cap/interior region {c}");
    }
    for path in 5..9 {
        assert!(peak[path][0] > 0.01, "masked feature nonzero {path}");
    }
    assert!(
        same.iter().all(|x| *x < 0.005),
        "same-side final-scale control"
    );
    if gate {
        assert!(
            failures.is_empty(),
            "PRISM angular continuity RED (.005 region/.01 RGB)"
        );
    }
}
#[test]
fn split_prism_controls() {
    run(false);
}
#[test]
fn split_prism_angular_gate() {
    run(true);
}
