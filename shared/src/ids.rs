//! Helpers for validating identifiers used in filesystem paths.

/// Returns true if a game ID is safe to use as a single path component on all platforms.
///
/// Rules:
/// - Must be non-empty and not "." or ".."
/// - Must not contain path separators ('/' or '\\')
/// - Must not contain control characters or NUL
/// - Must not contain Windows-reserved filename characters
/// - Must not end with '.' or space (Windows restriction)
pub fn is_safe_game_id(id: &str) -> bool {
    if id.is_empty() || id == "." || id == ".." {
        return false;
    }

    if id.ends_with('.') || id.ends_with(' ') {
        return false;
    }

    for c in id.chars() {
        if c == '/' || c == '\\' || c == '\0' {
            return false;
        }
        if c.is_control() {
            return false;
        }
        // Windows-reserved filename characters.
        if matches!(c, ':' | '*' | '?' | '"' | '<' | '>' | '|') {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::is_safe_game_id;

    #[test]
    fn accepts_common_ids() {
        assert!(is_safe_game_id("game-with-dashes"));
        assert!(is_safe_game_id("game_with_underscores"));
        assert!(is_safe_game_id("game.with.dots"));
        assert!(is_safe_game_id("My Game 2"));
        assert!(is_safe_game_id("unicode-标题"));
    }

    #[test]
    fn rejects_empty_and_special() {
        assert!(!is_safe_game_id(""));
        assert!(!is_safe_game_id("."));
        assert!(!is_safe_game_id(".."));
    }

    #[test]
    fn rejects_separators_and_reserved_chars() {
        assert!(!is_safe_game_id("../evil"));
        assert!(!is_safe_game_id("evil/dir"));
        assert!(!is_safe_game_id("evil\\dir"));
        assert!(!is_safe_game_id("C:evil"));
        assert!(!is_safe_game_id("bad|name"));
        assert!(!is_safe_game_id("bad?name"));
    }

    #[test]
    fn rejects_trailing_dot_or_space() {
        assert!(!is_safe_game_id("bad."));
        assert!(!is_safe_game_id("bad "));
    }
}
