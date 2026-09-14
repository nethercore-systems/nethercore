use super::*;

#[test]
fn patches_poles_and_nonfading_domain_preservation() {
    let p = probe_image(include_str!("fixtures/patches-poles/pole.wgsl"), 204, 240);
    assert!(p.iter().flatten().all(|v| v.is_finite()));
    let failures = p
        .as_chunks::<204>()
        .0
        .iter()
        .filter(|r| {
            (0..4).any(|c| {
                let s = &r[51..68];
                let lo = s.iter().map(|p| p[c]).fold(f32::INFINITY, f32::min);
                let hi = s.iter().map(|p| p[c]).fold(f32::NEG_INFINITY, f32::max);
                hi - lo > 0.005
            })
        })
        .count();
    let negative = p
        .as_chunks::<204>()
        .0
        .iter()
        .filter(|r| {
            (0..4).any(|c| {
                let s = &r[187..204];
                let lo = s.iter().map(|p| p[c]).fold(f32::INFINITY, f32::min);
                let hi = s.iter().map(|p| p[c]).fold(f32::NEG_INFINITY, f32::max);
                hi - lo > 0.005
            })
        })
        .count();
    for row in p.as_chunks::<204>().0.iter() {
        for level in 0..4 {
            for az in 0..16 {
                let v = row[68 + level * 17 + az];
                assert!(
                    v[0] > 0.96 && v[0] < 1.04 && v[1] == 1.,
                    "collapsed direction"
                );
            }
        }
        for value in &row[..68] {
            assert!(value[3] > 0.23, "must not fade PATCHES coverage");
        }
    }
    println!(
        "PATCHES_POLES rows=240 failures={failures} injected_jump_rejected={negative} limit=0.005 nonfading=true"
    );
    assert_eq!(negative, 240);
    let legacy = include_str!("fixtures/patches-poles/before.wgsl")
        .replace("patches_", "legacy_patches_")
        .replace("eval_patches(", "legacy_eval_patches(");
    let body = r#"
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let sample=p.x/2u;let which=p.x%2u;let az=sample/11u;
 let r=array<f32,11>(0.,0.00001,0.0001,0.01,0.04999,0.05,0.05001,0.1,0.3,0.8,1.)[sample%11u];
 let variant=p.y%8u;let domain=(p.y/8u)%4u;let axis_id=(p.y/32u)%3u;
 let bits=array<u32,3>(0x8080u,0xffffu,0x427du)[axis_id];let axis=decode_dir16(bits);
 let hint=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(axis.y)>0.9);
 let t=normalize(cross(hint,axis));let b=normalize(cross(axis,t));
 let centre=axis*select(1.,-1.,p.y>=192u);let angle=f32(az)*TAU/16.;
 let ray=normalize(centre*sqrt(1.-r*r)+r*(t*cos(angle)+b*sin(angle)));
 let seed=select(37u,255u,(p.y/96u)%2u==1u);
 let i=vec4u((seed<<24u)|(bits<<8u)|0xf7u,0xc0408080u,0x222244ccu,
  (6u<<27u)|(7u<<24u)|(((domain<<3u)|variant)<<16u)|0xeebbu);
 var v=eval_patches(ray,i,RegionWeights(1.,0.,0.));
 if which==1u {v=legacy_eval_patches(ray,i,RegionWeights(1.,0.,0.));}
 textureStore(result,p.xy,vec4f(v.regions.sky,v.regions.wall,v.regions.floor,v.sample.w));
}"#;
    let q = probe_image(&format!("{legacy}\n{body}"), 352, 384);
    assert!(q.iter().flatten().all(|v| v.is_finite()));
    let mut preserved = 0;
    let mut changed = 0;
    for (i, pair) in q.as_chunks::<2>().0.iter().enumerate() {
        let domain = (i / 176 / 8) % 4;
        let radius = i % 11;
        assert!(pair[0][3] > 0.23, "no cap opacity fade");
        assert!(
            (pair[0][..3].iter().sum::<f32>() - 1.).abs() < 0.002,
            "region partition"
        );
        if domain == 0 || domain == 3 || radius >= 6 {
            assert_eq!(pair[0], pair[1], "outside-cap change {i}");
            preserved += 1;
        } else if pair[0] != pair[1] {
            changed += 1;
        }
    }
    println!(
        "PATCHES_CAP_PRESERVATION records={} exact_outside={preserved} changed_inside={changed} no_added_coverage_gate=true",
        q.len()
    );
    assert_eq!(failures, 0);
    assert!(preserved > 0 && changed > 0);
}
