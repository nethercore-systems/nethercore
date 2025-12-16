# Save Data Functions

Persistent storage for game saves (8 slots, 64KB each).

## Overview

- **8 save slots** (indices 0-7)
- **64KB maximum** per slot
- Data persists across sessions
- Stored locally per-game

---

## Functions

### save

Saves data to a slot.

**Signature:**
```rust
fn save(slot: u32, data_ptr: *const u8, data_len: u32) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| slot | `u32` | Save slot (0-7) |
| data_ptr | `*const u8` | Pointer to data to save |
| data_len | `u32` | Size of data in bytes |

**Returns:**

| Value | Meaning |
|-------|---------|
| 0 | Success |
| 1 | Invalid slot |
| 2 | Data too large (>64KB) |

**Example:**
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

---

### load

Loads data from a slot.

**Signature:**
```rust
fn load(slot: u32, data_ptr: *mut u8, max_len: u32) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| slot | `u32` | Save slot (0-7) |
| data_ptr | `*mut u8` | Destination buffer |
| max_len | `u32` | Maximum bytes to read |

**Returns:** Number of bytes read (0 if empty or error)

**Example:**
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

---

### delete

Deletes data in a save slot.

**Signature:**
```rust
fn delete(slot: u32) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| slot | `u32` | Save slot (0-7) |

**Returns:**

| Value | Meaning |
|-------|---------|
| 0 | Success |
| 1 | Invalid slot |

**Example:**
```rust
fn delete_save(slot: u32) {
    unsafe {
        if delete(slot) == 0 {
            show_message(b"Save deleted");
        }
    }
}
```

---

## Save Data Patterns

### Simple Struct Save

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

### Multiple Save Slots UI

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

### Auto-Save

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

---

## Complete Example

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

**See Also:** [System Functions](./system.md)
