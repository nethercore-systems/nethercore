//! Pong Example
//!
//! A complete Pong game demonstrating core Emberware ZX features:
//! - 2D gameplay with draw_rect()
//! - Input handling for multiple players
//! - Simple physics (ball movement, collision)
//! - AI opponent for single-player
//! - Game states (title, playing, game over)
//! - Procedural sound effects
//! - Rollback-safe game state (all state in statics)
//!
//! This example is designed to accompany the "Build Pong" tutorial in the docs.
//!
//! Controls:
//! - Player 1: Left stick or D-pad Up/Down to move paddle
//! - Player 2: Left stick or D-pad Up/Down (or AI if single player)
//! - A button: Start game / Restart after game over
//!
//! Note: Rollback state is automatic (entire WASM memory is snapshotted).
//! When a second player connects, the game automatically becomes 2-player.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// === FFI Imports ===

#[link(wasm_import_module = "env")]
extern "C" {
    // Configuration
    fn set_clear_color(color: u32);

    // Input
    fn left_stick_y(player: u32) -> f32;
    fn button_pressed(player: u32, button: u32) -> u32;
    fn button_held(player: u32, button: u32) -> u32;
    fn player_count() -> u32;

    // 2D Drawing
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);

    // Audio
    fn load_sound(data_ptr: *const i16, byte_len: u32) -> u32;
    fn play_sound(sound: u32, volume: f32, pan: f32);

    // System
    fn random() -> u32;
}

// === Constants ===

// Screen dimensions (assuming 960x540 resolution)
const SCREEN_WIDTH: f32 = 960.0;
const SCREEN_HEIGHT: f32 = 540.0;

// Paddle dimensions and speed
const PADDLE_WIDTH: f32 = 15.0;
const PADDLE_HEIGHT: f32 = 80.0;
const PADDLE_SPEED: f32 = 8.0;
const PADDLE_MARGIN: f32 = 30.0;

// Ball dimensions and speed
const BALL_SIZE: f32 = 15.0;
const BALL_SPEED_INITIAL: f32 = 5.0;
const BALL_SPEED_MAX: f32 = 12.0;
const BALL_SPEED_INCREMENT: f32 = 0.5;

// Scoring
const WIN_SCORE: u32 = 5;

// Button constants
const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;
const BUTTON_A: u32 = 4;

// Colors
const COLOR_WHITE: u32 = 0xFFFFFFFF;
const COLOR_GRAY: u32 = 0x666666FF;
const COLOR_DARK: u32 = 0x1a1a2eFF;
const COLOR_PLAYER1: u32 = 0x4a9fffFF; // Blue
const COLOR_PLAYER2: u32 = 0xff6b6bFF; // Red
const COLOR_BALL: u32 = 0xFFFFFFFF;

// === Game State ===

#[derive(Clone, Copy, PartialEq)]
enum GameState {
    Title,
    Playing,
    GameOver,
}

#[derive(Clone, Copy)]
struct Paddle {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy)]
struct Ball {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}

// All game state is static for rollback safety
static mut STATE: GameState = GameState::Title;
static mut PADDLE1: Paddle = Paddle { x: 0.0, y: 0.0 };
static mut PADDLE2: Paddle = Paddle { x: 0.0, y: 0.0 };
static mut BALL: Ball = Ball { x: 0.0, y: 0.0, vx: 0.0, vy: 0.0 };
static mut SCORE1: u32 = 0;
static mut SCORE2: u32 = 0;
static mut WINNER: u32 = 0; // 1 or 2
static mut IS_TWO_PLAYER: bool = false;

// Sound handles
static mut SFX_HIT: u32 = 0;
static mut SFX_SCORE: u32 = 0;
static mut SFX_WIN: u32 = 0;

// === Sound Generation ===

// Generate a short beep sound (hit)
fn generate_hit_sound() -> [i16; 2205] {
    let mut samples = [0i16; 2205]; // 0.1 seconds at 22050 Hz
    let frequency = 440.0;
    let sample_rate = 22050.0;

    for i in 0..2205 {
        let t = i as f32 / sample_rate;
        // Envelope: quick attack, fast decay
        let envelope = 1.0 - (i as f32 / 2205.0);
        let value = libm::sinf(2.0 * core::f32::consts::PI * frequency * t) * envelope;
        samples[i] = (value * 32767.0 * 0.3) as i16;
    }
    samples
}

