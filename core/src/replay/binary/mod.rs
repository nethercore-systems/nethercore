//! Binary replay format (.ncrp)
//!
//! The binary format is optimized for compact storage and fast loading.
//! It supports optional delta + LZ4 compression for inputs and checkpoints.
//!
//! # File Structure
//!
//! ```text
//! ┌────────────────────────────────────────────────┐
//! │ Header (24 bytes)                               │
//! │ ├─ console_id: u8                              │
//! │ ├─ player_count: u8                            │
//! │ ├─ input_size: u8                              │
//! │ ├─ flags: u8                                   │
//! │ ├─ reserved: [u8; 4]                           │
//! │ ├─ seed: u64                                   │
//! │ └─ frame_count: u64                            │
//! ├────────────────────────────────────────────────┤
//! │ Input Stream (delta-compressed if flagged)     │
//! ├────────────────────────────────────────────────┤
//! │ Checkpoints (if flagged)                       │
//! ├────────────────────────────────────────────────┤
//! │ Assertions (if flagged, JSON)                  │
//! └────────────────────────────────────────────────┘
//! ```

mod reader;
mod writer;

pub use reader::BinaryReader;
pub use writer::BinaryWriter;
