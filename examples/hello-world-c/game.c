/**
 * Hello World - Emberware ZX (C Version)
 *
 * A simple game that draws a colored square and responds to input.
 * Demonstrates the core concepts of Emberware game development in C.
 *
 * Build with wasi-sdk:
 *   clang --target=wasm32-wasi -O2 -Wl,--no-entry \
 *         -Wl,--export=init -Wl,--export=update -Wl,--export=render \
 *         -Wl,--allow-undefined -o game.wasm game.c
 */

#include "emberware_zx.h"

/* Game state - stored in static variables for rollback safety */
static float square_y = 200.0f;

EWZX_EXPORT void init(void) {
    /* Set the background color (dark blue) */
    set_clear_color(0x1a1a2eFF);
}

EWZX_EXPORT void update(void) {
    /* Move square with D-pad */
    if (button_pressed(0, EWZX_BUTTON_UP)) {
        square_y -= 10.0f;
    }
    if (button_pressed(0, EWZX_BUTTON_DOWN)) {
        square_y += 10.0f;
    }

    /* Reset position with A button */
    if (button_pressed(0, EWZX_BUTTON_A)) {
        square_y = 200.0f;
    }

    /* Keep square on screen */
    square_y = ewzx_clampf(square_y, 20.0f, 450.0f);
}

EWZX_EXPORT void render(void) {
    /* Draw title text */
    EWZX_DRAW_TEXT("Hello Emberware!", 80.0f, 50.0f, 32.0f, EWZX_WHITE);

    /* Draw the moving square */
    draw_rect(200.0f, square_y, 80.0f, 80.0f, 0xFF6B6BFF);

    /* Draw instructions */
    EWZX_DRAW_TEXT("D-pad: Move   A: Reset", 60.0f, 500.0f, 18.0f, 0x888888FF);
}
