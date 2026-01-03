# Part 6: Scoring & Win States

Let's add proper scoring, win conditions, and a game state machine.

## What You'll Learn

- Game state machines (Title, Playing, GameOver)
- Tracking and displaying scores
- Win conditions
- Using `button_pressed()` for menu navigation

## Add Game State

Create a state enum and related variables:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[derive(Clone, Copy, PartialEq)]
enum GameState {
    Title,
    Playing,
    GameOver,
}

static mut STATE: GameState = GameState::Title;
static mut SCORE1: u32 = 0;
static mut SCORE2: u32 = 0;
static mut WINNER: u32 = 0;  // 1 or 2

const WIN_SCORE: u32 = 5;
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
typedef enum {
    TITLE,
    PLAYING,
    GAME_OVER
} GameState;

static GameState state = TITLE;
static uint32_t score1 = 0;
static uint32_t score2 = 0;
static uint32_t winner = 0;  // 1 or 2

#define WIN_SCORE 5
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const GameState = enum {
    title,
    playing,
    game_over,
};

var state: GameState = .title;
var score1: u32 = 0;
var score2: u32 = 0;
var winner: u32 = 0;  // 1 or 2

const WIN_SCORE: u32 = 5;
```
{{#endtab}}

{{#endtabs}}

## Add Button Constants

We need the A button for starting/restarting:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
const BUTTON_A: u32 = 4;
```

Add to FFI imports:

```rust
fn button_pressed(player: u32, button: u32) -> u32;
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
#define NCZX_BUTTON_A 4
```

Add to FFI imports:

```c
NCZX_IMPORT uint32_t button_pressed(uint32_t player, uint32_t button);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const Button = struct {
    pub const a: u32 = 4;
};
```

Add to FFI imports:

