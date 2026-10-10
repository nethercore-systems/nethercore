//! Tiny guest layout rules; the host supplies the actual font measurement.
pub fn aligned_x(left: f32, width: f32, measured: f32, alignment: u32) -> f32 {
    left + (width - measured)
        * match alignment {
            1 => 0.5,
            2 => 1.0,
            _ => 0.0,
        }
}

pub fn next_line<'a>(
    remaining: &mut &'a str,
    width: f32,
    measure: impl Fn(&str) -> f32,
) -> &'a str {
    let text = remaining.trim_start_matches([' ', '\n']);
    let mut end = 0;
    let mut word_end = None;
    for (start, ch) in text.char_indices() {
        if ch == '\n' {
            break;
        }
        let next = start + ch.len_utf8();
        // A separator may overflow even when the preceding whole word fits.
        if ch == ' ' {
            word_end = Some(start);
        }
        if measure(&text[..next]) > width {
            end = word_end
                .filter(|n| *n > 0)
                .unwrap_or(if end == 0 { next } else { end });
            break;
        }
        end = next;
    }
    *remaining = text[end..].trim_start_matches([' ', '\n']);
    text[..end].trim_end_matches(' ')
}

pub fn fallback(text: &str, output: &mut [u8]) -> usize {
    let mut count = 0;
    for (ch, dest) in text.chars().zip(output.iter_mut()) {
        *dest = if (' '..='~').contains(&ch) {
            ch as u8
        } else {
            b'?'
        };
        count += 1;
    }
    count
}

#[cfg(test)]
#[test]
fn layout_boundaries_and_missing_glyphs() {
    let measure = |s: &str| s.chars().count() as f32;
    let mut text = "  ab cd\nefgh  ";
    assert_eq!(next_line(&mut text, 3.0, measure), "ab");
    assert_eq!(next_line(&mut text, 3.0, measure), "cd");
    assert_eq!(next_line(&mut text, 3.0, measure), "efg");
    assert_eq!(next_line(&mut text, 3.0, measure), "h");
    assert!(text.is_empty());
    let mut boundary = "ab cd ef";
    assert_eq!(next_line(&mut boundary, 5.0, measure), "ab cd");
    assert_eq!(boundary, "ef");
    let mut wide = "ΩB";
    assert_eq!(next_line(&mut wide, 0.0, measure), "Ω");
    assert_eq!(wide, "B");
    assert_eq!(aligned_x(10.0, 90.0, 30.0, 0), 10.0);
    assert_eq!(aligned_x(10.0, 90.0, 30.0, 1), 40.0);
    assert_eq!(aligned_x(10.0, 90.0, 30.0, 2), 70.0);
    let mut output = [0; 8];
    let n = fallback("AΩB\n", &mut output);
    assert_eq!(&output[..n], b"A?B?");
    assert_eq!(fallback("long", &mut output[..2]), 2);
}
