use super::*;
fn failures(legacy: bool) -> usize {
    let body = format!(
        "{}\n{}",
        include_str!("fixtures/lobe-centre/legacy.wgsl"),
        include_str!("fixtures/lobe-centre/probe.wgsl")
    )
    .replace(
        "ACTUAL_LOBE",
        if legacy {
            "legacy_lobe"
        } else {
            "eval_lobe_radiance"
        },
    );
    let mut input = Vec::new();
    for axis in [0x8080u32, 0xff80, 0xe0a0] {
        for exponent in [0u32, 64, 255] {
            for wave in [0u32, 1, 2, 3, 255] {
                for phase in [0u32, 32, 64, 128, 192, 255] {
                    for alpha in [0u32, 7, 15] {
                        input.extend_from_slice(&[
                            (phase << 24) | (axis << 8) | (alpha << 4) | 13,
                            (128 << 24) | (exponent << 16) | (127 << 8) | wave,
                            0xaa8c5a28,
                            (18 << 27) | (7 << 24) | 0x466e,
                        ]);
                    }
                }
            }
        }
    }
    let height = (input.len() / 4) as u32;
    let rows = probe_image_input_bounds(&body, EPU_BOUNDS, 64, height, &input);
    let mut failures = 0;
    let mut max_error = 0.0f32;
    for row in &rows {
        assert!(row.iter().all(|v| v.is_finite()));
        assert_eq!(row[1], 0.0, "paint interpolation changed");
        assert_eq!(row[2], 0.0, "runtime input mismatch");
        assert_eq!(row[3], 1.0);
        max_error = max_error.max(row[0]);
        failures += usize::from(row[0] > 0.002);
    }
    println!(
        "LOBE_CENTRE legacy={legacy} records={} failures={failures} max_error={max_error} limit=.002 palette_unchanged=true",
        rows.len()
    );
    failures
}
#[test]
fn lobe_centre_bright_profile_preserves_palette_waveforms_and_rejects_old_cap() {
    assert_eq!(failures(false), 0);
    assert!(failures(true) > 100, "old centre suppression must reject");
}
