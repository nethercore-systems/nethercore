//! TOML replay script parser
//!
//! Parses .ncrs script files into a structured representation.

use super::ast::{AssertCondition, AssertValue, CompareOp, InputValue, ReplayScript};
use std::path::Path;

impl ReplayScript {
    /// Parse a TOML replay script from a string
    pub fn from_toml(toml_str: &str) -> Result<Self, ParseError> {
        toml::from_str(toml_str).map_err(|e| ParseError::TomlError(e.to_string()))
    }

    /// Parse a TOML replay script from a file
    pub fn from_file(path: &Path) -> Result<Self, ParseError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ParseError::IoError(e.to_string()))?;
        Self::from_toml(&content)
    }

    /// Serialize to TOML string
    pub fn to_toml(&self) -> Result<String, ParseError> {
        toml::to_string_pretty(self).map_err(|e| ParseError::TomlError(e.to_string()))
    }
}

impl AssertCondition {
    /// Parse an assertion string like "$velocity_y < 0"
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let s = s.trim();

        // Find operator (check longer operators first)
        let (variable, operator, value_str) = if let Some(pos) = s.find("==") {
            (&s[..pos], CompareOp::Eq, &s[pos + 2..])
        } else if let Some(pos) = s.find("!=") {
            (&s[..pos], CompareOp::Ne, &s[pos + 2..])
        } else if let Some(pos) = s.find("<=") {
            (&s[..pos], CompareOp::Le, &s[pos + 2..])
        } else if let Some(pos) = s.find(">=") {
            (&s[..pos], CompareOp::Ge, &s[pos + 2..])
        } else if let Some(pos) = s.find('<') {
            (&s[..pos], CompareOp::Lt, &s[pos + 1..])
        } else if let Some(pos) = s.find('>') {
            (&s[..pos], CompareOp::Gt, &s[pos + 1..])
        } else {
            return Err(ParseError::InvalidAssertion(format!(
                "No operator found in: {}",
                s
            )));
        };

        let variable = variable.trim().to_string();
        let value_str = value_str.trim();

        // Parse value
        let value = if let Some(prev_var) = value_str.strip_prefix("$prev_") {
            AssertValue::PrevValue(prev_var.to_string())
        } else if value_str.starts_with('$') {
            AssertValue::Variable(value_str.to_string())
        } else if value_str == "true" {
            AssertValue::Number(1.0)
        } else if value_str == "false" {
            AssertValue::Number(0.0)
        } else {
            let num: f64 = value_str.parse().map_err(|_| {
                ParseError::InvalidAssertion(format!("Invalid number: {}", value_str))
            })?;
            AssertValue::Number(num)
        };

        Ok(Self {
            variable,
            operator,
            value,
        })
    }
}

impl InputValue {
    /// Parse symbolic input like "right+a" into button list
    pub fn parse_symbolic(s: &str) -> Vec<String> {
        if s == "idle" || s.is_empty() {
            return Vec::new();
        }
        s.split('+')
            .map(|b| b.trim().to_lowercase())
            .filter(|b| !b.is_empty() && b != "idle")
            .collect()
    }

    /// Get buttons from any input type
    pub fn buttons(&self) -> Vec<String> {
        match self {
            InputValue::Symbolic(s) => Self::parse_symbolic(s),
            InputValue::Structured(s) => s.buttons.clone(),
            InputValue::HexBytes(_) => Vec::new(),
        }
    }

    /// Check if this is an idle input
    pub fn is_idle(&self) -> bool {
        match self {
            InputValue::Symbolic(s) => s == "idle" || s.is_empty(),
            InputValue::Structured(s) => {
                s.buttons.is_empty()
                    && s.lstick.is_none()
                    && s.rstick.is_none()
                    && s.lt.is_none()
                    && s.rt.is_none()
            }
            InputValue::HexBytes(bytes) => bytes.iter().all(|&b| b == 0),
        }
    }
}

