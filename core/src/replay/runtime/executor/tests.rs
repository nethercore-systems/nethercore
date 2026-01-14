//! Tests for script executor

use super::*;
use crate::replay::script::CompiledAssertion;
use crate::replay::types::InputSequence;

#[test]
fn test_executor_basic() {
    let mut inputs = InputSequence::new();
    inputs.push_frame(vec![vec![0x00]]);
    inputs.push_frame(vec![vec![0x10]]);

    let script = CompiledScript {
        console: "zx".to_string(),
        console_id: 1,
        seed: 0,
        player_count: 1,
        input_size: 1,
        inputs,
        snap_frames: vec![0],
        assertions: vec![],
        actions: vec![],
        frame_count: 2,
    };

    let mut executor = ScriptExecutor::new(script);

    assert_eq!(executor.current_frame(), 0);
    assert!(!executor.is_complete());
    assert!(executor.needs_snapshot());

    executor.advance_frame();
    assert_eq!(executor.current_frame(), 1);
    assert!(!executor.needs_snapshot());

    executor.advance_frame();
    assert!(executor.is_complete());
}

#[test]
fn test_assertion_evaluation_all_operators() {
    let inputs = InputSequence::new();

    // Test Eq operator
    let script = CompiledScript {
        console: "zx".to_string(),
        console_id: 1,
        seed: 0,
        player_count: 1,
        input_size: 1,
        inputs: inputs.clone(),
        snap_frames: vec![],
        assertions: vec![CompiledAssertion {
            frame: 0,
            condition: "$x == 100".to_string(),
            variable: "$x".to_string(),
            operator: CompareOp::Eq,
            value: CompiledAssertValue::Number(100.0),
        }],
        actions: vec![],
        frame_count: 1,
    };
    let mut executor = ScriptExecutor::new(script);
    let mut values = HashMap::new();
    values.insert("$x".to_string(), DebugValueData::F32(100.0));
    let assertion = &executor.current_assertions()[0].clone();
    assert!(executor.evaluate_assertion(assertion, &values, false));
    values.insert("$x".to_string(), DebugValueData::F32(99.0));
    assert!(!executor.evaluate_assertion(assertion, &values, false));

    // Test Ne operator
    let script = CompiledScript {
        console: "zx".to_string(),
        console_id: 1,
        seed: 0,
        player_count: 1,
        input_size: 1,
        inputs: inputs.clone(),
        snap_frames: vec![],
        assertions: vec![CompiledAssertion {
            frame: 0,
            condition: "$x != 0".to_string(),
            variable: "$x".to_string(),
            operator: CompareOp::Ne,
            value: CompiledAssertValue::Number(0.0),
        }],
        actions: vec![],
        frame_count: 1,
    };
    let mut executor = ScriptExecutor::new(script);
    values.insert("$x".to_string(), DebugValueData::F32(50.0));
    let assertion = &executor.current_assertions()[0].clone();
    assert!(executor.evaluate_assertion(assertion, &values, false));
    values.insert("$x".to_string(), DebugValueData::F32(0.0));
    assert!(!executor.evaluate_assertion(assertion, &values, false));

    // Test Lt operator
    let script = CompiledScript {
        console: "zx".to_string(),
        console_id: 1,
        seed: 0,
        player_count: 1,
        input_size: 1,
        inputs: inputs.clone(),
        snap_frames: vec![],
        assertions: vec![CompiledAssertion {
            frame: 0,
            condition: "$x < 100".to_string(),
            variable: "$x".to_string(),
            operator: CompareOp::Lt,
            value: CompiledAssertValue::Number(100.0),
        }],
        actions: vec![],
        frame_count: 1,
    };
    let mut executor = ScriptExecutor::new(script);
    values.insert("$x".to_string(), DebugValueData::F32(50.0));
    let assertion = &executor.current_assertions()[0].clone();
    assert!(executor.evaluate_assertion(assertion, &values, false));
    values.insert("$x".to_string(), DebugValueData::F32(100.0));
    assert!(!executor.evaluate_assertion(assertion, &values, false));

    // Test Gt operator
    let script = CompiledScript {
        console: "zx".to_string(),
        console_id: 1,
        seed: 0,
        player_count: 1,
        input_size: 1,
        inputs: inputs.clone(),
        snap_frames: vec![],
        assertions: vec![CompiledAssertion {
            frame: 0,
            condition: "$x > 100".to_string(),
            variable: "$x".to_string(),
            operator: CompareOp::Gt,
            value: CompiledAssertValue::Number(100.0),
        }],
        actions: vec![],
        frame_count: 1,
    };
    let mut executor = ScriptExecutor::new(script);
    values.insert("$x".to_string(), DebugValueData::F32(150.0));
    let assertion = &executor.current_assertions()[0].clone();
    assert!(executor.evaluate_assertion(assertion, &values, false));
    values.insert("$x".to_string(), DebugValueData::F32(100.0));
    assert!(!executor.evaluate_assertion(assertion, &values, false));

    // Test Le operator
    let script = CompiledScript {
        console: "zx".to_string(),
        console_id: 1,
        seed: 0,
        player_count: 1,
        input_size: 1,
        inputs: inputs.clone(),
        snap_frames: vec![],
        assertions: vec![CompiledAssertion {
            frame: 0,
            condition: "$x <= 100".to_string(),
            variable: "$x".to_string(),
            operator: CompareOp::Le,
            value: CompiledAssertValue::Number(100.0),
        }],
        actions: vec![],
        frame_count: 1,
    };
    let mut executor = ScriptExecutor::new(script);
    values.insert("$x".to_string(), DebugValueData::F32(100.0));
    let assertion = &executor.current_assertions()[0].clone();
    assert!(executor.evaluate_assertion(assertion, &values, false));
    values.insert("$x".to_string(), DebugValueData::F32(50.0));
    assert!(executor.evaluate_assertion(assertion, &values, false));
    values.insert("$x".to_string(), DebugValueData::F32(101.0));
    assert!(!executor.evaluate_assertion(assertion, &values, false));

    // Test Ge operator
    let script = CompiledScript {
        console: "zx".to_string(),
        console_id: 1,
        seed: 0,
        player_count: 1,
        input_size: 1,
        inputs,
        snap_frames: vec![],
        assertions: vec![CompiledAssertion {
            frame: 0,
            condition: "$x >= 100".to_string(),
            variable: "$x".to_string(),
            operator: CompareOp::Ge,
            value: CompiledAssertValue::Number(100.0),
        }],
        actions: vec![],
        frame_count: 1,
    };
    let mut executor = ScriptExecutor::new(script);
    values.insert("$x".to_string(), DebugValueData::F32(100.0));
    let assertion = &executor.current_assertions()[0].clone();
    assert!(executor.evaluate_assertion(assertion, &values, false));
    values.insert("$x".to_string(), DebugValueData::F32(150.0));
    assert!(executor.evaluate_assertion(assertion, &values, false));
    values.insert("$x".to_string(), DebugValueData::F32(99.0));
    assert!(!executor.evaluate_assertion(assertion, &values, false));
}

