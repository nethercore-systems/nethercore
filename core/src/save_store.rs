use std::ffi::OsString;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use crate::console::ConsoleInput;

pub const SAVE_MAGIC: [u8; 4] = *b"NCSV";
pub const SAVE_VERSION: u32 = 1;
pub const PERSISTENT_SLOTS: usize = 4;

fn read_u32_le(cursor: &mut std::io::Cursor<&[u8]>) -> Option<u32> {
    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf).ok()?;
    Some(u32::from_le_bytes(buf))
}

fn write_u32_le(dst: &mut Vec<u8>, v: u32) {
    dst.extend_from_slice(&v.to_le_bytes());
}

/// Map a session slot (0..player_count) to a local controller slot.
///
/// `controller_slot` is the index of `session_slot` within the ordered list of
/// local session slots (ascending).
pub fn map_session_slot_to_controller_slot(
    local_mask: u32,
    player_count: u32,
    session_slot: u32,
) -> Option<usize> {
    if session_slot >= player_count {
        return None;
    }

    let is_local = (local_mask & (1u32 << session_slot)) != 0;
    if !is_local {
        return None;
    }

    let mut controller_slot = 0usize;
    for s in 0..player_count {
        if (local_mask & (1u32 << s)) != 0 {
            if s == session_slot {
                return Some(controller_slot);
            }
            controller_slot += 1;
        }
    }

    None
}

pub struct SaveStore {
    path: PathBuf,
    slots: [Option<Vec<u8>>; PERSISTENT_SLOTS],
    dirty: [bool; PERSISTENT_SLOTS],
}

