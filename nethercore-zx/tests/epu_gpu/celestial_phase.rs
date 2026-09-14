use super::{EPU_BOUNDS, probe_image_input_bounds};
const LIMIT: f32 = 0.002;
const BODY: &str = include_str!("fixtures/celestial-phase/probe.wgsl");
const LEGACY: &str = include_str!("fixtures/celestial-phase/legacy-entry.wgsl");
fn measured(legacy: bool, wrong_input: bool) -> [[usize; 3]; 2] {
    let phases = [0u32, 32, 64, 96, 128, 160, 192, 255];
    let sizes = [0u32, 16, 64, 160, 255];
    let axes = [0x8080u32, 0xff80, 0xe0a0];
    let inputs: Vec<u32> = (0u32..240)
        .flat_map(|y| {
            let v = (y / 120) * 2;
            let phase = phases[(y % 8) as usize] ^ if wrong_input { 128 } else { 0 };
            [
                (127 << 24) | (axes[((y / 40) % 3) as usize] << 8) | 240,
                (128 << 24) | (sizes[((y / 8) % 5) as usize] << 16) | (127 << 8) | phase,
                0xaa8c5a28,
                (16 << 27) | (7 << 24) | (3 << 21) | (v << 16) | 0x466e,
            ]
        })
        .collect();
    let body = if legacy {
        format!(
            "{LEGACY}\n{}",
            BODY.replace("eval_celestial(dir,i,1.0)", "legacy_phase_entry(dir,i,1.0)")
        )
    } else {
        BODY.to_owned()
    };
    let rows = probe_image_input_bounds(&body, EPU_BOUNDS, 64, 240, &inputs);
    assert_eq!(rows.len(), 15360);
    let mut failures = [[0usize; 3]; 2];
    let mut maxima = [0.0f32; 3];
    for r in &rows {
        assert!(r.iter().all(|x| x.is_finite()));
        assert!(r[3] == 0.0 || r[3] == 2.0);
        let v = (r[3] as usize) / 2;
        for c in 0..3 {
            maxima[c] = maxima[c].max(r[c]);
            failures[v][c] += usize::from(r[c] > LIMIT);
        }
    }
    println!(
        "CELESTIAL_PHASE legacy={legacy} wrong_input={wrong_input} records={} failures={failures:?} maxima={maxima:?} limit={LIMIT}",
        rows.len()
    );
    failures
}
#[test]
fn celestial_phase_lights_the_projected_sphere_and_rejects_legacy_and_wrong_inputs() {
    let current = measured(false, false);
    let legacy = measured(true, false);
    let wrong = measured(false, true);
    for f in legacy {
        assert!(f[0] > 0 && f[2] > 0);
        assert_eq!(f[1], 0);
    }
    for f in wrong {
        assert!(f[0] > 0 && f[1] > 0);
    }
    assert_eq!(
        current, [[0; 3]; 2],
        "full/half/new phase must use the projected sphere and an unlit night side"
    );
}
