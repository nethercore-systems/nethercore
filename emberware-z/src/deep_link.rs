//! Deep link parsing (emberware://play/game_id)

#[derive(Debug, Clone)]
pub struct DeepLink {
    pub game_id: String,
}

pub fn parse(args: &[String]) -> Option<DeepLink> {
    for arg in args.iter().skip(1) {
        if let Some(rest) = arg.strip_prefix("emberware://play/") {
            let game_id = rest.trim_end_matches('/').to_string();
            if !game_id.is_empty() {
                return Some(DeepLink { game_id });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_deep_link() {
        let args = vec![
            "emberware".to_string(),
            "emberware://play/abc123".to_string(),
        ];
        let link = parse(&args).unwrap();
        assert_eq!(link.game_id, "abc123");
    }

    #[test]
    fn test_no_deep_link() {
        let args = vec!["emberware".to_string()];
        assert!(parse(&args).is_none());
    }
}
