//! Controller-edge state, kept in rollback-covered guest memory.
#[path = "../../../../include/zx/constants.rs"]
#[allow(dead_code)]
mod constants;
use constants::button;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub mode: u32,
    pub clear_depth: u32,
    pub paused: u32,
    pub mask: u32,
    pub tick: u32,
    pub rotation: u32, // Half-degrees, always in [0, 720).
}
impl State {
    pub const RESET: Self = Self {
        mode: 0,
        clear_depth: 1,
        paused: 1,
        mask: 0,
        tick: 0,
        rotation: 0,
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
            self.clear_depth ^= 1;
        }
        if pressed & (1 << button::X) != 0 {
            self.paused ^= 1;
        }
        if pressed & (1 << button::L1) != 0 {
            self.mask ^= 1;
        }
        if pressed & (1 << button::LEFT) != 0 {
            self.rotation = (self.rotation + 690) % 720;
        }
        if pressed & (1 << button::RIGHT) != 0 {
            self.rotation = (self.rotation + 30) % 720;
        }
        if self.paused == 0 {
            self.tick = self.tick.wrapping_add(1);
            self.rotation = (self.rotation + 1) % 720;
        }
    }
}

pub fn tint_color(color: u32, tint: u32) -> u32 {
    let channel =
        |shift: u32| (((color >> shift) & 255u32) * ((tint >> shift) & 255u32) / 255) << shift;
    channel(24) | channel(16) | channel(8) | (color & 255)
}

#[cfg(test)]
#[test]
fn edges_animation_wrap_and_reset() {
    assert_eq!(tint_color(0x804020ab, 0xffffffff), 0x804020ab);
    assert_eq!(tint_color(0x804020ab, 0x000000ff), 0x000000ab);
    let mut state = State::RESET;
    state.step(0);
    assert_eq!(state, State::RESET);
    for expected in [1, 2, 3, 0] {
        state.step(1 << button::A);
        assert_eq!(state.mode, expected);
    }
    state.step((1 << button::B) | (1 << button::L1) | (1 << button::LEFT));
    assert_eq!((state.clear_depth, state.mask, state.rotation), (0, 1, 690));
    state.step(1 << button::RIGHT);
    assert_eq!(state.rotation, 0);
    state.step(1 << button::X);
    state.step(0);
    assert_eq!((state.tick, state.rotation), (2, 2));
    state.step(1 << button::X);
    state.step(0);
    assert_eq!((state.tick, state.rotation), (2, 2));
    state.tick = u32::MAX;
    state.step(1 << button::X);
    assert_eq!(state.tick, 0);
    state.step((1 << button::Y) | (1 << button::A) | (1 << button::X));
    assert_eq!(state, State::RESET);
}
