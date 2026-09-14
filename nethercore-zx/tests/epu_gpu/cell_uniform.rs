//! CELL paint opacity is uniform across openings and applied once, without retagging regions.
use super::*;

#[test]
fn cell_gap_alpha_is_uniform_linear_and_geometry_independent() {
    const TOL: f32 = 0.002;
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let point=p.x%8u;
    let alpha=(p.x/8u)%16u;
    let fill=array<u32,3>(0u,128u,255u)[(p.x/128u)%3u];
    let variant=p.x/384u;
    let density=array<u32,8>(0u,1u,2u,63u,127u,129u,254u,255u)[point];
    let dir=normalize(vec3f(-.83+f32(point)*.21,.39-f32(point)*.13,.61));
    // Nonblack colors, no outline, seed 77, gap 96, oblique axis, ADD.
    let instr=vec4u((77u<<24u)|(0x9080u<<8u)|(alpha<<4u)|15u,
        (density<<16u)|(fill<<8u)|96u,
        0xc0806040u,(OP_CELL<<27u)|(REGION_ALL<<24u)|(variant<<16u)|0x4080u);
    var full=instr; full.x=(full.x&0xffffff0fu)|240u;
    let value=eval_cell(dir,instr,RegionWeights(.2,.3,.5));
    let opaque=eval_cell(dir,full,RegionWeights(.2,.3,.5));
    var layers:array<vec4u,8>;layers[0]=instr;
    var out=vec4f(value.sample.rgb,value.sample.w);
    if p.y==1u {out=vec4f(opaque.sample.rgb,opaque.sample.w);}
    if p.y==2u {out=vec4f(value.regions.sky,value.regions.wall,value.regions.floor,value.region_mix);}
    if p.y==3u {out=vec4f(opaque.regions.sky,opaque.regions.wall,opaque.regions.floor,opaque.region_mix);}
    if p.y==4u {out=vec4f(evaluate_epu_layers(dir,layers),instr_alpha_a_f32(instr));}
    if p.y==5u {let dispatched=evaluate_bounds_layer(dir,instr,OP_CELL,decode_dir16(instr_dir16(instr)),RegionWeights(.2,.3,.5));out=vec4f(dispatched.sample.rgb,dispatched.sample.w);}
    if p.y==6u {out=vec4f(instr_color_a(instr),f32(instr_variant_id(instr)));}
    if p.y==7u {out=vec4f(instr_color_b(instr),f32(instr_blend(instr)));}
    textureStore(result,p.xy,out);
}
"#,
        2304,
        8,
    );
    let mut failures = Vec::new();
    let color = [64. / 255., 128. / 255., 192. / 255.];
    assert_eq!(pixels.len(), 2304 * 8);
    assert!(pixels.iter().flatten().all(|v| v.is_finite()));
    let mut partial_regions = 0;
    for case in 0..2304 {
        let alpha = ((case / 8) % 16) as f32 / 15.;
        let decoded = pixels[2304 * 6 + case];
        assert_eq!(decoded[3], (case / 384) as f32);
        assert_eq!(pixels[2304 * 7 + case][3], 0.);
        for c in 0..3 {
            assert!((decoded[c] - color[c]).abs() <= TOL, "packed gap color");
        }
        let raw = pixels[case];
        let full = pixels[2304 + case];
        let regions = pixels[2304 * 2 + case];
        let composed = pixels[2304 * 4 + case];
        assert_eq!(
            regions,
            pixels[2304 * 3 + case],
            "alpha must not change region ownership, case {case}"
        );
        assert_eq!(
            raw,
            pixels[2304 * 5 + case],
            "direct/dispatch mismatch, case {case}"
        );
        assert!(
            (composed[3] - alpha).abs() <= TOL,
            "GPU-decoded alpha fixture"
        );
        if regions[0] > 0. && regions[0] < 1. {
            partial_regions += 1;
        }
        let weight = regions[0] * alpha + regions[1] + regions[2];
        for c in 0..3 {
            // Full-alpha paint minus the removed gap contribution: same geometry/material.
            let expected = full[c] * full[3] - color[c] * regions[0] * (1. - alpha);
            if (raw[3] - weight).abs() > TOL
                || (raw[c] * raw[3] - expected).abs() > TOL
                || (composed[c] - expected.clamp(0., 1.)).abs() > TOL
            {
                failures.push((case, c, raw, composed[c], expected, weight));
            }
        }
    }
    assert!(
        partial_regions > 0,
        "must exercise mixed gap/wall/floor paint, not just empty cells"
    );
    println!(
        "CELL_UNIFORM cases=2304 partial_regions={partial_regions} failed_channels={} tolerance={TOL}",
        failures.len()
    );
    assert!(
        failures.is_empty(),
        "CELL alpha ignored or applied twice; first failures: {:?}",
        &failures[..failures.len().min(12)]
    );
}

