//! Frozen rejected HEX prototypes: diagnostic falsifiers only, not production algorithms.
use super::cell_hex_2d_high::frozen_probe as probe_image;
fn medium_body() -> String {
    format!(
        "{}\n{}\n{}\n{}",
        include_str!("fixtures/cell-hex-2d/prototype.wgsl"),
        include_str!("fixtures/cell-hex-2d/evaluator.wgsl"),
        include_str!("fixtures/cell-hex-2d/medium-dispatch.wgsl"),
        include_str!("fixtures/cell-hex-2d/medium-controls.wgsl")
    )
}
#[test]
fn cell_hex_2d_medium_small_gap() {
    let p = probe_image(
        &medium_body().replace("select(0u,32u,", "select(0u,1u,"),
        32,
        14,
    );
    assert!(p.iter().flatten().all(|x| x.is_finite()));
    for family in 0..2 {
        for k in [0, 2, 4] {
            let a = &p[(family * 7 + k) * 32..];
            let b = &p[(family * 7 + k + 1) * 32..];
            let jump = (a[11][0] - b[11][0]).abs();
            println!(
                "SMALL_GAP family={family} step={k} old={:?}/{:?} proto={:?}/{:?} jump={jump}",
                a[10], b[10], a[11], b[11]
            );
            assert!(
                jump > 0.9,
                "positive gap below band retains authored hard transition"
            );
            if family == 1 {
                for c in 0..3 {
                    assert!((a[10][c] - a[11][c]).abs() < 0.001);
                    assert!((b[10][c] - b[11][c]).abs() < 0.001);
                }
            }
        }
    }
}

#[test]
fn cell_hex_2d_medium_controls() {
    let body = medium_body();
    let p = probe_image(&body, 32, 49);
    assert!(p.iter().flatten().all(|x| x.is_finite()));
    for (row, r) in p.chunks_exact(32).enumerate() {
        println!("MEDIUM row={row} {r:?}");
    }
    // Unchanged integral path and nonzero interior are hard controls.
    for r in p.chunks_exact(32).skip(28).take(7) {
        for c in 0..4 {
            assert_eq!(r[c * 8], r[c * 8 + 1]);
            assert_eq!(r[c * 8 + 2], r[c * 8 + 3]);
        }
    }
    assert!(p[35 * 32 + 19][2] > 0.9, "all-filled core preserved");
    // Old positive row interior becoming a zero-distance wall is a concrete incompatibility.
    assert!(p[48 * 32 + 16][2] > 0.06);
    assert!(p[48 * 32 + 17][2].abs() < 0.001);
    println!("MEDIUM_CONTROLS_PASS; compatibility remains rejected");
}
