//! HUD rendering for NEON DRIFT

use crate::ffi::*;
use crate::state::*;

pub fn render_hud(player_id: u32, vp_width: u32, vp_height: u32) {
    unsafe {
        font_bind(0);  // Use built-in font for readability
        let car = &CARS[player_id as usize];

        // Position indicator
        let pos_text = match car.race_position {
            1 => b"1ST" as &[u8],
            2 => b"2ND",
            3 => b"3RD",
            _ => b"4TH",
        };
        let pos_colors = [0xFFD700FF, 0xC0C0C0FF, 0xCD7F32FF, COLOR_WHITE];
        let pos_color = pos_colors[(car.race_position - 1).min(3) as usize];
        draw_text(pos_text.as_ptr(), 3, 10.0, 10.0, 28.0, pos_color);

        // Lap counter
        let lap_label = b"LAP";
        let lap_x = vp_width as f32 - 100.0;
        draw_text(lap_label.as_ptr(), 3, lap_x, 10.0, 16.0, 0x888888FF);

        let current_lap = car.current_lap.min(3);
        let lap_str = [b'0' + current_lap as u8, b'/', b'3'];
        draw_text(lap_str.as_ptr(), 3, lap_x, 30.0, 24.0, COLOR_CYAN);

        // Race time
        let mins = (RACE_TIME / 60.0) as u32;
        let secs = (RACE_TIME % 60.0) as u32;
        let centis = ((RACE_TIME * 100.0) as u32) % 100;

        let center_x = (vp_width as f32 / 2.0) - 40.0;
        let time_str = [
            b'0' + ((mins / 10) % 10) as u8,
            b'0' + (mins % 10) as u8,
            b':',
            b'0' + ((secs / 10) % 10) as u8,
            b'0' + (secs % 10) as u8,
            b'.',
            b'0' + ((centis / 10) % 10) as u8,
            b'0' + (centis % 10) as u8,
        ];
        draw_text(time_str.as_ptr(), 8, center_x, 10.0, 20.0, COLOR_WHITE);

        // Speed
        let speed = (car.velocity_forward.abs() * 10.0) as u32;
        let mut speed_str = [0u8; 16];
        let speed_len = format_number(speed, &mut speed_str);
        draw_text(speed_str.as_ptr(), speed_len, 10.0, vp_height as f32 - 40.0, 32.0, COLOR_WHITE);

        let kmh = b"KM/H";
        draw_text(kmh.as_ptr(), 4, 10.0 + (speed_len as f32) * 18.0 + 5.0, vp_height as f32 - 35.0, 14.0, 0x888888FF);

        // Boost meter
        let meter_w = 120.0;
        let meter_h = 12.0;
        let meter_x = 10.0;
        let meter_y = vp_height as f32 - 70.0;

        draw_rect(meter_x, meter_y, meter_w, meter_h, 0x222222FF);

        let fill_w = meter_w * car.boost_meter;
        let boost_color = if car.boost_meter >= BOOST_COST {
            if car.is_boosting { COLOR_CYAN } else { 0x00AAFFFF }
        } else {
            0x0066AAFF
        };
        draw_rect(meter_x, meter_y, fill_w, meter_h, boost_color);

        draw_rect(meter_x, meter_y, meter_w, 2.0, 0x444444FF);
        draw_rect(meter_x, meter_y + meter_h - 2.0, meter_w, 2.0, 0x444444FF);

        let boost_label = b"BOOST";
        draw_text(boost_label.as_ptr(), 5, meter_x + meter_w + 8.0, meter_y - 1.0, 12.0, 0x888888FF);

        if car.is_boosting {
            let boosting = b"BOOSTING!";
            draw_text(boosting.as_ptr(), 9, meter_x, meter_y - 20.0, 16.0, COLOR_CYAN);
        }

        if car.is_drifting {
            let drifting = b"DRIFT!";
            draw_text(drifting.as_ptr(), 6, vp_width as f32 - 80.0, vp_height as f32 - 40.0, 18.0, COLOR_MAGENTA);
        }
    }
}

pub fn format_number(mut num: u32, buf: &mut [u8]) -> u32 {
    if num == 0 {
        buf[0] = b'0';
        return 1;
    }

    let mut len = 0;
    let mut temp = [0u8; 16];
    let mut temp_len = 0;

    while num > 0 {
        temp[temp_len] = b'0' + (num % 10) as u8;
        temp_len += 1;
        num /= 10;
    }

    for i in 0..temp_len {
        buf[len] = temp[temp_len - 1 - i];
        len += 1;
    }

    len as u32
}
