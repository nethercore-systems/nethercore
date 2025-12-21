# Part 7: Sound Effects

Games feel incomplete without audio. So far we've been using `draw_rect()` for everything—but you can't draw a sound! This is where Emberware's asset pipeline comes in.

## What You'll Learn

- Setting up an assets folder
- Creating `ember.toml` to bundle assets
- Using `ember build` instead of `cargo build`
- Loading sounds with `rom_sound()`
- Playing sounds with `play_sound()` and stereo panning

## Why Assets Now?

Up until now, we've built and tested like this:

```bash
cargo build --target wasm32-unknown-unknown --release
ember run target/wasm32-unknown-unknown/release/paddle.wasm
```

This works great for graphics—`draw_rect()` handles everything. But sounds need actual audio files. That's where `ember build` comes in: it bundles your code *and* assets into a single ROM file.

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

**Download sample sounds** from the [tutorial assets](https://github.com/emberware-io/emberware/tree/main/docs/book/src/tutorials/paddle/assets), or create your own with:
- [JSFXR](https://sfxr.me) — Generate retro sound effects in your browser
- [Freesound.org](https://freesound.org) — CC-licensed sounds
- [Audacity](https://www.audacityteam.org) — Record and edit audio

Put them in your `assets/` folder:

```
paddle/
├── Cargo.toml
├── ember.toml          ← We'll create this next
├── assets/
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
- **path** — File location relative to `ember.toml`

## Build with ember build

Now use `ember build` instead of `cargo build`:

```bash
ember build
```

This command:
1. Compiles your Rust code to WASM
2. Converts WAV files to the optimized format (22050 Hz mono)
3. Bundles everything into a `paddle.ewz` ROM file

You'll see output like:

```
Building paddle...
  Compiling paddle v0.1.0
  Converting hit.wav → hit.ewzsnd
  Converting score.wav → score.ewzsnd
  Converting win.wav → win.ewzsnd
  Packing paddle.ewz (28 KB)
Done!
```

## Run Your Game

Now run the ROM:

```bash
ember run paddle.ewz
```

Or just:

```bash
ember run
```

This builds and runs in one step.

## Add Audio FFI

Add the audio functions to your FFI imports:

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

## Sound Handles

Add static variables to store sound handles:

```rust
static mut SFX_HIT: u32 = 0;
static mut SFX_SCORE: u32 = 0;
static mut SFX_WIN: u32 = 0;
```

## Load Sounds in init()

Update `init()` to load sounds from the ROM:

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

The `rom_sound()` function loads the sound directly from the bundled ROM—the string IDs match what you put in `ember.toml`.

## Play Sounds

Now add sound effects to game events:

### Wall Bounce

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

### Paddle Hit

```rust
// In paddle 1 collision:
play_sound(SFX_HIT, 0.5, -0.5);  // Pan left

// In paddle 2 collision:
play_sound(SFX_HIT, 0.5, 0.5);   // Pan right
```

### Scoring

```rust
// When player 2 scores (ball exits left):
SCORE2 += 1;
play_sound(SFX_SCORE, 0.6, 0.5);  // Pan right (scorer's side)

// When player 1 scores (ball exits right):
SCORE1 += 1;
play_sound(SFX_SCORE, 0.6, -0.5);  // Pan left (scorer's side)
```

### Win

```rust
// When either player wins:
if SCORE1 >= WIN_SCORE || SCORE2 >= WIN_SCORE {
    STATE = GameState::GameOver;
    play_sound(SFX_WIN, 0.8, 0.0);  // Center, louder
}
```

## Understanding play_sound()

```rust
fn play_sound(sound: u32, volume: f32, pan: f32);
```

| Parameter | Range | Description |
|-----------|-------|-------------|
| `sound` | Handle | Sound handle from `rom_sound()` |
| `volume` | 0.0 - 1.0 | 0 = silent, 1 = full volume |
| `pan` | -1.0 - 1.0 | -1 = left, 0 = center, 1 = right |

## Audio Specs

Emberware uses these audio settings:

| Property | Value |
|----------|-------|
| Sample rate | 22050 Hz |
| Format | 16-bit mono PCM |
| Channels | Stereo output |

The `ember build` command automatically converts your WAV files to this format.

## Sound Design Tips

1. **Keep sounds short** — 0.1 to 0.5 seconds is plenty for effects
2. **Use panning** — Stereo positioning helps players track action
3. **Vary volume** — Important sounds louder, ambient sounds quieter
4. **Match your aesthetic** — Simple sounds fit retro games

## Build and Test

Rebuild with your sound assets:

```bash
ember build
ember run
```

The game now has:
- "Ping" sound when ball hits walls or paddles
- Different panning for left/right paddle hits
- Descending "whomp" when someone scores
- Victory fanfare when a player wins

## New Workflow Summary

| Before (Parts 1-6) | Now (Part 7+) |
|-------------------|---------------|
| `cargo build --target wasm32-unknown-unknown --release` | `ember build` |
| `ember run target/.../paddle.wasm` | `ember run` |
| No assets needed | Assets bundled in ROM |

From now on, just use `ember build` and `ember run`!

---

**Next:** [Part 8: Polish & Publishing](./08-polish.md) — Final touches and releasing your game.
