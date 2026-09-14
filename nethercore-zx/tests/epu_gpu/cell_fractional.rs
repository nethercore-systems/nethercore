//! Remaining fractional CELL chart-wrap gate; fixed real rays and unchanged budgets.
use super::*;

const BODY: &str = r#"
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let byte=p.y%256u;let variant=array<u32,3>(2u,4u,5u)[p.y/256u];
 let density=mix(4.,64.,u8_to_01(byte));let v=1.37/density;
 let path=p.x/20u;let sample=p.x%20u;
 var delta=0.;
 if sample<15u {delta=f32(i32(sample%5u)-2)*array<f32,3>(.0001,.00001,.000001)[sample/5u];}
 else {delta=array<f32,5>(.13,.4,.71,1.9,2.8)[sample-15u];}
 let instr=vec4u((0x80ffu<<8u)|255u,(191u<<24u)|(byte<<16u)|(128u<<8u)|64u,
 0x4080c060u,(OP_CELL<<27u)|(REGION_ALL<<24u)|(3u<<21u)|(variant<<16u)|0x2040u);
 let axis=decode_dir16(instr_dir16(instr));
 let reference=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(axis.y)>.9);
 let t=normalize(cross(reference,axis));let b=normalize(cross(axis,t));
 let h=v*2.-1.;let dir=normalize(axis*h+sqrt(1.-h*h)*(-t*cos(delta)-b*sin(delta)));
 let uv=cell_axis_cylinder_uv(dir,axis);
 let value=evaluate_bounds_layer(dir,instr,OP_CELL,axis,RegionWeights(.2,.3,.5));
 var layers:array<vec4u,8>;layers[0]=instr;
 var out=vec4f(value.regions.sky,value.regions.wall,value.regions.floor,value.sample.w);
 if path==1u {out=vec4f(value.sample.rgb*value.sample.w,value.sample.w);}
 if path==2u {out=vec4f(evaluate_epu_layers(dir,layers),1.);}
 if path==4u {out=vec4f(apply_blend(vec3f(0.),value.sample,BLEND_LERP),1.);}
 if path==3u {
  var info=cell_voronoi(uv,density,0.);
  if variant==4u {info=cell_shatter(uv,density,0.);}
  if variant==5u {info=cell_brick(uv,density,0.);}
  out=vec4f(shortest_periodic_delta(uv.x,0.,1.)*10000.,info.z,info.xy);
 }
 // INJECT_WRAP_JUMP
 textureStore(result,p.xy,out);
}
"#;

const CURRENT_CELL: &str = include_str!("../../shaders/epu/bounds/04_cell.wgsl");
const LEGACY_CELL: &str = include_str!("fixtures/cell-periodic/legacy-cell.wgsl");

fn failures(inject: bool, legacy: bool) -> usize {
    let body = if inject {
        BODY.replace(
            "// INJECT_WRAP_JUMP",
            "if path==1u && sample<15u && sample%5u<2u {out.x+=.1;}",
        )
    } else {
        BODY.to_owned()
    };
    assert_eq!(EPU_BOUNDS.matches(CURRENT_CELL).count(), 1);
    let bounds = if legacy {
        EPU_BOUNDS.replace(CURRENT_CELL, LEGACY_CELL)
    } else {
        EPU_BOUNDS.to_owned()
    };
    let pixels = probe_image_bounds(&body, &bounds, 100, 768);
    assert_eq!(pixels.len(), 100 * 768);
    assert!(pixels.iter().flatten().all(|x| x.is_finite()));
    let mut bad = Vec::new();
    let mut parity = [0usize; 3];
    let mut continuity = [0usize; 3];
    let mut controls = [0.0f32; 3];
    for row in 0..768 {
        let group = row / 256;
        let at = |path: usize, sample: usize| pixels[row * 100 + path * 20 + sample];
        for sample in 0..20 {
            let region = at(0, sample);
            assert!((region[0] + region[1] + region[2] - 1.).abs() <= 0.005);
            for channel in 0..3 {
                parity[group] +=
                    usize::from((at(4, sample)[channel] - at(2, sample)[channel]).abs() > 0.01);
                if sample >= 15 {
                    controls[group] += at(2, sample)[channel].abs();
                }
            }
        }
        for scale in 0..3 {
            assert!(
                at(3, scale * 5 + 1)[0] * at(3, scale * 5 + 3)[0] < 0.,
                "observed straddle row={row} scale={scale}"
            );
        }
        for path in [0, 1, 2, 4] {
            let limit = if path == 0 { 0.005 } else { 0.01 };
            let mut previous = f32::INFINITY;
            for scale in 0..3 {
                let mut across = 0.0f32;
                let mut same = 0.0f32;
                for channel in 0..3 {
                    for (a, b) in [(1, 3), (1, 2), (2, 3)] {
                        across = across.max(
                            (at(path, scale * 5 + a)[channel] - at(path, scale * 5 + b)[channel])
                                .abs(),
                        );
                    }
                    for (a, b) in [(0, 1), (3, 4)] {
                        same = same.max(
                            (at(path, scale * 5 + a)[channel] - at(path, scale * 5 + b)[channel])
                                .abs(),
                        );
                    }
                }
                if (scale > 0 && across > previous * 0.35 + 0.001)
                    || (scale == 2 && across.max(same) > limit)
                {
                    continuity[group] += 1;
                    bad.push((row, path, scale, across, same));
                }
                previous = across;
            }
        }
    }
    assert!(controls.iter().all(|v| *v > 1.));
    println!(
        "CELL_FRACTIONAL legacy={legacy} variants=[VORONOI,SHATTER,BRICK] density_bytes=256 seed=0 fill=128 gap=64 samples={} continuity={continuity:?} parity={parity:?} first={:?}",
        pixels.len(),
        &bad[..bad.len().min(12)]
    );
    bad.len() + parity.iter().sum::<usize>()
}

