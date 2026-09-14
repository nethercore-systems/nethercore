//! Regular repeats and guest-owned cyclic GRID motion through actual shared shaders.
use super::*;

const WRAP: &str = include_str!("fixtures/grid/wrap.wgsl");
fn wrap(injected: bool) -> usize {
    let body = WRAP.replace(
        "@INJECT@",
        if injected {
            "+ select(0.,.25,dir.x>=0.)"
        } else {
            ""
        },
    );
    let p = probe_image(&body, 1024, 257);
    assert_eq!(p.len(), 1024 * 257);
    assert!(p.iter().flatten().all(|v| v.is_finite()));
    let mut failures = 0;
    let mut interiors = 0;
    for row in 0..256 {
        for byte in 0..256 {
            let samples: Vec<_> = (0..4).map(|k| p[row * 1024 + k * 256 + byte]).collect();
            assert!(samples.windows(2).all(|w| w[1][3] < w[0][3] * 0.2));
            assert!(
                samples[3][3] > 1.0,
                "bilateral directions must remain distinct"
            );
            if samples.iter().all(|s| s[2] > 0.02) {
                interiors += 1;
                if samples.iter().all(|s| s[1] == 0.0) && samples[3][0] > 0.01 {
                    failures += 1;
                }
            }
        }
    }
    assert!(interiors > 10000);
    assert!(
        p[256 * 1024..].iter().all(|s| s[0] == 1.0),
        "authored hard stripe edges stay sharp"
    );
    println!(
        "GRID_WRAP injected={injected} scenarios=65536 levels=4 interiors={interiors} failed={failures} limit=.01"
    );
    failures
}
#[test]
fn grid_regular_chart_wrap() {
    assert_eq!(wrap(false), 0);
}
#[test]
fn grid_chart_seam_control_rejected() {
    assert!(wrap(true) > 0);
}

