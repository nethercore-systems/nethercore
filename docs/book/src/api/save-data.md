# Save Data Functions

Persistent storage for game saves (8 slots, 64KB each).

## Overview

- **8 save slots** (indices 0-7)
- **Slots 0-3** are persistent and controller-backed (per-machine, per-game)
- **Slots 4-7** are reserved and ephemeral (in-memory only)
- **64KB maximum** per slot
- **Netplay-safe:** only local session slots persist; remote players cannot overwrite your save files

---

## Functions

### save

Saves data to a slot.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn save(slot: u32, data_ptr: *const u8, data_len: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t save(uint32_t slot, const uint8_t* data_ptr, uint32_t data_len);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn save(slot: u32, data_ptr: [*]const u8, data_len: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| slot | `u32` | Save slot (0-7). Slots 0-3 persist locally for local players; slots 4-7 are ephemeral |
| data_ptr | `*const u8` | Pointer to data to save |
| data_len | `u32` | Size of data in bytes |

**Returns:**

| Value | Meaning |
|-------|---------|
| 0 | Success |
| 1 | Invalid slot |
| 2 | Data too large (>64KB) |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn save_game() {
    unsafe {
        let save_data = SaveData {
            level: CURRENT_LEVEL,
            score: SCORE,
            health: PLAYER_HEALTH,
            position_x: PLAYER_X,
            position_y: PLAYER_Y,
            checksum: 0,
        };

        // Calculate checksum
        let bytes = &save_data as *const SaveData as *const u8;
        let size = core::mem::size_of::<SaveData>();

        let result = save(0, bytes, size as u32);
        if result == 0 {
            show_message(b"Game Saved!");
        }
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void save_game() {
    SaveData save_data = {
        .level = CURRENT_LEVEL,
        .score = SCORE,
        .health = PLAYER_HEALTH,
        .position_x = PLAYER_X,
        .position_y = PLAYER_Y,
        .checksum = 0
    };

    // Calculate checksum
    const uint8_t* bytes = (const uint8_t*)&save_data;
    uint32_t size = sizeof(SaveData);

    uint32_t result = save(0, bytes, size);
    if (result == 0) {
        show_message((const uint8_t*)"Game Saved!", 11);
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
fn save_game() void {
    var save_data = SaveData{
        .level = CURRENT_LEVEL,
        .score = SCORE,
        .health = PLAYER_HEALTH,
        .position_x = PLAYER_X,
        .position_y = PLAYER_Y,
        .checksum = 0,
    };

    // Calculate checksum
    const bytes: [*]const u8 = @ptrCast(&save_data);
    const size = @sizeOf(SaveData);

    const result = save(0, bytes, size);
    if (result == 0) {
        show_message("Game Saved!".ptr, 11);
    }
}
```
{{#endtab}}

{{#endtabs}}

---

### load

Loads data from a slot.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn load(slot: u32, data_ptr: *mut u8, max_len: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t load(uint32_t slot, uint8_t* data_ptr, uint32_t max_len);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn load(slot: u32, data_ptr: [*]u8, max_len: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| slot | `u32` | Save slot (0-7). Slots 0-3 persist locally for local players; slots 4-7 are ephemeral |
| data_ptr | `*mut u8` | Destination buffer |
| max_len | `u32` | Maximum bytes to read |

**Returns:** Number of bytes read (0 if empty or error)

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn load_game() -> bool {
    unsafe {
        let mut save_data = SaveData::default();
        let bytes = &mut save_data as *mut SaveData as *mut u8;
        let size = core::mem::size_of::<SaveData>();

        let read = load(0, bytes, size as u32);
        if read == size as u32 {
            // Validate checksum
            if validate_checksum(&save_data) {
                CURRENT_LEVEL = save_data.level;
                SCORE = save_data.score;
                PLAYER_HEALTH = save_data.health;
                PLAYER_X = save_data.position_x;
                PLAYER_Y = save_data.position_y;
                return true;
            }
        }
        false
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
bool load_game() {
    SaveData save_data = {0};
    uint8_t* bytes = (uint8_t*)&save_data;
    uint32_t size = sizeof(SaveData);

    uint32_t read = load(0, bytes, size);
    if (read == size) {
        // Validate checksum
        if (validate_checksum(&save_data)) {
            CURRENT_LEVEL = save_data.level;
            SCORE = save_data.score;
            PLAYER_HEALTH = save_data.health;
            PLAYER_X = save_data.position_x;
            PLAYER_Y = save_data.position_y;
            return true;
        }
    }
    return false;
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
fn load_game() bool {
    var save_data: SaveData = std.mem.zeroes(SaveData);
    const bytes: [*]u8 = @ptrCast(&save_data);
    const size = @sizeOf(SaveData);

    const read = load(0, bytes, size);
    if (read == size) {
        // Validate checksum
        if (validate_checksum(&save_data)) {
            CURRENT_LEVEL = save_data.level;
            SCORE = save_data.score;
            PLAYER_HEALTH = save_data.health;
            PLAYER_X = save_data.position_x;
            PLAYER_Y = save_data.position_y;
            return true;
        }
    }
    return false;
}
```
{{#endtab}}

{{#endtabs}}

---

### delete

Deletes data in a save slot.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn delete(slot: u32) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t delete_save(uint32_t slot);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn delete_save(slot: u32) u32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| slot | `u32` | Save slot (0-7). Slots 0-3 persist locally for local players; slots 4-7 are ephemeral |

**Returns:**

| Value | Meaning |
|-------|---------|
| 0 | Success |
| 1 | Invalid slot |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn delete_save(slot: u32) {
    unsafe {
        if delete(slot) == 0 {
            show_message(b"Save deleted");
        }
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void delete_saved_game(uint32_t slot) {
    if (delete_save(slot) == 0) {
        show_message((const uint8_t*)"Save deleted", 12);
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
fn delete_saved_game(slot: u32) void {
    if (delete_save(slot) == 0) {
        show_message("Save deleted".ptr, 12);
    }
}
```
{{#endtab}}

{{#endtabs}}

---

## Save Data Patterns

### Simple Struct Save

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[repr(C)]
struct SaveData {
    magic: u32,           // Identify valid saves
    version: u32,         // Save format version
    level: u32,
    score: u32,
    health: f32,
    position: [f32; 3],
    inventory: [u8; 64],
    checksum: u32,
}

impl SaveData {
    const MAGIC: u32 = 0x53415645; // "SAVE"

    fn new() -> Self {
        Self {
            magic: Self::MAGIC,
            version: 1,
            level: 0,
            score: 0,
            health: 100.0,
            position: [0.0; 3],
            inventory: [0; 64],
            checksum: 0,
        }
    }

    fn calculate_checksum(&self) -> u32 {
        // Simple checksum (XOR all bytes except checksum field)
        let bytes = self as *const Self as *const u8;
        let len = core::mem::size_of::<Self>() - 4; // Exclude checksum
        let mut sum: u32 = 0;
        for i in 0..len {
            unsafe { sum ^= (*bytes.add(i) as u32) << ((i % 4) * 8); }
        }
        sum
    }

    fn is_valid(&self) -> bool {
        self.magic == Self::MAGIC && self.checksum == self.calculate_checksum()
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
typedef struct {
    uint32_t magic;           // Identify valid saves
    uint32_t version;         // Save format version
    uint32_t level;
    uint32_t score;
    float health;
    float position[3];
    uint8_t inventory[64];
    uint32_t checksum;
} SaveData;

#define SAVE_MAGIC 0x53415645  // "SAVE"

SaveData SaveData_new() {
    SaveData data = {0};
    data.magic = SAVE_MAGIC;
    data.version = 1;
    data.health = 100.0f;
    return data;
}

uint32_t SaveData_calculate_checksum(const SaveData* self) {
    // Simple checksum (XOR all bytes except checksum field)
    const uint8_t* bytes = (const uint8_t*)self;
    uint32_t len = sizeof(SaveData) - 4; // Exclude checksum
    uint32_t sum = 0;
    for (uint32_t i = 0; i < len; i++) {
        sum ^= (bytes[i] << ((i % 4) * 8));
    }
    return sum;
}

bool SaveData_is_valid(const SaveData* self) {
    return self->magic == SAVE_MAGIC &&
           self->checksum == SaveData_calculate_checksum(self);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const SaveData = extern struct {
    magic: u32,           // Identify valid saves
    version: u32,         // Save format version
    level: u32,
    score: u32,
    health: f32,
    position: [3]f32,
    inventory: [64]u8,
    checksum: u32,

    const MAGIC: u32 = 0x53415645; // "SAVE"

    fn new() SaveData {
        return SaveData{
            .magic = MAGIC,
            .version = 1,
            .level = 0,
            .score = 0,
            .health = 100.0,
            .position = [_]f32{0.0} ** 3,
            .inventory = [_]u8{0} ** 64,
            .checksum = 0,
        };
    }

    fn calculate_checksum(self: *const SaveData) u32 {
        // Simple checksum (XOR all bytes except checksum field)
        const bytes: [*]const u8 = @ptrCast(self);
        const len = @sizeOf(SaveData) - 4; // Exclude checksum
        var sum: u32 = 0;
        for (0..len) |i| {
            sum ^= @as(u32, bytes[i]) << @intCast((i % 4) * 8);
        }
        return sum;
    }

    fn is_valid(self: *const SaveData) bool {
        return self.magic == MAGIC and
               self.checksum == self.calculate_checksum();
    }
};
```
{{#endtab}}

{{#endtabs}}

### Multiple Save Slots UI

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render_save_menu() {
    unsafe {
        draw_text(b"SAVE SLOTS".as_ptr(), 10, 200.0, 50.0, 24.0, 0xFFFFFFFF);

        for slot in 0..8 {
            let mut buffer = [0u8; 128];
            let read = load(slot, buffer.as_mut_ptr(), 128);

            let y = 100.0 + (slot as f32) * 40.0;

            if read > 0 {
                // Parse save info
                let save = &*(buffer.as_ptr() as *const SaveData);
                if save.is_valid() {
                    // Show save info
                    let text = format_save_info(slot, save.level, save.score);
                    draw_text(text.as_ptr(), text.len() as u32, 100.0, y, 16.0, 0xFFFFFFFF);
                } else {
                    draw_text(b"[Corrupted]".as_ptr(), 11, 100.0, y, 16.0, 0xFF4444FF);
                }
            } else {
                draw_text(b"[Empty]".as_ptr(), 7, 100.0, y, 16.0, 0x888888FF);
            }

            // Highlight selected slot
            if slot == SELECTED_SLOT {
                draw_rect(90.0, y - 5.0, 300.0, 30.0, 0xFFFFFF33);
            }
        }
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void render_save_menu() {
    draw_text((const uint8_t*)"SAVE SLOTS", 10, 200.0f, 50.0f, 24.0f, 0xFFFFFFFF);

    for (uint32_t slot = 0; slot < 8; slot++) {
        uint8_t buffer[128] = {0};
        uint32_t read = load(slot, buffer, 128);

        float y = 100.0f + (slot * 40.0f);

        if (read > 0) {
            // Parse save info
            const SaveData* save = (const SaveData*)buffer;
            if (SaveData_is_valid(save)) {
                // Show save info
                char text[64];
                format_save_info(text, slot, save->level, save->score);
                draw_text((const uint8_t*)text, strlen(text), 100.0f, y, 16.0f, 0xFFFFFFFF);
            } else {
                draw_text((const uint8_t*)"[Corrupted]", 11, 100.0f, y, 16.0f, 0xFF4444FF);
            }
        } else {
            draw_text((const uint8_t*)"[Empty]", 7, 100.0f, y, 16.0f, 0x888888FF);
        }

        // Highlight selected slot
        if (slot == SELECTED_SLOT) {
            draw_rect(90.0f, y - 5.0f, 300.0f, 30.0f, 0xFFFFFF33);
        }
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
fn render_save_menu() void {
    draw_text("SAVE SLOTS".ptr, 10, 200.0, 50.0, 24.0, 0xFFFFFFFF);

    for (0..8) |slot| {
        var buffer: [128]u8 = undefined;
        const read = load(@intCast(slot), &buffer, 128);

        const y = 100.0 + (@as(f32, @floatFromInt(slot)) * 40.0);

        if (read > 0) {
            // Parse save info
            const save: *const SaveData = @ptrCast(@alignCast(&buffer));
            if (save.is_valid()) {
                // Show save info
                const text = format_save_info(slot, save.level, save.score);
                draw_text(text.ptr, @intCast(text.len), 100.0, y, 16.0, 0xFFFFFFFF);
            } else {
                draw_text("[Corrupted]".ptr, 11, 100.0, y, 16.0, 0xFF4444FF);
            }
        } else {
            draw_text("[Empty]".ptr, 7, 100.0, y, 16.0, 0x888888FF);
        }

        // Highlight selected slot
        if (slot == SELECTED_SLOT) {
            draw_rect(90.0, y - 5.0, 300.0, 30.0, 0xFFFFFF33);
        }
    }
}
```
{{#endtab}}

{{#endtabs}}

### Auto-Save

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut LAST_SAVE_TIME: f32 = 0.0;
const AUTO_SAVE_INTERVAL: f32 = 60.0; // Every 60 seconds

fn update() {
    unsafe {
        // Auto-save every 60 seconds
        if elapsed_time() - LAST_SAVE_TIME > AUTO_SAVE_INTERVAL {
            auto_save();
            LAST_SAVE_TIME = elapsed_time();
        }
    }
}

fn auto_save() {
    unsafe {
        // Use slot 7 for auto-save
        let mut save = create_save_data();
        save.checksum = save.calculate_checksum();

        let bytes = &save as *const SaveData as *const u8;
        let size = core::mem::size_of::<SaveData>();
        save(7, bytes, size as u32);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static float LAST_SAVE_TIME = 0.0f;
const float AUTO_SAVE_INTERVAL = 60.0f; // Every 60 seconds

NCZX_EXPORT void update() {
    // Auto-save every 60 seconds
    if (elapsed_time() - LAST_SAVE_TIME > AUTO_SAVE_INTERVAL) {
        auto_save();
        LAST_SAVE_TIME = elapsed_time();
    }
}

void auto_save() {
    // Use slot 7 for auto-save
    SaveData save_data = create_save_data();
    save_data.checksum = SaveData_calculate_checksum(&save_data);

    const uint8_t* bytes = (const uint8_t*)&save_data;
    uint32_t size = sizeof(SaveData);
    save(7, bytes, size);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var LAST_SAVE_TIME: f32 = 0.0;
const AUTO_SAVE_INTERVAL: f32 = 60.0; // Every 60 seconds

export fn update() void {
    // Auto-save every 60 seconds
    if (elapsed_time() - LAST_SAVE_TIME > AUTO_SAVE_INTERVAL) {
        auto_save();
        LAST_SAVE_TIME = elapsed_time();
    }
}

fn auto_save() void {
    // Use slot 7 for auto-save
    var save_data = create_save_data();
    save_data.checksum = save_data.calculate_checksum();

    const bytes: [*]const u8 = @ptrCast(&save_data);
    const size = @sizeOf(SaveData);
    save(7, bytes, size);
}
```
{{#endtab}}

{{#endtabs}}

---

## Complete Example

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
#[repr(C)]
struct GameSave {
    magic: u32,
    version: u32,
    level: u32,
    score: u32,
    lives: u32,
    player_x: f32,
    player_y: f32,
    unlocked_levels: u32,  // Bitmask
    high_scores: [u32; 10],
    checksum: u32,
}

static mut CURRENT_SAVE: GameSave = GameSave {
    magic: 0x47414D45,
    version: 1,
    level: 1,
    score: 0,
    lives: 3,
    player_x: 0.0,
    player_y: 0.0,
    unlocked_levels: 1,
    high_scores: [0; 10],
    checksum: 0,
};

fn save_to_slot(slot: u32) -> bool {
    unsafe {
        CURRENT_SAVE.checksum = calculate_checksum(&CURRENT_SAVE);
        let bytes = &CURRENT_SAVE as *const GameSave as *const u8;
        let size = core::mem::size_of::<GameSave>();
        save(slot, bytes, size as u32) == 0
    }
}

fn load_from_slot(slot: u32) -> bool {
    unsafe {
        let bytes = &mut CURRENT_SAVE as *mut GameSave as *mut u8;
        let size = core::mem::size_of::<GameSave>();
        let read = load(slot, bytes, size as u32);

        if read == size as u32 {
            let expected = calculate_checksum(&CURRENT_SAVE);
            if CURRENT_SAVE.checksum == expected && CURRENT_SAVE.magic == 0x47414D45 {
                return true;
            }
        }
        // Reset to defaults on failure
        CURRENT_SAVE = GameSave::default();
        false
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
typedef struct {
    uint32_t magic;
    uint32_t version;
    uint32_t level;
    uint32_t score;
    uint32_t lives;
    float player_x;
    float player_y;
    uint32_t unlocked_levels;  // Bitmask
    uint32_t high_scores[10];
    uint32_t checksum;
} GameSave;

static GameSave CURRENT_SAVE = {
    .magic = 0x47414D45,
    .version = 1,
    .level = 1,
    .score = 0,
    .lives = 3,
    .player_x = 0.0f,
    .player_y = 0.0f,
    .unlocked_levels = 1,
    .high_scores = {0},
    .checksum = 0,
};

bool save_to_slot(uint32_t slot) {
    CURRENT_SAVE.checksum = calculate_checksum(&CURRENT_SAVE);
    const uint8_t* bytes = (const uint8_t*)&CURRENT_SAVE;
    uint32_t size = sizeof(GameSave);
    return save(slot, bytes, size) == 0;
}

bool load_from_slot(uint32_t slot) {
    uint8_t* bytes = (uint8_t*)&CURRENT_SAVE;
    uint32_t size = sizeof(GameSave);
    uint32_t read = load(slot, bytes, size);

    if (read == size) {
        uint32_t expected = calculate_checksum(&CURRENT_SAVE);
        if (CURRENT_SAVE.checksum == expected && CURRENT_SAVE.magic == 0x47414D45) {
            return true;
        }
    }
    // Reset to defaults on failure
    CURRENT_SAVE = (GameSave){
        .magic = 0x47414D45,
        .version = 1,
        .level = 1,
        .lives = 3,
        .unlocked_levels = 1,
    };
    return false;
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const GameSave = extern struct {
    magic: u32,
    version: u32,
    level: u32,
    score: u32,
    lives: u32,
    player_x: f32,
    player_y: f32,
    unlocked_levels: u32,  // Bitmask
    high_scores: [10]u32,
    checksum: u32,
};

var CURRENT_SAVE: GameSave = GameSave{
    .magic = 0x47414D45,
    .version = 1,
    .level = 1,
    .score = 0,
    .lives = 3,
    .player_x = 0.0,
    .player_y = 0.0,
    .unlocked_levels = 1,
    .high_scores = [_]u32{0} ** 10,
    .checksum = 0,
};

fn save_to_slot(slot: u32) bool {
    CURRENT_SAVE.checksum = calculate_checksum(&CURRENT_SAVE);
    const bytes: [*]const u8 = @ptrCast(&CURRENT_SAVE);
    const size = @sizeOf(GameSave);
    return save(slot, bytes, size) == 0;
}

fn load_from_slot(slot: u32) bool {
    const bytes: [*]u8 = @ptrCast(&CURRENT_SAVE);
    const size = @sizeOf(GameSave);
    const read = load(slot, bytes, size);

    if (read == size) {
        const expected = calculate_checksum(&CURRENT_SAVE);
        if (CURRENT_SAVE.checksum == expected and CURRENT_SAVE.magic == 0x47414D45) {
            return true;
        }
    }
    // Reset to defaults on failure
    CURRENT_SAVE = GameSave{
        .magic = 0x47414D45,
        .version = 1,
        .level = 1,
        .score = 0,
        .lives = 3,
        .player_x = 0.0,
        .player_y = 0.0,
        .unlocked_levels = 1,
        .high_scores = [_]u32{0} ** 10,
        .checksum = 0,
    };
    return false;
}
```
{{#endtab}}

{{#endtabs}}

**See Also:** [System Functions](./system.md)
