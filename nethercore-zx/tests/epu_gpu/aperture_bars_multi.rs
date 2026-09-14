//! Finite fixed-orientation BARS/MULTI gate. Production WGSL; no CPU evaluator.
use super::*;
const S: usize = 31;
const P: usize = 10;
const ROWS: usize = 960;
const BODY: &str = r#"
fn bm_sdf(q:vec2f,w:f32,h:f32,count:u32,variant:u32)->f32 {
 if variant==4u {return aperture_sdf_bars(q,w,h,count);}
 return aperture_sdf_multi(q,w,h,count);
}
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let k=p.y%16u;let kind=(p.y/16u)%5u;let config=(p.y/80u)%2u;
 let ci=(p.y/160u)%3u;let variant=4u+p.y/480u;
 let count=select(array<u32,3>(1u,4u,16u)[ci],array<u32,3>(1u,4u,8u)[ci],variant==5u);
 let pa=select(0u,137u,config==1u);let pb=select(0u,89u,config==1u);
 let pc=select(0u,28u,config==1u);let soft=select(0u,255u,config==1u);
 let w=mix(.1,1.5,f32(pa)/255.);let h=mix(.1,1.5,f32(pb)/255.);
 let frame=mix(.02,.5,f32(pc)/255.);let spacing=2.*w/f32(count+1u);
 let cell=vec2f(2.*w,2.*h)/f32(count);
 let along=(f32(k)+.37)/16.;let y=(along*1.8-.9)*h;
 let n=decode_dir16(0x8080u);let r=normalize(cross(vec3f(0.,1.,0.),n));let t=normalize(cross(n,r));
 let path=p.x/31u;let sample=p.x%31u;
 var q=vec2f(0.,y);var travel=vec2f(1.,0.);var applicable=true;
 if variant==4u {
  let index=1u+k%count;
  // Moving nearest-bar search window; NOT a bar's authored contour.
  q.x=(f32(k%(count+1u))+.5)*spacing-w;
  if kind==1u {
   // Discover first genuine bar entrance within a bounded nominal-bar interval.
   // Bisection uses production SDF sign, not a transcribed bow/profile formula.
   let start=f32(index)*spacing-w-.4*spacing;
   var lo=start;var hi=start;var found=false;
   for(var j=1u;j<=32u;j++) {
    let x=start+f32(j)/32.*.8*spacing;
    if !found && bm_sdf(vec2f(lo,y),w,h,count,variant)<0. && bm_sdf(vec2f(x,y),w,h,count,variant)>=0. {hi=x;found=true;}
    if !found {lo=x;}
   }
   applicable=found;
   for(var j=0u;j<24u;j++) {let mid=(lo+hi)*.5;if bm_sdf(vec2f(mid,y),w,h,count,variant)<0. {lo=mid;} else {hi=mid;}}
   q.x=(lo+hi)*.5;
  }
  if kind==2u {q=vec2f((along*1.8-.9)*w,h);travel=vec2f(0.,1.);}
  if kind==4u {q=vec2f(f32(index)*spacing-w,0.);travel=vec2f(0.,1.);}
 } else {
  // Interior floor(cell_phase) owner boundaries, including diagonal intersections.
  let id=1u+k%max(count-1u,1u);
  q=vec2f(f32(id)*cell.x-w,y);
  if kind<=2u {applicable=count>1u;}
  if kind==1u {q=vec2f((along*1.8-.9)*w,f32(id)*cell.y-h);travel=vec2f(0.,1.);}
  if kind==2u {q=vec2f(f32(id))*cell-vec2f(w,h);travel=vec2f(1.,1.);}
  // Centered-periodic coordinate fold: |local| has an intentional opening gap.
  if kind==4u {q.x=(f32(k%count)+.5)*cell.x-w;}
 }
 if kind==3u {q.x=w;}
 var delta=0.;if sample<15u {delta=f32(i32(sample%5u)-2)*array<f32,3>(.0001,.00001,.000001)[sample/5u];}
 q+=travel*delta;
 if sample>=15u {
  q=vec2f((f32(sample-15u)/13.*2.-1.)*w,y);
  if sample==29u {q=vec2f(w+frame*.5,0.);}
  if sample==30u {q=vec2f(w+frame*2.,0.);}
 }
 let dir=normalize(n+r*q.x+t*q.y);
 let projected=(dir-n*dot(dir,n))/dot(dir,n);let uv=vec2f(dot(projected,r),dot(projected,t));
 let instr=vec4u((count<<24u)|(0x8080u<<8u),(soft<<24u)|(pa<<16u)|(pb<<8u)|pc,0x2060c080u,(7u<<27u)|(7u<<24u)|(3u<<21u)|(variant<<16u)|0xe080u);
 let v=evaluate_bounds_layer(dir,instr,OP_APERTURE,n,RegionWeights(0.,1.,0.));
 var layers:array<vec4u,8>;layers[0]=instr;
 var out=vec4f(v.regions.sky,v.regions.wall,v.regions.floor,v.sample.w);
 if path==1u {out=vec4f(v.sample.rgb,v.sample.w);}
 if path==2u {out=vec4f(evaluate_epu_layers(dir,layers),1.);}
 if path>=3u && path<=7u {
  let mask=array<u32,5>(0u,4u,2u,1u,7u)[path-3u];
  // APERTURE paint deliberately stays present (its alphas are unused).
  // Later six-direction LOBEs exercise actual ordered masked composition.
  let axes=array<u32,6>(0x80ffu,0x8000u,0xff80u,0x0080u,0x8080u,0xffffu);
  for(var j=0u;j<6u;j++) {layers[j+1u]=vec4u((axes[j]<<8u)|240u,32u<<24u,0x80808080u,(18u<<27u)|(mask<<24u)|0x8080u);}
  out=vec4f(evaluate_epu_layers(dir,layers),1.);
 }
 if path==8u {out=vec4f(bm_sdf(uv,w,h,count,variant),(uv.x-q.x)*1000000.,(uv.y-q.y)*1000000.,select(0.,1.,applicable));}
 if path==9u {out=vec4f(f32(instr_variant_id(instr)),f32(instr_d(instr)),f32(instr_intensity(instr)),f32(instr_c(instr)));}
 textureStore(result,p.xy,out);
}
"#;
#[test]
fn continuity() {
    check(ROWS);
}
#[test]
fn bars_high() {
    check(480);
}
fn check(rows: usize) {
    let stage = std::env::var("APERTURE_BM_STAGE").unwrap_or("manual".into());
    // Isolated LOW candidate only; no production substitution or adoption.
    let candidate = stage.starts_with("candidate");
    let mut body = BODY.to_string();
    if candidate {
        let aperture = include_str!("../../shaders/epu/bounds/06_aperture.wgsl");
        let bars_old = "if bar_dist < 0.0 {\n            return -bar_dist; // Inside bar = positive SDF (outside opening)\n        }";
        assert_eq!(aperture.matches(bars_old).count(), 1);
        let aperture = aperture
            .replace(bars_old, "return max(base_sdf, -max(bar_dist, base_sdf));")
            .replace(
                "let cell_id = floor(cell_phase);",
                "let cell_id = floor(cell_phase + vec2f(0.5));",
            )
            .replace("aperture_", "candidate_aperture_")
            .replace("fn eval_aperture(", "fn candidate_eval_aperture(");
        let dispatch = include_str!("../../shaders/epu/epu_dispatch.wgsl")
            .replace("evaluate_bounds_layer", "candidate_evaluate_bounds_layer")
            .replace("evaluate_epu_layers", "candidate_evaluate_epu_layers")
            .replace("evaluate_layer", "candidate_evaluate_layer")
            .replace("eval_aperture(", "candidate_eval_aperture(");
        body = body
            .replace("aperture_sdf_", "candidate_aperture_sdf_")
            .replace("evaluate_bounds_layer", "candidate_evaluate_bounds_layer")
            .replace("evaluate_epu_layers", "candidate_evaluate_epu_layers");
        body = format!("{aperture}\n{dispatch}\n{body}");
    }
    let pixels = probe_image(&body, (S * P) as u32, rows as u32);
    // Diagnostic captures are opt-in; ordinary test runs must be repeatable.
    if std::env::var_os("APERTURE_BM_STAGE").is_some() {
        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../tmp/epu-review/aperture-bars-multi");
        std::fs::create_dir_all(&out).unwrap();
        let artifact = out.join(format!("{stage}.f32"));
        assert!(!artifact.exists());
        std::fs::write(artifact, bytemuck::cast_slice(&pixels)).unwrap();
    }
    assert_eq!(pixels.len(), rows * S * P);
    let mut failures = Vec::new();
    let mut parity_failures = Vec::new();
    let mut applicable = [0usize; 10];
    let mut maximum = [[[0f32; 3]; 8]; 10];
    let mut same = maximum;
    let mut peaks = vec![[0f32; 3]; rows / 80];
    for row in 0..rows {
        let at = |p: usize, s: usize| pixels[row * S * P + p * S + s];
        let variant = row / 480;
        let kind = (row / 16) % 5;
        let group = variant * 5 + kind;
        let config = (row / 80) % 2;
        let ci = (row / 160) % 3;
        let control = variant * 6 + ci * 2 + config;
        for s in 0..S {
            for p in 0..P {
                assert!(
                    at(p, s).iter().all(|v| v.is_finite()),
                    "finite {row}/{p}/{s}"
                );
            }
            assert_eq!(
                at(9, s),
                [
                    4. + variant as f32,
                    if variant == 0 {
                        [1., 4., 16.][ci]
                    } else {
                        [1., 4., 8.][ci]
                    },
                    [0., 255.][config],
                    [0., 28.][config]
                ]
            );
            let w = at(0, s);
            assert!((w[0] + w[1] + w[2] - 1.).abs() < 0.005);
            for c in 0..3 {
                let parity = (at(1, s)[c] - at(2, s)[c])
                    .abs()
                    .max((at(2, s)[c] - at(3, s)[c]).abs());
                if parity > 0.01 {
                    parity_failures.push((row, s, c, parity));
                }
                peaks[control][c] = peaks[control][c].max(w[c]);
            }
        }
        assert!(
            at(0, 29)[1] > 0.99 && at(0, 30)[2] > 0.99,
            "external frame/outside controls {row}"
        );
        if at(8, 12)[3] == 0. {
            continue;
        }
        applicable[group] += 1;
        for p in 0..8 {
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
                maximum[group][p][k] = maximum[group][p][k].max(j);
                same[group][p][k] = same[group][p][k].max(ss);
                if k == 2
                    && (j > if p == 0 { 0.005 } else { 0.01 }
                        || ss > if p == 0 { 0.005 } else { 0.01 })
                {
                    failures.push((row, p, j, ss));
                }
            }
        }
    }
    for g in 0..10 {
        println!(
            "APERTURE_BM group={g} applicable={} bilateral={:?} same_side={:?}",
            applicable[g], maximum[g], same[g]
        );
    }
    println!(
        "APERTURE_BM peaks={peaks:?} failures={} first={:?} parity_failures={} parity_first={:?}",
        failures.len(),
        &failures[..failures.len().min(12)],
        parity_failures.len(),
        parity_failures.first()
    );
    for w in peaks {
        assert!(
            w[0] > 0.5 && w[1] > 0.99 && w[2] > 0.99,
            "nonzero opening/frame/outside"
        );
    }
    assert!(
        failures.is_empty() && parity_failures.is_empty(),
        "APERTURE BARS/MULTI declared .005 region/.01 RGB limits"
    );
}

