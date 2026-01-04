//! Generic ID resolution from user queries
//!
//! This module provides fuzzy matching for IDs, supporting:
//! - Exact case-sensitive matches
//! - Case-insensitive matches
//! - Prefix matching (if unique)
//! - Typo suggestions using Levenshtein distance
//!
//! The generic `resolve_id` function works with any type via a closure,
//! while `resolve_game_id` provides a convenient wrapper for `LocalGame`.

use super::LocalGame;

#[derive(Debug, Clone)]
pub struct ResolutionError {
    pub message: String,
    pub suggestion: Option<Vec<String>>,
}

/// Backwards compatibility alias
pub type GameResolutionError = ResolutionError;

/// Generic ID resolution from a user query string.
///
/// Works with any item type by using a closure to extract the ID.
///
/// Priority order:
/// 1. Exact case-sensitive match
/// 2. Exact case-insensitive match
/// 3. Case-insensitive prefix match (if unique)
/// 4. Error with suggestions for similar IDs
///
/// # Example
/// ```ignore
/// let roms = vec![PathBuf::from("paddle.nczx"), PathBuf::from("cube.nczx")];
/// let result = resolve_id("pad", &roms, |p| {
///     p.file_stem().unwrap().to_str().unwrap()
/// });
/// ```
pub fn resolve_id<'a, T, F>(
    query: &str,
    items: &'a [T],
    get_id: F,
    item_kind: &str,
) -> Result<&'a T, ResolutionError>
where
    F: Fn(&T) -> &str,
{
    if query.is_empty() {
        return Err(ResolutionError {
            message: format!("Empty {} ID", item_kind),
            suggestion: None,
        });
    }

    // Fast path: exact case-sensitive match
    if let Some(item) = items.iter().find(|item| get_id(item) == query) {
        return Ok(item);
    }

    let lower_query = query.to_lowercase();

    // Exact case-insensitive match
    let case_insensitive_matches: Vec<&T> = items
        .iter()
        .filter(|item| get_id(item).to_lowercase() == lower_query)
        .collect();

    if case_insensitive_matches.len() == 1 {
        return Ok(case_insensitive_matches[0]);
    }

    // Prefix matching (case-insensitive)
    let prefix_matches: Vec<&T> = items
        .iter()
        .filter(|item| get_id(item).to_lowercase().starts_with(&lower_query))
        .collect();

    match prefix_matches.len() {
        0 => {
            // Not found - suggest similar items
            let suggestions = find_similar(query, items, &get_id);
            Err(ResolutionError {
                message: format!("{} '{}' not found", item_kind, query),
                suggestion: if suggestions.is_empty() {
                    None
                } else {
                    Some(suggestions)
                },
            })
        }
        1 => {
            // Unique prefix match
            Ok(prefix_matches[0])
        }
        _ => {
            // Multiple matches - ambiguous
            let candidates: Vec<String> = prefix_matches
                .iter()
                .map(|item| get_id(item).to_string())
                .collect();
            Err(ResolutionError {
                message: format!("Ambiguous {} '{}' matches multiple items", item_kind, query),
                suggestion: Some(candidates),
            })
        }
    }
}

/// Resolve a game ID from a user query string.
///
/// Convenience wrapper around `resolve_id` for `LocalGame`.
pub fn resolve_game_id(
    query: &str,
    available_games: &[LocalGame],
) -> Result<String, ResolutionError> {
    resolve_id(query, available_games, |g| g.id.as_str(), "Game").map(|g| g.id.clone())
}

/// Find items with similar IDs using Levenshtein distance.
pub fn find_similar<T, F>(query: &str, items: &[T], get_id: F) -> Vec<String>
where
    F: Fn(&T) -> &str,
{
    const DISTANCE_THRESHOLD: usize = 3;

    let mut matches: Vec<(String, usize)> = items
        .iter()
        .map(|item| {
            let id = get_id(item);
            (id.to_string(), levenshtein_distance(query, id))
        })
        .filter(|(_, dist)| *dist <= DISTANCE_THRESHOLD)
        .collect();

    matches.sort_by_key(|(_, dist)| *dist);
    matches.into_iter().take(3).map(|(id, _)| id).collect()
}

