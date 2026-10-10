//! A fixed two/three-level stack, not an arbitrary scene-graph allocator.
#[path = "../../../../include/zx/constants.rs"]
#[allow(dead_code)]
mod constants;
use constants::button;

pub const MAX_REFERENCE: u32 = 255; // Zero is reserved for the background.
pub type Rect = [f32; 4];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct State {
    pub deep: u32,
    pub cleanup: u32,
    pub paused: u32,
    pub tick: u32,
}
impl State {
    pub const RESET: Self = Self {
        deep: 1,
        cleanup: 1,
        paused: 1,
        tick: 0,
    };

    pub fn step(&mut self, pressed: u32) {
        if pressed & (1 << button::Y) != 0 {
            *self = Self::RESET;
            return;
        }
        if pressed & (1 << button::A) != 0 {
            self.deep ^= 1;
        }
        if pressed & (1 << button::B) != 0 {
            self.cleanup ^= 1;
        }
        if pressed & (1 << button::X) != 0 {
            self.paused ^= 1;
        }
        if self.paused == 0 {
            self.tick = self.tick.wrapping_add(1);
        }
    }

    pub fn levels(self) -> usize {
        2 + self.deep as usize
    }

    pub fn regions(self, panel: u32) -> [Rect; 4] {
        let offset = if (self.tick.wrapping_add(panel) & 1) == 0 {
            0.0
        } else {
            12.0
        };
        let mut regions = [
            [24.0, 24.0, 268.0, 280.0],
            [72.0, 64.0, 172.0, 196.0],
            [112.0 + offset, 112.0, 80.0, 112.0],
            [328.0 + offset, 104.0, 88.0, 196.0],
        ];
        if panel == 1 {
            for r in &mut regions {
                r[0] = 456.0 - r[0] - r[2];
            }
        }
        regions
    }
}

#[cfg(test)]
#[test]
fn bounded_nested_regions_controls_and_counter_wrap() {
    let mut state = State::RESET;
    for tick in [0, 1, u32::MAX] {
        state.tick = tick;
        for panel in 0..2 {
            let r = state.regions(panel);
            for [x, y, w, h] in r {
                assert!(x > 2.0 && y > 2.0 && x + w < 454.0 && y + h < 350.0);
            }
            for (outer, inner) in [(r[0], r[1]), (r[1], r[2])] {
                assert!(inner[0] > outer[0] && inner[1] > outer[1]);
                assert!(inner[0] + inner[2] < outer[0] + outer[2]);
                assert!(inner[1] + inner[3] < outer[1] + outer[3]);
            }
            assert!(r[0][0] + r[0][2] < r[3][0] || r[3][0] + r[3][2] < r[0][0]);
        }
    }
    assert!(state.levels() as u32 <= MAX_REFERENCE);
    state.step((1 << button::A) | (1 << button::B) | (1 << button::X));
    assert_eq!(
        state,
        State {
            deep: 0,
            cleanup: 0,
            paused: 0,
            tick: 0
        }
    );
    assert_eq!(state.levels(), 2);
    state.step((1 << button::Y) | (1 << button::X));
    assert_eq!(state, State::RESET);
}