const HEX_PERIODIC_BODY: &str = r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let path=p.x%4u;let step=(p.x/4u)%7u;let family=(p.x/28u)%4u;
    let axis_id=(p.x/112u)%3u;let config=p.x/336u;
    let e=array<f32,7>(.001,-.001,.0001,-.0001,.00001,-.00001,0.)[step];
    let density=mix(4.,64.,u8_to_01(p.y));let count=round(density);let hrow=.8660254037844386;
    var xy=vec2f(e,2.5*hrow);
    if family==1u {xy=vec2f(1.5,3.+e);}
    if family==2u {xy=vec2f(1.5,3.*hrow+e);}
    if family==3u {xy=vec2f(.5+e,2.5*hrow);}
    let fill=array<u32,6>(255u,128u,128u,128u,128u,0u)[config];
    let gap=array<u32,6>(0u,0u,1u,127u,255u,32u)[config];
    let alpha=array<u32,6>(15u,15u,7u,0u,7u,7u)[config];
    let seed=(p.y*73u+31u)%256u;
    let encoded_axis=array<u32,3>(0xff80u,0x80ffu,0xe0a0u)[axis_id];
    let instr=vec4u((seed<<24u)|(encoded_axis<<8u)|(alpha<<4u)|15u,
        (p.y<<16u)|(fill<<8u)|gap,0xc0806040u,(OP_CELL<<27u)|(REGION_ALL<<24u)|(1u<<16u)|0x4080u);
    let axis=decode_dir16(encoded_axis);
    let reference=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(axis.y)>.9);
    let t=normalize(cross(reference,axis));let b=normalize(cross(axis,t));
    let height=xy.y/count*2.-1.;let angle=xy.x/count*TAU;
    let dir=normalize(axis*height+sqrt(1.-height*height)*(-t*cos(angle)-b*sin(angle)));
    let value=eval_cell(dir,instr,RegionWeights(.2,.3,.5));
    var layers:array<vec4u,8>;layers[0]=instr;
    var out=vec4f(value.regions.sky,value.regions.wall,value.regions.floor,1.);
    if path==1u {out=vec4f(evaluate_epu_layers(dir,layers),1.);}
    if path==2u {out=vec4f(value.sample.rgb*value.sample.w,1.);}
    if path==3u {
        let uv=cell_axis_cylinder_uv(dir,axis);let info=cell_hex(uv,density);
        var observed=shortest_periodic_delta(uv.x,0.,1.)*count;
        if family==1u {observed=uv.y*count-3.;}
        if family==2u {observed=uv.y*count-3.*hrow;}
        if family==3u {observed=uv.x*count-.5;}
        out=vec4f(info,observed*1000000.);
    }
    // MUTATION_POINT
    textureStore(result,p.xy,out);
}
"#;

fn hex_periodic_failures(body: &str) -> usize {
    let pixels = probe_image(body, 2016, 256);
    assert_eq!(pixels.len(), 2016 * 256);
    assert!(pixels.iter().flatten().all(|v| v.is_finite()));
    let mut failures = Vec::new();
    let mut observed_straddles = 0;
    for byte in 0..256 {
        for case in 0..72 {
            let at =
                |sample: usize, path: usize| pixels[byte * 2016 + case * 28 + sample * 4 + path];
            let pos = at(4, 3)[3];
            let neg = at(5, 3)[3];
            if pos >= 0. && neg <= 0. && pos > neg {
                observed_straddles += 1;
            }
            for sample in 0..7 {
                for c in 0..3 {
                    assert!(
                        (at(sample, 1)[c] - at(sample, 2)[c]).abs() <= 0.002,
                        "premultiplied direct/full-dispatch mismatch {byte}/{case}/{sample}/{c}"
                    );
                }
            }
            for path in 0..3 {
                let mut jump = 0f32;
                for (a, b) in [(4, 5), (4, 6), (5, 6)] {
                    for c in 0..3 {
                        jump = jump.max((at(a, path)[c] - at(b, path)[c]).abs());
                    }
                }
                if jump > 0.01 {
                    failures.push((byte, case, path, jump));
                }
            }
        }
    }
    println!(
        "HEX_PERIODIC cases={} observed_straddles={observed_straddles} failures={} first={:?} budget=.01",
        256 * 72,
        failures.len(),
        &failures[..failures.len().min(10)]
    );
    assert_eq!(
        observed_straddles,
        256 * 72,
        "fixture must straddle every requested boundary"
    );
    failures.len()
}

#[test]
fn cell_hex_periodic_wrap_and_rows_all_density_bytes() {
    assert_eq!(hex_periodic_failures(HEX_PERIODIC_BODY), 0);
}

#[test]
fn cell_hex_periodic_gate_rejects_injected_seam() {
    let bad = HEX_PERIODIC_BODY.replace(
        "// MUTATION_POINT",
        "if path < 3u && step == 4u { out.x += .125; }",
    );
    assert_ne!(bad, HEX_PERIODIC_BODY);
    assert!(
        hex_periodic_failures(&bad) > 0,
        "constant seam must be detected"
    );
}
