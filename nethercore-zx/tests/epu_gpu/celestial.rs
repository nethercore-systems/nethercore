use super::probe_image;
const LIMIT: f32 = 0.002;
#[test]
fn celestial_single_brightness_gain_and_duplicate_gain_control() {
    let body = include_str!("fixtures/celestial/brightness.wgsl");
    let rows = probe_image(body, 32, 864);
    let counts = |rows: &[[f32; 4]]| {
        let mut bad = [0usize; 8];
        for r in rows {
            assert!(r.iter().all(|x| x.is_finite()));
            assert!(
                r[1] >= 0. && r[1] < 1.,
                "test must remain below RGB clipping"
            );
            if r[0] > LIMIT {
                bad[r[3] as usize] += 1;
            }
        }
        bad
    };
    let failures = counts(&rows);
    println!(
        "CELESTIAL_GAIN records={} failed={:?} limit={LIMIT}",
        rows.len(),
        failures
    );
    assert_eq!(failures, [0; 8]);
    for v in 0..7 {
        assert!(
            rows.iter()
                .filter(|r| r[3] == v as f32)
                .map(|r| r[1])
                .fold(0., f32::max)
                > 0.01,
            "variant {v} must paint"
        );
    }
    assert!(rows.iter().filter(|r| r[3] == 7.).all(|r| r[1] == 0.));
    let old = body
        .replace(
            "emitted(eval_celestial(dir,i,1.))",
            "emitted(duplicate_gain(dir,i))",
        )
        .replace(
            "emitted(eval_celestial(dir,twice,1.))",
            "emitted(duplicate_gain(dir,twice))",
        );
    let old = format!(
        "fn duplicate_gain(dir:vec3f,i:vec4u)->LayerSample {{let s=eval_celestial(dir,i,1.);return LayerSample(s.rgb*(u8_to_01(instr_intensity(i))*2.0),s.w);}}\n{old}"
    );
    let negative = counts(&probe_image(&old, 32, 864));
    println!("CELESTIAL_DUPLICATE_GAIN_REJECT counts={negative:?}");
    assert!(negative[..7].iter().all(|&n| n > 0));
    assert_eq!(negative[7], 0);
}

#[test]
fn celestial_all_intensities_blends_and_zero_disable() {
    let rows = probe_image(include_str!("fixtures/celestial/full-range.wgsl"), 256, 192);
    let mut maxima = [0.0f32; 4];
    let mut failed = 0usize;
    for r in &rows {
        assert!(r.iter().all(|x| x.is_finite()));
        for c in 0..4 {
            maxima[c] = maxima[c].max(r[c]);
        }
        if r.iter().any(|&x| x > LIMIT) {
            failed += 1;
        }
    }
    println!(
        "CELESTIAL_ALL_GAIN records={} failed={failed} RGB/weight/blends/off maxima={maxima:?} limit={LIMIT}",
        rows.len()
    );
    assert_eq!(failed, 0);
}