impl SaveStore {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            slots: std::array::from_fn(|_| None),
            dirty: [false; PERSISTENT_SLOTS],
        }
    }

    pub fn load_or_new(path: PathBuf) -> io::Result<Self> {
        let empty_path = path.clone();
        let mut store = Self::new(path);

        let mut file = match fs::File::open(&store.path) {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(store),
            Err(e) => return Err(e),
        };

        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        if bytes.len() < 8 {
            return Ok(store);
        }

        let mut cursor = std::io::Cursor::new(bytes.as_slice());

        let mut magic = [0u8; 4];
        if cursor.read_exact(&mut magic).is_err() {
            return Ok(Self::new(empty_path.clone()));
        }

        if magic != SAVE_MAGIC {
            return Ok(Self::new(empty_path.clone()));
        }

        let version = match read_u32_le(&mut cursor) {
            Some(v) => v,
            None => return Ok(Self::new(empty_path.clone())),
        };

        if version != SAVE_VERSION {
            return Ok(Self::new(empty_path.clone()));
        }

        for i in 0..PERSISTENT_SLOTS {
            let len = match read_u32_le(&mut cursor) {
                Some(v) => v,
                None => return Ok(Self::new(empty_path.clone())),
            };

            // `u32::MAX` is a sentinel for an empty slot (`None`).
            if len == u32::MAX {
                store.slots[i] = None;
                continue;
            }

            // NOTE: We intentionally treat oversized entries as corruption and
            // return an error. Other parse issues fall back to an empty store.
            if len as usize > crate::MAX_SAVE_SIZE {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "save file entry exceeds MAX_SAVE_SIZE",
                ));
            }

            let mut data = vec![0u8; len as usize];
            if cursor.read_exact(&mut data).is_err() {
                return Ok(Self::new(empty_path.clone()));
            }
            store.slots[i] = Some(data);
        }

        Ok(store)
    }

    pub fn set_controller_slot(&mut self, slot: usize, data: Option<Vec<u8>>) {
        assert!(slot < PERSISTENT_SLOTS);
        self.slots[slot] = data;
        self.dirty[slot] = true;
    }

    pub fn controller_slot(&self, slot: usize) -> Option<&[u8]> {
        assert!(slot < PERSISTENT_SLOTS);
        self.slots[slot].as_deref()
    }

    /// Prefills per-session save slots in a `GameState` from this store.
    ///
    /// Session slots are keyed by the session player index (0..player_count).
    /// Only local session slots (bits set in `local_player_mask`) are mapped to
    /// controller slots.
    pub fn prefill_game_save_data<I: ConsoleInput>(&self, game: &mut crate::wasm::GameState<I>) {
        // Clear session slots first so we don't keep stale data across reloads.
        for slot in 0..crate::wasm::MAX_PLAYERS {
            game.save_data[slot] = None;
        }

        let player_count = game.player_count.min(crate::wasm::MAX_PLAYERS as u32);
        for session_slot in 0..player_count {
            let Some(controller_slot) = map_session_slot_to_controller_slot(
                game.local_player_mask,
                player_count,
                session_slot,
            ) else {
                continue;
            };

            if controller_slot >= PERSISTENT_SLOTS {
                continue;
            }

            if let Some(data) = self.controller_slot(controller_slot) {
                game.save_data[session_slot as usize] = Some(data.to_vec());
            }
        }
    }

    pub fn flush(&mut self) -> io::Result<()> {
        if !self.dirty.iter().any(|d| *d) {
            return Ok(());
        }

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp_path = match self.path.file_name() {
            Some(name) => {
                let mut tmp_name = OsString::from(name);
                tmp_name.push(".tmp");
                self.path.with_file_name(tmp_name)
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "save store path has no file name",
                ))
            }
        };

        let mut out = Vec::new();
        out.extend_from_slice(&SAVE_MAGIC);
        write_u32_le(&mut out, SAVE_VERSION);

        for slot in &self.slots {
            match slot {
                Some(data) => {
                    if data.len() > crate::MAX_SAVE_SIZE {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "save data exceeds MAX_SAVE_SIZE",
                        ));
                    }
                    write_u32_le(&mut out, data.len() as u32);
                    out.extend_from_slice(data);
                }
                None => {
                    // `u32::MAX` encodes `None` and consumes no data bytes.
                    write_u32_le(&mut out, u32::MAX);
                }
            }
        }

        {
            let mut f = fs::File::create(&tmp_path)?;
            f.write_all(&out)?;
            f.sync_all()?;
        }

        #[cfg(windows)]
        {
            if self.path.exists() {
                // Windows rename fails if destination exists.
                fs::remove_file(&self.path)?;
            }
        }

        fs::rename(&tmp_path, &self.path)?;

        self.dirty = [false; PERSISTENT_SLOTS];
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wasm::GameState;

    #[test]
    fn map_session_slot_to_controller_slot_orders_locals() {
        let local_mask = 0b1010u32; // session slots 1 and 3 are local
        let player_count = 4;

        assert_eq!(
            map_session_slot_to_controller_slot(local_mask, player_count, 0),
            None
        );
        assert_eq!(
            map_session_slot_to_controller_slot(local_mask, player_count, 1),
            Some(0)
        );
        assert_eq!(
            map_session_slot_to_controller_slot(local_mask, player_count, 2),
            None
        );
        assert_eq!(
            map_session_slot_to_controller_slot(local_mask, player_count, 3),
            Some(1)
        );
    }

    #[test]
    fn save_file_roundtrips_four_controller_slots() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("save.bin");

        let mut store = SaveStore::load_or_new(path.clone()).unwrap();
        store.set_controller_slot(0, Some(vec![1, 2, 3]));
        store.set_controller_slot(1, None);
        store.set_controller_slot(2, Some(vec![0xAA, 0xBB]));
        store.set_controller_slot(3, Some(vec![]));
        store.flush().unwrap();

        let store2 = SaveStore::load_or_new(path).unwrap();

        assert_eq!(store2.controller_slot(0).unwrap(), &[1, 2, 3]);
        assert!(store2.controller_slot(1).is_none());
        assert_eq!(store2.controller_slot(2).unwrap(), &[0xAA, 0xBB]);
        assert_eq!(store2.controller_slot(3), Some(&[][..]));
    }

    #[test]
    fn prefill_session_slots_from_local_controller_slots() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("t.ncsav");
        let mut store = SaveStore::new(path);
        store.set_controller_slot(0, Some(vec![7]));

        let mut game_state = GameState::<crate::test_utils::TestInput>::new();
        game_state.player_count = 4;
        game_state.local_player_mask = 0b0100;
        SaveStore::prefill_game_save_data(&store, &mut game_state);
        assert_eq!(game_state.save_data[2].as_deref(), Some(&[7][..]));
    }

    #[test]
    fn map_session_slot_to_controller_slot_ignores_bits_above_player_count() {
        // local_mask includes bit 3, but player_count=2, so only slots 0..1 matter.
        let local_mask = 0b1000u32;
        let player_count = 2;
        assert_eq!(
            map_session_slot_to_controller_slot(local_mask, player_count, 0),
            None
        );
        assert_eq!(
            map_session_slot_to_controller_slot(local_mask, player_count, 1),
            None
        );
        assert_eq!(
            map_session_slot_to_controller_slot(local_mask, player_count, 3),
            None
        );
    }

    #[test]
    fn load_or_new_returns_empty_on_truncated_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("save.bin");

        // Valid magic/version, but truncated before any slot lengths.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&SAVE_MAGIC);
        bytes.extend_from_slice(&SAVE_VERSION.to_le_bytes());
        std::fs::write(&path, bytes).unwrap();

        let store = SaveStore::load_or_new(path).unwrap();
        for i in 0..PERSISTENT_SLOTS {
            assert!(store.controller_slot(i).is_none());
        }
    }

    #[test]
    fn load_or_new_errors_on_oversized_entry() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("save.bin");

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&SAVE_MAGIC);
        bytes.extend_from_slice(&SAVE_VERSION.to_le_bytes());

        // Slot 0 declares an oversized length.
        bytes.extend_from_slice(&((crate::MAX_SAVE_SIZE as u32) + 1).to_le_bytes());
        // Remaining slots are empty.
        for _ in 1..PERSISTENT_SLOTS {
            bytes.extend_from_slice(&u32::MAX.to_le_bytes());
        }

        std::fs::write(&path, bytes).unwrap();
        let err = SaveStore::load_or_new(path).err().unwrap();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn prefill_clears_stale_session_data() {
        let store = SaveStore::new(std::path::PathBuf::from("unused.ncsav"));

        let mut game_state = GameState::<crate::test_utils::TestInput>::new();
        game_state.player_count = 4;
        game_state.local_player_mask = 0b0001;
        game_state.save_data[0] = Some(vec![9]);

        store.prefill_game_save_data(&mut game_state);
        assert!(game_state.save_data[0].is_none());
    }

    #[test]
    fn prefill_maps_multiple_local_players_in_order() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("t.ncsav");
        let mut store = SaveStore::new(path);
        store.set_controller_slot(0, Some(vec![1]));
        store.set_controller_slot(1, Some(vec![2]));

        let mut game_state = GameState::<crate::test_utils::TestInput>::new();
        game_state.player_count = 4;
        // Session slots 0 and 2 are local; ordered locals => [0, 2]
        game_state.local_player_mask = 0b0101;

        store.prefill_game_save_data(&mut game_state);
        assert_eq!(game_state.save_data[0].as_deref(), Some(&[1][..]));
        assert_eq!(game_state.save_data[2].as_deref(), Some(&[2][..]));
    }
}
