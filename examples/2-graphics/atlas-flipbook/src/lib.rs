//! Guest clip metadata + fixed updates + draw_sprite_region, without an animation API.
#![no_std]
#![no_main]
use examples_common::{ffi::*, hud};
mod atlas;
mod playback;
use playback::{Playback, FRAMES};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

static mut CLIP: Playback = Playback::new();
static mut TEXTURE: u32 = 0;
const PIXELS: [u8; 64 * 16 * 4] = atlas::atlas();

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x142031ff);
        TEXTURE = load_texture(64, 16, PIXELS.as_ptr());
        for (name, ptr) in [
            ("mode", core::ptr::addr_of!(CLIP.mode)),
            ("frame", core::ptr::addr_of!(CLIP.frame)),
            ("age", core::ptr::addr_of!(CLIP.ticks)),
            ("paused", core::ptr::addr_of!(CLIP.paused)),
            ("finished", core::ptr::addr_of!(CLIP.finished)),
            ("forward", core::ptr::addr_of!(CLIP.forward)),
        ] {
            debug_watch_u32(name.as_ptr(), name.len() as u32, ptr.cast());
        }
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        let mut clip = CLIP;
        if button_pressed(0, button::Y) != 0 {
            CLIP = Playback::new();
            return;
        }
        if button_pressed(0, button::B) != 0 {
            clip.mode = (clip.mode + 1) % 3;
            clip.restart();
            CLIP = clip;
            return;
        }
        if button_pressed(0, button::L1) != 0 {
            clip.restart();
            CLIP = clip;
            return;
        }
        if button_pressed(0, button::A) != 0 {
            clip.paused ^= 1;
        }
        if clip.paused == 0 || button_pressed(0, button::X) != 0 {
            clip.step();
        }
        CLIP = clip;
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        let clip = CLIP;
        let frame = FRAMES[clip.frame as usize];
        // Shapes, sprites and text use explicit layers so batching cannot hide controls.
        z_index(0);
        set_color(0x202f44ff);
        draw_rect(32.0, 96.0, 896.0, 300.0);
        set_color(0x64829aff);
        draw_rect(336.0, 365.0, 288.0, 2.0);
        set_color(0xffcd66ff);
        draw_rect(260.0 + clip.frame as f32 * 108.0, 408.0, 80.0, 72.0);
        for i in 0..4 {
            set_color(0x202f44ff);
            draw_rect(264.0 + i as f32 * 108.0, 412.0, 72.0, 64.0);
        }
        z_index(1);
        texture_bind(TEXTURE);
        texture_filter(0);
        set_color(0xffffffff);
        draw_sprite_region(
            360.0,
            126.0,
            240.0,
            240.0,
            frame.0 as f32 / 64.0,
            0.0,
            0.25,
            1.0,
        );
        for (i, entry) in FRAMES.iter().enumerate() {
            draw_sprite_region(
                264.0 + i as f32 * 108.0,
                412.0,
                64.0,
                64.0,
                entry.0 as f32 / 64.0,
                0.0,
                0.25,
                1.0,
            );
        }
        z_index(2);
        hud::text(
            32.0,
            22.0,
            24.0,
            0xffffffff,
            format_args!("ATLAS FLIPBOOK / tick-owned playback"),
        );
        hud::text(
            32.0,
            64.0,
            14.0,
            0xffcd66ff,
            format_args!(
                "{} | frame {} | tick {}/{} | {}{}",
                ["LOOP", "ONCE", "PING-PONG"][clip.mode as usize],
                clip.frame,
                clip.ticks,
                frame.1,
                if clip.paused != 0 {
                    "PAUSED"
                } else {
                    "PLAYING"
                },
                if clip.finished != 0 { " / END" } else { "" }
            ),
        );
        for (i, entry) in FRAMES.iter().enumerate() {
            hud::text(
                266.0 + i as f32 * 108.0,
                484.0,
                10.0,
                0xffffffff,
                format_args!("{}: {}t", i, entry.1),
            );
        }
        hud::text(
            52.0,
            114.0,
            12.0,
            0xabc1d6ff,
            format_args!("One atlas. Four poses."),
        );
        hud::text(
            52.0,
            140.0,
            12.0,
            0xabc1d6ff,
            format_args!("Only update advances time."),
        );
        hud::text(
            52.0,
            348.0,
            11.0,
            0xabc1d6ff,
            format_args!(
                "A: {} | X: tick when paused",
                if clip.paused != 0 { "play" } else { "pause" }
            ),
        );
        hud::text(
            32.0,
            515.0,
            10.0,
            0xabc1d6ff,
            format_args!(
                "Pad/key: A/Z toggle | B/X mode | X/C tick | L1/Q rewind | Y/V reset | F3 info"
            ),
        );
    }
}
