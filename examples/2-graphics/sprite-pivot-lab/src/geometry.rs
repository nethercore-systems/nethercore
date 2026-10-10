//! Explicit two-link composition; no retained hierarchy or transform stack.
use core::f32::consts::FRAC_1_SQRT_2 as D;

pub fn along(point: (f32, f32), distance: f32, step: u32) -> (f32, f32) {
    // ponytail: eight 45-degree stops keep the math visible; use libm for arbitrary angles.
    let (c, s) = [
        (1.0, 0.0),
        (D, D),
        (0.0, 1.0),
        (-D, D),
        (-1.0, 0.0),
        (-D, -D),
        (0.0, -1.0),
        (D, -D),
    ][step as usize % 8];
    (point.0 + c * distance, point.1 + s * distance)
}

pub fn joints(step: u32, flip: u32) -> ((f32, f32), (f32, f32)) {
    let elbow = along((790.0, 270.0), 80.0, step);
    let tip = along(elbow, 60.0, step + if flip == 0 { 2 } else { 6 });
    (elbow, tip)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clockwise_quarters_and_relative_child() {
        for (step, elbow, tip) in [
            (0, (870.0, 270.0), (870.0, 330.0)),
            (2, (790.0, 350.0), (730.0, 350.0)),
            (4, (710.0, 270.0), (710.0, 210.0)),
            (6, (790.0, 190.0), (850.0, 190.0)),
        ] {
            assert_eq!(joints(step, 0), (elbow, tip));
            assert_eq!(joints(step + 8, 0), (elbow, tip));
        }
        assert_eq!(joints(0, 1), ((870.0, 270.0), (870.0, 210.0)));
        for step in 0..8 {
            let p = along((0.0, 0.0), 1.0, step);
            assert!((p.0 * p.0 + p.1 * p.1 - 1.0).abs() < 0.00001);
        }
    }
}