// Generate a descending tone (score against)
fn generate_score_sound() -> [i16; 4410] {
    let mut samples = [0i16; 4410]; // 0.2 seconds
    let sample_rate = 22050.0;

    for i in 0..4410 {
        let t = i as f32 / sample_rate;
        let progress = i as f32 / 4410.0;
        // Descending frequency from 880 to 220 Hz
        let frequency = 880.0 - (660.0 * progress);
        let envelope = 1.0 - progress;
        let value = libm::sinf(2.0 * core::f32::consts::PI * frequency * t) * envelope;
        samples[i] = (value * 32767.0 * 0.3) as i16;
    }
    samples
}

// Generate a victory fanfare
fn generate_win_sound() -> [i16; 11025] {
    let mut samples = [0i16; 11025]; // 0.5 seconds
    let sample_rate = 22050.0;

    for i in 0..11025 {
        let t = i as f32 / sample_rate;
        let progress = i as f32 / 11025.0;

        // Three ascending notes
        let frequency = if progress < 0.33 {
            523.25 // C5
        } else if progress < 0.66 {
            659.25 // E5
        } else {
            783.99 // G5
        };

        let envelope = 1.0 - (progress * 0.5);
        let value = libm::sinf(2.0 * core::f32::consts::PI * frequency * t) * envelope;
        samples[i] = (value * 32767.0 * 0.3) as i16;
    }
    samples
}

// === Helper Functions ===

fn draw_text_str(s: &[u8], x: f32, y: f32, size: f32, color: u32) {
    unsafe {
        draw_text(s.as_ptr(), s.len() as u32, x, y, size, color);
    }
}

fn reset_ball(direction: i32) {
    unsafe {
        BALL.x = SCREEN_WIDTH / 2.0 - BALL_SIZE / 2.0;
        BALL.y = SCREEN_HEIGHT / 2.0 - BALL_SIZE / 2.0;

        // Random vertical angle
        let rand = random() % 100;
        let angle = ((rand as f32 / 100.0) - 0.5) * 0.5; // -0.25 to 0.25

        BALL.vx = BALL_SPEED_INITIAL * direction as f32;
        BALL.vy = BALL_SPEED_INITIAL * angle;
    }
}

fn reset_game() {
    unsafe {
        // Reset paddles to center
        PADDLE1.x = PADDLE_MARGIN;
        PADDLE1.y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;

        PADDLE2.x = SCREEN_WIDTH - PADDLE_MARGIN - PADDLE_WIDTH;
        PADDLE2.y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;

        // Reset scores
        SCORE1 = 0;
        SCORE2 = 0;
        WINNER = 0;

        // Check player count for AI mode
        IS_TWO_PLAYER = player_count() >= 2;

        // Start ball moving toward player 1
        reset_ball(-1);
    }
}

fn clamp(v: f32, min: f32, max: f32) -> f32 {
    if v < min { min } else if v > max { max } else { v }
}

fn abs(v: f32) -> f32 {
    if v < 0.0 { -v } else { v }
}

// === Initialization ===

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(COLOR_DARK);

        // Generate and load sounds
        let hit_samples = generate_hit_sound();
        SFX_HIT = load_sound(hit_samples.as_ptr(), (hit_samples.len() * 2) as u32);

        let score_samples = generate_score_sound();
        SFX_SCORE = load_sound(score_samples.as_ptr(), (score_samples.len() * 2) as u32);

        let win_samples = generate_win_sound();
        SFX_WIN = load_sound(win_samples.as_ptr(), (win_samples.len() * 2) as u32);

        // Initialize game state
        reset_game();
        STATE = GameState::Title;
    }
}

// === Update ===

fn update_paddle_input(paddle: &mut Paddle, player: u32) {
    unsafe {
        // Read analog stick
        let stick_y = left_stick_y(player);

        // Read D-pad
        let up = button_held(player, BUTTON_UP) != 0;
        let down = button_held(player, BUTTON_DOWN) != 0;

        // Apply movement (stick Y is inverted: up is negative)
        let mut movement = -stick_y * PADDLE_SPEED;

        if up {
            movement -= PADDLE_SPEED;
        }
        if down {
            movement += PADDLE_SPEED;
        }

        paddle.y += movement;

        // Clamp to screen bounds
        paddle.y = clamp(paddle.y, 0.0, SCREEN_HEIGHT - PADDLE_HEIGHT);
    }
}

