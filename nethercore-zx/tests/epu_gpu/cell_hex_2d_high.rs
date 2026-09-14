//! Frozen rejected HEX prototypes: diagnostic falsifiers only, not production algorithms.

// A historical falsifier must not silently inherit repaired geometry or hashing.
pub(super) fn frozen_probe(body: &str, width: u32, height: u32) -> Vec<[f32; 4]> {
    let mut body = body.to_owned();
    for name in [
        "cell_periodic_candidates",
        "shortest_periodic_delta",
        "cell_axis_cylinder_uv",
        "cell_offset_id_x",
        "cell_hash2_vec2",
        "wrap_periodic_x",
        "cell_voronoi",
        "cell_shatter",
        "cell_radial",
        "cell_hash2",
        "cell_brick",
        "cell_grid",
        "eval_cell",
        "cell_hex",
    ] {
        body = body.replace(&format!("{name}("), &format!("frozen_{name}("));
    }
    body = body
        .replace("evaluate_epu_layers(", "frozen_scene(")
        .replace("evaluate_bounds_layer(", "frozen_bounds(");
    let dispatch =
        include_str!("fixtures/cell-hex-2d/high-dispatch.wgsl").replace("high_", "frozen_");
    let alias = "fn frozen_eval(d:vec3f,i:vec4u,a:RegionWeights)->BoundsResult{return frozen_eval_cell(d,i,a);}";
    super::probe_image(
        &format!(
            "{}\n{alias}\n{dispatch}\n{body}",
            include_str!("fixtures/cell-hex-2d/legacy-hex.wgsl")
        ),
        width,
        height,
    )
}
fn frozen_high_body(prototype: &str) -> String {
    format!(
        "{}\n{}\n{}\n{}",
        prototype,
        include_str!("fixtures/cell-hex-2d/high-evaluator.wgsl"),
        include_str!("fixtures/cell-hex-2d/high-dispatch.wgsl"),
        include_str!("fixtures/cell-hex-2d/high-controls.wgsl")
    )
}

#[test]
fn cell_hex_2d_high_controls() {
    let body = frozen_high_body(include_str!("fixtures/cell-hex-2d/high-prototype.wgsl"));
    let p = frozen_probe(&body, 324, 56);
    assert!(p.iter().flatten().all(|x| x.is_finite()));
    for (row, r) in p.chunks_exact(324).enumerate() {
        println!("HIGH_ROW {row} {r:?}");
    }
    for family in [2, 3, 4, 7] {
        for step in 0..7 {
            let r = &p[(family * 7 + step) * 324..];
            for c in 0..27 {
                let a = c * 12;
                assert_eq!(r[a], r[a + 1]);
                assert_eq!(r[a + 2], r[a + 3]);
                for field in [4, 6, 8, 10] {
                    assert_eq!(r[a + field], r[a + field + 1]);
                }
            }
        }
    }
    // Same-side exterior tie is NOT the intentional longitude owner edge.
    for step in [0, 2, 4] {
        let a = &p[(35 + step) * 324..];
        let b = &p[(36 + step) * 324..];
        let c = 3 * 12;
        assert_eq!(&a[c][..2], &b[c][..2]);
        assert_ne!(&a[c + 1][..2], &b[c + 1][..2]);
        assert!(a[c + 1][2] < 0.0 && b[c + 1][2] < 0.0);
        assert!((a[c + 3][0] - b[c + 3][0]).abs() > 0.8);
        for field in [5, 7, 9, 11] {
            assert!((a[c + field][1] - b[c + field][1]).abs() > 0.4);
        }
    }
    println!("HIGH_ALGEBRAIC_OWNER_SWITCH_REJECTED; integral/offcut/ordinary-edge controls PASS");
}

#[test]
fn cell_hex_2d_high_fixed_owner_falsifier() {
    let prototype = include_str!("fixtures/cell-hex-2d/high-prototype.wgsl")
        .replace("return best;", "return vec3f(old.xy,best.z);");
    let body = frozen_high_body(&prototype);
    let p = frozen_probe(&body, 324, 56);
    assert!(p.iter().flatten().all(|x| x.is_finite()));
    for step in [0, 2, 4] {
        let a = &p[(7 + step) * 324..];
        let b = &p[(8 + step) * 324..];
        let c = 12 * 12;
        assert_eq!(&a[c][..2], &a[c + 1][..2]);
        assert_eq!(&b[c][..2], &b[c + 1][..2]);
        assert!((a[c + 1][2] - b[c + 1][2]).abs() < 0.001);
        assert!((a[c + 3][0] - b[c + 3][0]).abs() > 0.99);
        println!(
            "HIGH_FIXED step={step} a={:?} b={:?}",
            &a[c..c + 12],
            &b[c..c + 12]
        );
    }
    println!("HIGH_FIXED_OWNER_REJECTED: converging field still leaves mixed cut jump");
}
