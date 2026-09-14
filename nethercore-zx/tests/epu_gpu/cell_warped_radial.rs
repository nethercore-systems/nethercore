//! Runtime-buffer parity against the independently frozen, qualified organic candidate.
use super::*;

const BODY: &str = r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@group(0) @binding(1) var<storage,read> words: array<vec4u>;
fn warped_max4(v:vec4f)->f32 { return max(max(v.x,v.y),max(v.z,v.w)); }
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let i=words[p.y/4u];
    var actual_i=i;
    // WRONG_VARIANT_CONTROL
    var expected_i=i;
    expected_i.w=(i.w&0xffe0ffffu)|(3u<<16u);
    let y=(f32(p.y%4u)+.5)/2.-1.;
    let angle=(f32(p.x)+.5)/64.*TAU;
    let r=sqrt(1.-y*y);
    let dir=vec3f(r*cos(angle),y,r*sin(angle));
    let base=RegionWeights(.2,.3,.5);
    let expected=warpref_eval_cell(dir,expected_i,base);
    let actual=evaluate_bounds_layer(dir,actual_i,OP_CELL,decode_dir16(instr_dir16(actual_i)),base);
    let sample_error=warped_max4(abs(vec4f(actual.sample.rgb,actual.sample.w)-vec4f(expected.sample.rgb,expected.sample.w)));
    let region_error=warped_max4(abs(vec4f(actual.regions.sky,actual.regions.wall,actual.regions.floor,actual.region_mix)-vec4f(expected.regions.sky,expected.regions.wall,expected.regions.floor,expected.region_mix)));
    var layers:array<vec4u,8>;layers[0]=actual_i;
    let composition_error=warped_max4(vec4f(abs(evaluate_epu_layers(dir,layers)-expected.sample.rgb*expected.sample.w),0.));
    let errors=vec3f(sample_error,region_error,composition_error);
    // Decide on f32 GPU values, not independently rounded half-float samples.
    textureStore(result,p.xy,vec4f(errors,select(0.,1.,any(errors>vec3f(.002)))));
}
"#;

fn capture(wrong_variant: bool) -> usize {
    let mut words = Vec::new();
    for axis in [0x8080u32, 0x80ff, 0xe0a0] {
        for density in [0u32, 63, 127, 254, 255] {
            for fill in [0u32, 128, 255] {
                for gap in [0u32, 64, 255] {
                    for seed in [0u32, 255] {
                        for alpha in [0u32, 7, 15] {
                            words.extend([
                                (seed << 24) | (axis << 8) | (alpha << 4) | 15,
                                (96 << 24) | (density << 16) | (fill << 8) | gap,
                                0x90604080,
                                (5 << 27) | (7 << 24) | (3 << 21) | (6 << 16) | 0x1020,
                            ]);
                        }
                    }
                }
            }
        }
    }
    let body = BODY.replace(
        "// WRONG_VARIANT_CONTROL",
        if wrong_variant {
            "actual_i.w=(i.w&0xffe0ffffu)|(3u<<16u);"
        } else {
            ""
        },
    );
    let source = format!(
        "{}\n{body}",
        include_str!("fixtures/cell-warped/reference.wgsl")
    );
    let pixels = probe_image_input_bounds(&source, EPU_BOUNDS, 64, words.len() as u32, &words);
    assert_eq!(pixels.len(), 64 * words.len());
    assert!(pixels.iter().flatten().all(|v| v.is_finite()));
    let failures = pixels.iter().filter(|v| v[3] != 0.0).count();
    let maxima: [f32; 3] = std::array::from_fn(|c| pixels.iter().map(|v| v[c]).fold(0.0, f32::max));
    println!(
        "WARPED_RADIAL wrong_variant={wrong_variant} configurations={} records={} failed={failures} maxima={maxima:?} tolerance=.002",
        words.len() / 4,
        pixels.len()
    );
    failures
}

#[test]
fn restored_variant_matches_frozen_candidate() {
    assert_eq!(
        capture(false),
        0,
        "public variant 6 must route the organic capability"
    );
}

#[test]
fn wrong_regular_variant_is_rejected() {
    assert!(
        capture(true) > 0,
        "regular RADIAL must not satisfy organic restoration"
    );
}

