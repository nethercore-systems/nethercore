use super::*;
fn failures(legacy: bool) -> usize {
    let body = format!(
        "{}\n{}",
        include_str!("fixtures/decal-facing/legacy.wgsl"),
        include_str!("fixtures/decal-facing/probe.wgsl")
    )
    .replace(
        "ACTUAL_DECAL_FACING",
        if legacy {
            "legacy_facing_decal"
        } else {
            "eval_decal"
        },
    );
    let mut input = Vec::new();
    for axis in [0x8080u32, 0xff80, 0xe0a0] {
        for shape in 0u32..4 {
            for size in [0u32, 128, 255] {
                for alpha in [0u32, 7, 15] {
                    for glow_alpha in [0u32, 7, 15] {
                        let phase = (input.len() as u32 / 4 * 37) & 255;
                        input.extend_from_slice(&[
                            (phase << 24) | (axis << 8) | (alpha << 4) | glow_alpha,
                            (128 << 24)
                                | (((shape << 4) | alpha) << 16)
                                | (size << 8)
                                | (glow_alpha * 17),
                            0xaa8c5a28,
                            (8 << 27) | (7 << 24) | (3 << 21) | 0x466e,
                        ]);
                    }
                }
            }
        }
    }
    let rows = probe_image_input_bounds(&body, EPU_BOUNDS, 64, (input.len() / 4) as u32, &input);
    let mut failed = 0;
    let mut max_error = 0.0f32;
    for row in &rows {
        assert!(row.iter().all(|v| v.is_finite()));
        assert_eq!(row[2], 0.0);
        assert_eq!(row[3], 1.0);
        if row[1] == 1.0 {
            assert_eq!(row[0], 0.0, "front-facing output must remain exact");
        }
        failed += usize::from(row[0] > 0.002);
        max_error = max_error.max(row[0]);
    }
    println!(
        "DECAL_FACING legacy={legacy} records={} failures={failed} max_error={max_error} limit=.002 front_exact=true",
        rows.len()
    );
    failed
}
#[test]
fn decal_front_support_preserved_backside_absent_old_duplicate_rejected() {
    assert_eq!(failures(false), 0);
    assert!(failures(true) > 100);
}
