# Part 7: Sound Effects

Games feel incomplete without audio. Let's add satisfying sound effects.

## What You'll Learn

- Generating sounds procedurally
- Loading sounds with `load_sound()`
- Playing sounds with `play_sound()` and stereo panning
- When to play sounds for best game feel

## Add Audio FFI

```rust
#[link(wasm_import_module = "env")]
extern "C" {
    // ... existing imports ...
    fn load_sound(data_ptr: *const i16, byte_len: u32) -> u32;
    fn play_sound(sound: u32, volume: f32, pan: f32);
}
```

## Sound Handles

Add static variables for sound handles:

```rust
static mut SFX_HIT: u32 = 0;
static mut SFX_SCORE: u32 = 0;
static mut SFX_WIN: u32 = 0;
```

## Generate Sounds Procedurally

Instead of loading audio files, we'll generate simple sounds with code. This keeps our game self-contained.

### Hit Sound (Short Beep)

```rust
fn generate_hit_sound() -> [i16; 2205] {
    let mut samples = [0i16; 2205];  // 0.1 seconds at 22050 Hz
    let frequency = 440.0;
    let sample_rate = 22050.0;

    for i in 0..2205 {
        let t = i as f32 / sample_rate;
        // Quick decay envelope
        let envelope = 1.0 - (i as f32 / 2205.0);
        let value = libm::sinf(2.0 * core::f32::consts::PI * frequency * t) * envelope;
        samples[i] = (value * 32767.0 * 0.3) as i16;
    }
    samples
}
```

### Score Sound (Descending Tone)

```rust
fn generate_score_sound() -> [i16; 4410] {
    let mut samples = [0i16; 4410];  // 0.2 seconds
    let sample_rate = 22050.0;

    for i in 0..4410 {
        let t = i as f32 / sample_rate;
        let progress = i as f32 / 4410.0;
        // Frequency slides from 880 Hz down to 220 Hz
        let frequency = 880.0 - (660.0 * progress);
        let envelope = 1.0 - progress;
        let value = libm::sinf(2.0 * core::f32::consts::PI * frequency * t) * envelope;
        samples[i] = (value * 32767.0 * 0.3) as i16;
    }
    samples
}
```

### Win Sound (Victory Fanfare)

```rust
fn generate_win_sound() -> [i16; 11025] {
    let mut samples = [0i16; 11025];  // 0.5 seconds
    let sample_rate = 22050.0;

    for i in 0..11025 {
        let t = i as f32 / sample_rate;
        let progress = i as f32 / 11025.0;

        // Three ascending notes (C-E-G chord arpeggio)
        let frequency = if progress < 0.33 {
            523.25  // C5
        } else if progress < 0.66 {
            659.25  // E5
        } else {
            783.99  // G5
        };

        let envelope = 1.0 - (progress * 0.5);
        let value = libm::sinf(2.0 * core::f32::consts::PI * frequency * t) * envelope;
        samples[i] = (value * 32767.0 * 0.3) as i16;
    }
    samples
}
```

## Load Sounds in Init

Update `init()` to generate and load sounds:

```rust
#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eFF);

        // Generate and load sounds
        let hit_samples = generate_hit_sound();
        SFX_HIT = load_sound(hit_samples.as_ptr(), (hit_samples.len() * 2) as u32);

        let score_samples = generate_score_sound();
        SFX_SCORE = load_sound(score_samples.as_ptr(), (score_samples.len() * 2) as u32);

        let win_samples = generate_win_sound();
        SFX_WIN = load_sound(win_samples.as_ptr(), (win_samples.len() * 2) as u32);

        reset_game();
        STATE = GameState::Title;
    }
}
```

## Play Sounds

Add sound effects to game events:

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
// When player 2 scores:
SCORE2 += 1;
play_sound(SFX_SCORE, 0.6, 0.5);  // Pan right (their side)

// When player 1 scores:
SCORE1 += 1;
play_sound(SFX_SCORE, 0.6, -0.5);  // Pan left (their side)
```

### Win
```rust
// When either player wins:
if SCORE1 >= WIN_SCORE || SCORE2 >= WIN_SCORE {
    STATE = GameState::GameOver;
    play_sound(SFX_WIN, 0.8, 0.0);  // Center, louder
}
```

## Understanding Audio Parameters

### `load_sound(data_ptr, byte_len)`
- `data_ptr`: Pointer to 16-bit PCM audio samples
- `byte_len`: Size in bytes (samples * 2 because i16 = 2 bytes)
- Returns: Handle to the loaded sound

### `play_sound(handle, volume, pan)`
- `handle`: Sound handle from `load_sound()`
- `volume`: 0.0 (silent) to 1.0 (full volume)
- `pan`: -1.0 (left) to 1.0 (right), 0.0 = center

### Sample Rate
Emberware uses 22050 Hz sample rate. This is lower than CD quality (44100 Hz) but:
- Uses half the memory
- Sounds fine for retro-style games
- Matches the aesthetic of older consoles

## Sound Design Tips

1. **Keep sounds short** - 0.1 to 0.5 seconds is plenty
2. **Use panning** - Stereo positioning helps players track the action
3. **Vary volume** - Important sounds louder, ambient sounds quieter
4. **Match the aesthetic** - Simple synthesized sounds fit retro games

## Build and Test

```bash
cargo build --target wasm32-unknown-unknown --release
ember run target/wasm32-unknown-unknown/release/paddle.wasm
```

The game now has:
- "Ping" sound when ball hits walls or paddles
- Different sounds for left/right paddle (stereo panning)
- Descending "whomp" when someone scores
- Victory fanfare when a player wins

---

**Next:** [Part 8: Polish & Publishing](./08-polish.md) - Final touches and releasing your game.
