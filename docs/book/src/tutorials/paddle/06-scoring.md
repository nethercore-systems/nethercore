# Part 6: Scoring & Win States

Let's add proper scoring, win conditions, and a game state machine.

## What You'll Learn

- Game state machines (Title, Playing, GameOver)
- Tracking and displaying scores
- Win conditions
- Using `button_pressed()` for menu navigation

## Add Game State

Create a state enum and related variables:

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

## Add Button Constants

We need the A button for starting/restarting:

```rust
const BUTTON_A: u32 = 4;
```

Add to FFI imports:

```rust
fn button_pressed(player: u32, button: u32) -> u32;
```

## Reset Game Function

Create a function to reset the entire game:

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

## Update Scoring Logic

Modify the ball update to handle scoring:

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

## State Machine in Update

Restructure `update()` to handle game states:

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

## Update Init

Start on title screen:

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

## Render Scores

Add a helper for drawing text:

```rust
fn draw_text_bytes(text: &[u8], x: f32, y: f32, size: f32, color: u32) {
    unsafe {
        draw_text(text.as_ptr(), text.len() as u32, x, y, size, color);
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
        draw_text(&[score1_char], 1, SCREEN_WIDTH / 4.0, 30.0, 48.0, COLOR_PLAYER1);
        draw_text(&[score2_char], 1, SCREEN_WIDTH * 3.0 / 4.0, 30.0, 48.0, COLOR_PLAYER2);
    }
}
```

## Render States

Update `render()` to show different screens:

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
        draw_text_bytes(b"PADDLE", SCREEN_WIDTH / 2.0 - 100.0, 150.0, 64.0, COLOR_WHITE);

        if IS_TWO_PLAYER {
            draw_text_bytes(b"2 PLAYER MODE", SCREEN_WIDTH / 2.0 - 100.0, 250.0, 24.0, COLOR_WHITE);
        } else {
            draw_text_bytes(b"1 PLAYER VS AI", SCREEN_WIDTH / 2.0 - 100.0, 250.0, 24.0, COLOR_WHITE);
        }

        draw_text_bytes(b"Press A to Start", SCREEN_WIDTH / 2.0 - 120.0, 350.0, 24.0, COLOR_GRAY);
    }
}

fn render_game_over() {
    unsafe {
        // Dark overlay
        draw_rect(SCREEN_WIDTH / 4.0, SCREEN_HEIGHT / 3.0,
                  SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 3.0, 0x000000CC);

        // Winner text
        let (text, color) = if WINNER == 1 {
            (b"PLAYER 1 WINS!" as &[u8], COLOR_PLAYER1)
        } else if IS_TWO_PLAYER {
            (b"PLAYER 2 WINS!" as &[u8], COLOR_PLAYER2)
        } else {
            (b"AI WINS!" as &[u8], COLOR_PLAYER2)
        };

        draw_text(text.as_ptr(), text.len() as u32,
                  SCREEN_WIDTH / 2.0 - 120.0, SCREEN_HEIGHT / 2.0 - 20.0, 32.0, color);

        draw_text_bytes(b"Press A to Play Again",
                       SCREEN_WIDTH / 2.0 - 140.0, SCREEN_HEIGHT / 2.0 + 30.0, 20.0, COLOR_GRAY);
    }
}

fn render_mode_indicator() {
    unsafe {
        if IS_TWO_PLAYER {
            draw_text_bytes(b"2P", 10.0, 10.0, 16.0, COLOR_GRAY);
        } else {
            draw_text_bytes(b"vs AI", 10.0, 10.0, 16.0, COLOR_GRAY);
        }
    }
}
```

## Build and Test

```bash
cargo build --target wasm32-unknown-unknown --release
ember run target/wasm32-unknown-unknown/release/paddle.wasm
```

The game now has:
- Title screen with mode indicator
- Score display during play
- Game over screen with winner
- Press A to start or restart
- First to 5 points wins

---

**Next:** [Part 7: Sound Effects](./07-sound.md) - Add audio feedback.
