//! Dynamic GRID wraps and zero-width CELL outlines; unchanged continuity budgets.
use super::*;
const CURRENT: &str = include_str!("../../shaders/epu/bounds/04_cell.wgsl");
const BEFORE: &str = include_str!("fixtures/cell-grid-zero/before.wgsl");
const WRAP: &str = include_str!("fixtures/cell-grid-zero/wrap.wgsl");
const OUTLINE: &str = include_str!("fixtures/cell-grid-zero/outline.wgsl");

fn wrap_gate(legacy: bool, injected: bool) -> usize {
    assert_eq!(EPU_BOUNDS.matches(CURRENT).count(), 1);
    let bounds = if legacy {
        EPU_BOUNDS.replace(CURRENT, BEFORE)
    } else {
        EPU_BOUNDS.to_owned()
    };
    let body = if injected {
        WRAP.replace(
            "// INJECT_WRAP_JUMP",
            "if path==2u && sample<15u && sample%5u==2u {out.x+=.125;}",
        )
    } else {
        WRAP.to_owned()
    };
    let pixels = probe_image_bounds(&body, &bounds, 100, 4096);
    assert_eq!(pixels.len(), 100 * 4096);
    assert!(pixels.iter().flatten().all(|x| x.is_finite()));
    let mut failures = [0usize; 4];
    let mut parity = 0;
    let mut peak = 0.0f32;
    for row in 0..4096 {
        let at = |path: usize, sample: usize| pixels[row * 100 + path * 20 + sample];
        for scale in 0..3 {
            assert!(
                at(3, scale * 5 + 1)[0] * at(3, scale * 5 + 3)[0] < 0.,
                "physical wrap straddle row={row} scale={scale}"
            );
        }
        for sample in 0..20 {
            for channel in 0..3 {
                parity +=
                    usize::from((at(2, sample)[channel] - at(4, sample)[channel]).abs() > 0.01);
                if sample >= 15 {
                    peak = peak.max(at(2, sample)[channel]);
                }
            }
        }
        for (index, path) in [0, 1, 2, 4].into_iter().enumerate() {
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
                    failures[index] += 1;
                }
                previous = across;
            }
        }
    }
    assert!(peak > 0.1);
    println!(
        "CELL_GRID_ZERO legacy={legacy} injected={injected} density_bytes=256 seeds=4 latitudes=4 samples={} failures={failures:?} parity={parity} peak={peak} excluded_samples=0",
        pixels.len()
    );
    failures.iter().sum::<usize>() + parity
}

fn outline_gate(legacy: bool) -> usize {
    assert_eq!(EPU_BOUNDS.matches(CURRENT).count(), 1);
    let bounds = if legacy {
        EPU_BOUNDS.replace(CURRENT, BEFORE)
    } else {
        EPU_BOUNDS.to_owned()
    };
    let pixels = probe_image_bounds(OUTLINE, &bounds, 256, 96);
    assert_eq!(pixels.len(), 256 * 96);
    assert!(pixels.iter().flatten().all(|x| x.is_finite()));
    let mut failures = [0usize; 6];
    let mut peaks = [0.0f32; 6];
    for (index, p) in pixels.iter().enumerate() {
        let variant = index / (256 * 16);
        failures[variant] += usize::from(p[0] > 0.002 || p[1] > 0.002);
        assert_eq!(p[2], 1.0, "opaque source weight");
        peaks[variant] = peaks[variant].max(p[3]);
    }
    assert!(peaks.iter().all(|x| *x > 0.1));
    println!(
        "CELL_ZERO_OUTLINE legacy={legacy} variants=6 samples={} failures={failures:?} peaks={peaks:?} limit=.002",
        pixels.len()
    );
    failures.iter().sum()
}

#[test]
fn cell_grid_zero_width_dynamic_wraps() {
    assert_eq!(wrap_gate(false, false), 0);
}
#[test]
fn cell_grid_zero_width_gate_rejects_on_cut_jump() {
    assert!(wrap_gate(false, true) > 0);
}
#[test]
fn cell_grid_zero_width_archived_wrap_rejects() {
    assert!(wrap_gate(true, false) > 0);
}
#[test]
fn cell_grid_zero_width_has_no_outline_contribution() {
    assert_eq!(outline_gate(false), 0);
}
#[test]
fn cell_grid_zero_width_archived_outline_rejects() {
    assert!(outline_gate(true) > 0);
}

#[test]
fn cell_grid_zero_width_density_codec_constant_and_dynamic() {
    let constants = (0..256u32)
        .map(|b| format!("cell_density({b}u)"))
        .collect::<Vec<_>>()
        .join(",");
    let body = format!(
        r#"
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {{
 let literal=array<f32,256>({constants});let d=cell_density(p.x);let steps=p.x*4u;
 let valid=d>=4. && d<=64. && abs(fract(d)-f32(steps%17u)/17.)<=.00001
   && bitcast<u32>(d)==bitcast<u32>(literal[p.x]);
 textureStore(result,p.xy,vec4f(floor(d),ceil(d),select(0.,1.,valid),d));
}}
"#
    );
    let pixels = probe_image(&body, 256, 1);
    assert_eq!(pixels.len(), 256);
    for (byte, p) in pixels.iter().enumerate() {
        assert_eq!(p[0], (4 + byte * 4 / 17) as f32, "floor byte={byte}");
        assert_eq!(
            p[1],
            (4 + (byte * 4).div_ceil(17)) as f32,
            "ceil byte={byte}"
        );
        assert_eq!(
            p[2], 1.,
            "fraction/domain/exact constant-dynamic byte={byte} value={p:?}"
        );
    }
    println!(
        "CELL_DENSITY_CODEC bytes=256 constant_dynamic_exact=true integer_cells_exact=true fractional_budget=.00001"
    );
}