#[test]
fn cell_fractional_remaining_chart_wraps() {
    assert_eq!(failures(false, false), 0);
}

#[test]
fn cell_fractional_gate_rejects_injected_jump() {
    assert!(failures(true, false) > 0);
}

#[test]
fn cell_fractional_archived_before_still_exposes_wrap_failures() {
    assert!(failures(false, true) > 0);
}

fn rectangle_paint_failures(variant: u32, missing_neighbour: bool) -> usize {
    let mut body = r#"
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let variant=@VARIANT@u;let byte=p.y%256u;let seed=array<u32,4>(0u,1u,127u,255u)[p.y/256u];
 let control=p.x/16u;let i=p.x%16u;
 let fill=array<u32,3>(0u,128u,255u)[control/5u];let gap=array<u32,5>(0u,1u,6u,64u,255u)[control%5u];
 let density=mix(4.,64.,u8_to_01(byte));let count=select(density,round(density),variant==5u);let height=select(count,count*.5,variant==5u);
 var requested_uv=vec2f(.73,.57);
 if i<5u {requested_uv=vec2f(fract(f32(i32(i)-2)*.000001),2.37/count);}
 else if i<10u {requested_uv=vec2f(.37/count,(2.+f32(i32(i)-7)*.000001)/count);}
 else if i<15u {requested_uv=vec2f(fract(f32(i32(i)-12)*.000001),2./count);}
 let instr=vec4u((seed<<24u)|(0x80ffu<<8u)|255u,(191u<<24u)|(byte<<16u)|(fill<<8u)|gap,
 0x4080c060u,(OP_CELL<<27u)|(REGION_ALL<<24u)|(variant<<16u)|0x2040u);
 let axis=decode_dir16(instr_dir16(instr));let t=normalize(cross(vec3f(1.,0.,0.),axis));let b=normalize(cross(axis,t));
 let h=requested_uv.y*2.-1.;let phi=(requested_uv.x-.5)*TAU;
 let dir=normalize(axis*h+sqrt(1.-h*h)*(t*cos(phi)+b*sin(phi)));
 let uv=cell_axis_cylinder_uv(dir,axis);let q=uv*vec2f(count,height);
 // Independent exhaustive union of literal rectangle half-planes, not cell_brick.
 var solid=-100.;let base=vec2i(floor(q));let n=i32(ceil(count));
 for(var y=base.y-3;y<=base.y+3;y++) {for(var x=base.x-3;x<=base.x+3;x++) {
  let shift=select(0.,.5,variant==5u&&(y&1)==0);let bottom=f32(y);
  let owner=(x+4*n)%n;let cycle=select(0.,1.,x>=n)-select(0.,1.,x<0);
  let left=cycle*count+f32(owner)-shift;let right=cycle*count+min(f32(owner)+1.,count)-shift;
  let id=vec2f(f32(owner),f32(y));
  let d=min(min(q.x-left,right-q.x),min(q.y-bottom,bottom+1.-q.y));
  if cell_hash2(id,f32(seed))<u8_to_01(fill) {solid=max(solid,d);}
 }}
 let width=u8_to_01(gap)*.2;var expected=RegionWeights(1.,0.,0.);
 if solid > -99. {expected=regions_from_signed_distance(solid-width,max(.005,width*.5));}
 var actual=evaluate_bounds_layer(dir,instr,OP_CELL,axis,RegionWeights(.2,.3,.5));
 // INJECT_MISSING_NEIGHBOUR
 let want=vec3f(expected.sky,expected.wall,expected.floor);
 let got=vec3f(actual.regions.sky,actual.regions.wall,actual.regions.floor);
 textureStore(result,p.xy,vec4f(abs(got-want),select(0.,1.,any(want>vec3f(0.))&&any(want<vec3f(1.))&&max(max(want.x,want.y),want.z)<1.)));
}
"#.replace("@VARIANT@", &variant.to_string());
    if missing_neighbour {
        body=body.replace("// INJECT_MISSING_NEIGHBOUR", "var owner=cell_grid(uv,density);if variant==5u {owner=cell_brick(uv,density,f32(seed));} if cell_hash2(owner.xy,f32(seed))>=u8_to_01(fill) {actual.regions=RegionWeights(1.,0.,0.);}");
    }
    let data = probe_image(&body, 240, 1024);
    assert_eq!(data.len(), 240 * 1024);
    assert!(data.iter().flatten().all(|v| v.is_finite()));
    let failures = data
        .iter()
        .filter(|p| p[..3].iter().any(|v| *v > 0.002))
        .count();
    let partial = data.iter().filter(|p| p[3] > 0.).count();
    assert!(partial > 0);
    println!(
        "CELL_RECTANGLE_UNION variant={variant} samples={} failures={failures} partial={partial} tolerance=.002 missing_neighbour={missing_neighbour}",
        data.len()
    );
    failures
}