// Same seam/mask discriminator at the full MULTI counts and parameter endpoints.
#[test]
fn multi_continuity_stress() {
    let mut configs = Vec::new();
    for count in [1, 4, 8] {
        for (a, b, c, softness) in [(0, 0, 0, 0), (137, 89, 28, 255)] {
            configs.push((a, b, count, softness, c, 0x8080));
        }
    }
    for count in 1..=8 {
        let even = count % 2 == 0;
        configs.push((
            if even { 0 } else { 255 },
            if even { 255 } else { 0 },
            count,
            255,
            255,
            if even { 0xff80 } else { 0x8080 },
        ));
    }
    configs.extend([
        (0, 0, 0, 0, 0, 0xffff),
        (255, 255, 255, 0, 0, 0xffff),
        (0, 255, 255, 255, 128, 0xff80),
        (255, 0, 0, 255, 128, 0x8080),
    ]);
    let mut failures = 0;
    let mut parity_failures = 0;
    for &(a, b, raw, soft, c, direction) in &configs {
        let mut body = BODY.replace("let variant=4u+p.y/480u;", "let variant=5u;");
        let start = body.find(" let count=select(").unwrap();
        let end = body[start..].find(" let w=mix(").unwrap() + start;
        body.replace_range(start..end, &format!(" let count=clamp({raw}u,1u,8u);let pa={a}u;let pb={b}u;let pc={c}u;let soft={soft}u;\n"));
        body = body.replace("let n=decode_dir16(0x8080u);let r=normalize(cross(vec3f(0.,1.,0.),n));let t=normalize(cross(n,r));", &format!("let n=decode_dir16({direction}u);let up=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(n.y)>.9);let r=normalize(cross(up,n));let t=normalize(cross(n,r));"));
        body = body.replace(
            "(count<<24u)|(0x8080u<<8u)",
            &format!("({raw}u<<24u)|({direction}u<<8u)"),
        );
        let pixels = probe_image(&body, 310, 80);
        assert_eq!(pixels.len(), 310 * 80);
        assert!(pixels.iter().flatten().all(|v| v.is_finite()));
        let mut peak = [0.0f32; 3];
        let before = failures;
        for row in 0..80 {
            let at = |path: usize, sample: usize| pixels[row * 310 + path * 31 + sample];
            for s in 0..31 {
                assert_eq!(at(9, s), [5., raw as f32, soft as f32, c as f32]);
                let w = at(0, s);
                assert!((w[0] + w[1] + w[2] - 1.).abs() <= 0.005);
                for channel in 0..3 {
                    peak[channel] = peak[channel].max(w[channel]);
                    parity_failures += usize::from(
                        (at(1, s)[channel] - at(2, s)[channel])
                            .abs()
                            .max((at(2, s)[channel] - at(3, s)[channel]).abs())
                            > 0.01,
                    );
                }
            }
            assert!(at(0, 29)[1] > 0.99 && at(0, 30)[2] > 0.99);
            if at(8, 12)[3] == 0. {
                continue;
            }
            for path in 0..9 {
                let channels = if path == 8 { 1 } else { 3 };
                let limit = if path == 0 || path == 8 { 0.005 } else { 0.01 };
                let mut previous = f32::INFINITY;
                for k in 0..3 {
                    let mut bilateral = 0.0f32;
                    let mut same_side = 0.0f32;
                    for channel in 0..channels {
                        for (x, y) in [(1, 3), (1, 2), (2, 3)] {
                            bilateral = bilateral.max(
                                (at(path, k * 5 + x)[channel] - at(path, k * 5 + y)[channel]).abs(),
                            );
                        }
                        for (x, y) in [(0, 1), (3, 4)] {
                            same_side = same_side.max(
                                (at(path, k * 5 + x)[channel] - at(path, k * 5 + y)[channel]).abs(),
                            );
                        }
                    }
                    if k > 0 && bilateral > previous * 0.35 + 0.001 {
                        failures += 1;
                    }
                    if k == 2 && bilateral.max(same_side) > limit {
                        failures += 1;
                    }
                    previous = bilateral;
                }
            }
        }
        assert!(peak[0] > 0.5 && peak[1] > 0.99 && peak[2] > 0.99);
        println!(
            "MULTI_STRESS a={a} b={b} count={raw} soft={soft} frame={c} direction={direction} continuity_failures={}",
            failures - before
        );
    }
    println!(
        "MULTI_STRESS cases={} failures={failures} parity_failures={parity_failures}",
        configs.len()
    );
    assert_eq!((failures, parity_failures), (0, 0));
}