fn count(byte: u32, pattern: u32) -> u32 {
    let raw = 1.0 + 63.0 * byte as f64 / 255.0;
    if pattern == 2 {
        ((raw / 2.0).round().max(1.0) * 2.0) as u32
    } else {
        raw.round() as u32
    }
}
fn words(byte: u32, pattern: u32, speed: u32, phase: u32) -> [u32; 4] {
    [
        (phase << 24) | 255,
        (255 << 24) | (byte << 16) | (128 << 8) | (pattern << 4) | speed,
        0xff000000,
        (9 << 27) | (7 << 24) | (3 << 21) | 0xffff,
    ]
}
const HELPERS: &str = r#"
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@group(0) @binding(1) var<storage,read> input:array<u32>;
fn instruction(row:u32)->vec4u {let n=row*5u;return vec4u(input[n],input[n+1u],input[n+2u],input[n+3u]);}
fn paint(dir:vec3f,i:vec4u)->f32 {let s=evaluate_layer(dir,i,vec3f(0.,1.,0.),RegionWeights(1.,0.,0.));return s.rgb.x*s.w;}
fn ray(angle:f32)->vec3f {return normalize(vec3f(sin(angle),.275,cos(angle)));}
"#;
#[test]
fn grid_all_density_pattern_contract() {
    let mut input = Vec::new();
    for pattern in [0, 1, 2, 15] {
        for byte in 0..256 {
            for speed in [0, 15] {
                for phase in [0, 127] {
                    input.extend(words(byte, pattern, speed, phase));
                    input.push(count(byte, pattern));
                }
            }
        }
    }
    let body = format!(
        "{HELPERS}{}",
        r#"
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let i=instruction(p.y);let n=f32(input[p.y*5u+4u]);let pattern=instr_c(i)>>4u;
 let dir=ray((f32(p.x)+.317)/64.*TAU);let u=atan2(dir.x,dir.z)/TAU;let v=dir.y*.5+.5;
 let offset=f32(instr_d(i))/256.*f32(instr_c(i)&15u)*select(1.,2.,pattern==2u);
 let coord=vec2f(u*n+offset,v*n);let f=fract(coord);let thick=mix(0.001f,0.1f,128.0f/255.0f);
 var expected:f32=select(0.,1.,abs(f.x-.5)<thick);
 if pattern==1u {expected=max(expected,select(0.,1.,abs(f.y-.5)<thick));}
 if pattern==2u {expected=select(.6,1.,((i32(floor(coord.x))+i32(floor(coord.y)))&1)==1);}
 let actual=paint(dir,i);textureStore(result,p.xy,vec4f(abs(actual-expected),actual,expected,1.));
}
"#
    );
    let p = probe_image_input_bounds(&body, EPU_BOUNDS, 64, (input.len() / 5) as u32, &input);
    assert!(p.iter().flatten().all(|v| v.is_finite()));
    let failed = p.iter().filter(|v| v[0] > 0.002).count();
    println!(
        "GRID_CONTRACT configurations={} records={} failed={failed} limit=.002",
        input.len() / 5,
        p.len()
    );
    assert_eq!(failed, 0);
}
fn motion(injected: bool) -> (usize, usize) {
    let mut input = Vec::new();
    for pattern in [0, 1, 2, 15] {
        for byte in [0, 1, 63, 127, 255] {
            for speed in [1, 7, 15] {
                input.extend(words(byte, pattern, speed, 0));
                input.push(count(byte, pattern));
            }
        }
    }
    let body = format!(
        "{HELPERS}{}",
        r#"
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let row=p.y/3u;let step=p.y%3u;let phase=p.x%256u;let r=p.x/256u;
 var i=instruction(row);let n=f32(input[row*5u+4u]);let pattern=instr_c(i)>>4u;
 i.x=(i.x&0xffffffu)|(phase<<24u);var next=i;next.x=(i.x&0xffffffu)|(((phase+step)&255u)<<24u);
 let angle=((f32(r)+.173)/8.-.5)*TAU;
 let shift=f32(step)*f32(instr_c(i)&15u)*select(1.,2.,pattern==2u)*TAU/(256.*n);
 var actual=paint(ray(angle),next);let expected=paint(ray(angle+shift),i);
 // INJECT_WRAP
 textureStore(result,p.xy,vec4f(abs(actual-expected),actual,select(0.,1.,phase+step>=256u),1.));
}
"#
    )
    .replace(
        "// INJECT_WRAP",
        if injected {
            "if phase+step>=256u {actual+=.25;}"
        } else {
            ""
        },
    );
    let height = input.len() / 5 * 3;
    let p = probe_image_input_bounds(&body, EPU_BOUNDS, 2048, height as u32, &input);
    assert_eq!(p.len(), 2048 * height);
    assert!(p.iter().flatten().all(|v| v.is_finite()));
    let failed = p.iter().filter(|v| v[0] > 0.002).count();
    let wrap_failed = p.iter().filter(|v| v[0] > 0.002 && v[2] > 0.).count();
    for band in p.chunks(p.len() / 4) {
        let lo = band.iter().map(|v| v[1]).fold(f32::INFINITY, f32::min);
        let hi = band.iter().map(|v| v[1]).fold(0., f32::max);
        assert!(
            hi > 0.5 && hi - lo > 0.1,
            "motion must preserve nonconstant authored patterns"
        );
    }
    println!(
        "GRID_MOTION injected={injected} configurations={} records={} failed={failed} wrap_failed={wrap_failed} held_single_double_steps=true limit=.002",
        input.len() / 5,
        p.len()
    );
    (failed, wrap_failed)
}
#[test]
fn grid_guest_phase_held_single_double_and_wrap() {
    assert_eq!(motion(false), (0, 0));
}
#[test]
fn grid_phase_wrap_pop_control_rejected() {
    let (all, wrap) = motion(true);
    assert!(all > 0 && wrap > 0);
}
