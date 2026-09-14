//! Diagnostic characterization, NOT a blanket continuity acceptance test.
//! Frozen terminal .005 region/.01 RGB; three scales and same-side controls.
use super::*;
const PREFIX: &str = r#"
fn bands_direction(h:f32, phi:f32, n:vec3f, basis:mat3x3f)->vec3f {
 return normalize(n*h+sqrt(1.-h*h)*(basis[0]*cos(phi)+basis[1]*sin(phi)));
}
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let az=p.y%2u;let kind=(p.y/2u)%3u;let ci=(p.y/6u)%4u;
 let oi=(p.y/24u)%3u;let wide=p.y/72u;
 let pa=select(0u,255u,wide==1u);let pb=128u;
 let pc=array<u32,4>(0u,127u,128u,255u)[ci];
 let pd=array<u32,3>(0u,129u,255u)[oi];let variant=3u;
 let n=decode_dir16(0x8080u);let basis=split_build_basis(n);
 let count=mix(2.,16.,u8_to_01(pc));let offset=u8_to_01(pd);
 let bw=max(u8_to_01(pa)*.2,.001);let phi=f32(az)*1.7;
 let mid=bands_phase(bands_direction(0.,phi,n,basis),n,basis,count,offset,bw).x;
 let base=floor(mid+.5);let boundary=base+array<f32,3>(.5,0.,-.25)[kind];
 let path=p.x/23u;let sample=p.x%23u;
 var goal=boundary;
 if sample>=15u {goal=base+array<f32,8>(-.4,-.25,-.1,0.,.1,.25,.4,.49)[sample-15u];}
 var lo=-.95;var hi=.95;
 let reachable=bands_phase(bands_direction(lo,phi,n,basis),n,basis,count,offset,bw).x<goal && bands_phase(bands_direction(hi,phi,n,basis),n,basis,count,offset,bw).x>goal;
 for(var i=0u;i<25u;i++) {
  let h=(lo+hi)*.5;
  if bands_phase(bands_direction(h,phi,n,basis),n,basis,count,offset,bw).x<goal {lo=h;} else {hi=h;}
 }
 var delta=0.;if sample<15u {delta=f32(i32(sample%5u)-2)*array<f32,3>(.0001,.00001,.000001)[sample/5u];}
 let h=(lo+hi)*.5+delta;let dir=bands_direction(h,phi,n,basis);
"#;
#[test]
fn fixed_axis_diagnostic() {
    // Reuse the existing packed raw/ordered-mask fixture without editing it.
    let planar = include_str!("split_planar.rs");
    let tail = planar
        .split(" let instr=vec4u")
        .nth(1)
        .unwrap()
        .split("\"#;")
        .next()
        .unwrap();
    let tail=format!(" let instr=vec4u{tail}").replace(" textureStore(result,p.xy,out);",r#"
 if path==10u {let q=bands_phase(dir,n,basis,count,offset,bw);out=vec4f((q.x-goal)*10000.,q.y,delta*1000000.,select(0.,1.,reachable));}
 if path==11u {out=vec4f(dir,h);}
 textureStore(result,p.xy,out);"#);
    // Extract ONLY a diagnostic coordinate reader from current production source.
    // Evaluated regions/RGB below still use unmodified eval_split/full dispatch.
    let source = include_str!("../../shaders/epu/bounds/03_split.wgsl");
    let function = source
        .split("fn split_bands(")
        .nth(1)
        .unwrap()
        .split("\n}")
        .next()
        .unwrap();
    // Stop at phase shaping, independently of how production returns its weights.
    let coordinates = function
        .split_once("    let d =")
        .expect("BANDS phase boundary")
        .0;
    let phase = format!(
        "fn split_bands({coordinates}    return vec2f(u * band_count + warp, local_bw);\n}}"
    )
    .replace("fn split_bands(", "fn bands_phase(")
    .replace("-> RegionWeights", "-> vec2f");
    let body = format!("{phase}\n{PREFIX}{tail}");
    let pixels = probe_image(&body, 276, 144);
    if let Some(artifact) = std::env::var_os("BANDS_DIAGNOSTIC_OUTPUT") {
        let artifact = std::path::PathBuf::from(artifact);
        assert!(!artifact.exists(), "preserve prior diagnostic readback");
        std::fs::write(artifact, bytemuck::cast_slice(&pixels)).unwrap();
    }
    assert_eq!(pixels.len(), 276 * 144);
    assert!(pixels.iter().flatten().all(|x| x.is_finite()));
    println!(
        "BANDS_CAPTURE rows=144 paths=12 samples=23; current production finite capture, not GPU_PASS continuity; historical verifier remains source-bound"
    );
}