```zig
pub extern fn button_pressed(player: u32, button: u32) u32;
```
{{#endtab}}

{{#endtabs}}

## Reset Game Function

Create a function to reset the entire game:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn reset_game() {
    unsafe {
        // Reset paddles
        PADDLE1_Y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
        PADDLE2_Y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;

        // Reset scores
        SCORE1 = 0;
        SCORE2 = 0;
        WINNER = 0;

        // Check player count
        IS_TWO_PLAYER = player_count() >= 2;

        // Reset ball
        reset_ball(-1);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void reset_game(void) {
    // Reset paddles
    paddle1_y = SCREEN_HEIGHT / 2.0f - PADDLE_HEIGHT / 2.0f;
    paddle2_y = SCREEN_HEIGHT / 2.0f - PADDLE_HEIGHT / 2.0f;

    // Reset scores
    score1 = 0;
    score2 = 0;
    winner = 0;

    // Check player count
    is_two_player = player_count() >= 2;

    // Reset ball
    reset_ball(-1);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
fn resetGame() void {
    // Reset paddles
    paddle1_y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
    paddle2_y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;

    // Reset scores
    score1 = 0;
    score2 = 0;
    winner = 0;

    // Check player count
    is_two_player = player_count() >= 2;

    // Reset ball
    resetBall(-1);
}
```
{{#endtab}}

{{#endtabs}}

## Update Scoring Logic

Modify the ball update to handle scoring:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn update_ball() {
    unsafe {
        // ... existing movement and collision code ...

        // Ball goes off left side - Player 2 scores
        if BALL_X < -BALL_SIZE {
            SCORE2 += 1;

            if SCORE2 >= WIN_SCORE {
                WINNER = 2;
                STATE = GameState::GameOver;
            } else {
                reset_ball(-1);  // Serve toward player 1
            }
        }

        // Ball goes off right side - Player 1 scores
        if BALL_X > SCREEN_WIDTH {
            SCORE1 += 1;

            if SCORE1 >= WIN_SCORE {
                WINNER = 1;
                STATE = GameState::GameOver;
            } else {
                reset_ball(1);  // Serve toward player 2
            }
        }
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void update_ball(void) {
    // ... existing movement and collision code ...

    // Ball goes off left side - Player 2 scores
    if (ball_x < -BALL_SIZE) {
        score2++;

        if (score2 >= WIN_SCORE) {
            winner = 2;
            state = GAME_OVER;
        } else {
            reset_ball(-1);  // Serve toward player 1
        }
    }

    // Ball goes off right side - Player 1 scores
    if (ball_x > SCREEN_WIDTH) {
        score1++;

        if (score1 >= WIN_SCORE) {
            winner = 1;
            state = GAME_OVER;
        } else {
            reset_ball(1);  // Serve toward player 2
        }
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
fn updateBall() void {
    // ... existing movement and collision code ...

    // Ball goes off left side - Player 2 scores
    if (ball_x < -BALL_SIZE) {
        score2 += 1;

        if (score2 >= WIN_SCORE) {
            winner = 2;
            state = .game_over;
        } else {
            resetBall(-1);  // Serve toward player 1
        }
    }

    // Ball goes off right side - Player 1 scores
    if (ball_x > SCREEN_WIDTH) {
        score1 += 1;

        if (score1 >= WIN_SCORE) {
            winner = 1;
            state = .game_over;
        } else {
            resetBall(1);  // Serve toward player 2
        }
    }
}
```
{{#endtab}}

{{#endtabs}}

## State Machine in Update

Restructure `update()` to handle game states:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Always check player count
        IS_TWO_PLAYER = player_count() >= 2;

        match STATE {
            GameState::Title => {
                // Press A to start
                if button_pressed(0, BUTTON_A) != 0 {
                    reset_game();
                    STATE = GameState::Playing;
                }
            }

            GameState::Playing => {
                // Normal gameplay
                update_paddle(&mut PADDLE1_Y, 0);

                if IS_TWO_PLAYER {
                    update_paddle(&mut PADDLE2_Y, 1);
                } else {
                    update_ai(&mut PADDLE2_Y);
                }

                update_ball();
            }

            GameState::GameOver => {
                // Press A to restart
                if button_pressed(0, BUTTON_A) != 0 || button_pressed(1, BUTTON_A) != 0 {
                    reset_game();
                    STATE = GameState::Playing;
                }
            }
        }
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void update(void) {
    // Always check player count
    is_two_player = player_count() >= 2;

    switch (state) {
        case TITLE:
            // Press A to start
            if (button_pressed(0, NCZX_BUTTON_A) != 0) {
                reset_game();
                state = PLAYING;
            }
            break;

        case PLAYING:
            // Normal gameplay
            update_paddle(&paddle1_y, 0);

            if (is_two_player) {
                update_paddle(&paddle2_y, 1);
            } else {
                update_ai(&paddle2_y);
            }

            update_ball();
            break;

        case GAME_OVER:
            // Press A to restart
            if (button_pressed(0, NCZX_BUTTON_A) != 0 || button_pressed(1, NCZX_BUTTON_A) != 0) {
                reset_game();
                state = PLAYING;
            }
            break;
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    // Always check player count
    is_two_player = player_count() >= 2;

    switch (state) {
        .title => {
            // Press A to start
            if (button_pressed(0, Button.a) != 0) {
                resetGame();
                state = .playing;
            }
        },

        .playing => {
            // Normal gameplay
            updatePaddle(&paddle1_y, 0);

            if (is_two_player) {
                updatePaddle(&paddle2_y, 1);
            } else {
                updateAi(&paddle2_y);
            }

            updateBall();
        },

        .game_over => {
            // Press A to restart
            if (button_pressed(0, Button.a) != 0 or button_pressed(1, Button.a) != 0) {
                resetGame();
                state = .playing;
            }
        },
    }
}
```
{{#endtab}}

{{#endtabs}}

## Update Init

Start on title screen:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eFF);
        reset_game();
        STATE = GameState::Title;
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void init(void) {
    set_clear_color(0x1a1a2eFF);
    reset_game();
    state = TITLE;
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn init() void {
    set_clear_color(0x1a1a2eFF);
    resetGame();
    state = .title;
}
```
{{#endtab}}

{{#endtabs}}

## Render Scores

Add a helper for drawing text:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn draw_text_bytes(text: &[u8], x: f32, y: f32, size: f32) {
    unsafe {
        draw_text(text.as_ptr(), text.len() as u32, x, y, size);
    }
}
```

Add score display in render:

```rust
fn render_scores() {
    unsafe {
        // Convert scores to single digits
        let score1_char = b'0' + (SCORE1 % 10) as u8;
        let score2_char = b'0' + (SCORE2 % 10) as u8;

        // Draw scores
        set_color(COLOR_PLAYER1);
        draw_text(&[score1_char], 1, SCREEN_WIDTH / 4.0, 30.0, 48.0);
        set_color(COLOR_PLAYER2);
        draw_text(&[score2_char], 1, SCREEN_WIDTH * 3.0 / 4.0, 30.0, 48.0);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void draw_text_bytes(const uint8_t* text, uint32_t len, float x, float y, float size) {
    draw_text(text, len, x, y, size);
}
```

Add score display in render:

```c
void render_scores(void) {
    // Convert scores to single digits
    uint8_t score1_char = '0' + (score1 % 10);
    uint8_t score2_char = '0' + (score2 % 10);

    // Draw scores
    set_color(COLOR_PLAYER1);
    draw_text(&score1_char, 1, SCREEN_WIDTH / 4.0f, 30.0f, 48.0f);
    set_color(COLOR_PLAYER2);
    draw_text(&score2_char, 1, SCREEN_WIDTH * 3.0f / 4.0f, 30.0f, 48.0f);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
fn drawTextBytes(text: []const u8, x: f32, y: f32, size: f32) void {
    draw_text(text.ptr, @intCast(text.len), x, y, size);
}
```

Add score display in render:

```zig
fn renderScores() void {
    // Convert scores to single digits
    const score1_char = '0' + @as(u8, @intCast(score1 % 10));
    const score2_char = '0' + @as(u8, @intCast(score2 % 10));

    // Draw scores
    set_color(COLOR_PLAYER1);
    draw_text(&score1_char, 1, SCREEN_WIDTH / 4.0, 30.0, 48.0);
    set_color(COLOR_PLAYER2);
    draw_text(&score2_char, 1, SCREEN_WIDTH * 3.0 / 4.0, 30.0, 48.0);
}
```
{{#endtab}}

{{#endtabs}}

## Render States

Update `render()` to show different screens:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        match STATE {
            GameState::Title => {
                render_court();
                render_title();
            }

            GameState::Playing => {
                render_court();
                render_scores();
                render_paddles();
                render_ball();
                render_mode_indicator();
            }

            GameState::GameOver => {
                render_court();
                render_scores();
                render_paddles();
                render_ball();
                render_game_over();
            }
        }
    }
}

fn render_title() {
    unsafe {
        set_color(COLOR_WHITE);
        draw_text_bytes(b"PADDLE", SCREEN_WIDTH / 2.0 - 100.0, 150.0, 64.0);

        if IS_TWO_PLAYER {
            draw_text_bytes(b"2 PLAYER MODE", SCREEN_WIDTH / 2.0 - 100.0, 250.0, 24.0);
        } else {
            draw_text_bytes(b"1 PLAYER VS AI", SCREEN_WIDTH / 2.0 - 100.0, 250.0, 24.0);
        }

        set_color(COLOR_GRAY);
        draw_text_bytes(b"Press A to Start", SCREEN_WIDTH / 2.0 - 120.0, 350.0, 24.0);
    }
}

fn render_game_over() {
    unsafe {
        // Dark overlay
        set_color(0x000000CC);
        draw_rect(SCREEN_WIDTH / 4.0, SCREEN_HEIGHT / 3.0,
                  SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 3.0);

        // Winner text
        let (text, color) = if WINNER == 1 {
            (b"PLAYER 1 WINS!" as &[u8], COLOR_PLAYER1)
        } else if IS_TWO_PLAYER {
            (b"PLAYER 2 WINS!" as &[u8], COLOR_PLAYER2)
        } else {
            (b"AI WINS!" as &[u8], COLOR_PLAYER2)
        };

        set_color(color);
        draw_text(text.as_ptr(), text.len() as u32,
                  SCREEN_WIDTH / 2.0 - 120.0, SCREEN_HEIGHT / 2.0 - 20.0, 32.0);

        set_color(COLOR_GRAY);
        draw_text_bytes(b"Press A to Play Again",
                       SCREEN_WIDTH / 2.0 - 140.0, SCREEN_HEIGHT / 2.0 + 30.0, 20.0);
    }
}

fn render_mode_indicator() {
    unsafe {
        set_color(COLOR_GRAY);
        if IS_TWO_PLAYER {
            draw_text_bytes(b"2P", 10.0, 10.0, 16.0);
        } else {
            draw_text_bytes(b"vs AI", 10.0, 10.0, 16.0);
        }
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
    switch (state) {
        case TITLE:
            render_court();
            render_title();
            break;

        case PLAYING:
            render_court();
            render_scores();
            render_paddles();
            render_ball();
            render_mode_indicator();
            break;

        case GAME_OVER:
            render_court();
            render_scores();
            render_paddles();
            render_ball();
            render_game_over();
            break;
    }
}

void render_title(void) {
    set_color(COLOR_WHITE);
    draw_text_bytes((const uint8_t*)"PADDLE", 6, SCREEN_WIDTH / 2.0f - 100.0f, 150.0f, 64.0f);

    if (is_two_player) {
        draw_text_bytes((const uint8_t*)"2 PLAYER MODE", 13, SCREEN_WIDTH / 2.0f - 100.0f, 250.0f, 24.0f);
    } else {
        draw_text_bytes((const uint8_t*)"1 PLAYER VS AI", 14, SCREEN_WIDTH / 2.0f - 100.0f, 250.0f, 24.0f);
    }

    set_color(COLOR_GRAY);
    draw_text_bytes((const uint8_t*)"Press A to Start", 16, SCREEN_WIDTH / 2.0f - 120.0f, 350.0f, 24.0f);
}

void render_game_over(void) {
    // Dark overlay
    set_color(0x000000CC);
    draw_rect(SCREEN_WIDTH / 4.0f, SCREEN_HEIGHT / 3.0f,
              SCREEN_WIDTH / 2.0f, SCREEN_HEIGHT / 3.0f);

    // Winner text
    const uint8_t* text;
    uint32_t text_len;
    uint32_t color;

    if (winner == 1) {
        text = (const uint8_t*)"PLAYER 1 WINS!";
        text_len = 14;
        color = COLOR_PLAYER1;
    } else if (is_two_player) {
        text = (const uint8_t*)"PLAYER 2 WINS!";
        text_len = 14;
        color = COLOR_PLAYER2;
    } else {
        text = (const uint8_t*)"AI WINS!";
        text_len = 8;
        color = COLOR_PLAYER2;
    }

    set_color(color);
    draw_text(text, text_len,
              SCREEN_WIDTH / 2.0f - 120.0f, SCREEN_HEIGHT / 2.0f - 20.0f, 32.0f);

    set_color(COLOR_GRAY);
    draw_text_bytes((const uint8_t*)"Press A to Play Again", 21,
                   SCREEN_WIDTH / 2.0f - 140.0f, SCREEN_HEIGHT / 2.0f + 30.0f, 20.0f);
}

void render_mode_indicator(void) {
    set_color(COLOR_GRAY);
    if (is_two_player) {
        draw_text_bytes((const uint8_t*)"2P", 2, 10.0f, 10.0f, 16.0f);
    } else {
        draw_text_bytes((const uint8_t*)"vs AI", 5, 10.0f, 10.0f, 16.0f);
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    switch (state) {
        .title => {
            renderCourt();
            renderTitle();
        },

        .playing => {
            renderCourt();
            renderScores();
            renderPaddles();
            renderBall();
            renderModeIndicator();
        },

        .game_over => {
            renderCourt();
            renderScores();
            renderPaddles();
            renderBall();
            renderGameOver();
        },
    }
}

fn renderTitle() void {
    set_color(COLOR_WHITE);
    drawTextBytes("PADDLE", SCREEN_WIDTH / 2.0 - 100.0, 150.0, 64.0);

    if (is_two_player) {
        drawTextBytes("2 PLAYER MODE", SCREEN_WIDTH / 2.0 - 100.0, 250.0, 24.0);
    } else {
        drawTextBytes("1 PLAYER VS AI", SCREEN_WIDTH / 2.0 - 100.0, 250.0, 24.0);
    }

    set_color(COLOR_GRAY);
    drawTextBytes("Press A to Start", SCREEN_WIDTH / 2.0 - 120.0, 350.0, 24.0);
}

fn renderGameOver() void {
    // Dark overlay
    set_color(0x000000CC);
    draw_rect(SCREEN_WIDTH / 4.0, SCREEN_HEIGHT / 3.0,
              SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 3.0);

    // Winner text
    const text: []const u8 = if (winner == 1)
        "PLAYER 1 WINS!"
    else if (is_two_player)
        "PLAYER 2 WINS!"
    else
        "AI WINS!";

    const color: u32 = if (winner == 1) COLOR_PLAYER1 else COLOR_PLAYER2;

    set_color(color);
    draw_text(text.ptr, @intCast(text.len),
              SCREEN_WIDTH / 2.0 - 120.0, SCREEN_HEIGHT / 2.0 - 20.0, 32.0);

    set_color(COLOR_GRAY);
    drawTextBytes("Press A to Play Again",
                 SCREEN_WIDTH / 2.0 - 140.0, SCREEN_HEIGHT / 2.0 + 30.0, 20.0);
}

fn renderModeIndicator() void {
    set_color(COLOR_GRAY);
    if (is_two_player) {
        drawTextBytes("2P", 10.0, 10.0, 16.0);
    } else {
        drawTextBytes("vs AI", 10.0, 10.0, 16.0);
    }
}
```
{{#endtab}}

{{#endtabs}}

## Build and Test

```bash
cargo build --target wasm32-unknown-unknown --release
nether run target/wasm32-unknown-unknown/release/paddle.wasm
```

The game now has:
- Title screen with mode indicator
- Score display during play
- Game over screen with winner
- Press A to start or restart
- First to 5 points wins

---

**Next:** [Part 7: Sound Effects](./07-sound.md) - Add audio feedback.
