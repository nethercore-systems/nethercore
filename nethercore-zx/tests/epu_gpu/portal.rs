use super::*;
fn portal_words(
    v: u32,
    a: u32,
    c: u32,
    d: u32,
    aa: u32,
    ab: u32,
    gain: u32,
    axis: u32,
) -> [u32; 4] {
    [
        (d << 24) | (axis << 8) | (aa << 4) | ab,
        (gain << 24) | (a << 16) | (64 << 8) | c,
        0x4dc870e8,
        (17 << 27) | (7 << 24) | (3 << 21) | ((24 | v) << 16) | 0x2638,
    ]
}
fn paint(legacy: bool) -> usize {
    let body = format!(
        "{}\n{}",
        include_str!("fixtures/portal-paint-vortex/legacy.wgsl"),
        include_str!("fixtures/portal-paint-vortex/paint.wgsl")
    )
    .replace("__LEGACY__", &legacy.to_string());
    let mut inputs = Vec::new();
    for axis in [0x8080, 0xff80] {
        for v in 0..8 {
            for a in [64, 200] {
                for aa in [0, 5, 15] {
                    for ab in [0, 7, 15] {
                        for gain in [0, 128, 255] {
                            for d in [0, 64] {
                                inputs.push(portal_words(
                                    v,
                                    a,
                                    if v == 3 { 0 } else { 192 },
                                    d,
                                    aa,
                                    ab,
                                    gain,
                                    axis,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    let rows = probe_image_input_bounds(
        &body,
        EPU_BOUNDS,
        64,
        inputs.len() as u32,
        bytemuck::cast_slice(&inputs),
    );
    assert_eq!(rows.len(), 64 * inputs.len());
    let mut failed = 0;
    let mut max = 0.0f32;
    for (row_index, row) in rows.iter().enumerate() {
        assert!(row.iter().all(|x| x.is_finite()) && row[2] == 0.0 && row[3] == 1.0);
        assert_eq!(
            row[1],
            0.0,
            "geometry/coverage changed row={row_index} input={:?} {row:?}",
            inputs[row_index / 64]
        );
        max = max.max(row[0]);
        if row[0] > 0.002 {
            failed += 1;
        }
    }
    println!(
        "PORTAL_PAINT legacy={legacy} records={} failures={failed} max_error={max} limit=.002 weights_exact=true",
        rows.len()
    );
    failed
}
#[test]
fn portal_paint_is_linear_in_component_alpha_and_rejects_old_double_coverage() {
    assert_eq!(paint(false), 0);
    assert!(paint(true) > 100);
}
fn motion(legacy: bool) -> (usize, usize) {
    let mut body = include_str!("fixtures/portal-paint-vortex/motion.wgsl").to_owned();
    if legacy {
        let source =
            include_str!("../../shaders/epu/features/09_portal.wgsl").replace("\r\n", "\n");
        let entry = source.split_once("fn eval_portal(").unwrap().1;
        let mutated = format!("fn legacy_vortex_eval({entry}").replace(
            "case PORTAL_VARIANT_VORTEX: {\n            sdf = portal_sdf_tear(warped_uv, size, roughness);\n        }",
            "case PORTAL_VARIANT_VORTEX: {\n            sdf = length(warped_uv) - size;\n        }",
        );
        assert!(mutated.contains("sdf = length(warped_uv) - size;"));
        body = format!(
            "{mutated}\n{}",
            body.replace("eval_portal(", "legacy_vortex_eval(")
        );
    }
    let mut inputs = Vec::new();
    for axis in [0x8080, 0xff80, 0x80ff] {
        for a in [64, 128, 240] {
            for c in [0, 64, 192, 255] {
                for d in [0, 1, 64, 128, 192, 255] {
                    inputs.push(portal_words(3, a, c, d, 15, 7, 128, axis));
                }
            }
        }
    }
    let rows = probe_image_input_bounds(
        &body,
        EPU_BOUNDS,
        128,
        inputs.len() as u32,
        bytemuck::cast_slice(&inputs),
    );
    assert_eq!(rows.len(), 128 * inputs.len());
    let mut failed = 0;
    let mut changed = 0;
    let mut max = 0.0f32;
    for row in &rows {
        assert!(
            row.iter().all(|x| x.is_finite()) && row[2] == 0.0 && row[3] == 1.0,
            "bad input/phase endpoint/zero-roughness control {row:?}"
        );
        max = max.max(row[0]);
        if row[0] > 0.002 {
            failed += 1;
        }
        if row[1] > 0.5 {
            changed += 1;
        }
    }
    println!(
        "PORTAL_VORTEX legacy={legacy} records={} failures={failed} moving_samples={changed} max_error={max} limit=.002",
        rows.len()
    );
    (failed, changed)
}
#[test]
fn portal_vortex_reuses_rough_shape_and_has_real_cyclic_phase() {
    let (errors, moving) = motion(false);
    assert_eq!(errors, 0);
    assert!(moving > 100);
    let (errors, _) = motion(true);
    assert!(errors > 100);
}
