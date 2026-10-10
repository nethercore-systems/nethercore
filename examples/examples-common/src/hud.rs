//! Small allocation-free labels shared by the input, clock and RNG lessons.
//! Not a UI framework: the caller owns layout, color and all state.
use core::fmt::{self, Write};

struct Line {
    bytes: [u8; 128],
    len: usize,
}
impl Line {
    fn new() -> Self {
        Self {
            bytes: [0; 128],
            len: 0,
        }
    }
    fn format(args: fmt::Arguments<'_>) -> Self {
        let mut line = Self::new();
        if line.write_fmt(args).is_err() {
            line = Self::new();
            let _ = line.write_str("Label exceeds 128 bytes");
        }
        line
    }
}
impl Write for Line {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self.len.checked_add(s.len()).ok_or(fmt::Error)?;
        if end > self.bytes.len() {
            return Err(fmt::Error);
        }
        self.bytes[self.len..end].copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub fn text(x: f32, y: f32, size: f32, color: u32, args: fmt::Arguments<'_>) {
    let line = Line::format(args);
    unsafe {
        crate::ffi::set_color(color);
        crate::ffi::draw_text(line.bytes.as_ptr(), line.len as u32, x, y, size);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn log(args: fmt::Arguments<'_>) {
    let line = Line::format(args);
    unsafe {
        crate::ffi::log(line.bytes.as_ptr(), line.len as u32);
    }
}

#[cfg(test)]
#[test]
fn labels_are_bounded_without_splitting_utf8() {
    let line = Line::format(format_args!("tick={} x={:.2}", 123, 0.25));
    assert_eq!(&line.bytes[..line.len], b"tick=123 x=0.25");
    let mut line = Line::new();
    line.write_str(&"x".repeat(127)).unwrap();
    assert!(line.write_str("é").is_err());
    assert_eq!(line.len, 127);
    let overflow = Line::format(format_args!("{}", "x".repeat(129)));
    assert_eq!(&overflow.bytes[..overflow.len], b"Label exceeds 128 bytes");
}
