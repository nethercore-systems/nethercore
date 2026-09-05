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
    /// List of registered debug actions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registered_actions: Option<Vec<DebugActionInfo>>,
    /// Automation error details, when execution could not complete.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ExecutionError>,
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
    /// Original hierarchical registry path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_path: Option<String>,
    /// Additional unambiguous names accepted by assertions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
}

/// Debug action metadata for discovery by automation clients.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DebugActionInfo {
    pub name: String,
    pub full_path: String,
    pub parameters: Vec<DebugActionParamInfo>,
}

/// Registered debug action parameter metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DebugActionParamInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub default: serde_json::Value,
}

/// Fatal execution error with phase and replay frame context.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionError {
    pub frame: u64,
    pub context: String,
    pub message: String,
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
    /// Replace the directory entry, never truncate a possibly hard-linked input inode.
    pub fn write_to_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        use std::io::Write;
        let temporary = path.with_extension(format!("report-{}.tmp", std::process::id()));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        let result = (|| -> anyhow::Result<()> {
            file.write_all(self.to_json()?.as_bytes())?;
            drop(file);
            std::fs::rename(&temporary, path)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        result
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Build a report for failures that happen before frame execution starts.
    pub fn startup_error(script: Option<String>, context: &str, message: String) -> Self {
        Self {
            version: default_version(),
            script,
            executed_at: Some(chrono::Utc::now().to_rfc3339()),
            duration_ms: Some(0),
            console: "zx".to_string(),
            seed: 0,
            frames_executed: 0,
            total_frames: 0,
            snapshots: Vec::new(),
            assertions: Vec::new(),
            registered_variables: None,
            registered_actions: None,
            error: Some(ExecutionError {
                frame: 0,
                context: context.to_string(),
                message,
            }),
            summary: ReportSummary {
                frames_with_snap: 0,
                assertions_passed: 0,
                assertions_failed: 0,
                status: "ERROR".to_string(),
            },
        }
    }

    pub fn succeeded(&self) -> bool {
        self.summary.status == "PASSED"
    }
}

#[cfg(test)]
mod tests {
    use super::ExecutionReport;

    #[test]
    fn old_reports_deserialize_without_new_metadata() {
        let report: ExecutionReport = serde_json::from_str(
            r#"{
                "version":"1.0","console":"zx","seed":1,
                "frames_executed":1,"total_frames":1,
                "snapshots":[],"assertions":[],
                "registered_variables":[{"name":"$score","type":"u32","description":"score"}],
                "summary":{"frames_with_snap":0,"assertions_passed":0,"assertions_failed":0,"status":"PASSED"}
            }"#,
        )
        .unwrap();

        assert!(report.error.is_none());
        assert!(report.registered_actions.is_none());
        assert!(report.registered_variables.unwrap()[0].aliases.is_empty());
    }
}
