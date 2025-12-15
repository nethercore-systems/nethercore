//! Console-specific debug statistics
//!
//! Provides a way for consoles to expose runtime statistics
//! to the debug overlay. These are read-only values that show
//! the current state of console subsystems (e.g., draw call count,
//! vertex count, texture memory usage).

/// A single debug statistic from a console implementation.
#[derive(Debug, Clone)]
pub struct DebugStat {
    /// Display name for the stat
    pub name: String,
    /// Current value as a formatted string
    pub value: String,
}

impl DebugStat {
    /// Create a new debug stat with a string value.
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }

    /// Create a debug stat from a numeric value.
    pub fn number(name: impl Into<String>, value: impl std::fmt::Display) -> Self {
        Self {
            name: name.into(),
            value: value.to_string(),
        }
    }

    /// Create a debug stat for a byte count, formatted as KB/MB.
    pub fn bytes(name: impl Into<String>, bytes: usize) -> Self {
        let value = if bytes >= 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else if bytes >= 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{} B", bytes)
        };
        Self {
            name: name.into(),
            value,
        }
    }
}
