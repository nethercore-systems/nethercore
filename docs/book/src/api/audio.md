# Audio Functions

Sound effects and music playback with 16 channels.

## Loading Sounds

### load_sound

Loads a sound from WASM memory.

**Signature:**
```rust
fn load_sound(data_ptr: *const u8, byte_len: u32) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| data_ptr | `*const u8` | Pointer to PCM audio data |
| byte_len | `u32` | Size of data in bytes |

**Returns:** Sound handle (non-zero on success)

**Audio Format:** 22.05 kHz, 16-bit signed, mono PCM

**Constraints:** Init-only.

**Example:**
```rust
static JUMP_DATA: &[u8] = include_bytes!("jump.raw");
static mut JUMP_SFX: u32 = 0;

fn init() {
    unsafe {
        JUMP_SFX = load_sound(JUMP_DATA.as_ptr(), JUMP_DATA.len() as u32);
    }
}
```

**Note:** Prefer `rom_sound()` for sounds bundled in the ROM data pack.

---

## Sound Effects

### play_sound

Plays a sound on the next available channel.

**Signature:**
```rust
fn play_sound(sound: u32, volume: f32, pan: f32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| sound | `u32` | Sound handle |
| volume | `f32` | Volume (0.0-1.0) |
| pan | `f32` | Stereo pan (-1.0 = left, 0.0 = center, 1.0 = right) |

**Example:**
```rust
fn update() {
    if button_pressed(0, BUTTON_A) != 0 {
        play_sound(JUMP_SFX, 1.0, 0.0);
    }

    // Positional audio
    let dx = enemy.x - player.x;
    let pan = (dx / 20.0).clamp(-1.0, 1.0);
    let dist = ((enemy.x - player.x).powi(2) + (enemy.z - player.z).powi(2)).sqrt();
    let vol = (1.0 - dist / 50.0).max(0.0);
    play_sound(ENEMY_GROWL, vol, pan);
}
```

---

### channel_play

Plays a sound on a specific channel with loop control.

**Signature:**
```rust
fn channel_play(channel: u32, sound: u32, volume: f32, pan: f32, looping: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| channel | `u32` | Channel index (0-15) |
| sound | `u32` | Sound handle |
| volume | `f32` | Volume (0.0-1.0) |
| pan | `f32` | Stereo pan (-1.0 to 1.0) |
| looping | `u32` | 1 to loop, 0 for one-shot |

**Example:**
```rust
fn update() {
    // Engine sound on dedicated channel (looping)
    if vehicle.engine_on && !ENGINE_PLAYING {
        channel_play(0, ENGINE_SFX, 0.8, 0.0, 1);
        ENGINE_PLAYING = true;
    }

    // Adjust engine pitch based on speed
    if ENGINE_PLAYING {
        let vol = 0.5 + vehicle.speed * 0.005;
        channel_set(0, vol.min(1.0), 0.0);
    }
}
```

---

### channel_set

Updates volume and pan for a playing channel.

**Signature:**
```rust
fn channel_set(channel: u32, volume: f32, pan: f32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| channel | `u32` | Channel index (0-15) |
| volume | `f32` | New volume (0.0-1.0) |
| pan | `f32` | New stereo pan |

**Example:**
```rust
fn update() {
    // Fade out channel 0
    if fading {
        fade_vol -= delta_time() * 0.5;
        if fade_vol <= 0.0 {
            channel_stop(0);
            fading = false;
        } else {
            channel_set(0, fade_vol, 0.0);
        }
    }
}
```

---

### channel_stop

Stops playback on a channel.

**Signature:**
```rust
fn channel_stop(channel: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| channel | `u32` | Channel index (0-15) |

**Example:**
```rust
fn update() {
    if vehicle.engine_off {
        channel_stop(0);
        ENGINE_PLAYING = false;
    }
}
```

---

## Music

Music uses a dedicated stereo channel, separate from the 16 SFX channels.

### music_play

Plays background music (looping).

**Signature:**
```rust
fn music_play(sound: u32, volume: f32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| sound | `u32` | Sound handle |
| volume | `f32` | Volume (0.0-1.0) |

**Example:**
```rust
fn init() {
    unsafe {
        MENU_MUSIC = rom_sound(b"menu_bgm".as_ptr(), 8);
        GAME_MUSIC = rom_sound(b"game_bgm".as_ptr(), 8);
    }
}

fn start_game() {
    music_play(GAME_MUSIC, 0.7);
}
```

---

### music_stop

Stops the currently playing music.

**Signature:**
```rust
fn music_stop()
```

**Example:**
```rust
fn game_over() {
    music_stop();
    play_sound(GAME_OVER_SFX, 1.0, 0.0);
}
```

---

### music_set_volume

Changes the music volume.

**Signature:**
```rust
fn music_set_volume(volume: f32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| volume | `f32` | New volume (0.0-1.0) |

**Example:**
```rust
fn render() {
    // Duck music during dialogue
    if dialogue_active {
        music_set_volume(0.3);
    } else {
        music_set_volume(0.7);
    }
}
```

---

## Audio Architecture

- **16 SFX channels** (0-15) for sound effects
- **1 Music channel** (separate) for background music
- **22.05 kHz** sample rate, 16-bit mono PCM
- **Rollback-safe**: Audio state is part of rollback snapshots
- Per-frame audio generation with ring buffer

---

## Complete Example

```rust
static mut JUMP_SFX: u32 = 0;
static mut LAND_SFX: u32 = 0;
static mut COIN_SFX: u32 = 0;
static mut MUSIC: u32 = 0;
static mut AMBIENT: u32 = 0;

fn init() {
    unsafe {
        // Load sounds from ROM
        JUMP_SFX = rom_sound(b"jump".as_ptr(), 4);
        LAND_SFX = rom_sound(b"land".as_ptr(), 4);
        COIN_SFX = rom_sound(b"coin".as_ptr(), 4);
        MUSIC = rom_sound(b"level1".as_ptr(), 6);
        AMBIENT = rom_sound(b"wind".as_ptr(), 4);

        // Start music and ambient
        music_play(MUSIC, 0.6);
        channel_play(15, AMBIENT, 0.3, 0.0, 1); // Looping ambient
    }
}

fn update() {
    unsafe {
        // Jump sound
        if button_pressed(0, BUTTON_A) != 0 && player.on_ground {
            play_sound(JUMP_SFX, 0.8, 0.0);
        }

        // Land sound
        if player.just_landed {
            play_sound(LAND_SFX, 0.6, 0.0);
        }

        // Coin pickup with positional audio
        for coin in &coins {
            if coin.just_collected {
                let dx = coin.x - player.x;
                let pan = (dx / 10.0).clamp(-1.0, 1.0);
                play_sound(COIN_SFX, 1.0, pan);
            }
        }

        // Pause menu - duck audio
        if game_paused {
            music_set_volume(0.2);
            channel_set(15, 0.1, 0.0);
        } else {
            music_set_volume(0.6);
            channel_set(15, 0.3, 0.0);
        }
    }
}
```

**See Also:** [rom_sound](./rom-loading.md#rom_sound)
