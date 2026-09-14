use super::{EPU_BOUNDS, probe_image_input_bounds};
use std::collections::HashSet;
const LIMIT: f32 = 0.002;
const SOURCE: &str = include_str!("../../shaders/epu/features/08_celestial.wgsl");
const BODY: &str = include_str!("fixtures/celestial-ring/probe.wgsl");
const LEGACY: &str = include_str!("fixtures/celestial-ring/legacy-ringed.wgsl");
fn measured(legacy: bool) -> (usize, usize, usize) {
    let mut body = BODY.to_owned();
    if legacy {
        // Keep every other production path, including gain/angle/UV repairs.
        let start = SOURCE.find("fn eval_celestial(").unwrap();
        let entry = SOURCE[start..]
            .replace("fn eval_celestial(", "fn legacy_entry(")
            .replace("eval_celestial_ringed(", "legacy_celestial_ringed(");
        body = format!(
            "{LEGACY}\n{entry}\n{}",
            body.replace("eval_celestial(", "legacy_entry(")
        );
    }
    let inputs: Vec<u32> = (0..256u32)
        .flat_map(|tilt| {
            [
                (tilt << 24) | (0x8080 << 8) | 255,
                (128 << 24) | (64 << 16) | (127 << 8),
                0xaa8c5a28,
                0x8704466e,
            ]
        })
        .collect();
    let rows = probe_image_input_bounds(&body, EPU_BOUNDS, 64, 1024, &inputs);
    assert_eq!(rows.len(), 65536);
    assert!(rows.iter().flatten().all(|x| x.is_finite()));
    assert!(
        rows.iter().all(|r| r[2] == 16.0 * 8.0 + 4.0),
        "decoded opcode/variant"
    );
    assert!(
        rows[..256].iter().all(|r| r[0] == 0.),
        "edge-on ring stays off"
    );
    let face = rows[255 * 256..]
        .iter()
        .filter(|r| (r[0] - 1.).abs() > LIMIT)
        .count();
    let masks: HashSet<Vec<bool>> = rows
        .chunks_exact(256)
        .skip(1)
        .map(|r| r.iter().map(|v| v[0] > 0.5).collect())
        .collect();
    let errors = rows.iter().filter(|r| r[3] > LIMIT).count();
    let max_error = rows.iter().map(|r| r[3]).fold(0.0f32, f32::max);
    println!(
        "CELESTIAL_RING legacy={legacy} records={} face_failures={face} masks={} ellipse_failures={errors} max_error={max_error} limit={LIMIT}",
        rows.len(),
        masks.len()
    );
    (face, masks.len(), errors)
}
#[test]
fn celestial_ring_tilt_shapes_the_annulus_and_rejects_cancelled_plane_test() {
    let current = measured(false);
    let old = measured(true);
    assert_eq!(old.0, 232);
    assert_eq!(old.1, 1);
    assert!(old.2 > 0, "legacy shape must reject");
    assert_eq!(current.0, 0, "face-on ring covers every bearing");
    assert!(current.1 > 1, "tilt changes geometry, not only brightness");
    assert_eq!(current.2, 0, "independent ellipse and uploaded input gate");
}
