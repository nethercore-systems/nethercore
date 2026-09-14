//! Full FLOW lane ownership, not nominal world-axis crossings.
use super::*;

#[test]
fn flow_streaks_lane_boundaries() {
    let flow = include_str!("../../shaders/epu/features/03_flow.wgsl");
    let start = flow.find("fn eval_flow(").unwrap();
    let diagnostic = flow[start..].replacen("fn eval_flow(", "fn flow_lane_coordinate(", 1);
    let needle = "            let lane = floor(across);";
    assert_eq!(diagnostic.matches(needle).count(), 1);
    let diagnostic = diagnostic.replace(
        needle,
        &format!("            return LayerSample(vec3f(across,floor(across),0.0),1.0);\n{needle}"),
    );
    let body = format!(
        "{diagnostic}\n{}",
        include_str!("fixtures/flow-streak-lanes.wgsl")
    );
    let pixels = probe_image(&body, 11, 6144);
    assert_eq!(pixels.len(), 67584);
    let mut failures = 0;
    let mut rejected = 0;
    let mut maximum = 0.0f32;
    let mut side_max = 0.0f32;
    let mut peak = 0.0f32;
    for row in 0..6144 {
        let values = &pixels[row * 11..row * 11 + 11];
        assert!(values.iter().flatten().all(|v| v.is_finite()));
        let lane = ((row / 64) % 32) as f32 - 16.0;
        let location = values[10];
        assert!(
            location[0] < 0.0 && location[1] > 0.0,
            "unbracketed actual lane {row}: {location:?}"
        );
        assert_eq!([location[2], location[3]], [lane - 1.0, lane]);
        if row % 4 != 0 {
            let previous = pixels[(row - 1) * 11 + 10];
            assert!(location[0].abs() < previous[0].abs() && location[1] < previous[1]);
        }
        peak = peak.max(values[..5].iter().map(|v| v[3]).fold(0.0, f32::max));
        if row % 4 != 3 {
            continue;
        }
        let mut error = 0.0f32;
        let mut same = 0.0f32;
        let mut negative = 0.0f32;
        for c in 0..4 {
            error = error
                .max((values[0][c] - values[2][c]).abs())
                .max((values[0][c] - values[1][c]).abs())
                .max((values[2][c] - values[1][c]).abs());
            same = same
                .max((values[0][c] - values[3][c]).abs())
                .max((values[2][c] - values[4][c]).abs());
            negative = negative.max((values[5][c] - values[7][c]).abs());
        }
        maximum = maximum.max(error);
        side_max = side_max.max(same);
        failures += usize::from(error > 0.002);
        rejected += usize::from(negative > 0.002);
    }
    println!(
        "FLOW_STREAK_LANES samples={} pairs=1536 failures={failures} max_error={maximum} same_side={side_max} rejected={rejected} peak={peak} limit=.002",
        pixels.len()
    );
    assert!(peak > 0.05, "blanked positive control");
    assert_eq!(rejected, 1536, "injected jump must reject");
    assert!(side_max <= 0.002, "ordinary same-side gradient budget");
    assert_eq!(failures, 0, "nonconverging streak lane ownership");
}

#[test]
fn flow_streaks_turbulence_lattice() {
    let body = r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let axis=p.y%2u; let knot=i32((p.y/2u)%33u)-16;
    let fraction=array<f32,3>(-0.37,0.31,1.73)[p.y/(2u*33u)];
    let eps=array<f32,4>(0.001,0.0001,0.00001,0.000003)[p.x];
    var q=vec2f(f32(knot),fraction); var delta=vec2f(eps,0.0);
    if axis==1u { q=q.yx; delta=delta.yx; }
    textureStore(result,p.xy,vec4f(value_noise(q-delta),value_noise(q),value_noise(q+delta),value_noise(q+3.0*delta)));
}
"#;
    let pixels = probe_image(body, 4, 198);
    assert!(pixels.iter().flatten().all(|v| v.is_finite()));
    let mut failures = 0;
    let mut maximum = 0.0f32;
    let mut same = 0.0f32;
    for row in 0..198 {
        let p = pixels[row * 4 + 3];
        let error = (p[0] - p[2])
            .abs()
            .max((p[0] - p[1]).abs())
            .max((p[2] - p[1]).abs());
        maximum = maximum.max(error);
        same = same.max((p[2] - p[3]).abs());
        failures += usize::from(error > 0.001);
    }
    println!(
        "FLOW_STREAK_TURBULENCE samples={} boundaries=198 failures={failures} max_error={maximum} same_side={same} limit=.001",
        pixels.len()
    );
    assert_eq!(failures, 0, "2D turbulence corner identity");
}