#[test]
fn test_assertion_prev_value() {
    let inputs = InputSequence::new();

    let script = CompiledScript {
        console: "zx".to_string(),
        console_id: 1,
        seed: 0,
        player_count: 1,
        input_size: 1,
        inputs,
        snap_frames: vec![0, 1],
        assertions: vec![CompiledAssertion {
            frame: 1,
            condition: "$x > $prev_x".to_string(),
            variable: "$x".to_string(),
            operator: CompareOp::Gt,
            value: CompiledAssertValue::PrevValue("x".to_string()),
        }],
        actions: vec![],
        frame_count: 2,
    };
    let mut executor = ScriptExecutor::new(script);

    // Frame 0: capture initial values
    let mut values = HashMap::new();
    values.insert("$x".to_string(), DebugValueData::F32(100.0));
    executor.capture_post_snapshot(values.clone(), values.clone(), "idle".to_string());
    executor.advance_frame();

    // Frame 1: x increased, assertion should pass
    values.insert("$x".to_string(), DebugValueData::F32(150.0));
    let assertion = &executor.current_assertions()[0].clone();
    assert!(executor.evaluate_assertion(assertion, &values, false));

    // Frame 1: x decreased, assertion should fail
    values.insert("$x".to_string(), DebugValueData::F32(50.0));
    assert!(!executor.evaluate_assertion(assertion, &values, false));
}

