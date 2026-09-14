// Exact Neon Metropolis cuts: generated directions in gpu-pairs.json.
use super::*;

#[test]
fn veil_variant_control_activity() {
    // Interleave baseline/changed/fallback through ONE evaluator call. Duplicated
    // calls can get different floating-point optimizations even for equal inputs.
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let case_id = p.y / 8u;
    let field = case_id % 4u;
    let variant = (case_id / 4u) % 8u;
    let domain = case_id / 32u;
    let height = (f32(p.y % 8u) + 0.5) / 4.0 - 1.0;
    let angle = (f32(p.x / 3u) + 0.5) / 128.0 * TAU;
    let radius = sqrt(1.0 - height * height);
    let ray = vec3f(sin(angle)*radius, height, cos(angle)*radius);
    var instr = vec4u((37u<<24u)|(0x8080u<<8u)|0xadu,
                     (200u<<24u)|(51u<<16u)|(50u<<8u)|64u,
                     0x66aa3399u, (13u<<27u)|(7u<<24u)|(((domain<<3u)|variant)<<16u)|0xcc66u);
    if p.x % 3u == 1u {
        switch field {
            case 0u: { instr.y = (instr.y & 0xffffff00u) | 219u; }
            case 1u: { instr.x = (instr.x & 0x00ffffffu) | (166u<<24u); }
            case 2u: { instr.z ^= 0x00ffffffu; }
            default: { instr.x ^= 15u; }
        }
    } else if p.x % 3u == 2u { instr.w &= ~0x70000u; }
    let sample = eval_veil(ray, instr, 1.0);
    textureStore(result,p.xy,vec4f(sample.rgb*sample.w,sample.w));
}"#,
        384,
        1024,
    );
    assert!(pixels.iter().flatten().all(|x| x.is_finite()));
    let mut active_cases = 0;
    for (case_id, rows) in pixels.as_chunks::<{ 384 * 8 }>().0.iter().enumerate() {
        let field = case_id % 4;
        let variant = (case_id / 4) % 8;
        let active = match field {
            0 => !matches!(variant, 1 | 2),
            1 => variant == 3,
            _ => variant != 1,
        };
        let peak = rows
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|p| (0..4).map(move |c| (p[0][c] - p[1][c]).abs()))
            .fold(0.0, f32::max);
        assert_eq!(peak > 0.002, active, "VEIL case={case_id} delta={peak}");
        if !active {
            assert_eq!(peak, 0.0, "VEIL inactive case={case_id}");
        }
        assert!(rows.iter().any(|p| p[3] > 0.002), "blank variant {case_id}");
        if variant > 4 {
            assert!(rows.as_chunks::<3>().0.iter().all(|p| p[0] == p[2]));
        }
        active_cases += usize::from(active);
    }
    println!(
        "VEIL_CONTROL records={} cases=128 active={active_cases} inactive exact; all domains; fallback exact",
        pixels.len()
    );
}

#[test]
fn rain_wall_phase_cycle_closes() {
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let u=(f32(p.x)+0.5)/128.; let v=(f32(p.y)+0.5)/64.*2.-1.;
 let a=eval_veil_rain_wall(u,v,8u,.025,.5,0.);
 let b=eval_veil_rain_wall(u,v,8u,.025,.5,1.);
 textureStore(result,p.xy,vec4f(a.y,b.y,abs(a.y-b.y),abs(a.z-b.z)));
}"#,
        128,
        64,
    );
    assert!(pixels.iter().flatten().all(|v| v.is_finite()));
    assert!(pixels.iter().any(|v| v[0] > 0.1));
    let failures = pixels
        .iter()
        .filter(|v| v[2] > 0.005 || v[3] > 0.005)
        .count();
    assert_eq!(
        failures, 0,
        "RAIN_WALL phase cycle must close without deleting rain"
    );
}

#[test]
fn rain_wall_byte_travel_and_caps() {
    // Actual shared helper must follow both one- and two-byte steps, including
    // 255->0, without respawning the bar or threshold-clipping its smooth cap.
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let h=veil_hash23(vec2f(f32(p.y)*11.3,17.7));
 let speed=1.+floor(h.x*2.); let phase=epu_loop_phase01(p.x);
 let drop=fract(h.y+phase*speed); let u=(f32(p.y)+.5)/16.;
 let a=eval_veil_rain_wall(u,drop*2.-1.,8u,.01,.5,phase);
 let b=eval_veil_rain_wall(u,fract(drop+speed/256.)*2.-1.,8u,.01,.5,epu_loop_phase01((p.x+1u)%256u));
 let c=eval_veil_rain_wall(u,fract(drop+speed/128.)*2.-1.,8u,.01,.5,epu_loop_phase01((p.x+2u)%256u));
 let v=fract(drop+1.3*(.02+h.z*.06))*2.-1.;
 let cap=eval_veil_rain_wall(u,v,8u,.01,.5,phase);
 // At 1.3*half_len the unchanged smooth envelope is 0.5, not a hard cut.
 textureStore(result,p.xy,vec4f(a.y,abs(b.y-a.y),abs(c.y-a.y),abs(cap.y-.5*a.y)));
}"#,
        256,
        16,
    );
    assert!(pixels.iter().flatten().all(|x| x.is_finite()));
    for (i, p) in pixels.iter().enumerate() {
        assert!(
            p[0] > 0.39 && p[1..].iter().all(|x| *x <= 0.001),
            "rain byte/cap {i}: {p:?}"
        );
    }
    println!(
        "RAIN 4096 column/phase cases: +1/+2 travel and half-envelope caps pass; tolerance=.001"
    );
}

