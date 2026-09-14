//! RADIAL polar geometry, pole coverage, and retained historical ownership rays.
use super::*;
const CURRENT: &str = include_str!("../../shaders/epu/bounds/04_cell.wgsl");
const LEGACY: &str = include_str!("fixtures/cell-radial/legacy.wgsl");
const REFERENCE: &str = include_str!("fixtures/cell-radial/reference.wgsl");
const ORACLE: &str = include_str!("fixtures/cell-radial/oracle.wgsl");
const POLES: &str = include_str!("fixtures/cell-radial/poles.wgsl");
const SUPPORT: &str = include_str!("fixtures/cell-radial/support.wgsl");
const FINITE_SLOPE: &str = include_str!("fixtures/cell-radial/finite-slope.wgsl");

fn locator() -> String {
    let mut source = LEGACY.to_string();
    let names: Vec<_> = LEGACY
        .lines()
        .filter_map(|l| l.strip_prefix("fn "))
        .map(|l| l.split('(').next().unwrap())
        .collect();
    for name in names {
        source = source.replace(&format!("{name}("), &format!("radial_before_{name}("));
    }
    source
}
fn assembled(cell: &str) -> String {
    assert_eq!(EPU_BOUNDS.matches(CURRENT).count(), 1);
    EPU_BOUNDS.replacen(CURRENT, cell, 1)
}
fn oracle(bounds: &str) -> usize {
    let values = probe_image_bounds(ORACLE, &assembled(bounds), 64, 1536);
    let mut failures = 0;
    let mut positive = 0;
    let mut worst = [0.0f32; 3];
    for p in &values {
        assert!(p.iter().all(|v| v.is_finite()));
        positive += usize::from(p[3] > 0.1);
        failures += usize::from(p[0] > 0.005 || p[1] > 0.005 || p[2] > 0.01);
        for c in 0..3 {
            worst[c] = worst[c].max(p[c]);
        }
    }
    assert_eq!(values.len(), 64 * 1536);
    assert!(positive > 100);
    println!(
        "RADIAL_POLAR_ORACLE samples={} failures={failures} positive={positive} worst={worst:?} budgets=[0.005,0.005,0.01]",
        values.len()
    );
    failures
}
fn poles(bounds: &str) -> usize {
    let values = probe_image_bounds(POLES, &assembled(bounds), 480, 6144);
    let mut failures = 0;
    let mut worst = [0.0f32; 2];
    let mut smallest = f32::MAX;
    for row in 0..6144 {
        for path in 0..2 {
            let start = row * 480 + path * 160;
            let reference = values[start];
            let mut error = 0.0f32;
            for level in 0..5 {
                for angle in 0..32 {
                    let p = values[start + level * 32 + angle];
                    assert!(p.iter().all(|v| v.is_finite()));
                    for c in 0..3 {
                        error = error.max((p[c] - reference[c]).abs());
                    }
                }
            }
            worst[path] = worst[path].max(error);
            failures += usize::from(error > [0.01, 0.005][path]);
        }
        for angle in 0..32 {
            let start = row * 480 + 320;
            let mut previous = values[start + 32 + angle][0];
            for level in 2..5 {
                let distance = values[start + level * 32 + angle][0];
                assert!(
                    distance < previous * 0.2,
                    "pole directions must genuinely shrink"
                );
                previous = distance;
            }
            smallest = smallest.min(previous);
        }
    }
    assert_eq!(values.len(), 480 * 6144);
    assert!(smallest > 1., "nonzero physical pole samples required");
    println!(
        "RADIAL_POLES samples={} failures={failures} worst={worst:?} smallest_scaled_direction={smallest} budgets=[0.01,0.005]",
        values.len()
    );
    failures
}
fn support(bounds: &str, injected: bool) -> usize {
    let mut body = format!("{}\n{SUPPORT}", locator());
    if injected {
        body = body.replace("// INJECT_ON_CUT_JUMP", "if scale==3u {paint[2].x+=0.125;}");
    }
    let values = probe_image_bounds(&body, &assembled(bounds), 512, 864);
    let mut total = 0;
    for part in 0..2 {
        let mut observed = 0;
        let mut bad = 0;
        let mut ambiguous = 0;
        let mut worst = [0.0f32; 3];
        for p in &values[part * 512 * 432..(part + 1) * 512 * 432] {
            assert!(p.iter().all(|v| v.is_finite()));
            if p[3] == 0. {
                continue;
            }
            observed += 1;
            ambiguous += usize::from(p[3] < 0.);
            bad += usize::from(p[0] > 0.01 || p[1] > 0.005 || p[2] != 0.);
            for c in 0..3 {
                worst[c] = worst[c].max(p[c]);
            }
        }
        assert!(observed > 1000);
        total += bad;
        println!(
            "RADIAL_HISTORICAL locator={part} observed={observed} ambiguous_measured={ambiguous} failures={bad} worst={worst:?} budgets=[0.01,0.005,0]"
        );
    }
    total
}
#[test]
fn cell_radial_live_polar_geometry() {
    assert_eq!(oracle(CURRENT), 0);
    assert!(oracle(LEGACY) > 0, "frozen noisy geometry must reject");
    let start = CURRENT
        .find("fn cell_radial_fields(")
        .expect("regular RADIAL fields");
    let end = start + CURRENT[start..].find("\nfn cell_radial(").unwrap();
    let body = &CURRENT[start..end];
    let old = "return vec4f(owner, all_edge, solid_edge);";
    assert_eq!(body.matches(old).count(), 1);
    let phantom = format!(
        "{}{}{}",
        &CURRENT[..start],
        body.replace(
            old,
            "return vec4f(owner,max(all_edge,0.1),max(solid_edge,0.1));"
        ),
        &CURRENT[end..]
    );
    assert!(oracle(&phantom) > 0, "phantom interior edge must reject");
}
#[test]
fn cell_radial_live_poles() {
    assert_eq!(poles(CURRENT), 0);
    assert!(
        poles(LEGACY) > 0,
        "frozen angular pole dependence must reject"
    );
}
#[test]
fn cell_radial_live_historical_support() {
    assert_eq!(support(CURRENT, false), 0);
    assert!(support(LEGACY, false) > 0, "frozen owner jumps must reject");
    assert!(
        support(CURRENT, true) > 0,
        "on-cut-only discontinuity must reject"
    );
    // Keep all original larger-step witnesses. Their finite slope is checked
    // against independent geometry, not excused by a looser colour budget.
    let body = format!("{}\n{REFERENCE}\n{FINITE_SLOPE}", locator());
    let values = probe_image_bounds(&body, &assembled(CURRENT), 512, 432);
    let mut measured = 0;
    let mut worst = 0.0f32;
    for p in values {
        assert!(p.iter().all(|v| v.is_finite()));
        if p[3] == 0. {
            continue;
        }
        measured += 1;
        worst = worst.max(p[2]);
        assert!(
            p[2] <= 0.002,
            "original finite-step witness disagrees with polar oracle: {p:?}"
        );
    }
    assert_eq!(measured, 168);
    println!(
        "RADIAL_ORIGINAL_FINITE_SLOPE measured={measured} max_oracle_error={worst} tolerance=0.002"
    );
}

