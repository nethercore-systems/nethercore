//! Production HEX wrap closure and frozen falsification of naive clipping.
use super::*;
fn focused(legacy: bool) -> Vec<[f32; 4]> {
    let body = r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let byte=array<u32,3>(1u,2u,0u)[p.y];
 let density=mix(4.,64.,u8_to_01(byte));
 let v=(2.+select(.5,.99,p.y==1u))/density;
 let point=p.x/4u; let field=p.x%4u;
 let delta=array<f32,7>(.0001,-.0001,.00001,-.00001,0.,.00002,-.00002)[point];
 let instr=vec4u((1u<<24u)|(0x80ffu<<8u)|255u,
 (191u<<24u)|(byte<<16u)|(255u<<8u),
 0x4080c060u,(OP_CELL<<27u)|(REGION_ALL<<24u)|(1u<<16u)|0x2040u);
 let axis=decode_dir16(instr_dir16(instr));
 let reference_axis=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(axis.y)>.9);
 let t=normalize(cross(reference_axis,axis));let b=normalize(cross(axis,t));
 let h=v*2.-1.;let dir=normalize(axis*h+sqrt(1.-h*h)*(-t*cos(delta)-b*sin(delta)));
 let uv=cell_axis_cylinder_uv(dir,axis);let info=cell_hex(uv,density);
 let value=evaluate_bounds_layer(dir,instr,OP_CELL,axis,RegionWeights(.2,.3,.5));
 let clipped=min(info.z,min(uv.x*density,(1.-uv.x)*density));
 let candidate=regions_from_signed_distance(clipped,.005);
 var out=vec4f(info,cell_hash2(info.xy,1.));
 if field==1u {out=vec4f(value.regions.sky,value.regions.wall,value.regions.floor,value.region_mix);}
 if field==2u {out=vec4f(candidate.sky,candidate.wall,candidate.floor,clipped);}
 if field==3u {out=vec4f(shortest_periodic_delta(uv.x,0.,1.)*1000000.,value.sample.rgb);}
 textureStore(result,p.xy,out);
}
"#;
    let body = if legacy {
        body.replace("cell_hex(", "frozen_cell_hex(")
    } else {
        body.to_owned()
    };
    probe_image(
        &format!(
            "{}\n{}",
            include_str!("fixtures/cell-hex-2d/legacy-hex.wgsl"),
            body
        ),
        28,
        3,
    )
}
#[test]
fn cell_offset_fractional_hex_production_closes() {
    let p = focused(false);
    for (row, r) in p.chunks_exact(28).enumerate() {
        println!("OFFSET_ROW {row} {r:?}");
    }
    assert!(p.iter().flatten().all(|x| x.is_finite()));
    let r = &p[..28];
    assert_eq!(
        &r[8][..2],
        &r[12][..2],
        "periodic HEX owns the same cell across this cut"
    );
    assert!(r[11][0] > 0. && r[15][0] < 0.);
    assert!(r[8][2] > 0.02 && r[12][2] > 0.02);
    let integral = &p[56..];
    assert_eq!(integral[8][0], integral[12][0]);
    assert!((integral[8][2] - integral[12][2]).abs() <= 0.005);
    let delta = (r[8][2] - r[12][2]).abs();
    println!(
        "HEX production byte1 seed1 center-row edge_jump={delta}, tolerance=.005; canonical integral control PASS"
    );
    assert!(
        delta <= 0.005,
        "fractional chart geometry does not close; identity-only wrapping cannot change this RED"
    );
}
#[test]
fn cell_offset_fractional_hex_clip_falsifier() {
    let p = focused(true);
    let r = &p[28..56];
    println!("HEX_CLIP_CORNER {r:?}");
    assert!(p.iter().flatten().all(|x| x.is_finite()));
    assert!(
        r[8][2] > 0. && r[12][2] < -0.01,
        "actual HEX polygon occupies only one cut side"
    );
    let jump = (r[10][0] - r[14][0]).abs().max((r[10][1] - r[14][1]).abs());
    assert!(
        jump > 0.01,
        "naive two-sided GRID clip must be rejected at zero gap"
    );
    assert!(r[11][0] > 0. && r[15][0] < 0.);
    println!(
        "NAIVE_CLIP_REJECTED byte2 seed1 row_phase=.99 gap0 region_jump={jump} budget=.01; not production GREEN"
    );
}
