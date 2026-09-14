//! Finite fixed-axis production boundary gate; no replacement SDF or shader repair.
use super::*;
const S: usize = 18;
const P: usize = 12;
const ROWS: usize = 5760;
const BODY: &str = r#"
fn simple_sdf(q:vec2f,w:f32,h:f32,d:u32,v:u32)->f32 {
 if v==0u {return aperture_sdf_circle(q,w,h);}
 if v==2u {return aperture_sdf_rounded_rect(q,w,h,f32(d)/255.*.5);}
 return aperture_sdf_arch(q,w,h,f32(d)/255.);
}
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let k=p.y%4u;let kind=(p.y/4u)%16u;let soft=(p.y/64u)%2u;
 let di=(p.y/128u)%3u;let config=(p.y/384u)%5u;let vi=p.y/1920u;
 let v=array<u32,3>(0u,2u,3u)[vi];let d=array<u32,3>(0u,128u,255u)[di];
 let pa=array<u32,5>(0u,137u,255u,0u,255u)[config];
 let pb=array<u32,5>(0u,89u,255u,255u,0u)[config];
 let pc=select(0u,255u,config!=0u);let si=soft*255u;
 let w=mix(.1,1.5,f32(pa)/255.);let h=mix(.1,1.5,f32(pb)/255.);
 let frame=mix(.02,.5,f32(pc)/255.);let width=min(mix(.005,.1,f32(si)/255.),frame*.45);
 let n=decode_dir16(0x8080u);let r=normalize(cross(vec3f(0.,1.,0.),n));let t=normalize(cross(n,r));
 let angle=(f32(k)+.5)/4.*TAU;let ray=vec2f(cos(angle),sin(angle));
 let path=p.x/18u;let sample=p.x%18u;
 var delta=0.;if sample<15u {delta=f32(i32(sample%5u)-2)*array<f32,3>(.0001,.00001,.000001)[sample/5u];}
 var q=vec2f(0.);var travel=ray;var level_sdf=0.;var bracket=true;
 if kind<6u {
  level_sdf=array<f32,6>(-width,0.,width,frame-width,frame,frame+width)[kind];
  var lo=0.;var hi=8.;
  bracket=simple_sdf(ray*lo,w,h,d,v)<level_sdf && simple_sdf(ray*hi,w,h,d,v)>level_sdf;
  for(var j=0u;j<25u;j++){let mid=(lo+hi)*.5;if simple_sdf(ray*mid,w,h,d,v)<level_sdf {lo=mid;} else {hi=mid;}}
  q=ray*(lo+hi)*.5;
 }
 if kind==6u {q=vec2f(w-min(f32(d)/255.*.5,min(w,h)),(f32(k)/3.*2.-1.)*(h+frame));travel=vec2f(1.,0.);}
 if kind==7u || kind==8u {
  q=vec2f(array<f32,4>(0.,.5,1.,1.1)[k]*w,select(h*(1.-1.5*f32(d)/255.),0.,kind==8u));travel=vec2f(0.,1.);
 }
 if kind==9u {q=vec2f(0.,h*(1.-1.5*f32(d)/255.))+ray*.001;}
 q+=travel*delta;
 if sample==15u {q=vec2f(0.);}
 if sample>=16u {
  // Discover actual right-side wall/floor controls from the production SDF.
  // Use the short/downward axis so controls stay outside horizon relief.
  let control_ray=select(vec2f(0.,-1.),vec2f(1.,0.),w<=h);
  let level=select(frame*.5,frame+width*2.,sample==17u);var lo=0.;var hi=32.;
  for(var j=0u;j<25u;j++){let mid=(lo+hi)*.5;if simple_sdf(control_ray*mid,w,h,d,v)<level {lo=mid;} else {hi=mid;}}
  q=control_ray*(lo+hi)*.5;
 }
 var dir=normalize(n+r*q.x+t*q.y);
 if kind>=10u && sample<15u {
  let dotn=array<f32,6>(0.,.06,.18,.22,.42,-.06)[kind-10u]+delta;
  dir=normalize(n*dotn+(r*ray.x+t*ray.y)*sqrt(1.-dotn*dotn));
 }
 let instr=vec4u((d<<24u)|(0x8080u<<8u),(si<<24u)|(pa<<16u)|(pb<<8u)|pc,0x2060c080u,(7u<<27u)|(7u<<24u)|(3u<<21u)|(v<<16u)|0xe080u);
 let b=evaluate_bounds_layer(dir,instr,OP_APERTURE,n,RegionWeights(0.,1.,0.));
 var layers:array<vec4u,8>;layers[0]=instr;
 var out=vec4f(b.regions.sky,b.regions.wall,b.regions.floor,b.sample.w);
 if path==1u {out=vec4f(b.sample.rgb*b.sample.w,b.sample.w);}
 if path==2u {out=vec4f(evaluate_epu_layers(dir,layers),1.);}
 if path>=3u && path<=10u {
  let mask=path-3u;
  let axes=array<u32,6>(0x80ffu,0x8000u,0xff80u,0x0080u,0x8080u,0xffffu);
  for(var j=0u;j<6u;j++){layers[j+1u]=vec4u((axes[j]<<8u)|240u,32u<<24u,0x80808080u,(18u<<27u)|(mask<<24u)|0x8080u);}
  out=vec4f(evaluate_epu_layers(dir,layers),1.);
 }
 // Raw SDF diagnostics include deliberately deep-inside ARCH branch changes;
 // continuity gates apply to rendered ownership/RGB, not a demand to redefine its interior field.
 if path==11u {out=vec4f(simple_sdf(q,w,h,d,v),level_sdf,select(0.,1.,bracket),f32(v));}
 textureStore(result,p.xy,out);
}
"#;
#[test]
fn boundaries() {
    // Frozen before GPU: shrinking eps 1e-4/1e-5/1e-6; final region .005,
    // linear RGB .01, same-side limits identical; convergence allows half-float .001.
    let pixels = probe_image(BODY, (S * P) as u32, ROWS as u32);
    if let Ok(stage) = std::env::var("APERTURE_SIMPLE_STAGE") {
        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../tmp/epu-review/aperture-simple-shapes");
        std::fs::create_dir_all(&out).unwrap();
        let artifact = out.join(format!("{stage}.f32"));
        assert!(!artifact.exists(), "preserve evidence");
        std::fs::write(artifact, bytemuck::cast_slice(&pixels)).unwrap();
    }
    assert_eq!(pixels.len(), ROWS * S * P);
    let mut failures = Vec::new();
    let mut maximum = [[[0f32; 3]; 11]; 3];
    let mut same = maximum;
    let mut parity = 0f32;
    let mut masks = [0f32; 8];
    let mut raw_join = 0f32;
    let mut unreachable = 0;
    for row in 0..ROWS {
        let at = |p: usize, s: usize| pixels[row * S * P + p * S + s];
        let vi = row / 1920;
        let kind = (row / 4) % 16;
        for s in 0..S {
            for p in 0..P {
                assert!(at(p, s).iter().all(|x| x.is_finite()));
            }
            assert_eq!(at(11, s)[3], [0., 2., 3.][vi]);
            let w = at(0, s);
            assert!(
                (w[0] + w[1] + w[2] - 1.).abs() < 0.005
                    && w[..3].iter().all(|v| (0.0..=1.0).contains(v))
            );
            for c in 0..3 {
                parity = parity
                    .max((at(1, s)[c] - at(2, s)[c]).abs())
                    .max((at(2, s)[c] - at(3, s)[c]).abs());
                for mask in 0..8 {
                    masks[mask] = masks[mask].max((at(3 + mask, s)[c] - at(2, s)[c]).abs());
                }
            }
        }
        assert!(
            at(0, 15)[0] > 0.5 && at(0, 16)[1] > 0.99 && at(0, 17)[2] > 0.99,
            "ownership {row}"
        );
        if kind == 10 || kind == 15 {
            assert_eq!(at(0, 12)[3], 0., "back/support zero {row}");
        }
        if at(11, 12)[2] == 0. {
            // The -softness level may be at/below the deepest interior point.
            // Retain this row as an origin diagnostic, not a claimed contour.
            assert_eq!(kind, 0);
            unreachable += 1;
        } else if kind < 6 {
            assert!(
                (at(11, 12)[0] - at(11, 12)[1]).abs() < 0.001,
                "real SDF contour {row}"
            );
            assert!(
                at(11, 10)[0] <= at(11, 10)[1] && at(11, 14)[0] >= at(11, 14)[1],
                "opposite SDF predicate {row}"
            );
        }
        if (6..10).contains(&kind) {
            raw_join = raw_join.max((at(11, 11)[0] - at(11, 13)[0]).abs());
        }
        for p in 0..11 {
            let tol = if p == 0 { 0.005 } else { 0.01 };
            let mut j = [0f32; 3];
            let mut ss = j;
            for k in 0..3 {
                for c in 0..if p == 0 { 4 } else { 3 } {
                    for (a, b) in [(1, 3), (1, 2), (2, 3)] {
                        j[k] = j[k].max((at(p, k * 5 + a)[c] - at(p, k * 5 + b)[c]).abs());
                    }
                    for (a, b) in [(0, 1), (3, 4)] {
                        ss[k] = ss[k].max((at(p, k * 5 + a)[c] - at(p, k * 5 + b)[c]).abs());
                    }
                }
                maximum[vi][p][k] = maximum[vi][p][k].max(j[k]);
                same[vi][p][k] = same[vi][p][k].max(ss[k]);
            }
            if j[2] > tol || ss[2] > tol || (1..3).any(|k| j[k] > j[k - 1] * 0.35 + 0.001) {
                failures.push((row, p, j, ss));
            }
        }
    }
    for vi in 0..3 {
        println!(
            "SIMPLE variant={} bilateral={:?} same={:?}",
            [0, 2, 3][vi],
            maximum[vi],
            same[vi]
        );
    }
    println!(
        "SIMPLE rows={ROWS} pixels={} unreachable_inner_shoulders={unreachable} parity={parity} mask_effects={masks:?} raw_interior_join={raw_join} failures={} first={:?}",
        pixels.len(),
        failures.len(),
        &failures[..failures.len().min(12)]
    );
    assert_eq!(unreachable, 72);
    assert!(parity <= 0.01 && masks[0] == 0. && masks[1..].iter().all(|v| *v > 0.01));
    assert!(
        failures.is_empty(),
        "declared .005 region/.01 RGB boundary gates"
    );
}
