# Part 7: Sound Effects

Games feel incomplete without audio. So far we've been using `draw_rect()` for everything—but you can't draw a sound! This is where Nethercore's asset pipeline comes in.

## What You'll Learn

- Setting up an assets folder
- Creating `nether.toml` to bundle assets
- Using `nether build` instead of `cargo build`
- Loading sounds with `rom_sound()`
- Playing sounds with `play_sound()` and stereo panning

## Why Assets Now?

Up until now, we've built and tested like this:

```bash
cargo build --target wasm32-unknown-unknown --release
nether run target/wasm32-unknown-unknown/release/paddle.wasm
```

This works great for graphics—`draw_rect()` handles everything. But sounds need actual audio files. That's where `nether build` comes in: it bundles your code *and* assets into a single ROM file.

## Create the Assets Folder

Create an `assets/` folder in your project:

```bash
mkdir assets
```

## Get Sound Files

You need three WAV files for the game:

| Sound | Description | Duration |
|-------|-------------|----------|
| `hit.wav` | Quick beep for paddle/wall hits | ~0.1s |
| `score.wav` | Descending tone when someone scores | ~0.2s |
| `win.wav` | Victory fanfare when game ends | ~0.5s |

**Download sample sounds** from the [tutorial assets](https://github.com/nethercore-systems/nethercore/tree/main/docs/book/src/tutorials/paddle/assets), or create your own with:
- [JSFXR](https://sfxr.me) — Generate retro sound effects in your browser
- [Freesound.org](https://freesound.org) — CC-licensed sounds
- [Audacity](https://www.audacityteam.org) — Record and edit audio

Put them in your `assets/` folder:

```
paddle/
├── Cargo.toml
├── nether.toml          ← We'll create this next
├── assets/
│   ├── hit.wav
│   ├── score.wav
│   └── win.wav
└── src/
    └── lib.rs
```

## Create nether.toml

Create `nether.toml` in your project root. This manifest tells Nethercore about your game and its assets:

```toml
[game]
id = "paddle"
title = "Paddle"
author = "Your Name"
version = "0.1.0"

# Sound assets
[[assets.sounds]]
id = "hit"
path = "assets/hit.wav"

[[assets.sounds]]
id = "score"
path = "assets/score.wav"

[[assets.sounds]]
id = "win"
path = "assets/win.wav"
```

Each asset has:
- **id** — The name you'll use to load it in code
- **path** — File location relative to `nether.toml`

## Build with nether build

Now use `nether build` instead of `cargo build`:

```bash
nether build
```

This command:
1. Compiles your Rust code to WASM
2. Converts WAV files to the optimized format (22050 Hz mono)
3. Bundles everything into a `paddle.nczx` ROM file

You'll see output like:

```
Building paddle...
  Compiling paddle v0.1.0
  Converting hit.wav → hit.ewzsnd
  Converting score.wav → score.ewzsnd
  Converting win.wav → win.ewzsnd
  Packing paddle.nczx (28 KB)
Done!
```

## Run Your Game

Now run the ROM:

```bash
nether run paddle.nczx
```

Or just:

```bash
nether run
```

This builds and runs in one step.

## Add Audio FFI

Add the audio functions to your FFI imports:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[link(wasm_import_module = "env")]
extern "C" {
    // ... existing imports ...

    // ROM loading
    fn rom_sound(id_ptr: *const u8, id_len: u32) -> u32;

    // Audio playback
    fn play_sound(sound: u32, volume: f32, pan: f32);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// ROM loading
EWZX_IMPORT uint32_t rom_sound(const uint8_t* id_ptr, uint32_t id_len);

// Audio playback
EWZX_IMPORT void play_sound(uint32_t sound, float volume, float pan);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// ROM loading
pub extern fn rom_sound(id_ptr: [*]const u8, id_len: u32) u32;

// Audio playback
pub extern fn play_sound(sound: u32, volume: f32, pan: f32) void;
```
{{#endtab}}

{{#endtabs}}

## Sound Handles

Add static variables to store sound handles:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut SFX_HIT: u32 = 0;
static mut SFX_SCORE: u32 = 0;
static mut SFX_WIN: u32 = 0;
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t SFX_HIT = 0;
static uint32_t SFX_SCORE = 0;
static uint32_t SFX_WIN = 0;
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var SFX_HIT: u32 = 0;
var SFX_SCORE: u32 = 0;
var SFX_WIN: u32 = 0;
```
{{#endtab}}

{{#endtabs}}

## Load Sounds in init()

Update `init()` to load sounds from the ROM:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eFF);

        // Load sounds from ROM
        SFX_HIT = rom_sound(b"hit".as_ptr(), 3);
        SFX_SCORE = rom_sound(b"score".as_ptr(), 5);
        SFX_WIN = rom_sound(b"win".as_ptr(), 3);

        reset_game();
        STATE = GameState::Title;
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void init() {
    set_clear_color(0x1a1a2eFF);

    // Load sounds from ROM
    SFX_HIT = rom_sound("hit", 3);
    SFX_SCORE = rom_sound("score", 5);
    SFX_WIN = rom_sound("win", 3);

    reset_game();
    STATE = TITLE;
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn init() void {
    set_clear_color(0x1a1a2eFF);

    // Load sounds from ROM
    SFX_HIT = rom_sound("hit", 3);
    SFX_SCORE = rom_sound("score", 5);
    SFX_WIN = rom_sound("win", 3);

    reset_game();
    STATE = .Title;
}
```
{{#endtab}}

{{#endtabs}}

The `rom_sound()` function loads the sound directly from the bundled ROM—the string IDs match what you put in `nether.toml`.

## Play Sounds

Now add sound effects to game events:

### Wall Bounce

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// In update_ball(), after wall bounce:
if BALL_Y <= 0.0 {
    BALL_Y = 0.0;
    BALL_VY = -BALL_VY;
    play_sound(SFX_HIT, 0.3, 0.0);  // Center pan
}

if BALL_Y >= SCREEN_HEIGHT - BALL_SIZE {
    BALL_Y = SCREEN_HEIGHT - BALL_SIZE;
    BALL_VY = -BALL_VY;
    play_sound(SFX_HIT, 0.3, 0.0);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// In update_ball(), after wall bounce:
if (BALL_Y <= 0.0f) {
    BALL_Y = 0.0f;
    BALL_VY = -BALL_VY;
    play_sound(SFX_HIT, 0.3f, 0.0f);  // Center pan
}

if (BALL_Y >= SCREEN_HEIGHT - BALL_SIZE) {
    BALL_Y = SCREEN_HEIGHT - BALL_SIZE;
    BALL_VY = -BALL_VY;
    play_sound(SFX_HIT, 0.3f, 0.0f);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// In update_ball(), after wall bounce:
if (BALL_Y <= 0.0) {
    BALL_Y = 0.0;
    BALL_VY = -BALL_VY;
    play_sound(SFX_HIT, 0.3, 0.0);  // Center pan
}

if (BALL_Y >= SCREEN_HEIGHT - BALL_SIZE) {
    BALL_Y = SCREEN_HEIGHT - BALL_SIZE;
    BALL_VY = -BALL_VY;
    play_sound(SFX_HIT, 0.3, 0.0);
}
```
{{#endtab}}

{{#endtabs}}

### Paddle Hit

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// In paddle 1 collision:
play_sound(SFX_HIT, 0.5, -0.5);  // Pan left

// In paddle 2 collision:
play_sound(SFX_HIT, 0.5, 0.5);   // Pan right
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// In paddle 1 collision:
play_sound(SFX_HIT, 0.5f, -0.5f);  // Pan left

// In paddle 2 collision:
play_sound(SFX_HIT, 0.5f, 0.5f);   // Pan right
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// In paddle 1 collision:
play_sound(SFX_HIT, 0.5, -0.5);  // Pan left

// In paddle 2 collision:
play_sound(SFX_HIT, 0.5, 0.5);   // Pan right
```
{{#endtab}}

{{#endtabs}}

### Scoring

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// When player 2 scores (ball exits left):
SCORE2 += 1;
play_sound(SFX_SCORE, 0.6, 0.5);  // Pan right (scorer's side)

// When player 1 scores (ball exits right):
SCORE1 += 1;
play_sound(SFX_SCORE, 0.6, -0.5);  // Pan left (scorer's side)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// When player 2 scores (ball exits left):
SCORE2 += 1;
play_sound(SFX_SCORE, 0.6f, 0.5f);  // Pan right (scorer's side)

// When player 1 scores (ball exits right):
SCORE1 += 1;
play_sound(SFX_SCORE, 0.6f, -0.5f);  // Pan left (scorer's side)
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// When player 2 scores (ball exits left):
SCORE2 += 1;
play_sound(SFX_SCORE, 0.6, 0.5);  // Pan right (scorer's side)

// When player 1 scores (ball exits right):
SCORE1 += 1;
play_sound(SFX_SCORE, 0.6, -0.5);  // Pan left (scorer's side)
```
{{#endtab}}

{{#endtabs}}

### Win

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// When either player wins:
if SCORE1 >= WIN_SCORE || SCORE2 >= WIN_SCORE {
    STATE = GameState::GameOver;
    play_sound(SFX_WIN, 0.8, 0.0);  // Center, louder
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// When either player wins:
if (SCORE1 >= WIN_SCORE || SCORE2 >= WIN_SCORE) {
    STATE = GAME_OVER;
    play_sound(SFX_WIN, 0.8f, 0.0f);  // Center, louder
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// When either player wins:
if (SCORE1 >= WIN_SCORE or SCORE2 >= WIN_SCORE) {
    STATE = .GameOver;
    play_sound(SFX_WIN, 0.8, 0.0);  // Center, louder
}
```
{{#endtab}}

{{#endtabs}}

## Understanding play_sound()

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn play_sound(sound: u32, volume: f32, pan: f32);
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void play_sound(uint32_t sound, float volume, float pan);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn play_sound(sound: u32, volume: f32, pan: f32) void;
```
{{#endtab}}

{{#endtabs}}

| Parameter | Range | Description |
|-----------|-------|-------------|
| `sound` | Handle | Sound handle from `rom_sound()` |
| `volume` | 0.0 - 1.0 | 0 = silent, 1 = full volume |
| `pan` | -1.0 - 1.0 | -1 = left, 0 = center, 1 = right |

## Audio Specs

Nethercore uses these audio settings:

| Property | Value |
|----------|-------|
| Sample rate | 22050 Hz |
| Format | 16-bit mono PCM |
| Channels | Stereo output |

The `nether build` command automatically converts your WAV files to this format.

## Sound Design Tips

1. **Keep sounds short** — 0.1 to 0.5 seconds is plenty for effects
2. **Use panning** — Stereo positioning helps players track action
3. **Vary volume** — Important sounds louder, ambient sounds quieter
4. **Match your aesthetic** — Simple sounds fit retro games

## Build and Test

Rebuild with your sound assets:

```bash
nether build
nether run
```

The game now has:
- "Ping" sound when ball hits walls or paddles
- Different panning for left/right paddle hits
- Descending "whomp" when someone scores
- Victory fanfare when a player wins

## Bonus: Sprite Graphics

Now that we have the asset pipeline set up, we can also use image sprites instead of `draw_rect()`. This is optional—the game works fine with rectangles—but sprites look nicer!

### Add Texture Assets

Download `paddle.png` and `ball.png` from the [tutorial assets](https://github.com/nethercore-systems/nethercore/tree/main/docs/book/src/tutorials/paddle/assets), then add them to `nether.toml`:

```toml
# Texture assets
[[assets.textures]]
id = "paddle"
path = "assets/paddle.png"

[[assets.textures]]
id = "ball"
path = "assets/ball.png"
```

### Add Texture FFI

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[link(wasm_import_module = "env")]
extern "C" {
    // ... existing imports ...

    // Texture loading and drawing
    fn rom_texture(id_ptr: *const u8, id_len: u32) -> u32;
    fn texture_bind(texture: u32);
    fn draw_sprite(x: f32, y: f32, w: f32, h: f32);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Texture loading and drawing
EWZX_IMPORT uint32_t rom_texture(const uint8_t* id_ptr, uint32_t id_len);
EWZX_IMPORT void texture_bind(uint32_t texture);
EWZX_IMPORT void draw_sprite(float x, float y, float w, float h);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Texture loading and drawing
pub extern fn rom_texture(id_ptr: [*]const u8, id_len: u32) u32;
pub extern fn texture_bind(texture: u32) void;
pub extern fn draw_sprite(x: f32, y: f32, w: f32, h: f32) void;
```
{{#endtab}}

{{#endtabs}}

### Load Textures

Add handles and load in `init()`:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut TEX_PADDLE: u32 = 0;
static mut TEX_BALL: u32 = 0;

// In init():
TEX_PADDLE = rom_texture(b"paddle".as_ptr(), 6);
TEX_BALL = rom_texture(b"ball".as_ptr(), 4);
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t TEX_PADDLE = 0;
static uint32_t TEX_BALL = 0;

// In init():
TEX_PADDLE = rom_texture("paddle", 6);
TEX_BALL = rom_texture("ball", 4);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var TEX_PADDLE: u32 = 0;
var TEX_BALL: u32 = 0;

// In init():
TEX_PADDLE = rom_texture("paddle", 6);
TEX_BALL = rom_texture("ball", 4);
```
{{#endtab}}

{{#endtabs}}

### Draw Sprites

Replace `draw_rect()` calls in `render()`:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// Instead of: draw_rect(PADDLE_MARGIN, PADDLE1_Y, PADDLE_WIDTH, PADDLE_HEIGHT, COLOR_PLAYER1);
texture_bind(TEX_PADDLE);
draw_sprite(PADDLE_MARGIN, PADDLE1_Y, PADDLE_WIDTH, PADDLE_HEIGHT);

// Second paddle
draw_sprite(SCREEN_WIDTH - PADDLE_MARGIN - PADDLE_WIDTH, PADDLE2_Y, PADDLE_WIDTH, PADDLE_HEIGHT);

// Instead of: draw_rect(BALL_X, BALL_Y, BALL_SIZE, BALL_SIZE, COLOR_WHITE);
texture_bind(TEX_BALL);
draw_sprite(BALL_X, BALL_Y, BALL_SIZE, BALL_SIZE);
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Instead of: draw_rect(PADDLE_MARGIN, PADDLE1_Y, PADDLE_WIDTH, PADDLE_HEIGHT, COLOR_PLAYER1);
texture_bind(TEX_PADDLE);
draw_sprite(PADDLE_MARGIN, PADDLE1_Y, PADDLE_WIDTH, PADDLE_HEIGHT);

// Second paddle
draw_sprite(SCREEN_WIDTH - PADDLE_MARGIN - PADDLE_WIDTH, PADDLE2_Y, PADDLE_WIDTH, PADDLE_HEIGHT);

// Instead of: draw_rect(BALL_X, BALL_Y, BALL_SIZE, BALL_SIZE, COLOR_WHITE);
texture_bind(TEX_BALL);
draw_sprite(BALL_X, BALL_Y, BALL_SIZE, BALL_SIZE);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Instead of: draw_rect(PADDLE_MARGIN, PADDLE1_Y, PADDLE_WIDTH, PADDLE_HEIGHT, COLOR_PLAYER1);
texture_bind(TEX_PADDLE);
draw_sprite(PADDLE_MARGIN, PADDLE1_Y, PADDLE_WIDTH, PADDLE_HEIGHT);

// Second paddle
draw_sprite(SCREEN_WIDTH - PADDLE_MARGIN - PADDLE_WIDTH, PADDLE2_Y, PADDLE_WIDTH, PADDLE_HEIGHT);

// Instead of: draw_rect(BALL_X, BALL_Y, BALL_SIZE, BALL_SIZE, COLOR_WHITE);
texture_bind(TEX_BALL);
draw_sprite(BALL_X, BALL_Y, BALL_SIZE, BALL_SIZE);
```
{{#endtab}}

{{#endtabs}}

The sprite will be tinted by the bound texture. You can also use `draw_sprite_colored()` if you want to tint sprites with different colors per player.

## New Workflow Summary

| Before (Parts 1-6) | Now (Part 7+) |
|-------------------|---------------|
| `cargo build --target wasm32-unknown-unknown --release` | `nether build` |
| `nether run target/.../paddle.wasm` | `nether run` |
| No assets needed | Assets bundled in ROM |

From now on, just use `nether build` and `nether run`!

---

**Next:** [Part 8: Polish & Publishing](./08-polish.md) — Final touches and releasing your game.
