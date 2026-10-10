/* Same first-cartridge lesson as Rust/Zig. Build and pack with `nether build`. */
#include "zx.h"

static float square_y = 240.0f;

NCZX_EXPORT void init(void) {
    set_clear_color(0x172334ff);
    debug_watch_f32((const uint8_t *)"square_y", 8, (const uint8_t *)&square_y);
}

NCZX_EXPORT void update(void) {
    if (button_pressed(0, NCZX_BUTTON_UP)) square_y -= 10.0f;
    if (button_pressed(0, NCZX_BUTTON_DOWN)) square_y += 10.0f;
    if (button_pressed(0, NCZX_BUTTON_A)) square_y = 240.0f;
    square_y = nczx_clampf(square_y, 160.0f, 384.0f);
}

NCZX_EXPORT void render(void) {
    NCZX_DRAW_TEXT("Hello Nethercore!", 48, 36, 32, 0xffffffff);
    NCZX_DRAW_TEXT("960 x 540 | top-left origin | x=448 | 64 x 64 | y=160..384", 48, 92, 18, 0xabc1d6ff);
    set_color(0x344c66ff);
    draw_rect(96, 156, 768, 4);
    draw_rect(96, 448, 768, 4);
    set_color(0xff967aff);
    draw_rect(448, square_y, 64, 64);
    const unsigned y = (unsigned)square_y;
    const uint8_t label[] = {'y', '=', '0' + y / 100, '0' + (y / 10) % 10, '0' + y % 10};
    set_color(0xffffffff);
    draw_text(label, sizeof(label), 532, square_y + 22, 18);
    NCZX_DRAW_TEXT("D-pad Up/Down: step 10 pixels   A: reset", 48, 480, 18, 0xffffffff);
    NCZX_DRAW_TEXT("Keyboard: arrows / Z   |   Hold does not repeat.   F3: inspect", 48, 512, 14, 0xabc1d6ff);
}
