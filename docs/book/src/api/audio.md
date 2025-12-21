# Audio Functions

Sound effects and music playback with 16 channels.

## Loading Sounds

### load_sound

Loads a sound from WASM memory.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn load_sound(data_ptr: *const u8, byte_len: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT uint32_t load_sound(const uint8_t* data_ptr, uint32_t byte_len);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn load_sound(data_ptr: [*]const u8, byte_len: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| data_ptr | `*const u8` | Pointer to PCM audio data |
| byte_len | `u32` | Size of data in bytes |

**Returns:** Sound handle (non-zero on success)

**Audio Format:** 22.05 kHz, 16-bit signed, mono PCM

**Constraints:** Init-only.

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static JUMP_DATA: &[u8] = include_bytes!("jump.raw");
static mut JUMP_SFX: u32 = 0;

fn init() {
    unsafe {
        JUMP_SFX = load_sound(JUMP_DATA.as_ptr(), JUMP_DATA.len() as u32);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static const uint8_t jump_data[] = {
    #include "jump.raw.h"
};
static uint32_t jump_sfx = 0;

EWZX_EXPORT void init() {
    jump_sfx = load_sound(jump_data, sizeof(jump_data));
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const jump_data = @embedFile("jump.raw");
var jump_sfx: u32 = 0;

export fn init() void {
    jump_sfx = load_sound(jump_data.ptr, jump_data.len);
}
```
{{#endtab}}

{{#endtabs}}

**Note:** Prefer `rom_sound()` for sounds bundled in the ROM data pack.

---

## Sound Effects

### play_sound

Plays a sound on the next available channel.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn play_sound(sound: u32, volume: f32, pan: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void play_sound(uint32_t sound, float volume, float pan);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn play_sound(sound: u32, volume: f32, pan: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| sound | `u32` | Sound handle |
| volume | `f32` | Volume (0.0-1.0) |
| pan | `f32` | Stereo pan (-1.0 = left, 0.0 = center, 1.0 = right) |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void update() {
    if (button_pressed(0, BUTTON_A) != 0) {
        play_sound(JUMP_SFX, 1.0f, 0.0f);
    }

    // Positional audio
    float dx = enemy.x - player.x;
    float pan = fmaxf(-1.0f, fminf(1.0f, dx / 20.0f));
    float ex = enemy.x - player.x;
    float ez = enemy.z - player.z;
    float dist = sqrtf(ex * ex + ez * ez);
    float vol = fmaxf(0.0f, 1.0f - dist / 50.0f);
    play_sound(ENEMY_GROWL, vol, pan);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    if (button_pressed(0, BUTTON_A) != 0) {
        play_sound(JUMP_SFX, 1.0, 0.0);
    }

    // Positional audio
    const dx = enemy.x - player.x;
    const pan = @max(-1.0, @min(1.0, dx / 20.0));
    const ex = enemy.x - player.x;
    const ez = enemy.z - player.z;
    const dist = @sqrt(ex * ex + ez * ez);
    const vol = @max(0.0, 1.0 - dist / 50.0);
    play_sound(ENEMY_GROWL, vol, pan);
}
```
{{#endtab}}

{{#endtabs}}

---

### channel_play

Plays a sound on a specific channel with loop control.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn channel_play(channel: u32, sound: u32, volume: f32, pan: f32, looping: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void channel_play(uint32_t channel, uint32_t sound, float volume, float pan, uint32_t looping);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn channel_play(channel: u32, sound: u32, volume: f32, pan: f32, looping: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| channel | `u32` | Channel index (0-15) |
| sound | `u32` | Sound handle |
| volume | `f32` | Volume (0.0-1.0) |
| pan | `f32` | Stereo pan (-1.0 to 1.0) |
| looping | `u32` | 1 to loop, 0 for one-shot |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void update() {
    // Engine sound on dedicated channel (looping)
    if (vehicle.engine_on && !ENGINE_PLAYING) {
        channel_play(0, ENGINE_SFX, 0.8f, 0.0f, 1);
        ENGINE_PLAYING = true;
    }

    // Adjust engine pitch based on speed
    if (ENGINE_PLAYING) {
        float vol = 0.5f + vehicle.speed * 0.005f;
        channel_set(0, fminf(vol, 1.0f), 0.0f);
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    // Engine sound on dedicated channel (looping)
    if (vehicle.engine_on and !ENGINE_PLAYING) {
        channel_play(0, ENGINE_SFX, 0.8, 0.0, 1);
        ENGINE_PLAYING = true;
    }

    // Adjust engine pitch based on speed
    if (ENGINE_PLAYING) {
        const vol = 0.5 + vehicle.speed * 0.005;
        channel_set(0, @min(vol, 1.0), 0.0);
    }
}
```
{{#endtab}}

{{#endtabs}}

---

### channel_set

Updates volume and pan for a playing channel.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn channel_set(channel: u32, volume: f32, pan: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void channel_set(uint32_t channel, float volume, float pan);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn channel_set(channel: u32, volume: f32, pan: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| channel | `u32` | Channel index (0-15) |
| volume | `f32` | New volume (0.0-1.0) |
| pan | `f32` | New stereo pan |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void update() {
    // Fade out channel 0
    if (fading) {
        fade_vol -= delta_time() * 0.5f;
        if (fade_vol <= 0.0f) {
            channel_stop(0);
            fading = false;
        } else {
            channel_set(0, fade_vol, 0.0f);
        }
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    // Fade out channel 0
    if (fading) {
        fade_vol -= delta_time() * 0.5;
        if (fade_vol <= 0.0) {
            channel_stop(0);
            fading = false;
        } else {
            channel_set(0, fade_vol, 0.0);
        }
    }
}
```
{{#endtab}}

{{#endtabs}}

---

### channel_stop

Stops playback on a channel.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn channel_stop(channel: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void channel_stop(uint32_t channel);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn channel_stop(channel: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| channel | `u32` | Channel index (0-15) |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn update() {
    if vehicle.engine_off {
        channel_stop(0);
        ENGINE_PLAYING = false;
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void update() {
    if (vehicle.engine_off) {
        channel_stop(0);
        ENGINE_PLAYING = false;
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn update() void {
    if (vehicle.engine_off) {
        channel_stop(0);
        ENGINE_PLAYING = false;
    }
}
```
{{#endtab}}

{{#endtabs}}

---

## Music

Music uses a dedicated stereo channel, separate from the 16 SFX channels.

### music_play

Plays background music (looping).

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn music_play(sound: u32, volume: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void music_play(uint32_t sound, float volume);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn music_play(sound: u32, volume: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| sound | `u32` | Sound handle |
| volume | `f32` | Volume (0.0-1.0) |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void init() {
    MENU_MUSIC = rom_sound("menu_bgm", 8);
    GAME_MUSIC = rom_sound("game_bgm", 8);
}

void start_game() {
    music_play(GAME_MUSIC, 0.7f);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn init() void {
    MENU_MUSIC = rom_sound("menu_bgm".ptr, 8);
    GAME_MUSIC = rom_sound("game_bgm".ptr, 8);
}

fn start_game() void {
    music_play(GAME_MUSIC, 0.7);
}
```
{{#endtab}}

{{#endtabs}}

---

### music_stop

Stops the currently playing music.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn music_stop()
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void music_stop();
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn music_stop() void;
```
{{#endtab}}

{{#endtabs}}

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn game_over() {
    music_stop();
    play_sound(GAME_OVER_SFX, 1.0, 0.0);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void game_over() {
    music_stop();
    play_sound(GAME_OVER_SFX, 1.0f, 0.0f);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
fn game_over() void {
    music_stop();
    play_sound(GAME_OVER_SFX, 1.0, 0.0);
}
```
{{#endtab}}

{{#endtabs}}

---

### music_set_volume

Changes the music volume.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn music_set_volume(volume: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void music_set_volume(float volume);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn music_set_volume(volume: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| volume | `f32` | New volume (0.0-1.0) |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render() {
    // Duck music during dialogue
    if (dialogue_active) {
        music_set_volume(0.3f);
    } else {
        music_set_volume(0.7f);
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Duck music during dialogue
    if (dialogue_active) {
        music_set_volume(0.3);
    } else {
        music_set_volume(0.7);
    }
}
```
{{#endtab}}

{{#endtabs}}

---

## Audio Architecture

- **16 SFX channels** (0-15) for sound effects
- **1 Music channel** (separate) for background music
- **22.05 kHz** sample rate, 16-bit mono PCM
- **Rollback-safe**: Audio state is part of rollback snapshots
- Per-frame audio generation with ring buffer

---

## Complete Example

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t JUMP_SFX = 0;
static uint32_t LAND_SFX = 0;
static uint32_t COIN_SFX = 0;
static uint32_t MUSIC = 0;
static uint32_t AMBIENT = 0;

EWZX_EXPORT void init() {
    // Load sounds from ROM
    JUMP_SFX = rom_sound("jump", 4);
    LAND_SFX = rom_sound("land", 4);
    COIN_SFX = rom_sound("coin", 4);
    MUSIC = rom_sound("level1", 6);
    AMBIENT = rom_sound("wind", 4);

    // Start music and ambient
    music_play(MUSIC, 0.6f);
    channel_play(15, AMBIENT, 0.3f, 0.0f, 1); // Looping ambient
}

EWZX_EXPORT void update() {
    // Jump sound
    if (button_pressed(0, BUTTON_A) != 0 && player.on_ground) {
        play_sound(JUMP_SFX, 0.8f, 0.0f);
    }

    // Land sound
    if (player.just_landed) {
        play_sound(LAND_SFX, 0.6f, 0.0f);
    }

    // Coin pickup with positional audio
    for (int i = 0; i < coin_count; i++) {
        if (coins[i].just_collected) {
            float dx = coins[i].x - player.x;
            float pan = fmaxf(-1.0f, fminf(1.0f, dx / 10.0f));
            play_sound(COIN_SFX, 1.0f, pan);
        }
    }

    // Pause menu - duck audio
    if (game_paused) {
        music_set_volume(0.2f);
        channel_set(15, 0.1f, 0.0f);
    } else {
        music_set_volume(0.6f);
        channel_set(15, 0.3f, 0.0f);
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var JUMP_SFX: u32 = 0;
var LAND_SFX: u32 = 0;
var COIN_SFX: u32 = 0;
var MUSIC: u32 = 0;
var AMBIENT: u32 = 0;

export fn init() void {
    // Load sounds from ROM
    JUMP_SFX = rom_sound("jump".ptr, 4);
    LAND_SFX = rom_sound("land".ptr, 4);
    COIN_SFX = rom_sound("coin".ptr, 4);
    MUSIC = rom_sound("level1".ptr, 6);
    AMBIENT = rom_sound("wind".ptr, 4);

    // Start music and ambient
    music_play(MUSIC, 0.6);
    channel_play(15, AMBIENT, 0.3, 0.0, 1); // Looping ambient
}

export fn update() void {
    // Jump sound
    if (button_pressed(0, BUTTON_A) != 0 and player.on_ground) {
        play_sound(JUMP_SFX, 0.8, 0.0);
    }

    // Land sound
    if (player.just_landed) {
        play_sound(LAND_SFX, 0.6, 0.0);
    }

    // Coin pickup with positional audio
    for (coins) |coin| {
        if (coin.just_collected) {
            const dx = coin.x - player.x;
            const pan = @max(-1.0, @min(1.0, dx / 10.0));
            play_sound(COIN_SFX, 1.0, pan);
        }
    }

    // Pause menu - duck audio
    if (game_paused) {
        music_set_volume(0.2);
        channel_set(15, 0.1, 0.0);
    } else {
        music_set_volume(0.6);
        channel_set(15, 0.3, 0.0);
    }
}
```
{{#endtab}}

{{#endtabs}}

**See Also:** [rom_sound](./rom-loading.md#rom_sound)
