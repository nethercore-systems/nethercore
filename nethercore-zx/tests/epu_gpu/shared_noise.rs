//! Shared carrier lattice continuity; independent of FLOW/PATCHES acceptance.
use super::*;

#[test]
fn transport_chart_pole_limits() {
    const LIMIT: f32 = 0.005;
    let image = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) id:vec3u) {
    let family=id.y/4u;
    let level=id.y%4u;
    let profile=family%2u;
    let pole=(family/2u)%2u;
    let axis_code=array<u32,3>(0u,0x80ffu,0xe070u)[(family/4u)%3u];
    let domain=(family/12u)%2u+1u;
    let pair=array<vec2u,10>(vec2u(21u,0u),vec2u(21u,1u),vec2u(21u,2u),
        vec2u(21u,3u),vec2u(21u,4u),vec2u(21u,5u),vec2u(23u,0u),
        vec2u(23u,1u),vec2u(23u,2u),vec2u(23u,3u))[family/24u];
    let a=select(0u,128u,profile==1u);
    let c=select(128u,255u,profile==1u);
    let phase=select(64u,128u,profile==1u);
    let ins=vec4u((phase<<24u)|(axis_code<<8u)|0xf0u,
        (64u<<24u)|(a<<16u)|(255u<<8u)|c,0x66224466u,
        (pair.x<<27u)|(7u<<24u)|(3u<<21u)|(domain<<19u)|(pair.y<<16u)|0xff99u);
    let axis=normalize(select(vec3f(-1.0,0.08,0.0),decode_dir16(instr_dir16(ins)),axis_code!=0u));
    let basis=advect_build_basis(axis);
    let centre=axis*select(1.0,-1.0,pole==1u);
    let eps=array<f32,4>(0.01,0.001,0.0001,0.00001)[level];
    if id.x==9u {
        let t=basis[0];let b=basis[2];
        textureStore(result,id.xy,1000.0*vec4f(dot(normalize(centre+eps*t)-centre,t),
            dot(normalize(centre-eps*t)-centre,t),dot(normalize(centre+eps*b)-centre,b),
            dot(normalize(centre-eps*b)-centre,b)));return;
    }
    if id.x==10u {textureStore(result,id.xy,vec4f(f32(instr_opcode(ins)),f32(instr_variant_id(ins)),f32(instr_domain_id(ins)),eps*1000.0));return;}
    let offsets=array<vec2f,9>(vec2f(0,0),vec2f(1,0),vec2f(-1,0),vec2f(0,1),
        vec2f(0,-1),vec2f(2,0),vec2f(-2,0),vec2f(0,2),vec2f(0,-2));
    let offset=offsets[select(id.x,0u,id.x==11u)];
    var dir=centre;
    if any(offset!=vec2f(0)) {dir=normalize(centre+eps*(offset.x*basis[0]+offset.y*basis[2]));}
    var sample=LayerSample(vec3f(0),0.0);
    if pair.x==21u {sample=eval_advect(dir,ins,1.0);} else {sample=eval_mass(dir,ins,1.0);}
    var color=vec4f(apply_blend(vec3f(0.17,0.21,0.29),sample,3u),epu_saturate(sample.w));
    if id.x==11u {color.x+=0.04;}
    textureStore(result,id.xy,color);
}
"#,
        12,
        10 * 2 * 3 * 2 * 2 * 4,
    );
    assert!(image.iter().flatten().all(|v| v.is_finite()));
    let mut failures = Vec::new();
    let mut peak = [0.0f32; 4];
    let mut max_error = 0.0f32;
    let mut max_same = 0.0f32;
    let mut rejected = 0;
    for family in 0..240 {
        let pair = family / 24;
        let op = if pair < 6 { 21 } else { 23 };
        let variant = if pair < 6 { pair } else { pair - 6 };
        let domain = (family / 12) % 2 + 1;
        let mut previous = [f32::INFINITY; 4];
        let mut levels = [0.0f32; 4];
        for level in 0..4 {
            let row = &image[(family * 4 + level) * 12..(family * 4 + level + 1) * 12];
            assert_eq!(&row[10][..3], &[op as f32, variant as f32, domain as f32]);
            let loc = row[9];
            assert!(
                loc[0] > 0.0 && loc[1] < 0.0 && loc[2] > 0.0 && loc[3] < 0.0,
                "unbracketed pole approach"
            );
            for c in 0..4 {
                assert!(loc[c].abs() < previous[c]);
                previous[c] = loc[c].abs();
            }
            let mut jump = 0.0f32;
            let mut same = 0.0f32;
            for c in 0..4 {
                for a in &row[..5] {
                    for b in &row[..5] {
                        jump = jump.max((a[c] - b[c]).abs());
                    }
                }
                for k in 1..5 {
                    same = same.max((row[k][c] - row[k + 4][c]).abs());
                }
            }
            for p in &row[..5] {
                peak[usize::from(op == 23) * 2 + domain - 1] =
                    peak[usize::from(op == 23) * 2 + domain - 1].max(p[3]);
            }
            assert!(
                (row[11][0] - row[0][0]).abs() > 0.035,
                "injected on-cut error escaped"
            );
            rejected += 1;
            levels[level] = jump;
            if level == 3 {
                max_error = max_error.max(jump);
                max_same = max_same.max(same);
                if jump > LIMIT {
                    failures.push((family, op, variant, domain, levels, same));
                }
            }
        }
    }
    println!(
        "TRANSPORT_POLES samples={} families=240 failures={} max_error={max_error} max_same_side={max_same} peaks={peak:?} rejected={rejected} limit={LIMIT}",
        image.len(),
        failures.len()
    );
    println!(
        "TRANSPORT_FIRST_WITNESSES {:?}",
        &failures[..failures.len().min(6)]
    );
    assert!(
        peak.iter().all(|p| *p > 0.03),
        "blank opcode/domain controls"
    );
    assert!(
        failures.is_empty(),
        "transport chart pole limits do not converge"
    );
}

