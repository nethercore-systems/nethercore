use super::*;

#[test]
fn veil_polar_poles_envelope_and_other_domains() {
    let pole = probe_image(include_str!("fixtures/veil-polar/pole.wgsl"), 68, 20);
    assert!(pole.iter().flatten().all(|x| x.is_finite()));
    let failures = pole
        .as_chunks::<68>()
        .0
        .iter()
        .filter(|row| {
            (0..4).any(|c| {
                let samples = &row[51..68];
                let lo = samples.iter().map(|p| p[c]).fold(f32::INFINITY, f32::min);
                let hi = samples
                    .iter()
                    .map(|p| p[c])
                    .fold(f32::NEG_INFINITY, f32::max);
                hi - lo > 0.005
            })
        })
        .count();
    println!(
        "VEIL_POLAR_LIMIT records={} failures={failures} limit=.005",
        pole.len()
    );

    // Keep the old near-pole-only envelope as a diagnostic, not a production fallback.
    let source = include_str!("../../shaders/epu/features/05_veil.wgsl");
    let legacy = source[source.find("fn eval_veil(").unwrap()..]
        .replace("fn eval_veil(", "fn old_veil(")
        .replace(
            "smoothstep(0.05, 0.2, min(uv.y, 1.0 - uv.y))",
            "smoothstep(0.05, 0.2, uv.y)",
        );
    assert_eq!(
        legacy
            .matches("domain_w = smoothstep(0.05, 0.2, uv.y);")
            .count(),
        1
    );
    let body = r#"
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let sample=p.x/3u; let which=p.x%3u; let radius_id=sample/8u;
    let radius=array<f32,24>(0.,.01,.04999,.05,.05001,.1,.19999,.2,.20001,.3,.4,.5,.6,.7,.79999,.8,.80001,.9,.94999,.95,.95001,.99,.999,1.)[radius_id];
    let v=p.y%8u; let domain=(p.y/8u)%4u; let axis_id=(p.y/32u)%3u;
    let bits=array<u32,3>(0u,0x8080u,0xffffu)[axis_id];
    let axis=select(vec3f(0.,1.,0.),decode_dir16(bits),bits!=0u);
    let hint=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(axis.y)>.9);
    let t=normalize(cross(hint,axis));let b=normalize(cross(axis,t));
    let angle=f32(sample%8u)*TAU/8.;
    let ray=normalize(axis*cos(radius*PI)+(t*cos(angle)+b*sin(angle))*sin(radius*PI));
    let phase=select(37u,128u,p.y>=96u);
    let instr=vec4u((phase<<24u)|(bits<<8u)|0xf7u,(192u<<24u)|(64u<<16u)|(20u<<8u)|128u,
        0x22u<<24u|0x2244ccu,(13u<<27u)|(7u<<24u)|(((domain<<3u)|v)<<16u)|0xeebb);
    var value=old_veil(ray,instr,1.);
    if which==0u { value=eval_veil(ray,instr,1.); }
    if which==2u && domain==2u {
        let angular_radius=acos(clamp(dot(ray,axis),-1.,1.))/PI;
        let old_support=smoothstep(.05,.2,angular_radius);
        let new_support=smoothstep(.05,.2,min(angular_radius,1.-angular_radius));
        if old_support>0. {value.w*=new_support/old_support;} else {value.w=0.;}
    }
    textureStore(result,p.xy,vec4f(value.rgb,value.w));
}"#;
    let values = probe_image(&format!("{legacy}\n{body}"), 576, 192);
    assert!(values.iter().flatten().all(|x| x.is_finite()));
    let mut wrong = 0;
    let mut preserved = 0;
    let mut live = 0;
    for (i, triplet) in values.as_chunks::<3>().0.iter().enumerate() {
        let domain = (i / 192 / 8) % 4;
        let radius_id = (i % 192) / 8;
        assert_eq!(&triplet[0][..3], &triplet[1][..3], "RGB changed at {i}");
        if (triplet[0][3] - triplet[2][3]).abs() > 0.002 {
            wrong += 1;
        }
        if domain != 2 || radius_id <= 15 {
            assert_eq!(triplet[0], triplet[1], "outside-cap change at {i}");
            preserved += 1;
        }
        if domain == 2 && (7..=15).contains(&radius_id) && triplet[0][3] > 0.01 {
            live += 1;
        }
    }
    println!(
        "VEIL_POLAR_ENVELOPE records={} failures={wrong} off_cap_exact={preserved} nonzero_mid_chart={live} limit=.002",
        values.len()
    );
    assert!(live > 100, "retain visible polar interiors");
    assert_eq!((failures, wrong), (0, 0));
}
