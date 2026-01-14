//! Execution report types and serialization

use crate::replay::types::{AssertionResult, Snapshot};

/// Execution report
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionReport {
    /// Report format version
    #[serde(default = "default_version")]
    pub version: String,
    /// Script file name (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    /// Execution timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executed_at: Option<String>,
    /// Execution duration in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    /// Console name
    pub console: String,
    /// Random seed used
    pub seed: u64,
    /// Frames executed
    pub frames_executed: u64,
    /// Total frames in script
    pub total_frames: u64,
    /// Captured snapshots
    pub snapshots: Vec<Snapshot>,
    /// Assertion results
    pub assertions: Vec<AssertionResult>,
    /// List of registered debug variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registered_variables: Option<Vec<DebugVariableInfo>>,
    /// Summary statistics
    pub summary: ReportSummary,
}

fn default_version() -> String {
    "1.0".to_string()
}

/// Debug variable metadata for reporting
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DebugVariableInfo {
    /// Variable name (e.g., "$player_x")
    pub name: String,
    /// Type name (e.g., "i32", "f32", "bool")
    #[serde(rename = "type")]
    pub type_name: String,
    /// Human-readable description
    pub description: String,
}

/// Report summary
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReportSummary {
    /// Number of frames with snapshots
    pub frames_with_snap: usize,
    /// Number of passed assertions
    pub assertions_passed: usize,
    /// Number of failed assertions
    pub assertions_failed: usize,
    /// Overall status
    pub status: String,
}

impl ExecutionReport {
    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