#[test]
fn transport_chart_cap_join_and_preservation() {
    let image = probe_image(
        r#"
fn transport_original_coords(dir: vec3f, axis: vec3f, domain: u32) -> vec3f {
    let basis = advect_build_basis(axis);
    let x = dot(dir, basis[0]); let y = dot(dir, basis[1]); let z = dot(dir, basis[2]);
    let angle = atan2(x, z);
    if domain == 1u { return vec3f(cos(angle), sin(angle), y); }
    let radius = acos(clamp(y, -1.0, 1.0)) / PI;
    return vec3f(cos(angle) * radius, sin(angle) * radius, radius * 2.0 - 1.0);
}
fn transport_cap_ray(centre: vec3f, tangent: vec3f, r: f32) -> vec3f {
    return normalize(centre*sqrt(1.0-r*r)+tangent*r);
}
fn transport_transverse(dir: vec3f, basis: mat3x3f) -> f32 {
    return length(vec2f(dot(dir,basis[0]),dot(dir,basis[2])));
}

@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) id:vec3u) {
    let family=(id.y/4u)/4u;
    let meridian=(id.y/4u)%4u;
    let level=id.y%4u;
    let profile=family%2u;
    let pole=(family/2u)%2u;
    let axis_code=array<u32,3>(0u,0x80ffu,0xe070u)[(family/4u)%3u];
    let domain=(family/12u)%2u+1u;
    let pair=array<vec2u,10>(vec2u(21u,0u),vec2u(21u,1u),vec2u(21u,2u),
        vec2u(21u,3u),vec2u(21u,4u),vec2u(21u,5u),vec2u(23u,0u),
        vec2u(23u,1u),vec2u(23u,2u),vec2u(23u,3u))[family/24u];
    let a=select(0u,128u,profile==1u);
    let c=select(128u,255u,profile==1u);
    let phase=select(64u,128u,profile==1u);
    let ins=vec4u((phase<<24u)|(axis_code<<8u)|0xf0u,
        (64u<<24u)|(a<<16u)|(255u<<8u)|c,0x66224466u,
        (pair.x<<27u)|(7u<<24u)|(3u<<21u)|(domain<<19u)|(pair.y<<16u)|0xff99u);
    let axis=normalize(select(vec3f(-1.0,0.08,0.0),decode_dir16(instr_dir16(ins)),axis_code!=0u));
    let basis=advect_build_basis(axis);
    let centre=axis*select(1.0,-1.0,pole==1u);
    let tangent=select(basis[0],basis[2],meridian>=2u)*select(1.0,-1.0,meridian%2u==1u);
    let eps=array<f32,4>(0.001,0.0001,0.00001,0.000001)[level];
    if id.x==9u {
        textureStore(result,id.xy,1000.0*vec4f(
            transport_transverse(transport_cap_ray(centre,tangent,0.05-eps),basis)-0.05,
            transport_transverse(transport_cap_ray(centre,tangent,0.05+eps),basis)-0.05,
            transport_transverse(transport_cap_ray(centre,tangent,0.05),basis)-0.05,eps));return;
    }
    if id.x==10u {textureStore(result,id.xy,vec4f(f32(instr_opcode(ins)),f32(instr_variant_id(ins)),f32(instr_domain_id(ins)),eps*1000.0));return;}
    if id.x>=5u && id.x<=8u {
        let r=array<f32,4>(0.051,0.1,0.5,0.9)[id.x-5u];
        let dir=transport_cap_ray(centre,tangent,r);
        var actual=advect_coords_cyl(dir,axis);
        if domain==2u {actual=advect_coords_polar(dir,axis);}
        let prior=transport_original_coords(dir,axis,domain);
        let difference=abs(actual-prior);
        textureStore(result,id.xy,vec4f(select(0.0,1.0,all(actual==prior)),max(difference.x,max(difference.y,difference.z)),0,1));return;
    }
    let offset=select(f32(i32(id.x)-2),0.0,id.x==11u);
    let dir=transport_cap_ray(centre,tangent,0.05+offset*eps);
    var sample=LayerSample(vec3f(0),0.0);
    if pair.x==21u {sample=eval_advect(dir,ins,1.0);} else {sample=eval_mass(dir,ins,1.0);}
    var color=vec4f(apply_blend(vec3f(0.17,0.21,0.29),sample,3u),epu_saturate(sample.w));
    if id.x==11u {color.x+=0.04;}
    textureStore(result,id.xy,color);
}
"#,
        12,
        10 * 2 * 3 * 2 * 2 * 4 * 4,
    );

    assert!(image.iter().flatten().all(|v| v.is_finite()));
    let mut failures = Vec::new();
    let mut outside_failures = 0;
    let mut max_error = 0.0f32;
    let mut max_same = 0.0f32;
    let mut max_outside = 0.0f32;
    let mut rejected = 0;
    for family in 0..960 {
        let mut previous = [f32::INFINITY; 2];
        for level in 0..4 {
            let row = &image[(family * 4 + level) * 12..(family * 4 + level + 1) * 12];
            let loc = row[9];
            assert!(
                loc[0] < 0.0 && loc[1] > 0.0 && loc[2].abs() < 0.0001,
                "unbracketed cap join {family} {level} {loc:?}"
            );
            for c in 0..2 {
                assert!(loc[c].abs() < previous[c]);
                previous[c] = loc[c].abs();
            }
            for p in &row[5..9] {
                outside_failures += usize::from(p[0] != 1.0);
                max_outside = max_outside.max(p[1]);
            }
            assert!(
                (row[11][0] - row[2][0]).abs() > 0.035,
                "injected join jump escaped"
            );
            rejected += 1;
            if level == 3 {
                let mut jump = 0.0f32;
                let mut same = 0.0f32;
                for c in 0..4 {
                    for (a, b) in [(1, 2), (2, 3), (1, 3)] {
                        jump = jump.max((row[a][c] - row[b][c]).abs());
                    }
                    for (a, b) in [(0, 1), (3, 4)] {
                        same = same.max((row[a][c] - row[b][c]).abs());
                    }
                }
                max_error = max_error.max(jump);
                max_same = max_same.max(same);
                if jump > 0.005 {
                    failures.push((family, jump, same));
                }
            }
        }
    }
    println!(
        "TRANSPORT_CAP_JOIN samples={} families=960 failures={} outside_records=15360 outside_failures={outside_failures} max_error={max_error} max_same_side={max_same} max_outside={max_outside} rejected={rejected} limit=.005",
        image.len(),
        failures.len()
    );
    assert_eq!(outside_failures, 0, "off-cap coordinates must remain exact");
    assert!(
        failures.is_empty(),
        "new cap join discontinuity: {failures:?}"
    );
}

