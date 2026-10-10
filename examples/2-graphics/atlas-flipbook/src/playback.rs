//! Small guest-owned clip table: (source pixel X, duration in update ticks).
pub const FRAMES: [(u32, u32); 4] = [(0, 8), (16, 12), (32, 8), (48, 12)];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Playback {
    pub mode: u32, // 0 loop, 1 once, 2 ping-pong
    pub frame: u32,
    pub ticks: u32,
    pub paused: u32,
    pub finished: u32,
    pub forward: u32,
}

impl Playback {
    pub const fn new() -> Self {
        Self {
            mode: 0,
            frame: 0,
            ticks: 0,
            paused: 1,
            finished: 0,
            forward: 1,
        }
    }

    pub fn restart(&mut self) {
        *self = Self {
            mode: self.mode,
            paused: self.paused,
            ..Self::new()
        };
    }

    pub fn step(&mut self) {
        if self.finished != 0 {
            return;
        }
        self.ticks += 1;
        if self.ticks < FRAMES[self.frame as usize].1 {
            return;
        }
        self.ticks = 0;
        let last = (FRAMES.len() - 1) as u32;
        match self.mode {
            0 => self.frame = (self.frame + 1) % FRAMES.len() as u32,
            1 => {
                if self.frame == last {
                    self.finished = 1;
                } else {
                    self.frame += 1;
                }
            }
            _ => {
                if self.forward != 0 {
                    if self.frame == last {
                        self.forward = 0;
                        self.frame -= 1;
                    } else {
                        self.frame += 1;
                    }
                } else if self.frame == 0 {
                    self.forward = 1;
                    self.frame += 1;
                } else {
                    self.frame -= 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_boundaries_modes_and_restart() {
        for mode in 0..3 {
            let mut clip = Playback {
                mode,
                ..Playback::new()
            };
            // This expected visit order is independent of the step implementation.
            let sequence: &[u32] = match mode {
                0 => &[0, 1, 2, 3, 0, 1],
                1 => &[0, 1, 2, 3, 3],
                _ => &[0, 1, 2, 3, 2, 1, 0, 1],
            };
            for pair in sequence.windows(2) {
                assert_eq!(clip.frame, pair[0]);
                let duration = FRAMES[clip.frame as usize].1;
                for tick in 1..duration {
                    clip.step();
                    assert_eq!((clip.frame, clip.ticks), (pair[0], tick));
                }
                clip.step();
                assert_eq!((clip.frame, clip.ticks), (pair[1], 0));
            }
            if mode == 1 {
                assert_eq!(clip.finished, 1);
                let end = clip;
                for _ in 0..100 {
                    clip.step();
                }
                assert_eq!(clip, end);
            }
            clip.restart();
            assert_eq!(
                clip,
                Playback {
                    mode,
                    ..Playback::new()
                }
            );
        }
    }
}
