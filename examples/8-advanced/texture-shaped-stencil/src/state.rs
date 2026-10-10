//! Fixed alpha tile and rollback-covered presentation controls.
#[path = "../../../../include/zx/constants.rs"]
#[allow(dead_code)]
mod constants;
use constants::button;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct State {
    pub alpha: u32,
    pub offset: u32,
    pub box_mask: u32,
}
impl State {
    pub const RESET: Self = Self {
        alpha: 0,
        offset: 0,
        box_mask: 0,
    };
    pub fn step(&mut self, pressed: u32) {
        if pressed & (1 << button::Y) != 0 {
            *self = Self::RESET;
            return;
        }
        if pressed & (1 << button::A) != 0 {
            self.alpha = (self.alpha + 1) % 3;
        }
        if pressed & (1 << button::B) != 0 {
            self.offset = (self.offset + 1) % 4;
        }
        if pressed & (1 << button::X) != 0 {
            self.box_mask ^= 1;
        }
    }
    pub fn level(self) -> u32 {
        [15, 8, 0][self.alpha as usize]
    }
}

// A small authored diamond leaf, central window, asymmetric small hole and
// three fractional-alpha edge bands. It is created once, never during update.
pub const fn alpha_at(x: u32, y: u32) -> u8 {
    if (x >= 12 && x < 20 && y >= 12 && y < 20) || (x >= 8 && x < 11 && y >= 14 && y < 17) {
        return 0;
    }
    let d = (2 * x as i32 - 31).abs() + (2 * y as i32 - 31).abs();
    match d {
        0..=20 => 255,
        21..=24 => 192,
        25..=28 => 128,
        29..=30 => 64,
        _ => 0,
    }
}
pub const fn pixels() -> [u8; 4096] {
    let mut result = [255; 4096];
    let mut i = 0;
    while i < 1024 {
        result[i * 4 + 3] = alpha_at((i % 32) as u32, (i / 32) as u32);
        i += 1;
    }
    result
}

#[cfg(test)]
#[test]
fn authored_alpha_bands_holes_and_control_cycles() {
    let tile = pixels();
    for (x, y, alpha) in [
        (0, 0, 0),
        (16, 16, 0),
        (8, 15, 0),
        (23, 15, 255),
        (16, 1, 64),
        (16, 2, 128),
        (16, 4, 192),
        (16, 6, 255),
    ] {
        assert_eq!(tile[(y * 32 + x) * 4 + 3], alpha);
    }
    assert!(tile.chunks_exact(4).all(|p| p[..3] == [255, 255, 255]));
    let mut state = State::RESET;
    for level in [8, 0, 15] {
        state.step(1 << button::A);
        assert_eq!(state.level(), level);
    }
    for phase in [1, 2, 3, 0] {
        state.step(1 << button::B);
        assert_eq!(state.offset, phase);
    }
    assert_eq!(state, State::RESET);
    state.step((1 << button::A) | (1 << button::B) | (1 << button::X));
    assert_eq!(
        state,
        State {
            alpha: 1,
            offset: 1,
            box_mask: 1
        }
    );
    state.step((1 << button::Y) | (1 << button::A) | (1 << button::X));
    assert_eq!(state, State::RESET);
}
