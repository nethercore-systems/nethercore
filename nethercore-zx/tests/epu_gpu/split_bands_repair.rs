//! Frozen BANDS reverse-edge behavioral gate; real production WGSL and ordered masks.
use super::*;
const PREFIX: &str = r#"
fn bands_direction(h:f32, phi:f32, n:vec3f, basis:mat3x3f)->vec3f {
 return normalize(n*h+sqrt(1.-h*h)*(basis[0]*cos(phi)+basis[1]*sin(phi)));
}
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let az=p.y%2u;let kind=(p.y/2u)%4u;let ci=(p.y/8u)%4u;
 let oi=(p.y/32u)%3u;let wide=p.y/96u;
 let pa=array<u32,3>(0u,32u,255u)[wide];let pb=128u;
 let pc=array<u32,4>(0u,127u,128u,255u)[ci];
 let pd=array<u32,3>(0u,129u,255u)[oi];let variant=3u;
 let n=decode_dir16(0x8080u);let basis=split_build_basis(n);
 let count=mix(2.,16.,u8_to_01(pc));let offset=u8_to_01(pd);
 let bw=max(u8_to_01(pa)*.2,.001);let phi=f32(az)*1.7;
 let mid=bands_phase(bands_direction(0.,phi,n,basis),n,basis,count,offset,bw).x;
 let base=floor(mid+.5);let boundary=base+array<f32,4>(.5,0.,-.25,.25)[kind];
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
fn reverse_continuity() {
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
 if path==12u {let w=bands_original(dir,n,basis,count,offset,bw);out=vec4f(w.sky,w.wall,w.floor,1.);}
 textureStore(result,p.xy,out);"#);
    // Frozen phase locates authored boundaries; original preserves pre-repair parity.
    // Keep independent of production split_bands: do not regenerate these oracles.
    let phase = include_str!("fixtures/split-bands/phase.wgsl");
    let original = include_str!("fixtures/split-bands/original-function.wgsl");
    let pixels = probe_image(&format!("{phase}\n{original}\n{PREFIX}{tail}"), 299, 288);
    if let Ok(stage) = std::env::var("BANDS_REPAIR_STAGE") {
        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../tmp/epu-review/split-bands-repair");
        let artifact = out.join(format!("{stage}.f32"));
        assert!(!artifact.exists());
        std::fs::create_dir_all(&out).unwrap();
        std::fs::write(artifact, bytemuck::cast_slice(&pixels)).unwrap();
    }
    let mut failures = Vec::new();
    let mut maxima = [[0f32; 3]; 9];
    let mut parity = 0f32;
    let mut parity_count = 0;
    for row in 0..288 {
        let at = |p: usize, s: usize| pixels[row * 299 + p * 23 + s];
        let wide = row / 96;
        for s in 0..23 {
            for p in 0..13 {
                assert!(at(p, s).iter().all(|x| x.is_finite()));
            }
            assert_eq!(
                at(9, s),
                [
                    128.,
                    [0., 127., 128., 255.][(row / 8) % 4],
                    [0., 129., 255.][(row / 32) % 3],
                    1.
                ]
            );
            let w = at(0, s);
            assert!(
                (w[0] + w[1] + w[2] - 1.).abs() < 0.005
                    && w[..3].iter().all(|x| *x >= 0. && *x <= 1.)
            );
            for c in 0..3 {
                assert!((at(1, s)[c] - at(2, s)[c]).abs() <= 0.01);
            }
            for p in [3, 8] {
                assert_eq!(&at(p, s)[..3], &[0.; 3]);
            }
            if s >= 15 {
                let x = [-0.4f32, -0.25, -0.1, 0., 0.1, 0.25, 0.4, 0.49][s - 15];
                let width = at(10, s)[1];
                // Outside reverse support, or in the untouched inner quarter: old profile parity.
                if x.abs() <= 0.25 || 0.5 - x.abs() >= width + 0.001 {
                    parity_count += 1;
                    for c in 0..3 {
                        parity = parity.max((at(0, s)[c] - at(12, s)[c]).abs());
                    }
                }
            }
        }
        for c in 0..3 {
            assert!(
                (15..23).map(|s| at(0, s)[c]).fold(0f32, f32::max)
                    > if wide == 0 { 0.99 } else { 0.01 },
                "nonzero/sharp {row}/{c}"
            );
        }
        for k in 0..3 {
            assert!(at(10, k * 5 + 1)[0] < 0. && at(10, k * 5 + 3)[0] > 0.);
            assert!(at(10, k * 5 + 2)[0].abs() < 0.1);
        }
        for p in 0..9 {
            let mut prev = 0.;
            for k in 0..3 {
                let mut j = 0f32;
                let mut ss = 0f32;
                for c in 0..3 {
                    for (a, b) in [(1, 3), (1, 2), (2, 3)] {
                        j = j.max((at(p, k * 5 + a)[c] - at(p, k * 5 + b)[c]).abs());
                    }
                    for (a, b) in [(0, 1), (3, 4)] {
                        ss = ss.max((at(p, k * 5 + a)[c] - at(p, k * 5 + b)[c]).abs());
                    }
                }
                maxima[p][k] = maxima[p][k].max(j);
                let tol = if p == 0 { 0.005 } else { 0.01 };
                if (k > 0 && j > prev * 0.35 + 0.001) || (k == 2 && (j > tol || ss > tol)) {
                    failures.push((row, p, k, j, ss));
                }
                prev = j;
            }
        }
    }
    println!(
        "BANDS_REPAIR maxima={maxima:?} parity={parity} parity_count={parity_count} failures={} first={:?}",
        failures.len(),
        failures.first()
    );
    assert!(parity <= 0.005 && parity_count > 0);
    assert!(failures.is_empty(), "frozen continuity failed");
}
