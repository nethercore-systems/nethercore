//! Replay script validation.

use hashbrown::HashSet;

use super::ast::ReplayScript;

/// Validation errors for replay scripts.
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Console identifier is missing.
    EmptyConsole,
    /// Player count outside allowed range.
    InvalidPlayerCount(u8),
    /// Duplicate frame entry.
    DuplicateFrame(u64),
    /// Inputs provided for a player beyond the configured count.
    UnexpectedPlayerInput { frame: u64, player: u8 },
    /// Action parameters provided without an action.
    OrphanedActionParams(u64),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptyConsole => write!(f, "console is required"),
            ValidationError::InvalidPlayerCount(count) => {
                write!(f, "invalid player count: {}", count)
            }
            ValidationError::DuplicateFrame(frame) => write!(f, "duplicate frame: {}", frame),
            ValidationError::UnexpectedPlayerInput { frame, player } => write!(
                f,
                "input for player {} provided at frame {} but players < {}",
                player,
                frame,
                player + 1
            ),
            ValidationError::OrphanedActionParams(frame) => {
                write!(
                    f,
                    "action params provided without action at frame {}",
                    frame
                )
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validate a parsed replay script before compilation.
pub fn validate_script(script: &ReplayScript) -> Result<(), ValidationError> {
    if script.console.trim().is_empty() {
        return Err(ValidationError::EmptyConsole);
    }

    if !(1..=4).contains(&script.players) {
        return Err(ValidationError::InvalidPlayerCount(script.players));
    }

    let mut frames = HashSet::new();
    for entry in &script.frames {
        if !frames.insert(entry.f) {
            return Err(ValidationError::DuplicateFrame(entry.f));
        }

        if entry.action_params.is_some() && entry.action.is_none() {
            return Err(ValidationError::OrphanedActionParams(entry.f));
        }

        if script.players < 4 && entry.p4.is_some() {
            return Err(ValidationError::UnexpectedPlayerInput {
                frame: entry.f,
                player: 4,
            });
        }
        if script.players < 3 && entry.p3.is_some() {
            return Err(ValidationError::UnexpectedPlayerInput {
                frame: entry.f,
                player: 3,
            });
        }
        if script.players < 2 && entry.p2.is_some() {
            return Err(ValidationError::UnexpectedPlayerInput {
                frame: entry.f,
                player: 2,
            });
        }
    }

    Ok(())
}
