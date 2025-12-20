# Part 7: Sound Effects

Games feel incomplete without audio. Let's add satisfying sound effects using the assets we set up in [Part 1.5](./01b-assets.md).

## What You'll Learn

- Loading sounds from the ROM with `rom_sound()`
- Playing sounds with `play_sound()` and stereo panning
- When to play sounds for best game feel

## Our Sound Assets

In Part 1.5, we added three sound files to `ember.toml`:

```toml
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

Make sure you have these WAV files in your `assets/` folder:

| Sound | Description | Duration |
|-------|-------------|----------|
| `hit.wav` | Quick beep for paddle/wall hits | ~0.1s |
| `score.wav` | Descending tone when someone scores | ~0.2s |
| `win.wav` | Victory fanfare when game ends | ~0.5s |

You can download sample sounds from the [tutorial assets](https://github.com/emberware-io/emberware/tree/main/docs/book/src/tutorials/paddle/assets) or create your own!

## Add Audio FFI

Add the audio functions to your FFI imports:

```rust
#[link(wasm_import_module = "env")]
extern "C" {
    // ... existing imports ...

    // ROM loading (from Part 1.5)
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

That's it! The `rom_sound()` function loads the sound directly from the bundled ROM data—no need to generate anything procedurally.

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

The 22050 Hz sample rate:
- Uses half the memory of CD quality (44100 Hz)
- Sounds great for retro-style games
- Matches the aesthetic of older consoles

## Sound Design Tips

1. **Keep sounds short** — 0.1 to 0.5 seconds is plenty for effects
2. **Use panning** — Stereo positioning helps players track action
3. **Vary volume** — Important sounds louder, ambient sounds quieter
4. **Match your aesthetic** — Simple sounds fit retro games

## Creating Your Own Sounds

Want custom sounds? Here are some options:

### Free Sound Resources
- [Freesound.org](https://freesound.org) — CC-licensed sounds
- [SFXR/JSFXR](https://sfxr.me) — Generate retro sound effects
- [Audacity](https://www.audacityteam.org) — Record and edit audio

### Sound Requirements
- **Format:** WAV (16-bit PCM)
- **Sample rate:** 22050 Hz recommended
- **Channels:** Mono (stereo will be converted)

The `ember build` command automatically converts WAV files to the correct format.

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

## Complete Audio Code

Here's the complete audio-related code:

```rust
// Sound handles
static mut SFX_HIT: u32 = 0;
static mut SFX_SCORE: u32 = 0;
static mut SFX_WIN: u32 = 0;

// In init():
SFX_HIT = rom_sound(b"hit".as_ptr(), 3);
SFX_SCORE = rom_sound(b"score".as_ptr(), 5);
SFX_WIN = rom_sound(b"win".as_ptr(), 3);

// Play on events:
play_sound(SFX_HIT, 0.5, pan);      // Paddle/wall hit
play_sound(SFX_SCORE, 0.6, pan);    // Point scored
play_sound(SFX_WIN, 0.8, 0.0);      // Game over
```

---

**Next:** [Part 8: Polish & Publishing](./08-polish.md) — Final touches and releasing your game.
