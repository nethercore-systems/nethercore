//! Column-major matrices, column vectors: mul(a,b) applies b before a.
//! Deliberately local to this lesson; not a scene graph or transform stack.
pub type Matrix = [f32; 16];

pub fn identity() -> Matrix {
    [
        1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
    ]
}

pub fn mul(a: Matrix, b: Matrix) -> Matrix {
    let mut out = [0.; 16];
    for c in 0..4 {
        for r in 0..4 {
            for k in 0..4 {
                out[c * 4 + r] += a[k * 4 + r] * b[c * 4 + k];
            }
        }
    }
    out
}

pub fn translate(x: f32, y: f32, z: f32) -> Matrix {
    let mut m = identity();
    m[12] = x;
    m[13] = y;
    m[14] = z;
    m
}

pub fn scale(x: f32, y: f32, z: f32) -> Matrix {
    let mut m = identity();
    m[0] = x;
    m[5] = y;
    m[10] = z;
    m
}

pub fn rotate(step: u32) -> Matrix {
    // Eight repeatable teaching stops; independent native probes use sin/cos.
    const Q: f32 = core::f32::consts::FRAC_1_SQRT_2;
    let (c, s) = [
        (1., 0.),
        (Q, Q),
        (0., 1.),
        (-Q, Q),
        (-1., 0.),
        (-Q, -Q),
        (0., -1.),
        (Q, -Q),
    ][(step % 8) as usize];
    [c, s, 0., 0., -s, c, 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.]
}

pub fn point(m: Matrix, p: [f32; 3]) -> [f32; 3] {
    core::array::from_fn(|r| m[r] * p[0] + m[4 + r] * p[1] + m[8 + r] * p[2] + m[12 + r])
}

pub fn left(step: u32) -> Matrix {
    // The -7 panel offset is shared presentation, not the illustrated T.
    mul(
        translate(-7., 0., 0.),
        mul(translate(2., 0., 0.), rotate(step)),
    )
}

pub fn right(step: u32) -> Matrix {
    mul(rotate(step), translate(2., 0., 0.))
}

pub fn crane(step: u32, flip: u32) -> (Matrix, Matrix) {
    let parent = mul(translate(7., 0., 0.), rotate(step));
    let child = mul(
        mul(parent, translate(1.5, 0., 0.)),
        rotate(if flip == 0 { 1 } else { 7 }),
    );
    (parent, child)
}

#[test]
fn composition_order_and_relative_child() {
    let p = [1., 2., 3.];
    let t = translate(2., 3., 4.);
    assert_eq!(mul(identity(), t), t);
    assert_eq!(mul(t, identity()), t);
    assert_eq!(point(mul(t, scale(2., 3., 4.)), p), [4., 9., 16.]);
    assert_eq!(point(mul(scale(2., 3., 4.), t), p), [6., 15., 28.]);
    assert_eq!(point(left(2), [0.; 3]), [-5., 0., 0.]);
    assert_eq!(point(right(2), [0.; 3]), [0., 2., 0.]);
    for step in 0..8 {
        assert_eq!(left(step), left(step + 8));
        let (a, b) = crane(step, 0);
        let (_, flipped) = crane(step, 1);
        assert_eq!(point(b, [0.; 3]), point(flipped, [0.; 3]));
        assert_eq!(point(a, [1.5, 0., 0.]), point(b, [0.; 3]));
        let tip = point(b, [1., 0., 0.]);
        let other = point(flipped, [1., 0., 0.]);
        let elbow = point(b, [0.; 3]);
        let distance2: f32 = (0..3).map(|i| (tip[i] - elbow[i]).powi(2)).sum();
        assert!((distance2 - 1.).abs() < 0.00001);
        assert_ne!(tip, other);
    }
}
