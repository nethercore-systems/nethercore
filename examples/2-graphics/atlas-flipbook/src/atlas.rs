//! Four original squash/stretch poses; generated at compile time, loaded once.
pub const fn atlas() -> [u8; 64 * 16 * 4] {
    let poses = [(7i32, 5i32, 6i32), (10, 6, 3), (7, 4, 7), (5, 5, 5)];
    let mut pixels = [0; 64 * 16 * 4];
    let mut y = 0;
    while y < 16 {
        let mut x = 0;
        while x < 64 {
            let (cy, rx, ry) = poses[x / 16];
            let lx = (x % 16) as i32;
            let dx = lx - 7;
            let dy = y as i32 - cy;
            if dx * dx * ry * ry + dy * dy * rx * rx <= rx * rx * ry * ry {
                let face = (dy == -1 && (lx == 5 || lx == 9)) || (dy == 2 && lx >= 6 && lx <= 8);
                let color = if face {
                    [20, 40, 65, 255]
                } else if lx < 6 {
                    [150, 240, 255, 255]
                } else {
                    [60, 210, 230, 255]
                };
                let mut c = 0;
                while c < 4 {
                    pixels[(y * 64 + x) * 4 + c] = color[c];
                    c += 1;
                }
            }
            x += 1;
        }
        y += 1;
    }
    pixels
}