#[test]
fn test_assertion_variable_to_variable() {
    let inputs = InputSequence::new();

    let script = CompiledScript {
        console: "zx".to_string(),
        console_id: 1,
        seed: 0,
        player_count: 1,
        input_size: 1,
        inputs,
        snap_frames: vec![],
        assertions: vec![CompiledAssertion {
            frame: 0,
            condition: "$player_x > $enemy_x".to_string(),
            variable: "$player_x".to_string(),
            operator: CompareOp::Gt,
            value: CompiledAssertValue::Variable("$enemy_x".to_string()),
        }],
        actions: vec![],
        frame_count: 1,
    };
    let mut executor = ScriptExecutor::new(script);

    let mut values = HashMap::new();
    values.insert("$player_x".to_string(), DebugValueData::F32(200.0));
    values.insert("$enemy_x".to_string(), DebugValueData::F32(100.0));

    let assertion = &executor.current_assertions()[0].clone();
    assert!(executor.evaluate_assertion(assertion, &values, false));

    // Flip positions - should fail
    values.insert("$player_x".to_string(), DebugValueData::F32(50.0));
    assert!(!executor.evaluate_assertion(assertion, &values, false));

    // Missing variable - should fail
    values.remove("$enemy_x");
    assert!(!executor.evaluate_assertion(assertion, &values, false));
}

#[test]
fn test_assertion_different_value_types() {
    let inputs = InputSequence::new();

    let script = CompiledScript {
        console: "zx".to_string(),
        console_id: 1,
        seed: 0,
        player_count: 1,
        input_size: 1,
        inputs,
        snap_frames: vec![],
        assertions: vec![CompiledAssertion {
            frame: 0,
            condition: "$x == 100".to_string(),
            variable: "$x".to_string(),
            operator: CompareOp::Eq,
            value: CompiledAssertValue::Number(100.0),
        }],
        actions: vec![],
        frame_count: 1,
    };
    let mut executor = ScriptExecutor::new(script);
    let assertion = &executor.current_assertions()[0].clone();

    // Test I32
    let mut values = HashMap::new();
    values.insert("$x".to_string(), DebugValueData::I32(100));
    assert!(executor.evaluate_assertion(assertion, &values, false));

    // Test U32
    values.insert("$x".to_string(), DebugValueData::U32(100));
    assert!(executor.evaluate_assertion(assertion, &values, false));

    // Test F64
    values.insert("$x".to_string(), DebugValueData::F64(100.0));
    assert!(executor.evaluate_assertion(assertion, &values, false));

    // Test Bool (true = 1.0)
    values.insert("$x".to_string(), DebugValueData::Bool(true));
    let script2 = CompiledScript {
        console: "zx".to_string(),
        console_id: 1,
        seed: 0,
        player_count: 1,
        input_size: 1,
        inputs: InputSequence::new(),
        snap_frames: vec![],
        assertions: vec![CompiledAssertion {
            frame: 0,
            condition: "$x == 1".to_string(),
            variable: "$x".to_string(),
            operator: CompareOp::Eq,
            value: CompiledAssertValue::Number(1.0),
        }],
        actions: vec![],
        frame_count: 1,
    };
    let mut executor2 = ScriptExecutor::new(script2);
    let assertion2 = &executor2.current_assertions()[0].clone();
    assert!(executor2.evaluate_assertion(assertion2, &values, false));
}

#[test]
fn test_delta_computation() {
    use super::helpers::compute_delta;

    let mut pre = HashMap::new();
    pre.insert("$x".to_string(), DebugValueData::F32(100.0));
    pre.insert("$y".to_string(), DebugValueData::F32(200.0));
    pre.insert("$grounded".to_string(), DebugValueData::Bool(true));

    let mut post = HashMap::new();
    post.insert("$x".to_string(), DebugValueData::F32(105.0));
    post.insert("$y".to_string(), DebugValueData::F32(200.0)); // unchanged
    post.insert("$grounded".to_string(), DebugValueData::Bool(false));

    let delta = compute_delta(&pre, &post).unwrap();

    assert!(delta.contains_key("$x"));
    assert!(!delta.contains_key("$y")); // No change
    assert!(delta.contains_key("$grounded"));
    assert_eq!(delta.get("$grounded").unwrap(), "true -> false");
}

