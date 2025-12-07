//! Console-specific ROM formats
//!
//! Each fantasy console has its own ROM format (`.ewz`, `.ewc`, etc.) with
//! console-specific metadata and settings. ROMs are binary files using bitcode
//! serialization for fast loading and compact size.
//!
//! # Structure
//!
//! - `z.rs` - Emberware Z ROM format (`.ewz`)
//! - Future: `classic.rs` - Emberware Classic ROM format (`.ewc`)
//!
//! # Design
//!
//! - Console type is implicit in file extension (type-safe)
//! - Each ROM has a magic bytes header for validation
//! - Metadata includes optional platform foreign keys for syncing
//! - Screenshots stored in ROM but not extracted locally (save disk space)

pub mod z;