#[test]
fn carrier_phase_wrap_and_packed_controls() {
    let mut cases: Vec<[u32; 4]> = Vec::new();
    for (op, count) in [(20u32, 4u32), (21, 6), (23, 4)] {
        for variant in 0..count {
            for domain in 0..if op == 20 { 1 } else { 3 } {
                for (pa, pb, pc) in [(0, 255, 128), (192, 128, 255)] {
                    for axis in [0, 0xe070] {
                        cases.push([
                            (64 << 24) | (axis << 8) | 0xf0,
                            (64 << 24) | (pa << 16) | (pb << 8) | pc,
                            (0x66 << 24) | 0x224466,
                            (op << 27)
                                | (7 << 24)
                                | (3 << 21)
                                | (domain << 19)
                                | (variant << 16)
                                | 0xff99,
                        ]);
                    }
                }
            }
        }
    }
    assert_eq!(cases.len(), 136);
    let mut clones = Vec::new();
    for (name, source) in [
        (
            "mottle",
            include_str!("../../shaders/epu/features/12_mottle.wgsl"),
        ),
        (
            "advect",
            include_str!("../../shaders/epu/features/13_advect.wgsl"),
        ),
        (
            "mass",
            include_str!("../../shaders/epu/features/15_mass.wgsl"),
        ),
    ] {
        let signature = format!("fn eval_{name}(");
        let f = &source[source.find(&signature).unwrap()..];
        let decode = "let phase01 = epu_loop_phase01(instr_d(instr));";
        assert_eq!(f.matches(decode).count(), 1);
        clones.push(
            f.replacen(&signature, &format!("fn audit_{name}("), 1)
                .replace(decode, "let phase01 = audit_phase;"),
        );
    }
    let words = cases
        .iter()
        .map(|w| format!("vec4u({}u,{}u,{}u,{}u)", w[0], w[1], w[2], w[3]))
        .collect::<Vec<_>>()
        .join(",");
    let body = r#"@group(0) @binding(0) var out_tex: texture_storage_2d<rgba16float,write>;
var<private> audit_phase: f32;
__CLONES__
const CASES=array<vec4u,__COUNT__>(__WORDS__);
fn audit_eval(d:vec3f,i:vec4u,w:f32,clone:bool)->LayerSample {
 if instr_opcode(i)==20u {if clone{return audit_mottle(d,i,w);}return eval_mottle(d,i,w);}
 if instr_opcode(i)==21u {if clone{return audit_advect(d,i,w);}return eval_advect(d,i,w);}
 if clone{return audit_mass(d,i,w);}return eval_mass(d,i,w);
}
fn audit_pixel(d:vec3f,i:vec4u,w:f32,clone:bool)->vec4f {
 let s=audit_eval(d,i,w,clone);
 return vec4f(apply_blend(vec3f(.17,.21,.29),s,3u),epu_saturate(s.w));
}
@compute @workgroup_size(1)
fn probe(@builtin(global_invocation_id) id:vec3u){
 if id.x>=320u || id.y>=__HEIGHT__u{return;}
 var i=CASES[id.y/39u];let op=instr_opcode(i);let ray=id.y%39u;
 let raw=instr_dir16(i);let fallback=select(vec3f(-1,.08,0),vec3f(0,1,0),op==20u);
 let axis=normalize(select(fallback,decode_dir16(raw),raw!=0u));let basis=advect_build_basis(axis);
 let dirs=array<vec3f,7>(axis,-axis,basis[0],-basis[0],basis[2],-basis[2],normalize(axis+basis[0]+basis[2]));
 var dir=dirs[min(ray,6u)];
 if ray>=7u {
  let j=ray-7u;let theta=f32(j%16u)*TAU/16.0;
  dir=normalize(vec3f(cos(theta),select(-.12,.12,j>=16u),sin(theta)));
 }
 var pixel=vec4f(0);
 if id.x<56u {
  let eps=array<f32,4>(.001,.0001,.00001,.000001)[id.x/14u];
  audit_phase=array<f32,14>(-2.0*eps,-eps,0,eps,2.0*eps,1.0-eps,1,1.0+eps,2.0-eps,2,2.0+eps,8.0-eps,8,8.0+eps)[id.x%14u];
  pixel=audit_pixel(dir,i,1.0,true);
 }else if id.x<312u {
  let phase=id.x-56u;i.x=(i.x&0x00ffffffu)|(phase<<24u);audit_phase=f32(phase)/256.0;
  let real=audit_pixel(dir,i,1.0,false);let clone=audit_pixel(dir,i,1.0,true);
  let error=abs(real-clone);pixel=vec4f(max(max(error.x,error.y),max(error.z,error.w)),real.w,real.x,real.y);
 }else if id.x==312u {
  i.y=i.y&0x00ffffffu;pixel=audit_pixel(dir,i,1.0,false);
 }else if id.x==313u {
  let full=audit_pixel(dir,i,1.0,false);let half=audit_pixel(dir,i,.5,false);
  pixel=abs(half-mix(vec4f(.17,.21,.29,0),full,.5));
 }else if id.x==314u {pixel=audit_pixel(dir,i,0.0,false);
 }else if id.x<318u {
  let original=audit_pixel(dir,i,1.0,false);let domain=id.x-314u;
  i.w=(i.w&~(3u<<19u))|(domain<<19u);
  let changed=audit_pixel(dir,i,1.0,false);
  pixel=vec4f(select(0.0,1.0,all(original==changed)),abs(original-changed).xyz);
 }else if id.x==318u {audit_phase=-.000001;pixel=audit_pixel(dir,i,1.0,true);pixel.w+=.04;
 }else {pixel=vec4f(f32(op),f32(instr_variant_id(i)),f32(instr_domain_id(i)),f32(ray));}
 textureStore(out_tex,vec2i(id.xy),pixel);
}
"#
        .replace("__CLONES__", &clones.join("\n"))
        .replace("__COUNT__", &cases.len().to_string())
        .replace("__WORDS__", &words)
        .replace("__HEIGHT__", &(cases.len()*39).to_string());
    let p = probe_image(&body, 320, (cases.len() * 39) as u32);
    let mut failures = 0;
    let mut wrap01 = 0;
    let mut domain_failures = 0;
    let mut bridge_failures = 0;
    let mut rejected = 0;
    let mut maximum = 0.0f32;
    let mut max_bridge = 0.0f32;
    let mut max_same = 0.0f32;
    let mut positive = std::collections::BTreeMap::<(u32, u32, u32), f32>::new();
    let delta = |a: [f32; 4], b: [f32; 4]| {
        a.into_iter()
            .zip(b)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max)
    };
    for (family, words) in cases.iter().enumerate() {
        let key = (words[3] >> 27, (words[3] >> 16) & 7, (words[3] >> 19) & 3);
        for ray in 0..39 {
            let row = &p[(family * 39 + ray) * 320..(family * 39 + ray + 1) * 320];
            assert!(row.iter().flatten().all(|v| v.is_finite()));
            assert_eq!(
                row[319],
                [key.0 as f32, key.1 as f32, key.2 as f32, ray as f32]
            );
            let mut error = 0.0f32;
            let mut error01 = 0.0f32;
            for a in [43, 44, 45, 47, 48, 49, 50, 51, 52, 53, 54, 55] {
                for b in [43, 44, 45, 47, 48, 49, 50, 51, 52, 53, 54, 55] {
                    error = error.max(delta(row[a], row[b]));
                }
            }
            for a in [44, 45, 47, 48] {
                for b in [44, 45, 47, 48] {
                    error01 = error01.max(delta(row[a], row[b]));
                }
            }
            let same = delta(row[42], row[43]).max(delta(row[45], row[46]));
            let bridge = row[56..312].iter().map(|v| v[0]).fold(0.0f32, f32::max);
            let mask = row[313].into_iter().fold(0.0f32, f32::max);
            let zero = delta(row[312], [0.17, 0.21, 0.29, 0.0])
                .max(delta(row[314], [0.17, 0.21, 0.29, 0.0]));
            let peak = row[56..312].iter().map(|v| v[1]).fold(0.0f32, f32::max);
            positive
                .entry(key)
                .and_modify(|v| *v = v.max(peak))
                .or_insert(peak);
            rejected += usize::from(delta(row[318], row[43]) > 0.03);
            if key.0 == 20 {
                domain_failures += row[315..318].iter().filter(|v| v[0] != 1.0).count();
            }
            failures += usize::from(
                error > 0.002 || same > 0.002 || bridge > 0.002 || mask > 0.002 || zero > 0.002,
            );
            wrap01 += usize::from(error01 > 0.002);
            bridge_failures += usize::from(bridge > 0.002);
            maximum = maximum.max(error);
            max_bridge = max_bridge.max(bridge);
            max_same = max_same.max(same);
        }
    }
    let missing = positive
        .iter()
        .filter(|(_, v)| **v <= 0.005)
        .collect::<Vec<_>>();
    println!(
        "CARRIER_PHASE records={} rays={} failures={failures} wrap01_failures={wrap01} max_error={maximum} max_same_side={max_same} bridge_failures={bridge_failures} max_bridge={max_bridge} domain_failures={domain_failures} missing_positive={} rejected={rejected} limit=.002",
        p.len(),
        cases.len() * 39,
        missing.len()
    );
    assert_eq!(rejected, cases.len() * 39);
    assert_eq!(domain_failures, 0);
    assert!(missing.is_empty(), "{missing:?}");
    assert_eq!(failures, 0);
}