impl Default for InputValue {
    fn default() -> Self {
        InputValue::Symbolic("idle".to_string())
    }
}

/// Parse errors
#[derive(Debug)]
pub enum ParseError {
    /// TOML parsing error
    TomlError(String),
    /// File I/O error
    IoError(String),
    /// Invalid assertion syntax
    InvalidAssertion(String),
    /// Invalid input format
    InvalidInput(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::TomlError(e) => write!(f, "TOML parse error: {}", e),
            ParseError::IoError(e) => write!(f, "IO error: {}", e),
            ParseError::InvalidAssertion(e) => write!(f, "Invalid assertion: {}", e),
            ParseError::InvalidInput(e) => write!(f, "Invalid input: {}", e),
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::script::{ActionParamValue, FrameEntry};

    #[test]
    fn test_parse_basic_script() {
        let toml_str = r#"
console = "zx"
seed = 12345
players = 1

frames = [
  { f = 0, p1 = "idle", snap = true },
  { f = 1, p1 = "right+a" },
  { f = 60, p1 = "right+a", snap = true, assert = "$player_x > 0" },
]
"#;

        let script = ReplayScript::from_toml(toml_str).unwrap();

        assert_eq!(script.console, "zx");
        assert_eq!(script.seed, 12345);
        assert_eq!(script.players, 1);
        assert_eq!(script.frames.len(), 3);
        assert!(script.frames[0].snap);
        assert!(script.frames[2].assert.is_some());
    }

    #[test]
    fn test_parse_assertion_all_operators() {
        // Test all 6 comparison operators
        let cond = AssertCondition::parse("$x == 100").unwrap();
        assert_eq!(cond.variable, "$x");
        assert_eq!(cond.operator, CompareOp::Eq);
        assert!(matches!(cond.value, AssertValue::Number(n) if n == 100.0));

        let cond = AssertCondition::parse("$x != 0").unwrap();
        assert_eq!(cond.operator, CompareOp::Ne);

        let cond = AssertCondition::parse("$velocity_y < 0").unwrap();
        assert_eq!(cond.variable, "$velocity_y");
        assert_eq!(cond.operator, CompareOp::Lt);
        assert!(matches!(cond.value, AssertValue::Number(n) if n == 0.0));

        let cond = AssertCondition::parse("$player_x > 50").unwrap();
        assert_eq!(cond.operator, CompareOp::Gt);

        let cond = AssertCondition::parse("$health <= 100").unwrap();
        assert_eq!(cond.operator, CompareOp::Le);

        let cond = AssertCondition::parse("$score >= 1000").unwrap();
        assert_eq!(cond.operator, CompareOp::Ge);
    }

    #[test]
    fn test_parse_assertion_value_types() {
        // Test $prev_* values
        let cond = AssertCondition::parse("$player_x == $prev_x").unwrap();
        assert_eq!(cond.variable, "$player_x");
        assert_eq!(cond.operator, CompareOp::Eq);
        assert!(matches!(cond.value, AssertValue::PrevValue(ref s) if s == "x"));

        // Test variable-to-variable comparison
        let cond = AssertCondition::parse("$player_x > $enemy_x").unwrap();
        assert_eq!(cond.variable, "$player_x");
        assert_eq!(cond.operator, CompareOp::Gt);
        assert!(matches!(cond.value, AssertValue::Variable(ref s) if s == "$enemy_x"));

        // Test boolean literals
        let cond = AssertCondition::parse("$on_ground == true").unwrap();
        assert_eq!(cond.operator, CompareOp::Eq);
        assert!(matches!(cond.value, AssertValue::Number(n) if n == 1.0));

        let cond = AssertCondition::parse("$is_dead == false").unwrap();
        assert!(matches!(cond.value, AssertValue::Number(n) if n == 0.0));

        // Test negative numbers
        let cond = AssertCondition::parse("$velocity_y < -10").unwrap();
        assert!(matches!(cond.value, AssertValue::Number(n) if n == -10.0));

        // Test floating point numbers
        let cond = AssertCondition::parse("$position_x == 123.456").unwrap();
        assert!(matches!(cond.value, AssertValue::Number(n) if (n - 123.456).abs() < 0.001));
    }

    #[test]
    fn test_parse_assertion_errors() {
        // Missing operator
        assert!(AssertCondition::parse("$x 100").is_err());

        // Invalid number
        assert!(AssertCondition::parse("$x == abc").is_err());

        // Empty string
        assert!(AssertCondition::parse("").is_err());
    }

    #[test]
    fn test_parse_symbolic_input() {
        assert_eq!(InputValue::parse_symbolic("idle"), Vec::<String>::new());
        assert_eq!(InputValue::parse_symbolic("a"), vec!["a".to_string()]);
        assert_eq!(
            InputValue::parse_symbolic("right+a"),
            vec!["right".to_string(), "a".to_string()]
        );
        assert_eq!(
            InputValue::parse_symbolic("up+right+b"),
            vec!["up".to_string(), "right".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn test_structured_input() {
        let toml_str = r#"
console = "zx"
players = 1

[[frames]]
f = 0
[frames.p1]
buttons = ["a"]
lstick = [1.0, 0.0]

[[frames]]
f = 1
[frames.p1]
buttons = []
lstick = [0.5, 0.5]
rt = 0.8
"#;

        let script = ReplayScript::from_toml(toml_str).unwrap();
        assert_eq!(script.frames.len(), 2);

        if let Some(InputValue::Structured(s)) = &script.frames[0].p1 {
            assert_eq!(s.buttons, vec!["a".to_string()]);
            assert_eq!(s.lstick, Some([1.0, 0.0]));
        } else {
            panic!("Expected structured input");
        }

        if let Some(InputValue::Structured(s)) = &script.frames[1].p1 {
            assert!(s.buttons.is_empty());
            assert_eq!(s.lstick, Some([0.5, 0.5]));
            assert_eq!(s.rt, Some(0.8));
        } else {
            panic!("Expected structured input");
        }
    }

    #[test]
    fn test_roundtrip() {
        let script = ReplayScript {
            console: "zx".to_string(),
            seed: 42,
            players: 2,
            frames: vec![
                FrameEntry {
                    f: 0,
                    p1: Some(InputValue::Symbolic("idle".to_string())),
                    p2: Some(InputValue::Symbolic("idle".to_string())),
                    p3: None,
                    p4: None,
                    snap: true,
                    screenshot: false,
                    assert: None,
                    action: None,
                    action_params: None,
                },
                FrameEntry {
                    f: 1,
                    p1: Some(InputValue::Symbolic("a".to_string())),
                    p2: Some(InputValue::Symbolic("b".to_string())),
                    p3: None,
                    p4: None,
                    snap: false,
                    screenshot: false,
                    assert: Some("$velocity_y < 0".to_string()),
                    action: None,
                    action_params: None,
                },
            ],
        };

        let toml = script.to_toml().unwrap();
        let parsed = ReplayScript::from_toml(&toml).unwrap();

        assert_eq!(parsed.console, script.console);
        assert_eq!(parsed.seed, script.seed);
        assert_eq!(parsed.players, script.players);
        assert_eq!(parsed.frames.len(), script.frames.len());
    }

    #[test]
    fn test_action_parsing_basic() {
        let toml = r#"
console = "zx"
seed = 0
players = 1

[[frames]]
f = 0
action = "Load Level"
action_params = { level = 2 }

[[frames]]
f = 1
p1 = "a"
"#;
        let script = ReplayScript::from_toml(toml).unwrap();
        assert_eq!(script.frames.len(), 2);
        assert_eq!(script.frames[0].action, Some("Load Level".to_string()));
        assert!(script.frames[0].action_params.is_some());
        assert!(script.frames[1].action.is_none());
    }

    #[test]
    fn test_action_all_parameter_types() {
        let toml = r#"
console = "zx"
seed = 0
players = 1

[[frames]]
f = 0
action = "Configure Game"
action_params = { level = 5, speed = 1.5, name = "test_run", enabled = true }
"#;
        let script = ReplayScript::from_toml(toml).unwrap();
        let params = script.frames[0].action_params.as_ref().unwrap();

        // Test Int parameter
        assert!(matches!(
            params.get("level"),
            Some(ActionParamValue::Int(5))
        ));

        // Test Float parameter
        match params.get("speed") {
            Some(ActionParamValue::Float(f)) => assert!((*f - 1.5).abs() < 0.001),
            _ => panic!("Expected Float parameter"),
        }

        // Test String parameter
        assert!(matches!(
            params.get("name"),
            Some(ActionParamValue::String(s)) if s == "test_run"
        ));

        // Test Bool parameter
        assert!(matches!(
            params.get("enabled"),
            Some(ActionParamValue::Bool(true))
        ));
    }

    #[test]
    fn test_action_multiple_on_same_frame() {
        // Note: TOML array format allows multiple frame entries with same f value
        // Each becomes a separate action
        let toml = r#"
console = "zx"
seed = 0
players = 1

[[frames]]
f = 0
action = "Set Difficulty"
action_params = { level = 3 }

[[frames]]
f = 0
action = "Set Player Count"
action_params = { count = 2 }

[[frames]]
f = 0
p1 = "idle"
snap = true
"#;
        let script = ReplayScript::from_toml(toml).unwrap();

        // All three entries for frame 0 should be parsed
        assert_eq!(script.frames.len(), 3);

        let frame0_actions: Vec<_> = script
            .frames
            .iter()
            .filter(|f| f.f == 0 && f.action.is_some())
            .collect();
        assert_eq!(frame0_actions.len(), 2);
        assert_eq!(frame0_actions[0].action, Some("Set Difficulty".to_string()));
        assert_eq!(
            frame0_actions[1].action,
            Some("Set Player Count".to_string())
        );
    }

    #[test]
    fn test_action_without_params() {
        let toml = r#"
console = "zx"
seed = 0
players = 1

[[frames]]
f = 0
action = "Reset State"
"#;
        let script = ReplayScript::from_toml(toml).unwrap();
        assert_eq!(script.frames[0].action, Some("Reset State".to_string()));
        assert!(script.frames[0].action_params.is_none());
    }

    #[test]
    fn test_action_roundtrip() {
        use hashbrown::HashMap;

        let mut params = HashMap::new();
        params.insert("level".to_string(), ActionParamValue::Int(5));
        params.insert("speed".to_string(), ActionParamValue::Float(2.5));
        params.insert(
            "name".to_string(),
            ActionParamValue::String("test".to_string()),
        );
        params.insert("debug".to_string(), ActionParamValue::Bool(true));

        let script = ReplayScript {
            console: "zx".to_string(),
            seed: 0,
            players: 1,
            frames: vec![FrameEntry {
                f: 0,
                p1: None,
                p2: None,
                p3: None,
                p4: None,
                snap: false,
                screenshot: false,
                assert: None,
                action: Some("Test Action".to_string()),
                action_params: Some(params),
            }],
        };

        let toml = script.to_toml().unwrap();
        let parsed = ReplayScript::from_toml(&toml).unwrap();

        assert_eq!(parsed.frames[0].action, Some("Test Action".to_string()));
        let parsed_params = parsed.frames[0].action_params.as_ref().unwrap();
        assert!(matches!(
            parsed_params.get("level"),
            Some(ActionParamValue::Int(5))
        ));
        assert!(matches!(
            parsed_params.get("debug"),
            Some(ActionParamValue::Bool(true))
        ));
    }
}
