//! WASM error parsing utilities
//!
//! Parses wasmtime errors into user-friendly `GameError` structures with
//! helpful suggestions for debugging.

use super::types::{GameError, GameErrorPhase};

/// Parse a WASM runtime error into a structured `GameError`
///
/// Extracts useful information from wasmtime error messages and provides
/// user-friendly summaries with debugging suggestions.
///
/// # Arguments
/// * `error` - The error to parse (typically from wasmtime)
/// * `tick` - The tick number when the error occurred (if known)
/// * `phase` - Which game lifecycle phase failed (init/update/render)
pub fn parse_wasm_error(error: &anyhow::Error, tick: Option<u64>, phase: GameErrorPhase) -> GameError {
    let error_str = format!("{:#}", error);

    // Classify the trap type and get suggestions
    let (summary, suggestions) = classify_trap(&error_str);

    // Extract stack trace if present
    let stack_trace = extract_stack_trace(&error_str);

    GameError {
        summary,
        details: error_str,
        stack_trace,
        tick,
        phase,
        suggestions,
    }
}

/// Classify the type of WASM trap and provide helpful information
fn classify_trap(error: &str) -> (String, Vec<String>) {
    let error_lower = error.to_lowercase();

    if error_lower.contains("out of bounds memory access") {
        (
            "Memory Access Error".to_string(),
            vec![
                "Check array bounds before access".to_string(),
                "Verify pointer arithmetic".to_string(),
                "Ensure data structures are properly sized".to_string(),
                "Check for use-after-free in unsafe code".to_string(),
            ],
        )
    } else if error_lower.contains("unreachable") {
        (
            "Assertion Failed / Panic".to_string(),
            vec![
                "Check panic!() or unreachable!() locations".to_string(),
                "Review Option::unwrap() and Result::unwrap() calls".to_string(),
                "Verify match statements cover all cases".to_string(),
            ],
        )
    } else if error_lower.contains("integer overflow") {
        (
            "Integer Overflow".to_string(),
            vec![
                "Use checked arithmetic (checked_add, checked_mul, etc.)".to_string(),
                "Use saturating arithmetic if overflow is acceptable".to_string(),
                "Verify value ranges before operations".to_string(),
            ],
        )
    } else if error_lower.contains("integer divide by zero") || error_lower.contains("division by zero") {
        (
            "Division by Zero".to_string(),
            vec![
                "Check divisor is non-zero before division".to_string(),
                "Use checked_div() for safe division".to_string(),
                "Verify loop counters and indices".to_string(),
            ],
        )
    } else if error_lower.contains("call stack exhausted") || error_lower.contains("stack overflow") {
        (
            "Stack Overflow".to_string(),
            vec![
                "Check for infinite recursion".to_string(),
                "Reduce recursion depth or use iteration".to_string(),
                "Avoid large stack allocations".to_string(),
            ],
        )
    } else if error_lower.contains("indirect call type mismatch") {
        (
            "Function Type Mismatch".to_string(),
            vec![
                "Check function pointer casts".to_string(),
                "Verify callback signatures match expectations".to_string(),
                "Review dynamic dispatch code".to_string(),
            ],
        )
    } else if error_lower.contains("null") {
        (
            "Null Reference".to_string(),
            vec![
                "Check Option values before unwrapping".to_string(),
                "Verify references are initialized".to_string(),
                "Use Option::as_ref() for safe access".to_string(),
            ],
        )
    } else if error_lower.contains("uninitialized") {
        (
            "Uninitialized Memory".to_string(),
            vec![
                "Ensure all variables are initialized".to_string(),
                "Check for read-before-write in unsafe code".to_string(),
            ],
        )
    } else if error_lower.contains("memory") {
        (
            "Memory Error".to_string(),
            vec![
                "Check memory allocation limits".to_string(),
                "Verify data sizes and alignments".to_string(),
            ],
        )
    } else if error_lower.contains("failed") && error_lower.contains("init") {
        (
            "Initialization Error".to_string(),
            vec![
                "Check init() function implementation".to_string(),
                "Verify resource loading succeeds".to_string(),
                "Check for errors in FFI calls".to_string(),
            ],
        )
    } else {
        (
            "Runtime Error".to_string(),
            vec![
                "Check the error details below for more information".to_string(),
                "Review recent code changes".to_string(),
            ],
        )
    }
}

/// Extract stack trace information from the error message
///
/// Wasmtime errors may include stack frame information when debug info
/// is available in the WASM module.
fn extract_stack_trace(error: &str) -> Option<Vec<String>> {
    let mut frames = Vec::new();

    for line in error.lines() {
        let trimmed = line.trim();

        // Look for common stack frame patterns from wasmtime
        // Pattern: "    at <function> (offset 0xNNNN)"
        // Pattern: "    <N>: 0xNNNN - <function>"
        // Pattern: "wasm backtrace:"
        if trimmed.starts_with("at ") || trimmed.contains("wasm backtrace") {
            frames.push(trimmed.to_string());
        } else if trimmed.starts_with("0x") || (trimmed.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) && trimmed.contains(":")) {
            // Numbered frames like "0: 0x1234 - func_name"
            frames.push(trimmed.to_string());
        }
    }

    if frames.is_empty() {
        None
    } else {
        Some(frames)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_oob() {
        let (summary, suggestions) = classify_trap("wasm trap: out of bounds memory access");
        assert_eq!(summary, "Memory Access Error");
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_classify_unreachable() {
        let (summary, _) = classify_trap("wasm trap: unreachable");
        assert_eq!(summary, "Assertion Failed / Panic");
    }

    #[test]
    fn test_classify_overflow() {
        let (summary, _) = classify_trap("wasm trap: integer overflow");
        assert_eq!(summary, "Integer Overflow");
    }

    #[test]
    fn test_classify_divide_by_zero() {
        let (summary, _) = classify_trap("wasm trap: integer divide by zero");
        assert_eq!(summary, "Division by Zero");
    }

    #[test]
    fn test_classify_stack_overflow() {
        let (summary, _) = classify_trap("wasm trap: call stack exhausted");
        assert_eq!(summary, "Stack Overflow");
    }

    #[test]
    fn test_classify_unknown() {
        let (summary, _) = classify_trap("some unknown error");
        assert_eq!(summary, "Runtime Error");
    }

    #[test]
    fn test_parse_wasm_error() {
        let error = anyhow::anyhow!("WASM update() failed at tick 42: wasm trap: out of bounds memory access");
        let game_error = parse_wasm_error(&error, Some(42), GameErrorPhase::Update);

        assert_eq!(game_error.summary, "Memory Access Error");
        assert_eq!(game_error.tick, Some(42));
        assert_eq!(game_error.phase, GameErrorPhase::Update);
        assert!(!game_error.suggestions.is_empty());
    }

    #[test]
    fn test_extract_stack_trace() {
        let error = "wasm trap: unreachable\nwasm backtrace:\n    at update (offset 0x1234)\n    at main (offset 0x5678)";
        let trace = extract_stack_trace(error);
        assert!(trace.is_some());
        let frames = trace.unwrap();
        assert!(frames.len() >= 2);
    }

    #[test]
    fn test_extract_stack_trace_empty() {
        let error = "simple error with no stack trace";
        let trace = extract_stack_trace(error);
        assert!(trace.is_none());
    }
}