#[test]
fn cell_brick_paint_matches_rectangle_union() {
    assert_eq!(rectangle_paint_failures(5, false), 0);
}
#[test]
fn cell_brick_paint_rejects_missing_neighbour() {
    assert!(rectangle_paint_failures(5, true) > 0);
}

#[test]
fn cell_grid_paint_matches_rectangle_union() {
    assert_eq!(rectangle_paint_failures(0, false), 0);
}
#[test]
fn cell_grid_paint_rejects_missing_neighbour() {
    assert!(rectangle_paint_failures(0, true) > 0);
}

#[test]
fn cell_integer_periodic_identity_matches_euclidean_remainder() {
    let data = probe_image(
        r#"
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let x=i32(p.x)-128;let n=i32(p.y)+4;
    textureStore(result,p.xy,vec4f(cell_offset_id_x(f32(x),f32(n)),f32(x),f32(n),1.));
}
"#,
        257,
        61,
    );
    assert_eq!(data.len(), 257 * 61);
    let mut failures = Vec::new();
    for (k, got) in data.iter().enumerate() {
        let x = (k % 257) as i32 - 128;
        let n = (k / 257) as i32 + 4;
        let expected = x.rem_euclid(n) as f32;
        assert!(got.iter().all(|v| v.is_finite()));
        assert_eq!(got[1], x as f32);
        assert_eq!(got[2], n as f32);
        if got[0] != expected {
            failures.push((x, n, got[0], expected));
        }
    }
    println!(
        "CELL_INTEGER_ID samples={} failures={} first={:?}",
        data.len(),
        failures.len(),
        &failures[..failures.len().min(8)]
    );
    assert!(
        failures.is_empty(),
        "canonical periodic IDs must use mathematical Euclidean remainder"
    );
}

#[test]
fn cell_integer_wrapped_brick_retains_filled_neighbour_at_frozen_rays() {
    let data = probe_image(
        r#"
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let d0=bitcast<vec3f>(vec3u(904682832u,1051471574u,3211859703u));let d1=bitcast<vec3f>(vec3u(3045970326u,1051471574u,3211859703u));
    let i=vec4u((127u<<24u)|(0xff80u<<8u)|255u,(96u<<24u)|(127u<<16u)|(128u<<8u),
        (0x58u<<24u)|0xc09050u,(OP_CELL<<27u)|(REGION_ALL<<24u)|(3u<<21u)|(5u<<16u)|0x1828u);
    let q0=eval_cell(d0,i,RegionWeights(.2,.3,.5));let q1=eval_cell(d1,i,RegionWeights(.2,.3,.5));
    let axis=decode_dir16(instr_dir16(i));let density=mix(4.,64.,u8_to_01(instr_a(i)));
    let f0=cell_brick_fields(cell_axis_cylinder_uv(d0,axis),density,127.,128./255.);
    let f1=cell_brick_fields(cell_axis_cylinder_uv(d1,axis),density,127.,128./255.);
    let rgb=abs(q0.sample.rgb-q1.sample.rgb);
    let regions=abs(vec3f(q0.regions.sky,q0.regions.wall,q0.regions.floor)-vec3f(q1.regions.sky,q1.regions.wall,q1.regions.floor));
    textureStore(result,p.xy,vec4f(max(rgb.x,max(rgb.y,rgb.z)),max(regions.x,max(regions.y,regions.z)),f0.w,f1.w));
}
"#,
        1,
        1,
    );
    let q = data[0];
    assert!(q.iter().all(|x| x.is_finite()));
    assert!(
        q[0] <= 0.01 && q[1] <= 0.01 && q[2] >= 0. && q[2] < 0.001 && q[3] <= 0. && q[3] > -0.001,
        "frozen wrap lost its filled neighbour: {q:?}"
    );
}