#[test]
fn carrier_phase_zero_shape_is_preserved() {
    let body = r#"
@group(0) @binding(0) var out_tex:texture_storage_2d<rgba16float,write>;
fn phase_zero_legacy_body(p: vec3f, breakup: f32, phase: f32) -> vec3f {
    let amt = breakup * breakup;
    if amt <= 1e-5 {
        return p;
    }

    let bend0 = epu_relief_wave(vec2f(p.y * 0.43, p.z * 0.31), phase + p.x * 0.19);
    let bend1 = epu_relief_wave(vec2f(p.z * 0.37, p.x * 0.29), phase * 0.73 + p.y * 0.23);
    let bend2 = epu_relief_wave(vec2f(p.x * 0.34, p.y * 0.27), phase * 1.21 - p.z * 0.17);
    let bend_amt = mix(0.0, 0.34, amt);

    return vec3f(
        p.x + (p.y * 0.22 + bend0 * 0.58 + bend2 * 0.16) * bend_amt,
        p.y + (p.z * -0.12 + bend1 * 0.44 + bend0 * 0.09) * bend_amt,
        p.z + (p.x * 0.19 + bend1 * 0.31 + bend2 * 0.23) * bend_amt
    );
}
@compute @workgroup_size(1)
fn probe(@builtin(global_invocation_id) id:vec3u){
 if id.x>=256u || id.y>=32u{return;}
 let v=vec3f(f32(id.x%8u)/7.0,f32((id.x/8u)%8u)/7.0,f32(id.y%16u)/15.0)*2.0-1.0;
 let breakup=array<f32,4>(0,.25,.5,1)[id.x/64u];let seed=select(.19,.27,id.y>=16u);
 let old=phase_zero_legacy_body(v,breakup,seed);
 let now=epu_body_curve_coords(v,breakup,0.0,seed);
 let d=abs(old-now);
 textureStore(out_tex,vec2i(id.xy),vec4f(max(d.x,max(d.y,d.z)),select(0.0,1.0,all(old==now)),length(old),seed));
}
"#;
    let p = probe_image(body, 256, 32);
    let failures = p.iter().filter(|p| p[1] != 1.0).count();
    let max = p.iter().map(|p| p[0]).fold(0.0f32, f32::max);
    assert!(p.iter().flatten().all(|v| v.is_finite()));
    assert!(p.iter().any(|p| p[2] > 1.0));
    println!(
        "CARRIER_ZERO_SHAPE records={} failures={failures} max_error={max} tolerance=0",
        p.len()
    );
    assert_eq!(failures, 0);
}