// Independent rectangle-union oracle: includes real edges and rejects phantom zeros.
const MULTI_RECT_ORACLE: &str = r#"
fn window_box(w:f32,h:f32,count:u32,ix:u32,iy:u32)->vec4f {
 let size=vec2f(2.*w,2.*h)/f32(count);let g=min(size.x,size.y)*.05;
 let lo=vec2f(-w,-h)+vec2f(f32(ix),f32(iy))*size+vec2f(select(0.,g,ix>0u),select(0.,g,iy>0u));
 let hi=vec2f(-w,-h)+vec2f(f32(ix+1u),f32(iy+1u))*size-vec2f(select(0.,g,ix+1u<count),select(0.,g,iy+1u<count));
 return vec4f(lo,hi);
}
fn union_oracle(q:vec2f,w:f32,h:f32,count:u32)->f32 {
 var value=1e6;
 for(var y=0u;y<count;y++) {for(var x=0u;x<count;x++) {
  let b=window_box(w,h,count,x,y);
  value=min(value,max(max(b.x-q.x,q.x-b.z),max(b.y-q.y,q.y-b.w)));
 }}
 return value;
}
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let raw=array<u32,10>(0u,1u,2u,3u,4u,5u,6u,7u,8u,255u)[p.y/4u];let count=clamp(raw,1u,8u);
 let cfg=p.y%4u;let pa=array<u32,4>(0u,137u,0u,255u)[cfg];let pb=array<u32,4>(0u,89u,255u,0u)[cfg];let pc=array<u32,4>(0u,28u,128u,255u)[cfg];
 let w=mix(.1,1.5,f32(pa)/255.);let h=mix(.1,1.5,f32(pb)/255.);let frame=mix(.02,.5,f32(pc)/255.);
 let size=vec2f(2.*w,2.*h)/f32(count);let id=p.x%64u;let ix=id%count;let iy=(id/count)%count;
 let rect=window_box(w,h,count,ix,iy);let mid=(rect.xy+rect.zw)*.5;
 var q=(vec2f(f32(p.x%32u)+.371,f32((p.x/32u)%32u)+.413)/32.*2.-1.)*(vec2f(w,h)+vec2f(frame+.1));
 var kind=0u;var normal=vec2f(1.,0.);
 if p.x>=1024u && p.x<1088u {kind=1u;q=vec2f(-w+f32(1u+id%max(count-1u,1u))*size.x,mid.y);if count==1u {q=vec2f(0.);}}
 if p.x>=1088u && p.x<1152u {kind=2u;q=mid;}
 if p.x>=1152u && p.x<1408u {
  kind=3u;let side=(p.x-1152u)/64u;
  q=mid;
  if side==0u {q.x=rect.x;normal=vec2f(-1.,0.);}
  if side==1u {q.x=rect.z;normal=vec2f(1.,0.);}
  if side==2u {q.y=rect.y;normal=vec2f(0.,-1.);}
  if side==3u {q.y=rect.w;normal=vec2f(0.,1.);}
 }
 if p.x>=1408u && p.x<1472u {kind=4u;q=select(vec2f(w+frame,0.),vec2f(0.,h+frame),id%2u==1u);}
 if p.x>=1472u {kind=1u;q=vec2f(mid.x,-h+f32(1u+id%max(count-1u,1u))*size.y);if count==1u {q=vec2f(0.);}}
 var got=aperture_sdf_multi(q,w,h,raw);
 // INJECT_PHANTOM_ZERO
 let expected=union_oracle(q,w,h,count);var ok=true;
 if count==1u {ok=abs(got-max(abs(q.x)-w,abs(q.y)-h))<=.000002;}
 if kind==1u && count>1u {ok=ok && got>0.;}
 if kind==2u {ok=ok && got<0.;}
 if got==0. && abs(expected)>.000002 {ok=false;}
 if kind==3u {
  ok=ok && abs(got)<=.000002;
  ok=ok && aperture_sdf_multi(q-normal*.000002,w,h,raw)<0.;
  ok=ok && aperture_sdf_multi(q+normal*.000002,w,h,raw)>0.;
 }
 if kind==4u {
  let n=decode_dir16(0x8080u);let r=normalize(cross(vec3f(0.,1.,0.),n));let t=normalize(cross(n,r));let dir=normalize(n+r*q.x+t*q.y);
  let instr=vec4u((raw<<24u)|(0x8080u<<8u),(pa<<16u)|(pb<<8u)|pc,0x2060c080u,(7u<<27u)|(7u<<24u)|(3u<<21u)|(5u<<16u)|0xe080u);
  let v=evaluate_bounds_layer(dir,instr,OP_APERTURE,n,RegionWeights(0.,1.,0.));
  ok=ok && abs(v.regions.wall-.5)<=.005 && abs(v.regions.floor-.5)<=.005 && v.regions.sky<.005;
 }
 textureStore(result,p.xy,vec4f(abs(got-expected),select(1.,0.,ok),got,expected));
}
"#;