#[test]
fn showcase_flow_identified_cell_boundaries() {
    const TOL: f32 = 0.001; // frozen absolute linear/sample budget, rgba16float readback
    let pixels = probe_image(include_str!("flow-pairs.wgsl"), 24, 1);
    let mut worst = [0.0f32; 4];
    for (i, pair) in pixels.as_chunks::<2>().0.iter().enumerate() {
        assert!(pair.iter().flatten().all(|x| x.is_finite()));
        for c in 0..4 {
            worst[c] = worst[c].max((pair[0][c] - pair[1][c]).abs());
        }
        println!("FLOW exact boundary {i}: {pair:?}");
    }
    println!("FLOW maximum jumps [weight, noise x/y/z] {worst:?}; tolerance={TOL}");
    assert!(
        worst.iter().all(|x| *x <= TOL),
        "continuous FLOW / trilinear noise violated: {worst:?}"
    );
}

#[test]
fn showcase_flow_lattice_regression() {
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let eps=select(-0.00001,0.00001,p.x==1u);
 let k=f32(i32(p.y%257u)-128);
 let axis=p.y/257u;
 let coord=select(select(vec3f(k+eps,0.37,-1.21),vec3f(0.37,k+eps,-1.21),axis==1u),vec3f(0.37,-1.21,k+eps),axis==2u);
 let n=value_noise3(coord);
 textureStore(result,p.xy,vec4f(n,n,n,1.));
}"#,
        2,
        771,
    );
    let mut worst = 0.0f32;
    let mut lo = 1.0f32;
    let mut hi = -1.0f32;
    for pair in pixels.as_chunks::<2>().0.iter() {
        assert!(pair.iter().flatten().all(|x| x.is_finite()));
        worst = worst.max((pair[0][0] - pair[1][0]).abs());
        lo = lo.min(pair[0][0]);
        hi = hi.max(pair[0][0]);
    }
    println!(
        "FLOW 771 lattice boundaries +/-128 all axes max_jump={worst}, range={lo}..{hi}, tolerance=.001"
    );
    assert!(worst <= 0.001 && hi - lo > 0.5);
}

#[test]
fn showcase_veil_endpoint_continuity() {
    // The existing smooth segment envelope must remain continuous at its former hard gate.
    let body = r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
let i=p.y; let h=veil_hash23(vec2f(f32(i)*11.3,17.7));
let phase=32./256.; let drop=fract(h.y+phase*(1.+floor(h.x*2.)));
let half_len=.02+h.z*.06;
let side=select(-1.,1.,p.x>=2u); let eps=select(-.000001,.000001,(p.x%2u)==1u);
let v=fract(drop+side*half_len*1.3)*2.-1.+eps;
let u=(f32(i)+.5)/16.-v*(142./255.-.5)*.15;
let axis=decode_dir16(128u);let hint=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(axis.y)>.9);
let t=normalize(cross(hint,axis));let b=normalize(cross(axis,t));
let angle=(u-.5)*TAU;let dir=axis*v+sqrt(max(0.,1.-v*v))*(t*sin(angle)+b*cos(angle));
let instr=vec4u(0x20008093u,0x5834148eu,0xa81d2936u,0x6f8b5f7eu);
let s=eval_veil(dir,instr,1.);
textureStore(result,p.xy,vec4f(s.rgb*s.w,v));
}"#;
    let pixels = probe_image(body, 4, 16);
    let mut max = 0.0f32;
    for (i, pair) in pixels.as_chunks::<2>().0.iter().enumerate() {
        assert!(pair.iter().flatten().all(|v| v.is_finite()));
        let jump = (0..3)
            .map(|c| (pair[0][c] - pair[1][c]).abs())
            .fold(0., f32::max);
        if jump > 0.001 {
            println!("VEIL bar/end {i}, pair {pair:?}, jump {jump}");
        }
        max = max.max(jump);
    }
    println!("VEIL segment maximum linear weighted-RGB jump={max}; continuity budget .001");
    assert!(max <= 0.001, "rain envelope discontinuity: {max}");
}

