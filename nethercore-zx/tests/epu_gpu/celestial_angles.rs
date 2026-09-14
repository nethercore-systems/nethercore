use super::probe_image;
const BODY: &str = include_str!("fixtures/celestial-angles/probe.wgsl");
const REFERENCE: &str = include_str!("fixtures/celestial-angles/signed-reference.wgsl");
const LIMIT: f32 = 0.002;
fn count(legacy_reference: bool) -> ([usize; 8], [usize; 8]) {
    let mut reference = REFERENCE.to_owned();
    if legacy_reference {
        let old = "clamp(dot(dir, body_dir), -1.0, 1.0)";
        assert_eq!(reference.matches(old).count(), 3);
        reference = reference.replace(old, "epu_saturate(dot(dir, body_dir))");
        let old = "clamp(dot(dir, secondary_dir), -1.0, 1.0)";
        assert_eq!(reference.matches(old).count(), 1);
        reference = reference.replace(old, "epu_saturate(dot(dir, secondary_dir))");
    }
    let p = probe_image(&(reference + "\n" + BODY), 65, 48);
    assert_eq!(p.len(), 65 * 48);
    let mut front = [0; 8];
    let mut back = [0; 8];
    let mut peaks = [0.0f32; 8];
    for (index, r) in p.iter().enumerate() {
        assert!(r.iter().all(|v| v.is_finite()));
        let v = r[3] as usize;
        assert!(v < 8);
        peaks[v] = peaks[v].max(r[1]);
        if r[0] > LIMIT {
            if index % 65 <= 32 {
                front[v] += 1
            } else {
                back[v] += 1
            }
        }
    }
    for (v, peak) in peaks.iter().enumerate() {
        if v == 7 {
            assert_eq!(*peak, 0.0)
        } else {
            assert!(*peak > 0.01)
        }
    }
    println!(
        "CELESTIAL_ANGLE legacy={legacy_reference} records={} front={front:?} back={back:?} limit={LIMIT}",
        p.len()
    );
    (front, back)
}
#[test]
fn celestial_full_sphere_distance_and_legacy_control() {
    let (front, back) = count(false);
    assert_eq!(front, [0; 8], "front-hemisphere control");
    assert_eq!(back, [0; 8], "signed cosine required beyond 90 degrees");
    let (front, back) = count(true);
    assert_eq!(front, [0; 8]);
    assert_eq!(
        back,
        [0, 78, 0, 0, 96, 0, 96, 0],
        "frozen unsigned-angle defect must be rejected on the same rays"
    );
}
