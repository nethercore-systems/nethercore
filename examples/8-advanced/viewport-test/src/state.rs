//! Small rollback-covered controls and deterministic viewport rectangles.
#[path = "../../../../include/zx/constants.rs"]
#[allow(dead_code)]
mod constants;
use constants::button;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct State {
    pub quad: u32,
    pub mask: u32,
    pub ink: u32,
    pub paused: u32,
    pub tick: u32,
    pub angles: [f32; 4],
}
impl State {
    pub const RESET: Self = Self {
        quad: 0,
        mask: 0,
        ink: 0,
        paused: 1,
        tick: 0,
        angles: [0.0, 90.0, 180.0, 270.0],
    };
    pub fn step(&mut self, pressed: u32, sticks: [f32; 4]) {
        if pressed & (1 << button::Y) != 0 {
            *self = Self::RESET;
            return;
        }
        if pressed & (1 << button::A) != 0 {
            self.quad ^= 1;
        }
        if pressed & (1 << button::B) != 0 {
            self.mask ^= 1;
        }
        if pressed & (1 << button::L1) != 0 {
            self.ink ^= 1;
        }
        if pressed & (1 << button::X) != 0 {
            self.paused ^= 1;
        }
        if self.paused == 0 {
            self.tick = self.tick.wrapping_add(1);
        }
        for (angle, x) in self.angles.iter_mut().zip(sticks) {
            if x.abs() > 0.1 {
                *angle += x * 2.0;
                if *angle >= 360.0 {
                    *angle -= 360.0;
                }
                if *angle < 0.0 {
                    *angle += 360.0;
                }
            }
        }
    }
    pub fn count(self) -> usize {
        if self.quad == 0 {
            2
        } else {
            4
        }
    }
    pub fn rect(self, player: usize) -> [u32; 4] {
        let h = if self.quad == 0 { 428 } else { 214 };
        [
            (player as u32 % 2) * 480,
            64 + (player as u32 / 2) * h,
            480,
            h,
        ]
    }
}

#[cfg(test)]
#[test]
fn controls_angles_and_layout_partition() {
    let mut s = State::RESET;
    s.step(0, [0.1, -0.1, 0.0, 0.0]);
    assert_eq!(s, State::RESET);
    s.step(0, [-1.0, 0.5, 1.0, 0.0]);
    assert_eq!(s.angles, [358.0, 91.0, 182.0, 270.0]);
    for _ in 0..2 {
        let mut area = 0;
        let mut rows = [0; 540];
        for p in 0..s.count() {
            let [x, y, w, h] = s.rect(p);
            assert!(x + w <= 960 && y + h <= 492 && y >= 64);
            area += w * h;
            for row in &mut rows[y as usize..(y + h) as usize] {
                *row += w;
            }
        }
        assert_eq!(area, 960 * 428);
        assert!(rows[64..492].iter().all(|&w| w == 960));
        s.step(1 << button::A, [0.0; 4]);
    }
    s.step(
        (1 << button::B) | (1 << button::L1) | (1 << button::X),
        [0.0; 4],
    );
    assert_eq!((s.mask, s.ink, s.paused, s.tick), (1, 1, 0, 1));
    s.tick = u32::MAX;
    s.step(0, [0.0; 4]);
    assert_eq!(s.tick, 0);
    s.step((1 << button::Y) | (1 << button::A), [1.0; 4]);
    assert_eq!(s, State::RESET);
}
