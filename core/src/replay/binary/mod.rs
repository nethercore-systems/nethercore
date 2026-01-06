//! Binary replay format (.ncrp)
//!
//! The binary format is optimized for compact storage and fast loading.
//! It supports optional delta + LZ4 compression for inputs and checkpoints.
//!
//! # File Structure
//!
//! ```text
//! ┌────────────────────────────────────────────────━E
//! ━EHeader (24 bytes)                               ━E
//! ━E├─ console_id: u8                              ━E
//! ━E├─ player_count: u8                            ━E
//! ━E├─ input_size: u8                              ━E
//! ━E├─ flags: u8                                   ━E
//! ━E├─ reserved: [u8; 4]                           ━E
//! ━E├─ seed: u64                                   ━E
//! ━E└─ frame_count: u64                            ━E
//! ├────────────────────────────────────────────────┤
//! ━EInput Stream (delta-compressed if flagged)     ━E
//! ├────────────────────────────────────────────────┤
//! ━ECheckpoints (if flagged)                       ━E
//! ├────────────────────────────────────────────────┤
//! ━EAssertions (if flagged, JSON)                  ━E
//! └────────────────────────────────────────────────━E
//! ```

mod reader;
mod writer;

pub use reader::BinaryReader;
pub use writer::BinaryWriter;
