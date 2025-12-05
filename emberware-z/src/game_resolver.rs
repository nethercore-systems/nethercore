//! Game ID resolution from user queries
//!
//! This module provides fuzzy matching for game IDs, supporting:
//! - Exact case-sensitive matches
//! - Case-insensitive matches
//! - Prefix matching (if unique)
//! - Typo suggestions using Levenshtein distance

use crate::library::LocalGame;

#[derive(Debug, Clone)]
pub struct GameResolutionError {
    pub message: String,
    pub suggestion: Option<Vec<String>>,
}

/// Resolve a game ID from a user query string
///
/// Priority order:
/// 1. Exact case-sensitive match
/// 2. Exact case-insensitive match
/// 3. Case-insensitive prefix match (if unique)
/// 4. Error with suggestions for similar games
pub fn resolve_game_id(
    query: &str,
    available_games: &[LocalGame],
) -> Result<String, GameResolutionError> {
    if query.is_empty() {
        return Err(GameResolutionError {
            message: "Empty game ID".to_string(),
            suggestion: None,
        });
    }

    // Fast path: exact case-sensitive match
    if let Some(game) = available_games.iter().find(|g| g.id == query) {
        return Ok(game.id.clone());
    }

    let lower_query = query.to_lowercase();

    // Exact case-insensitive match
    let case_insensitive_matches: Vec<&LocalGame> = available_games
        .iter()
        .filter(|g| g.id.to_lowercase() == lower_query)
        .collect();

    if case_insensitive_matches.len() == 1 {
        return Ok(case_insensitive_matches[0].id.clone());
    }

    // Prefix matching (case-insensitive)
    let prefix_matches: Vec<&LocalGame> = available_games
        .iter()
        .filter(|g| g.id.to_lowercase().starts_with(&lower_query))
        .collect();

    match prefix_matches.len() {
        0 => {
            // Not found - suggest similar games
            let suggestions = find_similar_games(query, available_games);
            Err(GameResolutionError {
                message: format!("Game '{}' not found", query),
                suggestion: if suggestions.is_empty() {
                    None
                } else {
                    Some(suggestions)
                },
            })
        }
        1 => {
            // Unique prefix match
            Ok(prefix_matches[0].id.clone())
        }
        _ => {
            // Multiple matches - ambiguous
            let candidates: Vec<String> = prefix_matches.iter().map(|g| g.id.clone()).collect();
            Err(GameResolutionError {
                message: format!("Ambiguous game ID '{}' matches multiple games", query),
                suggestion: Some(candidates),
            })
        }
    }
}

/// Find games with similar IDs using Levenshtein distance
fn find_similar_games(query: &str, available_games: &[LocalGame]) -> Vec<String> {
    const DISTANCE_THRESHOLD: usize = 3;

    let mut matches: Vec<(String, usize)> = available_games
        .iter()
        .map(|g| (g.id.clone(), levenshtein_distance(query, &g.id)))
        .filter(|(_, dist)| *dist <= DISTANCE_THRESHOLD)
        .collect();

    matches.sort_by_key(|(_, dist)| *dist);
    matches.into_iter().take(3).map(|(id, _)| id).collect()
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1.chars().nth(i - 1) == s2.chars().nth(j - 1) {
                0
            } else {
                1
            };
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
        assert_eq!(err.message, "Empty game ID");
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
        assert!(suggestions.contains(&"cube".to_string()) ||
                suggestions.contains(&"tube".to_string()) ||
                suggestions.contains(&"lube".to_string()));
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
