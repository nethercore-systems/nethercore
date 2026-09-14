use super::*;
fn failures(legacy: bool) -> usize {
    let body = format!(
        "{}\n{}",
        include_str!("fixtures/decal-paint/legacy.wgsl"),
        include_str!("fixtures/decal-paint/probe.wgsl")
    )
    .replace(
        "ACTUAL_DECAL",
        if legacy { "legacy_decal" } else { "eval_decal" },
    );
    let color_a = 0x6ac8e0u32;
    let input = [
        0x8080u32 << 8,
        (255u32 << 24) | (128u32 << 8) | 64u32,
        ((color_a & 255) << 24) | 0xe07038,
        (8u32 << 27) | (7u32 << 24) | (3u32 << 21) | (color_a >> 8),
    ];
    let rows = probe_image_input_bounds(&body, EPU_BOUNDS, 256, 36, &input);
    let mut failures = 0;
    let mut visible = 0;
    let mut max_error = 0.0f32;
    for p in &rows {
        assert!(p.iter().all(|v| v.is_finite()));
        assert_eq!(p[3], 8.0, "runtime instruction opcode");
        assert_eq!(p[1], 0.0, "paint correction changed coverage weight");
        visible += usize::from(p[2] > 0.002);
        failures += usize::from(p[0] > 0.002);
        max_error = max_error.max(p[0]);
    }
    assert!(visible > 100);
    println!(
        "DECAL_PAINT legacy={legacy} records={} visible={visible} failures={failures} max_error={max_error} limit=0.002 exact_weights=true",
        rows.len()
    );
    failures
}
#[test]
fn decal_straight_paint_keeps_legacy_coverage_and_rejects_squared_alpha() {
    assert_eq!(failures(false), 0);
    assert!(
        failures(true) > 100,
        "restored double-coverage defect must reject"
    );
}
