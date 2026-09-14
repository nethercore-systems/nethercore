use super::probe_image;
const LIMIT: f32 = 0.002;
const SOURCE: &str = include_str!("../../shaders/epu/features/08_celestial.wgsl");
const BODY: &str = include_str!("fixtures/celestial-uv/probe.wgsl");
fn measured(legacy: bool) -> usize {
    let start = SOURCE.find("fn eval_celestial(").unwrap();
    let end = start + SOURCE[start..].find("    // Extract colors").unwrap();
    let mut prefix = SOURCE[start..end].replace("fn eval_celestial(", "fn inspect_celestial_uv(");
    assert_eq!(prefix.matches("celestial_surface_uv(").count(), 1);
    if legacy {
        assert!(prefix.contains("celestial_surface_uv(dir, body_dir, sin(angular_size_rad))"));
        prefix = prefix.replace(
            "celestial_surface_uv(dir, body_dir, sin(angular_size_rad))",
            "celestial_surface_uv(dir, body_dir, r)",
        );
    }
    prefix.push_str("    return LayerSample(vec3f(surface_uv, r), angular_size_rad);\n}\n");
    let rows = probe_image(&(prefix + BODY), 64, 768);
    assert_eq!(rows.len(), 49152);
    let mut failures = [0usize; 4];
    let mut maxima = [0.0f32; 4];
    for (index, r) in rows.iter().enumerate() {
        assert!(r.iter().all(|x| x.is_finite()));
        let bucket = index % 4;
        maxima[bucket] = maxima[bucket].max(r[0]);
        if r[0] > LIMIT {
            failures[bucket] += 1;
        }
    }
    println!(
        "CELESTIAL_UV legacy={legacy} records={} failures={failures:?} maxima={maxima:?} limit={LIMIT}",
        rows.len()
    );
    assert_eq!(failures[0], 0, "centre must remain zero");
    if !legacy {
        for r in rows.iter().skip(3).step_by(4) {
            assert!((r[1] - 1.).abs() <= LIMIT, "limb must span the unit disk");
        }
    }
    failures.iter().sum()
}
#[test]
fn celestial_uv_caller_maps_the_projected_disk_and_rejects_per_pixel_radius() {
    assert_eq!(measured(false), 0);
    assert_eq!(measured(true), 36744);
}
