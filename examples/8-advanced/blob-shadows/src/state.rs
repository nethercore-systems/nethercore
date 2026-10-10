//! Rollback-safe motion, a fixed receiver and an authored radial alpha texture.
#[path = "../../../../include/zx/constants.rs"]
#[allow(dead_code)]
mod constants;
use constants::button;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct State {
    pub lane: u32,
    pub bad_depth: u32,
    pub write_depth: u32,
    pub shadow: u32,
    pub paused: u32,
    pub tick: u32,
}
impl State {
    pub const RESET: Self = Self {
        lane: 0,
        bad_depth: 0,
        write_depth: 0,
        shadow: 1,
        paused: 1,
        tick: 30,
    };
    pub fn step(&mut self, pressed: u32) {
        if pressed & (1 << button::Y) != 0 {
            *self = Self::RESET;
            return;
        }
        if pressed & (1 << button::A) != 0 {
            self.lane = (self.lane + 1) % 3;
        }
        if pressed & (1 << button::B) != 0 {
            self.bad_depth ^= 1;
        }
        if pressed & (1 << button::R1) != 0 {
            self.write_depth ^= 1;
        }
        if pressed & (1 << button::L1) != 0 {
            self.shadow ^= 1;
        }
        if pressed & (1 << button::X) != 0 {
            self.paused ^= 1;
        }
        if self.paused == 0 {
            self.tick = self.tick.wrapping_add(1);
        }
    }
    pub fn pose(self) -> (f32, f32, f32, u32) {
        let phase = self.tick & 63;
        let bounce = phase.min(64 - phase);
        let height = bounce as f32 / 16.0;
        (
            [-1.0, 4.0, 5.5][self.lane as usize] + bounce as f32 / 32.0,
            height,
            1.0 + height / 4.0,
            12 - bounce / 8,
        )
    }
}
// [x0,x1,z0,z1] of one known receiver; no arbitrary scene/depth query.
pub fn clip_floor([x0, x1, z0, z1]: [f32; 4]) -> Option<[f32; 4]> {
    let r = [x0.max(-5.0), x1.min(5.0), z0.max(-3.0), z1.min(3.0)];
    (r[0] < r[1] && r[2] < r[3]).then_some(r)
}
pub const fn pixels() -> [u8; 4096] {
    let mut result = [255; 4096];
    let mut i = 0;
    while i < 1024 {
        let x = 2 * (i % 32) as i32 - 31;
        let y = 2 * (i / 32) as i32 - 31;
        let d = x * x + y * y;
        result[i * 4 + 3] = if d >= 900 {
            0
        } else {
            ((900 - d) * 255 / 900) as u8
        };
        i += 1;
    }
    result
}

#[cfg(test)]
#[test]
fn radial_texture_motion_controls_and_floor_clipping() {
    let p = pixels();
    assert_eq!(p[3], 0);
    assert_eq!(p[(15 * 32 + 15) * 4 + 3], 254);
    for y in 0..32 {
        for x in 0..32 {
            assert_eq!(p[(y * 32 + x) * 4 + 3], p[((31 - y) * 32 + 31 - x) * 4 + 3]);
        }
    }
    assert_eq!(
        clip_floor([4.5, 6.0, -1.0, 1.0]),
        Some([4.5, 5.0, -1.0, 1.0])
    );
    assert_eq!(clip_floor([5.0, 6.0, -1.0, 1.0]), None);
    let mut s = State::RESET;
    for tick in 0..128 {
        s.tick = tick;
        let (x, h, r, a) = s.pose();
        assert!((-1.0..=0.0).contains(&x));
        assert!((0.0..=2.0).contains(&h));
        assert!((1.0..=1.5).contains(&r));
        assert!((8..=12).contains(&a));
    }
    s.tick = u32::MAX;
    s.step(1 << button::X);
    assert_eq!(s.tick, 0);
    s.step((1 << button::A) | (1 << button::B) | (1 << button::R1) | (1 << button::L1));
    assert_eq!((s.lane, s.bad_depth, s.write_depth, s.shadow), (1, 1, 1, 0));
    s.step((1 << button::Y) | (1 << button::X));
    assert_eq!(s, State::RESET);
}