fn multi_regular_geometry_failures(inject_zero: bool) -> usize {
    let body = if inject_zero {
        MULTI_RECT_ORACLE.replace(
            "// INJECT_PHANTOM_ZERO",
            "if kind==1u && count>1u {got=0.;}",
        )
    } else {
        MULTI_RECT_ORACLE.to_owned()
    };
    let pixels = probe_image(&body, 1536, 40);
    assert_eq!(pixels.len(), 1536 * 40);
    assert!(pixels.iter().flatten().all(|x| x.is_finite()));
    let failures: Vec<_> = pixels
        .iter()
        .enumerate()
        .filter(|(_, p)| p[0] > 0.005 || p[1] != 0.)
        .map(|(i, p)| (i / 1536, i % 1536, *p))
        .collect();
    println!(
        "MULTI_REGULAR_RECTANGLE_ORACLE samples={} failures={} first={:?}",
        pixels.len(),
        failures.len(),
        &failures[..failures.len().min(10)]
    );
    failures.len()
}

#[test]
fn multi_regular_grid_and_exact_contour_contract() {
    assert_eq!(multi_regular_geometry_failures(false), 0);
}

#[test]
fn multi_regular_contour_rejects_phantom_zero() {
    assert!(multi_regular_geometry_failures(true) > 0);
}