#[test]
fn shared_carrier_lattice_limits() {
    const TOL: f32 = 0.001; // Frozen absolute RGBA16F budget, not a relative error.
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) id:vec3u) {
    let eps = array<f32,3>(0.001, 0.0001, 0.00001)[id.x / 5u];
    let offset = f32(i32(id.x % 5u) - 2) * eps;
    let k = f32(i32(id.y % 33u) - 16);
    var p = vec3f(k + offset, 0.37, -1.21);
    switch id.y / 33u {
        case 1u: { p = vec3f(0.37, k + offset, -1.21); }
        case 2u: { p = vec3f(0.37, -1.21, k + offset); }
        case 3u: { p = vec3f(k + offset); }
        default: {}
    }
    textureStore(result, id.xy, vec4f(
        epu_value_noise3(p), mottle_value_noise3(p),
        advect_value_noise3(p), epu_fbm3(p,3u)));
}"#,
        15,
        132,
    );
    assert!(pixels.iter().flatten().all(|v| v.is_finite()));
    let mut worst = [[0.0f32; 4]; 3];
    let mut range = [(f32::INFINITY, f32::NEG_INFINITY); 4];
    let mut failures = Vec::new();
    for (row, samples) in pixels.chunks_exact(15).enumerate() {
        let mut previous = [0.0; 4];
        for (level, five) in samples.chunks_exact(5).enumerate() {
            for c in 0..4 {
                for p in five {
                    range[c].0 = range[c].0.min(p[c]);
                    range[c].1 = range[c].1.max(p[c]);
                }
                // Bilateral, both on-cut limits, and equal-distance same-side controls.
                let jump = [(1, 3), (1, 2), (2, 3), (0, 1), (3, 4)]
                    .iter()
                    .map(|&(a, b)| (five[a][c] - five[b][c]).abs())
                    .fold(0.0f32, f32::max);
                worst[level][c] = worst[level][c].max(jump);
                if (level == 2 && jump > TOL) || (level > 0 && jump > previous[c] * 0.35 + TOL) {
                    failures.push((row, level, c, jump));
                }
                previous[c] = jump;
            }
        }
    }
    println!(
        "SHARED_NOISE boundaries=132 samples={} worst={worst:?} range={range:?} failures={failures:?} tolerance={TOL}",
        pixels.len()
    );
    assert!(
        range.iter().all(|(lo, hi)| hi - lo > 0.25),
        "nonconstant controls"
    );
    assert!(
        failures.is_empty(),
        "shared carrier lattice limits must converge"
    );
}