/// Calculate Levenshtein distance between two strings.
pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    // Pre-collect chars to avoid O(n) .chars().nth() calls
    let chars1: Vec<char> = s1.chars().collect();
    let chars2: Vec<char> = s2.chars().collect();
    let len1 = chars1.len();
    let len2 = chars2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }
    for (j, val) in matrix[0].iter_mut().enumerate().take(len2 + 1) {
        *val = j;
    }

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[len1][len2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_game(id: &str, title: &str) -> LocalGame {
        LocalGame {
            id: id.to_string(),
            title: title.to_string(),
            author: "Test Author".to_string(),
            version: "1.0.0".to_string(),
            rom_path: PathBuf::from("dummy.wasm"),
            console_type: "zx".to_string(),
        }
    }

    #[test]
    fn test_exact_case_sensitive_match() {
        let games = vec![create_test_game("platformer", "Platformer Game")];
        let result = resolve_game_id("platformer", &games);
        assert_eq!(result.unwrap(), "platformer");
    }

    #[test]
    fn test_case_insensitive_exact_match() {
        let games = vec![create_test_game("platformer", "Platformer Game")];
        let result = resolve_game_id("PLATFORMER", &games);
        assert_eq!(result.unwrap(), "platformer");
    }

    #[test]
    fn test_case_insensitive_exact_match_mixed_case() {
        let games = vec![create_test_game("platformer", "Platformer Game")];
        let result = resolve_game_id("PlatFormer", &games);
        assert_eq!(result.unwrap(), "platformer");
    }

    #[test]
    fn test_prefix_match_unique() {
        let games = vec![create_test_game("platformer", "Platformer Game")];
        let result = resolve_game_id("plat", &games);
        assert_eq!(result.unwrap(), "platformer");
    }

    #[test]
    fn test_prefix_match_case_insensitive() {
        let games = vec![create_test_game("platformer", "Platformer Game")];
        let result = resolve_game_id("PLAT", &games);
        assert_eq!(result.unwrap(), "platformer");
    }

    #[test]
    fn test_prefix_match_ambiguous() {
        let games = vec![
            create_test_game("billboard", "Billboard"),
            create_test_game("billboard-lite", "Billboard Lite"),
        ];
        let result = resolve_game_id("bill", &games);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Ambiguous"));
        assert!(err.suggestion.is_some());
        let suggestions = err.suggestion.unwrap();
        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.contains(&"billboard".to_string()));
        assert!(suggestions.contains(&"billboard-lite".to_string()));
    }

    #[test]
    fn test_not_found() {
        let games = vec![create_test_game("platformer", "Platformer Game")];
        let result = resolve_game_id("notgame", &games);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("not found"));
    }

    #[test]
    fn test_empty_query() {
        let games = vec![create_test_game("platformer", "Platformer Game")];
        let result = resolve_game_id("", &games);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Empty Game ID");
    }

    #[test]
    fn test_similar_game_suggestion() {
        let games = vec![
            create_test_game("platformer", "Platformer Game"),
            create_test_game("cube", "Cube"),
        ];
        let result = resolve_game_id("platfrm", &games);
        assert!(result.is_err());
        let err = result.unwrap_err();
        // "platfrm" is 2 chars away from "platformer"
        assert!(err.suggestion.is_some());
        let suggestions = err.suggestion.unwrap();
        assert!(suggestions.contains(&"platformer".to_string()));
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", "ab"), 1);
        assert_eq!(levenshtein_distance("abc", "abcd"), 1);
        assert_eq!(levenshtein_distance("abc", "def"), 3);
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
    }

    #[test]
    fn test_multiple_exact_matches_case_insensitive() {
        // Edge case: should prefer the first case-insensitive match
        let games = vec![
            create_test_game("cube", "Cube 1"),
            create_test_game("CUBE", "Cube 2"),
        ];
        let result = resolve_game_id("cube", &games);
        // Should return exact case-sensitive match
        assert_eq!(result.unwrap(), "cube");
    }

    #[test]
    fn test_shortest_prefix_wins() {
        let games = vec![
            create_test_game("cube", "Cube"),
            create_test_game("platformer", "Platformer"),
        ];
        let result = resolve_game_id("c", &games);
        assert_eq!(result.unwrap(), "cube");
    }

    #[test]
    fn test_special_characters() {
        let games = vec![
            create_test_game("hello-world", "Hello World"),
            create_test_game("textured-quad", "Textured Quad"),
        ];
        let result = resolve_game_id("hello", &games);
        assert_eq!(result.unwrap(), "hello-world");
    }

    #[test]
    fn test_no_games_available() {
        let games: Vec<LocalGame> = vec![];
        let result = resolve_game_id("anything", &games);
        assert!(result.is_err());
    }

    #[test]
    fn test_prefix_with_hyphen() {
        let games = vec![
            create_test_game("hello-world", "Hello World"),
            create_test_game("textured-quad", "Textured Quad"),
        ];
        let result = resolve_game_id("hello-", &games);
        assert_eq!(result.unwrap(), "hello-world");
    }

    #[test]
    fn test_similar_games_multiple_suggestions() {
        let games = vec![
            create_test_game("cube", "Cube"),
            create_test_game("tube", "Tube"),
            create_test_game("lube", "Lube"),
            create_test_game("platformer", "Platformer"),
        ];
        let result = resolve_game_id("dube", &games);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let suggestions = err.suggestion.unwrap();
        // Should suggest cube, tube, lube (all distance 1), max 3 suggestions
        assert!(suggestions.len() <= 3);
        assert!(
            suggestions.contains(&"cube".to_string())
                || suggestions.contains(&"tube".to_string())
                || suggestions.contains(&"lube".to_string())
        );
    }

    #[test]
    fn test_no_similar_games() {
        let games = vec![create_test_game("platformer", "Platformer Game")];
        let result = resolve_game_id("xyz", &games);
        assert!(result.is_err());
        let err = result.unwrap_err();
        // "xyz" is too far from "platformer" (distance > 3)
        assert!(err.suggestion.is_none() || err.suggestion.unwrap().is_empty());
    }
}
