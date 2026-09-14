//! Filled-site edge support: real rays, on-cut and shrinking/same-side controls.
use super::*;
const BODY: &str = include_str!("fixtures/cell-sites/support.wgsl");
const CURRENT: &str = include_str!("../../shaders/epu/bounds/04_cell.wgsl");
const BEFORE: &str = include_str!("fixtures/cell-sites/selected-owner.wgsl");

fn gate_body(legacy: bool, injected: bool, input: &str, height: usize) -> usize {
    assert_eq!(EPU_BOUNDS.matches(CURRENT).count(), 1);
    let bounds = if legacy {
        EPU_BOUNDS.replace(CURRENT, BEFORE)
    } else {
        EPU_BOUNDS.to_owned()
    };
    let body = if injected {
        input.replace("// INJECT_ON_CUT_JUMP", "paint[2].x += .125;")
    } else {
        input.to_owned()
    };
    let pixels = probe_image_bounds(&body, &bounds, 512, height as u32);
    assert_eq!(pixels.len(), 512 * height);
    assert!(pixels.iter().flatten().all(|v| v.is_finite()));
    let mut observed = [0usize; 2];
    let mut mixed = [0usize; 2];
    let mut ambiguous = [0usize; 2];
    let mut bad = [0usize; 2];
    let mut worst = [[0.0f32; 3]; 2];
    let mut first = Vec::new();
    for (index, p) in pixels.iter().enumerate() {
        let variant = index / (height / 2 * 512);
        if p[3] == 0. {
            continue;
        }
        observed[variant] += 1;
        mixed[variant] += usize::from(p[3].abs() == 2.);
        ambiguous[variant] += usize::from(p[3] < 0.);
        for c in 0..3 {
            worst[variant][c] = worst[variant][c].max(p[c]);
        }
        if p[0] > 0.01 || p[1] > 0.005 || p[2] != 0. {
            bad[variant] += 1;
            if first.len() < 12 {
                first.push((index / 512, index % 512, *p));
            }
        }
    }
    assert!(observed.iter().all(|n| *n > 1000));
    assert!(mixed.iter().all(|n| *n > 100));
    println!(
        "CELL_SITE_SUPPORT legacy={legacy} injected={injected} variants=[VORONOI,SHATTER] observed={observed:?} mixed={mixed:?} ambiguous_measured={ambiguous:?} failures={bad:?} worst={worst:?} first={first:?} limits=[.01,.005] excluded_pairs=0"
    );
    bad.iter().sum()
}
#[test]
fn cell_site_support_continuity_and_dispatch() {
    assert_eq!(gate_body(false, false, BODY, 864), 0);
}
#[test]
fn cell_site_support_rejects_selected_owner() {
    assert!(gate_body(true, false, BODY, 864) > 0);
}
#[test]
fn cell_site_support_rejects_on_cut_jump() {
    assert!(gate_body(false, true, BODY, 864) > 0);
}

#[test]
fn cell_site_support_all_seeds() {
    let start = BODY.find("    let row=").unwrap();
    let end = BODY.find("    let i=").unwrap();
    let body = format!(
        "{}{}{}",
        &BODY[..start],
        r#"
    let row=p.y%768u;let variant=array<u32,2>(2u,4u)[p.y/768u];
    let density_byte=array<u32,3>(0u,127u,255u)[row/256u];
    let seed=row%256u;let v=.37;let gap=0u;let alpha=15u;
"#,
        &BODY[end..]
    );
    assert_eq!(gate_body(false, false, &body, 1536), 0);
}

// Site-support-only preservation under the current common density/zero-width
// contracts. The raw BEFORE fixture and its historical falsifier remain intact.
fn matched_site_reference() -> String {
    let mut source = BEFORE.to_owned();
    assert_eq!(
        source
            .matches(r#"let density = mix(4.0, 64.0, u8_to_01(instr_a(instr)));"#)
            .count(),
        1
    );
    source = source.replace(
        r#"let density = mix(4.0, 64.0, u8_to_01(instr_a(instr)));"#,
        r#"let density_steps = instr_a(instr) * 4u;
    let density = 4.0 + f32(density_steps / 17u) + f32(density_steps % 17u) / 17.0;"#,
    );
    assert_eq!(
        source
            .matches(
                r#"    let outline = smoothstep(outline_width, 0.0, outline_dist)
        * outline_alpha
        * outline_brightness
        * outline_taper
        * solid_w;"#
            )
            .count(),
        1
    );
    source = source.replace(
        r#"    let outline = smoothstep(outline_width, 0.0, outline_dist)
        * outline_alpha
        * outline_brightness
        * outline_taper
        * solid_w;"#,
        r#"    // A zero-width outline contributes nothing; never evaluate a zero denominator.
    var outline = 0.0;
    if outline_width > 0.0 {
        outline = smoothstep(outline_width, 0.0, outline_dist)
            * outline_alpha
            * outline_brightness
            * outline_taper
            * solid_w;
    }"#,
    );
    source
}

#[test]
fn cell_site_support_geometry_interiors_and_siblings() {
    let mut frozen = matched_site_reference();
    for line in BEFORE.lines() {
        if let Some(function) = line.strip_prefix("fn ") {
            let name = function.split('(').next().unwrap();
            frozen = frozen.replace(&format!("{name}("), &format!("site_before_{name}("));
        }
    }
    let helper = BODY.split("@group").next().unwrap();
    let body = include_str!("fixtures/cell-sites/preservation.wgsl");
    let radial = include_str!("fixtures/cell-radial/reference.wgsl");
    let pixels = probe_image(&format!("{frozen}\n{helper}\n{radial}\n{body}"), 256, 864);
    assert_eq!(pixels.len(), 256 * 864);
    let mut bad = [0usize; 6];
    let mut interiors = [0usize; 6];
    let mut changes = [0usize; 6];
    let mut worst = [[0.0f32; 3]; 6];
    for (index, p) in pixels.iter().enumerate() {
        assert!(p.iter().all(|v| v.is_finite()));
        let variant = index / (256 * 144);
        for c in 0..3 {
            worst[variant][c] = worst[variant][c].max(p[c]);
        }
        bad[variant] += usize::from(p[0] != 0. || p[1] > 0.002 || p[2] != 0.);
        interiors[variant] += usize::from(p[3] % 2. == 1.);
        changes[variant] += usize::from(p[3] >= 2.);
    }
    println!(
        "CELL_SITE_PRESERVATION samples={} failures={bad:?} worst={worst:?} positive_interiors={interiors:?} changed_support={changes:?} geometry_budget=.00001 paint_budget=.002 siblings_exact=true common_contract=density_and_zero_width",
        pixels.len()
    );
    assert!(interiors.iter().all(|n| *n > 10));
    assert!(changes[2] > 0 && changes[4] > 0);
    assert_eq!(bad, [0; 6]);
}