fn update_ai(paddle: &mut Paddle) {
    unsafe {
        // Simple AI: follow the ball with some lag
        let paddle_center = paddle.y + PADDLE_HEIGHT / 2.0;
        let ball_center = BALL.y + BALL_SIZE / 2.0;

        let diff = ball_center - paddle_center;

        // Only move if difference is significant
        if abs(diff) > 5.0 {
            // AI moves slower than max speed to be beatable
            let ai_speed = PADDLE_SPEED * 0.7;
            if diff > 0.0 {
                paddle.y += ai_speed;
            } else {
                paddle.y -= ai_speed;
            }
        }

        // Clamp to screen bounds
        paddle.y = clamp(paddle.y, 0.0, SCREEN_HEIGHT - PADDLE_HEIGHT);
    }
}

fn update_ball() {
    unsafe {
        // Move ball
        BALL.x += BALL.vx;
        BALL.y += BALL.vy;

        // Bounce off top and bottom walls
        if BALL.y <= 0.0 {
            BALL.y = 0.0;
            BALL.vy = -BALL.vy;
            play_sound(SFX_HIT, 0.3, 0.0);
        }
        if BALL.y >= SCREEN_HEIGHT - BALL_SIZE {
            BALL.y = SCREEN_HEIGHT - BALL_SIZE;
            BALL.vy = -BALL.vy;
            play_sound(SFX_HIT, 0.3, 0.0);
        }

        // Check collision with paddle 1 (left)
        if BALL.vx < 0.0 {
            let paddle = &PADDLE1;
            if BALL.x <= paddle.x + PADDLE_WIDTH
                && BALL.x + BALL_SIZE >= paddle.x
                && BALL.y + BALL_SIZE >= paddle.y
                && BALL.y <= paddle.y + PADDLE_HEIGHT
            {
                // Bounce off paddle
                BALL.x = paddle.x + PADDLE_WIDTH;
                BALL.vx = -BALL.vx;

                // Add spin based on where ball hit paddle
                let paddle_center = paddle.y + PADDLE_HEIGHT / 2.0;
                let ball_center = BALL.y + BALL_SIZE / 2.0;
                let offset = (ball_center - paddle_center) / (PADDLE_HEIGHT / 2.0);
                BALL.vy += offset * 2.0;

                // Speed up
                let speed = libm::sqrtf(BALL.vx * BALL.vx + BALL.vy * BALL.vy);
                if speed < BALL_SPEED_MAX {
                    let factor = (speed + BALL_SPEED_INCREMENT) / speed;
                    BALL.vx *= factor;
                    BALL.vy *= factor;
                }

                play_sound(SFX_HIT, 0.5, -0.5); // Pan left
            }
        }

        // Check collision with paddle 2 (right)
        if BALL.vx > 0.0 {
            let paddle = &PADDLE2;
            if BALL.x + BALL_SIZE >= paddle.x
                && BALL.x <= paddle.x + PADDLE_WIDTH
                && BALL.y + BALL_SIZE >= paddle.y
                && BALL.y <= paddle.y + PADDLE_HEIGHT
            {
                // Bounce off paddle
                BALL.x = paddle.x - BALL_SIZE;
                BALL.vx = -BALL.vx;

                // Add spin based on where ball hit paddle
                let paddle_center = paddle.y + PADDLE_HEIGHT / 2.0;
                let ball_center = BALL.y + BALL_SIZE / 2.0;
                let offset = (ball_center - paddle_center) / (PADDLE_HEIGHT / 2.0);
                BALL.vy += offset * 2.0;

                // Speed up
                let speed = libm::sqrtf(BALL.vx * BALL.vx + BALL.vy * BALL.vy);
                if speed < BALL_SPEED_MAX {
                    let factor = (speed + BALL_SPEED_INCREMENT) / speed;
                    BALL.vx *= factor;
                    BALL.vy *= factor;
                }

                play_sound(SFX_HIT, 0.5, 0.5); // Pan right
            }
        }

        // Check for scoring
        if BALL.x < -BALL_SIZE {
            // Player 2 scores
            SCORE2 += 1;
            play_sound(SFX_SCORE, 0.6, 0.5);

            if SCORE2 >= WIN_SCORE {
                WINNER = 2;
                STATE = GameState::GameOver;
                play_sound(SFX_WIN, 0.8, 0.0);
            } else {
                reset_ball(-1); // Serve toward player 1
            }
        }

        if BALL.x > SCREEN_WIDTH {
            // Player 1 scores
            SCORE1 += 1;
            play_sound(SFX_SCORE, 0.6, -0.5);

            if SCORE1 >= WIN_SCORE {
                WINNER = 1;
                STATE = GameState::GameOver;
                play_sound(SFX_WIN, 0.8, 0.0);
            } else {
                reset_ball(1); // Serve toward player 2
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Check if player count changed (someone connected/disconnected)
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
                // Update player 1 paddle (always player-controlled)
                update_paddle_input(&mut PADDLE1, 0);

                // Update player 2 paddle (player or AI)
                if IS_TWO_PLAYER {
                    update_paddle_input(&mut PADDLE2, 1);
                } else {
                    update_ai(&mut PADDLE2);
                }

                // Update ball
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

// === Render ===

fn render_court() {
    unsafe {
        // Center line (dashed)
        let dash_height = 20.0;
        let dash_gap = 15.0;
        let dash_width = 4.0;
        let center_x = SCREEN_WIDTH / 2.0 - dash_width / 2.0;

        let mut y = 10.0;
        while y < SCREEN_HEIGHT - 10.0 {
            draw_rect(center_x, y, dash_width, dash_height, COLOR_GRAY);
            y += dash_height + dash_gap;
        }
    }
}

fn render_paddles() {
    unsafe {
        // Player 1 paddle (blue)
        draw_rect(PADDLE1.x, PADDLE1.y, PADDLE_WIDTH, PADDLE_HEIGHT, COLOR_PLAYER1);

        // Player 2 paddle (red)
        draw_rect(PADDLE2.x, PADDLE2.y, PADDLE_WIDTH, PADDLE_HEIGHT, COLOR_PLAYER2);
    }
}

fn render_ball() {
    unsafe {
        draw_rect(BALL.x, BALL.y, BALL_SIZE, BALL_SIZE, COLOR_BALL);
    }
}

fn render_scores() {
    unsafe {
        // Score digits
        let score1_digit = b'0' + (SCORE1 % 10) as u8;
        let score2_digit = b'0' + (SCORE2 % 10) as u8;

        let score1_text = [score1_digit];
        let score2_text = [score2_digit];

        // Player 1 score (left side)
        draw_text(score1_text.as_ptr(), 1, SCREEN_WIDTH / 4.0, 30.0, 48.0, COLOR_PLAYER1);

        // Player 2 score (right side)
        draw_text(score2_text.as_ptr(), 1, SCREEN_WIDTH * 3.0 / 4.0, 30.0, 48.0, COLOR_PLAYER2);
    }
}

fn render_title() {
    unsafe {
        // Title
        draw_text_str(b"PONG", SCREEN_WIDTH / 2.0 - 80.0, 150.0, 64.0, COLOR_WHITE);

        // Mode indicator
        if IS_TWO_PLAYER {
            draw_text_str(b"2 PLAYER MODE", SCREEN_WIDTH / 2.0 - 100.0, 250.0, 24.0, COLOR_WHITE);
        } else {
            draw_text_str(b"1 PLAYER VS AI", SCREEN_WIDTH / 2.0 - 100.0, 250.0, 24.0, COLOR_WHITE);
        }

        // Instructions
        draw_text_str(b"Press A to Start", SCREEN_WIDTH / 2.0 - 120.0, 350.0, 24.0, COLOR_GRAY);

        // Controls hint
        draw_text_str(b"Controls: Left Stick or D-Pad Up/Down", 250.0, 450.0, 18.0, COLOR_GRAY);
    }
}

fn render_game_over() {
    unsafe {
        // Overlay
        draw_rect(SCREEN_WIDTH / 4.0, SCREEN_HEIGHT / 3.0,
                  SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 3.0, 0x000000CC);

        // Winner text
        if WINNER == 1 {
            draw_text_str(b"PLAYER 1 WINS!", SCREEN_WIDTH / 2.0 - 130.0, SCREEN_HEIGHT / 2.0 - 30.0, 32.0, COLOR_PLAYER1);
        } else {
            if IS_TWO_PLAYER {
                draw_text_str(b"PLAYER 2 WINS!", SCREEN_WIDTH / 2.0 - 130.0, SCREEN_HEIGHT / 2.0 - 30.0, 32.0, COLOR_PLAYER2);
            } else {
                draw_text_str(b"AI WINS!", SCREEN_WIDTH / 2.0 - 70.0, SCREEN_HEIGHT / 2.0 - 30.0, 32.0, COLOR_PLAYER2);
            }
        }

        // Restart prompt
        draw_text_str(b"Press A to Play Again", SCREEN_WIDTH / 2.0 - 150.0, SCREEN_HEIGHT / 2.0 + 30.0, 20.0, COLOR_GRAY);
    }
}

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

                // Show mode indicator
                if IS_TWO_PLAYER {
                    draw_text_str(b"2P", 10.0, 10.0, 16.0, COLOR_GRAY);
                } else {
                    draw_text_str(b"vs AI", 10.0, 10.0, 16.0, COLOR_GRAY);
                }
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
