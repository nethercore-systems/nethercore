use std::fmt;

/// Error type for shader generation failures
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShaderGenError {
    /// Invalid render mode (must be 0-3)
    InvalidRenderMode(u8),
    /// Render mode requires NORMAL flag but format doesn't have it
    MissingNormalFlag { mode: u8, format: u8 },
}

impl fmt::Display for ShaderGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShaderGenError::InvalidRenderMode(mode) => {
                write!(f, "Invalid render mode: {} (must be 0-3)", mode)
            }
            ShaderGenError::MissingNormalFlag { mode, format } => {
                write!(
                    f,
                    "Render mode {} requires NORMAL flag, but format {} doesn't have it",
                    mode, format
                )
            }
        }
    }
}

impl std::error::Error for ShaderGenError {}
