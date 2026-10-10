//! Finite controls and source-authored mask rectangles; all state is rollback-covered.
#[path = "../../../../include/zx/constants.rs"]
#[allow(dead_code)]
mod constants;
use constants::button;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct State {
    pub mode: u32,
    pub layout: u32,
    pub swap: u32,
}
impl State {
    pub const RESET: Self = Self {
        mode: 0,
        layout: 0,
        swap: 0,
    };

    pub fn step(&mut self, pressed: u32) {
        if pressed & (1 << button::Y) != 0 {
            *self = Self::RESET;
            return;
        }
        if pressed & (1 << button::A) != 0 {
            self.mode = (self.mode + 1) % 4;
        }
        if pressed & (1 << button::B) != 0 {
            self.layout = (self.layout + 1) % 3;
        }
        if pressed & (1 << button::X) != 0 {
            self.swap ^= 1;
        }
    }

    pub fn shapes(self) -> [[f32; 4]; 2] {
        let a = [38.0, 42.0, 176.0, 188.0];
        let b = match self.layout {
            0 => [128.0, 130.0, 122.0, 164.0],
            1 => [86.0, 96.0, 80.0, 80.0],
            _ => [225.0, 20.0, 45.0, 75.0],
        };
        if self.swap == 0 {
            [a, b]
        } else {
            [b, a]
        }
    }
}

#[cfg(test)]
#[test]
fn modes_layouts_operand_order_and_reset() {
    let mut s = State::RESET;
    for _ in 0..4 {
        s.step(1 << button::A);
    }
    assert_eq!(s, State::RESET);
    for _ in 0..3 {
        let pair = s.shapes();
        s.step(1 << button::X);
        assert_eq!(s.shapes(), [pair[1], pair[0]]);
        s.step(1 << button::X);
        for [x, y, w, h] in s.shapes() {
            assert!(x > 2.0 && y > 2.0 && x + w < 286.0 && y + h < 318.0);
        }
        s.step(1 << button::B);
    }
    assert_eq!(s, State::RESET);
    s.step((1 << button::A) | (1 << button::B) | (1 << button::X));
    assert_eq!(
        s,
        State {
            mode: 1,
            layout: 1,
            swap: 1
        }
    );
    s.step((1 << button::Y) | (1 << button::A));
    assert_eq!(s, State::RESET);
}
