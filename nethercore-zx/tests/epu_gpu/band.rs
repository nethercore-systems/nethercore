//! BAND depth is independent of cyclic phase; original phase-zero RED rays retained.
use super::*;
const LIMIT: f32 = 0.002;
const PHASE: &str = include_str!("fixtures/band/phase.wgsl");
const DEPTH: &str = include_str!("fixtures/band/depth.wgsl");

fn phase_failures(body: &str) -> usize {
    let rows = probe_image(body, 64, 486);
    assert!(rows.iter().flatten().all(|v| v.is_finite()));
    let mut counts = [0usize; 4];
    let mut failed = [0usize; 4];
    for r in &rows {
        let category = if r[3] == 0.0 {
            0
        } else if r[2] + r[3] >= 256.0 {
            3
        } else if r[2] == 0.0 {
            2
        } else {
            1
        };
        counts[category] += 1;
        failed[category] += usize::from(r[0] > LIMIT);
    }
    assert_eq!(counts, [10368, 14976, 2304, 3456]);
    assert!(rows.iter().any(|r| r[1] > 0.99) && rows.iter().any(|r| r[1] < 0.41));
    println!(
        "BAND_PHASE records={} failed={failed:?} held/ordinary/leaving-zero/wrapping-zero limit={LIMIT}",
        rows.len()
    );
    failed.iter().sum()
}

#[test]
fn band_frozen_cyclic_phase_and_jump_control() {
    assert_eq!(phase_failures(PHASE), 0);
    let broken = PHASE.replace("abs(a-b)", "abs(a-b+select(0.0f,0.125f,phase+step>=256u))");
    assert_ne!(broken, PHASE);
    assert!(
        phase_failures(&broken) > 0,
        "constant wrap jump must reject"
    );
}

#[test]
fn band_all_depths_all_phases_and_ignored_depth_control() {
    let rows = probe_image(DEPTH, 256, 288);
    assert!(rows.iter().flatten().all(|v| v.is_finite()));
    let failed = rows.iter().filter(|r| r[0] > LIMIT || r[1] > LIMIT).count();
    let contrast = rows.iter().map(|r| r[2]).fold(0.0f32, f32::max);
    assert!(
        rows.iter().all(|r| r[3] > 0.99),
        "zero depth must retain the plain ring at every phase"
    );
    println!(
        "BAND_DEPTH records={} failed={failed} max_contrast={contrast} limit={LIMIT}",
        rows.len()
    );
    assert_eq!(failed, 0);
    assert!(contrast > 0.59, "modulation must remain visible");
    let broken = DEPTH.replace("let actual=paint(ray,i);", "let actual=paint(ray,full_i);");
    assert_ne!(broken, DEPTH);
    let bad = probe_image(&broken, 256, 288);
    assert!(
        bad.iter().any(|r| r[0] > LIMIT),
        "ignoring depth must reject"
    );
}