#[test]
fn cell_radial_live_full_primitive_and_siblings() {
    let helpers = SUPPORT.split("@group(0)").next().unwrap();
    let body = format!(
        "{}\n{}\n{}\n{}\n{}",
        locator(),
        helpers,
        REFERENCE,
        include_str!("fixtures/cell-warped/reference.wgsl"),
        include_str!("fixtures/cell-radial/siblings.wgsl")
    );
    let pixels = probe_image_bounds(&body, &assembled(CURRENT), 256, 1152);
    for variant in 0..8usize {
        let samples = &pixels[variant * 144 * 256..(variant + 1) * 144 * 256];
        let bad = samples
            .iter()
            .filter(|p| {
                p.iter().any(|v| !v.is_finite()) || p[0] != 0. || p[1] > 0.002 || p[2] != 0.
            })
            .count();
        println!(
            "RADIAL_SIBLINGS variant={variant} samples={} failures={bad}; full primitive oracle for RADIAL, exact components elsewhere",
            samples.len()
        );
        assert_eq!(bad, 0);
    }
    // The removed selector-6 fallback is a negative, not silently discarded evidence.
    let wrong_body = body.replace(
        "// OLD_SELECTOR_SIX_FALLBACK_CONTROL",
        "if variant==6u {old=radial_before_eval_cell(dir,i,RegionWeights(.2,.3,.5));}",
    );
    let wrong = probe_image_bounds(&wrong_body, &assembled(CURRENT), 256, 1152);
    let rejected = wrong[6 * 144 * 256..7 * 144 * 256]
        .iter()
        .filter(|p| p[0] != 0. || p[1] > 0.002 || p[2] != 0.)
        .count();
    println!("RADIAL_SIBLINGS_OLD_SIX_FALLBACK rejected={rejected}");
    assert!(
        rejected > 0,
        "the former GRID fallback must not qualify WARPED_RADIAL"
    );
}