#[test]
fn test_executor_with_actions() {
    use crate::replay::script::{ActionParamValue, CompiledAction};

    let inputs = InputSequence::new();
    let mut params = HashMap::new();
    params.insert("level".to_string(), ActionParamValue::Int(2));

    let script = CompiledScript {
        console: "zx".to_string(),
        console_id: 1,
        seed: 0,
        player_count: 1,
        input_size: 1,
        inputs,
        snap_frames: vec![],
        assertions: vec![],
        actions: vec![CompiledAction {
            frame: 0,
            name: "Load Level".to_string(),
            params,
        }],
        frame_count: 2,
    };

    let executor = ScriptExecutor::new(script);

    // Frame 0 has actions
    assert!(executor.has_actions());
    let actions = executor.current_actions();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].name, "Load Level");
    assert!(actions[0].params.contains_key("level"));
}

#[test]
fn test_executor_multiple_actions_same_frame() {
    use crate::replay::script::{ActionParamValue, CompiledAction};

    let inputs = InputSequence::new();

    let mut params1 = HashMap::new();
    params1.insert("difficulty".to_string(), ActionParamValue::Int(3));

    let mut params2 = HashMap::new();
    params2.insert("players".to_string(), ActionParamValue::Int(2));

    let mut params3 = HashMap::new();
    params3.insert("speed".to_string(), ActionParamValue::Float(1.5));

    let script = CompiledScript {
        console: "zx".to_string(),
        console_id: 1,
        seed: 0,
        player_count: 1,
        input_size: 1,
        inputs,
        snap_frames: vec![],
        assertions: vec![],
        actions: vec![
            CompiledAction {
                frame: 0,
                name: "Set Difficulty".to_string(),
                params: params1,
            },
            CompiledAction {
                frame: 0,
                name: "Set Players".to_string(),
                params: params2,
            },
            CompiledAction {
                frame: 1,
                name: "Set Speed".to_string(),
                params: params3,
            },
        ],
        frame_count: 3,
    };

    let mut executor = ScriptExecutor::new(script);

    // Frame 0 should have 2 actions
    assert!(executor.has_actions());
    let actions = executor.current_actions();
    assert_eq!(actions.len(), 2);
    assert_eq!(actions[0].name, "Set Difficulty");
    assert_eq!(actions[1].name, "Set Players");

    // Advance to frame 1
    executor.advance_frame();
    assert!(executor.has_actions());
    let actions = executor.current_actions();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].name, "Set Speed");

    // Advance to frame 2 - no actions
    executor.advance_frame();
    assert!(!executor.has_actions());
    assert!(executor.current_actions().is_empty());
}

#[test]
fn test_executor_action_all_param_types() {
    use crate::replay::script::{ActionParamValue, CompiledAction};

    let inputs = InputSequence::new();

    let mut params = HashMap::new();
    params.insert("int_val".to_string(), ActionParamValue::Int(42));
    params.insert("float_val".to_string(), ActionParamValue::Float(3.14));
    params.insert(
        "string_val".to_string(),
        ActionParamValue::String("hello".to_string()),
    );
    params.insert("bool_val".to_string(), ActionParamValue::Bool(true));

    let script = CompiledScript {
        console: "zx".to_string(),
        console_id: 1,
        seed: 0,
        player_count: 1,
        input_size: 1,
        inputs,
        snap_frames: vec![],
        assertions: vec![],
        actions: vec![CompiledAction {
            frame: 0,
            name: "Test All Types".to_string(),
            params,
        }],
        frame_count: 1,
    };

    let executor = ScriptExecutor::new(script);
    let actions = executor.current_actions();
    assert_eq!(actions.len(), 1);

    let params = &actions[0].params;
    assert!(matches!(
        params.get("int_val"),
        Some(ActionParamValue::Int(42))
    ));
    match params.get("float_val") {
        Some(ActionParamValue::Float(f)) => assert!((*f - 3.14).abs() < 0.001),
        _ => panic!("Expected Float"),
    }
    assert!(matches!(
        params.get("string_val"),
        Some(ActionParamValue::String(s)) if s == "hello"
    ));
    assert!(matches!(
        params.get("bool_val"),
        Some(ActionParamValue::Bool(true))
    ));
}
