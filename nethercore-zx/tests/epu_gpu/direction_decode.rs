//! Exact byte-lattice folds and complete oct-u16 direction coverage.
use super::probe_image;

const PROBE: &str = r#"
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let encoded=(p.y/3u)*256u+p.x;
 let direction=decode_dir16(encoded);
 // Preserve the actual GPU f32 bits despite the harness's half-float target.
 let bits=bitcast<u32>(direction[p.y%3u]);
 textureStore(result,p.xy,vec4f(f32(bits&255u),f32((bits>>8u)&255u),f32((bits>>16u)&255u),f32(bits>>24u)));
}
"#;

fn failures(body: &str) -> usize {
    let pixels = probe_image(body, 256, 768);
    assert_eq!(pixels.len(), 256 * 768);
    let mut bad = 0;
    let mut max_error = 0.0_f64;
    let mut bad_zeros = 0;
    for encoded in 0..=u16::MAX {
        // Independent double-precision oracle: the public normalized oct chart.
        let u = f64::from(encoded & 255) / 255.0 * 2.0 - 1.0;
        let v = f64::from(encoded >> 8) / 255.0 * 2.0 - 1.0;
        let z = 1.0 - u.abs() - v.abs();
        let mut expected = [u, v, z];
        if z < 0.0 {
            expected[0] = (1.0 - v.abs()) * if u < 0.0 { -1.0 } else { 1.0 };
            expected[1] = (1.0 - u.abs()) * if v < 0.0 { -1.0 } else { 1.0 };
        }
        let len = expected.iter().map(|x| x * x).sum::<f64>().sqrt();
        let mut actual = [0.0_f64; 3];
        let mut invalid = false;
        for component in 0..3 {
            expected[component] /= len;
            let index =
                ((usize::from(encoded) / 256) * 3 + component) * 256 + usize::from(encoded) % 256;
            let bytes = pixels[index];
            assert!(
                bytes
                    .iter()
                    .all(|b| b.is_finite() && *b >= 0.0 && *b <= 255.0 && b.fract() == 0.0)
            );
            let bits = (bytes[0] as u32)
                | ((bytes[1] as u32) << 8)
                | ((bytes[2] as u32) << 16)
                | ((bytes[3] as u32) << 24);
            actual[component] = f64::from(f32::from_bits(bits));
            assert!(actual[component].is_finite());
            let error = (actual[component] - expected[component]).abs();
            max_error = max_error.max(error);
            invalid |= error > 2.5e-7;
            if expected[component] == 0.0 && actual[component] != 0.0 {
                bad_zeros += 1;
                invalid = true;
            }
        }
        invalid |= (actual.iter().map(|x| x * x).sum::<f64>().sqrt() - 1.0).abs() > 2.5e-7;
        bad += usize::from(invalid);
    }
    println!(
        "OCT_U16 count=65536 failed={bad} nonzero_edge_components={bad_zeros} max_component_error={max_error} limit=0.00000025"
    );
    bad
}

#[test]
fn every_packed_axis_matches_the_oct_chart_and_exact_edges() {
    assert_eq!(failures(PROBE), 0);
}

#[test]
fn swapped_direction_bytes_are_rejected() {
    let wrong = PROBE.replace(
        "decode_dir16(encoded)",
        "decode_dir16((encoded>>8u)|((encoded&255u)<<8u))",
    );
    assert!(failures(&wrong) > 0);
}
