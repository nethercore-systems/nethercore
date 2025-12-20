# Part 1.5: Game Assets

Before we add gameplay, let's set up a proper asset workflow. Instead of generating graphics and sounds procedurally in code, we'll load them from files—just like a real game.

## What You'll Learn

- Creating an `ember.toml` manifest
- Setting up an assets folder
- Using `ember build` to compile your game
- Loading textures and sounds from the ROM

## Why Use Assets?

Procedural generation is great for prototyping, but real games use asset files:

| Approach | Pros | Cons |
|----------|------|------|
| Procedural | Self-contained, small binaries | Limited quality, hard to iterate |
| Asset files | Professional quality, easy to update | Requires build step |

With Emberware's asset pipeline, you get the best of both worlds: easy asset management with automatic format conversion and compression.

## Create the Assets Folder

Create an `assets/` folder in your project:

```bash
mkdir assets
```

## Download Sample Assets

We've provided sample assets for this tutorial. Download them to your `assets/` folder:

**Images:**
- `paddle.png` — Blue paddle sprite (16×64)
- `ball.png` — White ball sprite (16×16)

**Sounds:**
- `hit.wav` — Paddle/wall hit sound
- `score.wav` — Point scored sound
- `win.wav` — Victory fanfare

You can find these in the [Emberware examples](https://github.com/emberware-io/emberware/tree/main/docs/book/src/tutorials/paddle/assets), or create your own!

Your project structure should now look like:

```
paddle/
├── Cargo.toml
├── ember.toml          ← We'll create this next
├── assets/
│   ├── paddle.png
│   ├── ball.png
│   ├── hit.wav
│   ├── score.wav
│   └── win.wav
└── src/
    └── lib.rs
```

## Create ember.toml

Create `ember.toml` in your project root. This manifest tells Emberware about your game and its assets:

```toml
[game]
id = "paddle"
title = "Paddle"
author = "Your Name"
version = "0.1.0"

# Texture assets
[[assets.textures]]
id = "paddle"
path = "assets/paddle.png"

[[assets.textures]]
id = "ball"
path = "assets/ball.png"

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

### Understanding ember.toml

| Section | Purpose |
|---------|---------|
| `[game]` | Game metadata (ID, title, author) |
| `[[assets.textures]]` | Image files to bundle |
| `[[assets.sounds]]` | Audio files to bundle |

Each asset has:
- **id** — The name you'll use to load it in code
- **path** — File location relative to `ember.toml`

## Build Your Game

Now use `ember build` to compile your game with assets:

```bash
ember build
```

This command:
1. Compiles your Rust code to WASM
2. Converts assets to optimized formats
3. Bundles everything into a `.ewz` ROM file

You'll see output like:

```
Building paddle...
  Compiling paddle v0.1.0
  Converting paddle.png → paddle.ewztex
  Converting ball.png → ball.ewztex
  Converting hit.wav → hit.ewzsnd
  Converting score.wav → score.ewzsnd
  Converting win.wav → win.ewzsnd
  Packing paddle.ewz (42 KB)
Done!
```

## Run Your Game

Test your built ROM:

```bash
ember run paddle.ewz
```

Or run directly (ember build + run):

```bash
ember run
```

## Loading Assets in Code

Now let's update our code to use the bundled assets. We'll use `rom_*` functions instead of creating graphics procedurally.

### Add FFI Imports

Update your FFI imports in `src/lib.rs`:

```rust
#[link(wasm_import_module = "env")]
extern "C" {
    // Existing imports...
    fn set_clear_color(color: u32);
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);

    // NEW: ROM asset loading
    fn rom_texture(id_ptr: *const u8, id_len: u32) -> u32;
    fn rom_sound(id_ptr: *const u8, id_len: u32) -> u32;

    // NEW: Texture and sprite drawing
    fn texture_bind(texture: u32);
    fn draw_sprite(x: f32, y: f32, w: f32, h: f32);

    // NEW: Audio playback
    fn play_sound(sound: u32, volume: f32, pan: f32);
}
```

### Add Asset Handles

Add static variables to store loaded asset handles:

```rust
// Asset handles (loaded in init)
static mut TEX_PADDLE: u32 = 0;
static mut TEX_BALL: u32 = 0;
static mut SFX_HIT: u32 = 0;
static mut SFX_SCORE: u32 = 0;
static mut SFX_WIN: u32 = 0;
```

### Load Assets in init()

Update `init()` to load assets from the ROM:

```rust
#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eFF);

        // Load textures from ROM
        TEX_PADDLE = rom_texture(b"paddle".as_ptr(), 6);
        TEX_BALL = rom_texture(b"ball".as_ptr(), 4);

        // Load sounds from ROM
        SFX_HIT = rom_sound(b"hit".as_ptr(), 3);
        SFX_SCORE = rom_sound(b"score".as_ptr(), 5);
        SFX_WIN = rom_sound(b"win".as_ptr(), 3);

        // Initialize game state (same as before)
        PADDLE1_Y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
        PADDLE2_Y = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
        BALL_X = SCREEN_WIDTH / 2.0 - BALL_SIZE / 2.0;
        BALL_Y = SCREEN_HEIGHT / 2.0 - BALL_SIZE / 2.0;
    }
}
```

### Draw Sprites (Optional Enhancement)

You can now draw paddles and ball using sprites instead of rectangles:

```rust
#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Draw center line (unchanged)
        // ... center line code ...

        // Draw paddle 1 using sprite
        texture_bind(TEX_PADDLE);
        draw_sprite(PADDLE_MARGIN, PADDLE1_Y, PADDLE_WIDTH, PADDLE_HEIGHT);

        // Draw paddle 2 using sprite
        draw_sprite(
            SCREEN_WIDTH - PADDLE_MARGIN - PADDLE_WIDTH,
            PADDLE2_Y,
            PADDLE_WIDTH,
            PADDLE_HEIGHT,
        );

        // Draw ball using sprite
        texture_bind(TEX_BALL);
        draw_sprite(BALL_X, BALL_Y, BALL_SIZE, BALL_SIZE);
    }
}
```

> **Note:** For this tutorial, we'll continue using `draw_rect()` for simplicity. The sprite approach is shown here so you know it's available. We'll use the loaded sounds in [Part 7: Sound Effects](./07-sound.md).

## Build Workflow Summary

| Command | Purpose |
|---------|---------|
| `ember build` | Compile + convert assets + pack ROM |
| `ember run` | Build and run immediately |
| `ember run game.ewz` | Run a specific ROM file |

## What You've Learned

- **ember.toml** — The manifest that describes your game and assets
- **Asset types** — Textures, sounds, meshes, fonts, and raw data
- **ember build** — Compiles and packs everything into a ROM
- **rom_* functions** — Load bundled assets by string ID

Your project is now set up for professional game development!

---

**Next:** [Part 2: Paddle Movement](./02-paddles.md) — Make the paddles respond to input.