#[test]
fn showcase_rain_support_and_bar_transitions() {
    // Production helper: isolated finite caps/core, then wide overlapping bars at
    // every nearest-bar boundary (including u wrap), multiple heights and phases.
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let eps=select(-0.000001,0.000001,p.x==1u);
 let i=p.y%16u; let h=veil_hash23(vec2f(f32(i)*11.3,17.7));
 let drop=fract(h.y+(32./256.)*(1.+floor(h.x*2.)));
 var s=vec3f(0.);
 if(p.y<48u) {
  let side=f32(i32(p.y/16u)-1);
  let v=2.*fract(drop+side*(1.6*(.02+h.z*.06)+.001))-1.;
  s=eval_veil_rain_wall((f32(i)+.5)/16.,v,8u,.01,.5,32./256.);
 } else {
  let row=p.y-48u; let v=-.9+1.8*f32((row/16u)%19u)/18.;
  let phase=f32(row/(16u*19u))/4.;
  let u=f32(i)/16.+eps;
  s=eval_veil_rain_wall(u,v,8u,.0625,.5,phase);
 }
 textureStore(result,p.xy,vec4f(s,1.));
}"#,
        2,
        48 + 16 * 19 * 4,
    );
    let mut worst = 0.0f32;
    let mut active = 0;
    for (i, pair) in pixels.as_chunks::<2>().0.iter().enumerate() {
        assert!(pair.iter().flatten().all(|v| v.is_finite()));
        if i < 48 {
            if (16..32).contains(&i) {
                assert!(pair[0][1] > 0.39, "nonzero core {i}: {pair:?}");
            } else {
                assert_eq!(&pair[0][1..3], &[0., 0.], "finite support {i}");
            }
        } else {
            active += usize::from(pair[0][1] + pair[0][2] > 0.01);
            for c in 1..3 {
                worst = worst.max((pair[0][c] - pair[1][c]).abs());
            }
        }
    }
    println!(
        "RAIN support/core 48 cases; 1216 wrap/neighbor pairs, active={active}, max_jump={worst}, tolerance=.001"
    );
    assert!(active > 0 && worst <= 0.001);
}

#[test]
fn trail_native_endpoint_mapping() {
    let body = r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
let locations=array<vec2f,5>(vec2f(145.,82.),vec2f(197.,232.),vec2f(798.,165.),vec2f(743.,389.),vec2f(558.,492.));
let xy=locations[p.y]+vec2f(0.,f32(p.x)-4.);
let eye=vec3f(0.,2.294095516204834,4.829628944396973);
let forward=normalize(-eye);let right=normalize(cross(forward,vec3f(0.,1.,0.)));let up=cross(right,forward);
let dir=normalize(forward+right*((xy.x+.5)/960.*2.-1.)*tan(PI/6.)*960./540.+up*(1.-(xy.y+.5)/540.*2.)*tan(PI/6.));
let uv=veil_cyl_uv(dir,decode_dir16(128u));
var best=100.;var residual=100.;var bar=0.;
for(var i=0u;i<16u;i++) {
 let h=veil_hash23(vec2f(f32(i)*11.3,17.7));let drop=fract(h.y+(32./256.)*(.3+h.x*1.4));
 let d=ribbon_dist_wrapped(fract(uv.x)+uv.y*(142./255.-.5)*.15,(f32(i)+.5)/16.);
 if(d<best){best=d;residual=abs(uv.y*.5+.5-drop)-1.3*(.02+h.z*.06);bar=f32(i);}
}
let s=eval_veil(dir,vec4u(0x20008093u,0x5834148eu,0xa81d2936u,0x6f8b5f7eu),1.);
textureStore(result,p.xy,vec4f(residual,best,bar,s.w));
}"#;
    // The current envelope is also checked at the original native-image rays.
    // Restore ONLY historical motion for this control: approved loop-safe travel
    // intentionally moves these drops. This is not current-placement parity.
    let source = include_str!("../../shaders/epu/features/05_veil.wgsl");
    let start = source.find("fn eval_veil_rain_wall(").unwrap();
    let end = source[start..].find("// SHARDS variant:").unwrap() + start;
    let historical_rain = source[start..end]
        .replace("eval_veil_rain_wall(", "historical_rain(")
        .replace("1.0 + floor(h.x * 2.0)", "0.3 + h.x * 1.4")
        .replace(
            "abs(fract(v01 - drop_pos + 0.5) - 0.5)",
            "abs(v01 - drop_pos)",
        );
    let historical_eval = source[source.find("fn eval_veil(").unwrap()..]
        .replacen("fn eval_veil(", "fn historical_veil(", 1)
        .replace("eval_veil_rain_wall(", "historical_rain(");
    let body = body.replace("let s=eval_veil(", "let s=historical_veil(");
    let pixels = probe_image(
        &format!("{historical_rain}\n{historical_eval}\n{body}"),
        9,
        5,
    );
    for (i, row) in pixels.as_chunks::<9>().0.iter().enumerate() {
        println!("native endpoint {i} residual/distance/bar/weight: {row:?}");
        assert!(row.iter().flatten().all(|v| v.is_finite()));
        assert!(
            row.iter().any(|v| v[0] < 0.) && row.iter().any(|v| v[0] > 0.),
            "native ROI must straddle the historical cap"
        );
        assert!(
            row.iter().all(|v| v[1] < 0.001 && v[3] > 0.01),
            "native cap must carry nonzero envelope"
        );
        assert!(
            row.windows(2).all(|p| (p[0][3] - p[1][3]).abs() < 0.01),
            "native cap still jumps"
        );
    }
}
