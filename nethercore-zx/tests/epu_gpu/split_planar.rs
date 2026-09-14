//! Finite fixed-axis planar slice; no PRISM/BANDS, no production substitution.
use super::*;
const SAMPLES: usize = 23;
const PATHS: usize = 10;
const BODY: &str = r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let variant=VARIANTu; let boundaries=BOUNDARIESu;
 let az=p.y%8u; let edge=(p.y/8u)%boundaries;
 let control=(p.y/(8u*boundaries))%9u; let wide=p.y/(72u*boundaries);
 let pa=select(0u,255u,wide==1u);
 var pb=select(0u,255u,(control&1u)!=0u);
 var pc=select(0u,255u,(control&2u)!=0u);
 var pd=select(0u,255u,(control&4u)!=0u);
 if control==8u {pb=127u;pc=128u;pd=129u;}
 let n=decode_dir16(0x8080u);let basis=split_build_basis(n);
 let bw=max(u8_to_01(pa)*0.2,0.001);let angle=u8_to_01(pb)*PI;
 let n1=split_rotate_around_axis(n,basis[1],angle);
 var normal=n;var level=0.;
 if variant==0u {level=f32(i32(edge)-1)*bw;}
 if variant==1u {
   if edge<6u {normal=select(n,n1,edge>=3u);level=f32(i32(edge%3u)-1)*bw;}
   else {normal=n+n1;if length(normal)<0.0001 {normal=basis[0];}}
 }
 if variant==2u {normal=basis[edge/3u];level=f32(i32(edge%3u)-1)*bw;}
 if variant==4u {normal=select(n,basis[0],edge==1u);}
 if variant==6u {
   normal=n+basis[1]*mix(-0.35,0.35,angle/PI);
   let span=max(u8_to_01(pc)*0.5,0.02);let center=u8_to_01(pd)*2.-1.;
   level=center+select(-span,span,edge>=3u)+f32(i32(edge%3u)-1)*bw;
 }
 if variant==7u {
   normal=n+basis[1]*mix(-0.25,0.25,angle/PI);
   let width=mix(0.08,0.45,u8_to_01(pc));let center=u8_to_01(pd)*1.2-0.6;
   let edges=array<f32,7>(0.,-width*0.4,width*0.4,-width,width,-width-bw,width+bw);
   level=center+edges[edge];
 }
 let path=p.x/23u;let sample=p.x%23u;
 var delta=0.;if sample<15u {delta=f32(i32(sample%5u)-2)*array<f32,3>(0.0001,0.00001,0.000001)[sample/5u];}
 let unit=normalize(normal);let ring=split_build_basis(unit);
 let h=level/length(normal);var reachable=abs(h)<0.9999;
 let height=clamp(h+delta,-1.,1.);let phi=f32(az)*TAU/8.;
 var dir=normalize(unit*height+sqrt(max(0.,1.-height*height))*(ring[0]*cos(phi)+ring[1]*sin(phi)));
 if variant==4u && edge>=2u {
   // Product support d=-q0*q1 at +/-bw. Preserve the quadrant planes/intersection.
   let q1=select(-0.6,0.6,edge>=4u);let product_level=select(-bw,bw,(edge%2u)==1u);
   let q0=product_level/q1+delta;let residual=sqrt(max(0.,1.-q0*q0-q1*q1));
   dir=normalize(n*q0+basis[0]*q1+basis[1]*select(-residual,residual,(az%2u)==1u));reachable=true;
 }
 if sample>=15u {
   let k=sample-15u;
   dir=normalize(n*select(-1.,1.,(k&1u)!=0u)+basis[0]*select(-1.,1.,(k&2u)!=0u)+basis[1]*select(-1.,1.,(k&4u)!=0u));
 }
 let instr=vec4u((pd<<24u)|(0x8080u<<8u)|240u,(pa<<16u)|(pb<<8u)|pc,0x4060c080u,(4u<<27u)|(7u<<24u)|(3u<<21u)|(variant<<16u)|0xe080u);
 let v=eval_split(dir,instr,RegionWeights(0.,1.,0.));
 var layers:array<vec4u,8>;layers[0]=instr;
 var out=vec4f(v.regions.sky,v.regions.wall,v.regions.floor,v.region_mix);
 if path==1u {out=vec4f(v.sample.rgb,v.sample.w);}
 if path==2u {out=vec4f(evaluate_epu_layers(dir,layers),1.);}
 if path>=3u && path<=8u {
   let mask=array<u32,6>(0u,1u,2u,4u,7u,2u)[path-3u];
   layers[0].x=layers[0].x&0xffffff0fu;
   let axes=array<u32,6>(0x80ffu,0x8000u,0xff80u,0x0080u,0x8080u,0xffffu);
   for(var j=0u;j<6u;j++) {layers[j+1u]=vec4u((axes[j]<<8u)|240u,32u<<24u,0x80808080u,(18u<<27u)|(mask<<24u)|0x8080u);}
   if path==8u {let last=layers[0];layers[0]=layers[6];layers[6]=vec4u(0u);layers[7]=last;}
   out=vec4f(evaluate_epu_layers(dir,layers),1.);
 }
 if path==9u {out=vec4f(f32(instr_b(instr)),f32(instr_c(instr)),f32(instr_d(instr)),select(0.,1.,reachable));}
 textureStore(result,p.xy,out);
}
"#;
fn run(variant: u32, boundaries: usize) {
    let rows = 2 * 9 * boundaries * 8;
    let width = SAMPLES * PATHS;
    let body = BODY
        .replace("VARIANT", &variant.to_string())
        .replace("BOUNDARIES", &boundaries.to_string());
    let pixels = probe_image(&body, width as u32, rows as u32);
    if let Some(out) = std::env::var_os("EPU_DIAGNOSTIC_OUTPUT_DIR") {
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        let artifact = out.join(format!("split-planar-{variant}.f32"));
        assert!(!artifact.exists(), "preserve evidence");
        std::fs::write(artifact, bytemuck::cast_slice(&pixels)).unwrap();
    }
    assert_eq!(pixels.len(), rows * width);
    let mut maximum = [[0f32; 3]; 9];
    let mut same = [[0f32; 3]; 9];
    let mut peaks = [[0f32; 3]; 9];
    let mut applicable = 0;
    for row in 0..rows {
        let at = |path: usize, s: usize| pixels[row * width + path * SAMPLES + s];
        let control = (row / (8 * boundaries)) % 9;
        let expected = if control == 8 {
            [127., 128., 129.]
        } else {
            std::array::from_fn(|c| if control & (1 << c) != 0 { 255. } else { 0. })
        };
        for s in 0..SAMPLES {
            assert_eq!(&at(9, s)[..3], &expected, "packing");
            for path in 0..PATHS {
                assert!(
                    at(path, s).iter().all(|v| v.is_finite()),
                    "finite {variant}/{row}/{path}/{s}"
                );
            }
            let w = at(0, s);
            assert!(
                (w[0] + w[1] + w[2] - 1.).abs() < 0.005
                    && w[..3].iter().all(|x| *x >= 0. && *x <= 1.)
            );
            for c in 0..3 {
                assert!(
                    (at(1, s)[c] - at(2, s)[c]).abs() <= 0.01,
                    "raw/full RGB budget"
                );
            }
            for path in [3, 8] {
                assert_eq!(&at(path, s)[..3], &[0.; 3], "zero mask/before ownership");
            }
            for path in 0..9 {
                for c in 0..3 {
                    peaks[path][c] = peaks[path][c].max(at(path, s)[c]);
                }
            }
        }
        if at(9, 0)[3] == 0. {
            continue;
        }
        applicable += 1;
        for path in 0..9 {
            let tol = if path == 0 { 0.005 } else { 0.01 };
            let mut previous = 0.;
            for scale in 0..3 {
                let mut jump = 0f32;
                let mut ss = 0f32;
                for c in 0..3 {
                    let v = |s: usize| at(path, scale * 5 + s)[c];
                    jump = jump
                        .max((v(1) - v(3)).abs())
                        .max((v(1) - v(2)).abs())
                        .max((v(2) - v(3)).abs());
                    ss = ss.max((v(0) - v(1)).abs()).max((v(3) - v(4)).abs());
                }
                maximum[path][scale] = maximum[path][scale].max(jump);
                same[path][scale] = same[path][scale].max(ss);
                if scale > 0 {
                    assert!(
                        jump <= previous * 0.35 + 0.001,
                        "noncontracting {variant}/{row}/{path}/{scale}: {previous} -> {jump}"
                    );
                }
                if scale == 2 {
                    assert!(
                        jump <= tol && ss <= tol,
                        "boundary {variant}/{row}/{path} jump={jump} same={ss}"
                    );
                }
                previous = jump;
            }
        }
    }
    assert!(
        peaks[0].iter().all(|p| *p > 0.1),
        "nonzero sky/wall/floor {peaks:?}"
    );
    for path in [1, 2, 4, 5, 6, 7] {
        assert!(peaks[path].iter().any(|p| *p > 0.01), "nonzero path {path}");
    }
    println!(
        "SPLIT_PLANAR variant={variant} rows={rows} applicable={applicable} unreachable={} pixels={} maxima={maximum:?} same={same:?} peaks={peaks:?} GPU_PASS",
        rows - applicable,
        pixels.len()
    );
}
#[test]
fn half() {
    run(0, 3);
}
#[test]
fn wedge() {
    run(1, 7);
}
#[test]
fn corner() {
    run(2, 9);
}
#[test]
fn cross() {
    run(4, 6);
}
#[test]
fn tier() {
    run(6, 6);
}
#[test]
fn face() {
    run(7, 7);
}