fn boundary_words() -> Vec<u32> {
    let mut words = Vec::new();
    for axis in [0xff80u32, 0x80ff, 0xe0a0] {
        for density in 0..256 {
            words.extend([
                (axis << 8) | 255,
                (96 << 24) | (density << 16),
                0x58c09050,
                (5 << 27) | (7 << 24) | (3 << 21) | (6 << 16) | 0x1828,
            ]);
        }
    }
    words.resize(864 * 4, 0);
    words
}

fn chart_capture(injected: bool) -> usize {
    let body = include_str!("fixtures/cell-warped/chart.wgsl").replace(
        "// INJECT_CHART_JUMP",
        if injected {
            "if s>2u {paint[s].x+=.03125;}"
        } else {
            ""
        },
    );
    let pixels = probe_image_input_bounds(&body, EPU_BOUNDS, 108, 3072, &boundary_words());
    assert_eq!(pixels.len(), 108 * 3072);
    assert!(pixels.iter().flatten().all(|v| v.is_finite()));
    let failed = pixels.iter().filter(|v| v[2] != 0.).count();
    println!(
        "WARPED_CHART injected={injected} records={} failed={failed}",
        pixels.len()
    );
    failed
}

// The fourth path is additive: retain every original ray, plus its actual
// axis-relative offset, so collapsed meridians cannot fake pole convergence.
fn pole_capture(defect: u8) -> usize {
    let body = include_str!("fixtures/cell-warped/poles.wgsl")
        .replace(
            "// POLE_SEAM_CONTROL",
            if defect == 1 {
                "if path==0u && level>0u && dot(dir,t)>0. {value.x+=.03125;}"
            } else {
                ""
            },
        )
        .replace(
            "// POLE_DIRECTION_CONTROL",
            if defect == 2 {
                "if level==4u {dir=normalize(axis*sign*sqrt(1.-radius*radius)+radius*t);}"
            } else {
                ""
            },
        );
    let pixels = probe_image_input_bounds(&body, EPU_BOUNDS, 640, 6144, &boundary_words());
    assert_eq!(pixels.len(), 640 * 6144);
    assert!(pixels.iter().flatten().all(|v| v.is_finite()));
    let mut failures = 0;
    let mut maxima = [0.0f32; 2];
    for row in 0..6144 {
        let at = |path: usize, level: usize, angle: usize| {
            pixels[row * 640 + path * 160 + level * 32 + angle]
        };
        let mut bad = false;
        for path in 0..2 {
            let expected = at(path, 0, 0);
            let mut error = 0.0f32;
            for level in 0..5 {
                for angle in 0..32 {
                    for c in 0..3 {
                        error = error.max((at(path, level, angle)[c] - expected[c]).abs());
                    }
                }
            }
            maxima[path] = maxima[path].max(error);
            bad |= error > [0.01, 0.005][path];
        }
        for angle in 0..32 {
            let mut prior = f32::INFINITY;
            for level in 0..5 {
                let v = at(2, level, angle);
                bad |= v[3] != 1.;
                if level > 0 {
                    bad |= v[0] >= prior * 0.2;
                    prior = v[0];
                }
                if level == 4 {
                    bad |= v[0] <= 1.;
                }
            }
        }
        let distinct: std::collections::HashSet<[u32; 3]> = (0..32)
            .map(|angle| std::array::from_fn(|c| at(3, 4, angle)[c].to_bits()))
            .collect();
        bad |= distinct.len() != 32;
        failures += usize::from(bad);
    }
    println!(
        "WARPED_POLES defect={defect} configurations=6144 failed={failures} maxima={maxima:?}"
    );
    failures
}

#[test]
fn wrapped_chart_is_continuous() {
    assert_eq!(chart_capture(false), 0);
}
#[test]
fn chart_seam_is_rejected() {
    assert!(chart_capture(true) > 0);
}
#[test]
fn both_poles_converge() {
    assert_eq!(pole_capture(0), 0);
}
#[test]
fn pole_seam_is_rejected() {
    assert!(pole_capture(1) > 0);
}
#[test]
fn collapsed_pole_rays_are_rejected() {
    assert!(pole_capture(2) > 0);
}
