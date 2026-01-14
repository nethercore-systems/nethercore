//! Debug value registry
//!
//! Stores registered debug values and provides read/write access to WASM memory.

mod memory;
mod tests;
mod tree;

pub use tree::*;

use super::types::{ActionParam, ActionParamValue, Constraints, ValueType};

/// A registered debug value
#[derive(Debug, Clone)]
pub struct RegisteredValue {
    /// Display name of the value
    pub name: String,
    /// Full hierarchical path (e.g., "player/attacks/punch_hitbox")
    pub full_path: String,
    /// Pointer into WASM linear memory
    pub wasm_ptr: u32,
    /// Type of the value
    pub value_type: ValueType,
    /// Optional range constraints (for sliders)
    pub constraints: Option<Constraints>,
    /// Whether this value is read-only (watch mode)
    pub read_only: bool,
}

/// A registered debug action (button that calls a WASM function)
#[derive(Debug, Clone)]
pub struct RegisteredAction {
    /// Display name of the action (button label)
    pub name: String,
    /// Full hierarchical path (e.g., "spawning/spawn_enemy")
    pub full_path: String,
    /// Name of the WASM function to call
    pub func_name: String,
    /// Parameters to pass to the function
    pub params: Vec<ActionParam>,
}

/// Builder state for action registration
struct PendingAction {
    name: String,
    full_path: String,
    func_name: String,
    params: Vec<ActionParam>,
}

/// Registry of debug values and actions
#[derive(Default)]
pub struct DebugRegistry {
    /// All registered values
    pub values: Vec<RegisteredValue>,
    /// All registered actions
    pub actions: Vec<RegisteredAction>,
    /// Current group path stack during registration
    group_stack: Vec<String>,
    /// Whether registration has been finalized (after init completes)
    pub finalized: bool,
    /// Pending action being built (between action_begin and action_end)
    pending_action: Option<PendingAction>,
}

impl Clone for DebugRegistry {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
            actions: self.actions.clone(),
            group_stack: self.group_stack.clone(),
            finalized: self.finalized,
            pending_action: None, // Don't clone pending state
        }
    }
}

impl DebugRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Begin a new group for organizing values
    pub fn group_begin(&mut self, name: &str) {
        if self.finalized {
            tracing::warn!("debug_group_begin called after init - ignored");
            return;
        }
        self.group_stack.push(name.to_string());
    }

    /// End the current group
    pub fn group_end(&mut self) {
        if self.finalized {
            tracing::warn!("debug_group_end called after init - ignored");
            return;
        }
        if self.group_stack.pop().is_none() {
            tracing::warn!("debug_group_end called without matching begin");
        }
    }

    /// Register a value for debug inspection
    pub fn register(
        &mut self,
        name: &str,
        wasm_ptr: u32,
        value_type: ValueType,
        constraints: Option<Constraints>,
    ) {
        self.register_internal(name, wasm_ptr, value_type, constraints, false);
    }

    /// Register a read-only watch value for debug inspection
    pub fn watch(&mut self, name: &str, wasm_ptr: u32, value_type: ValueType) {
        self.register_internal(name, wasm_ptr, value_type, None, true);
    }

    // =========================================================================
    // Action Registration (builder pattern)
    // =========================================================================

    /// Begin building an action with optional parameters
    ///
    /// Call `action_param_*` methods to add parameters, then `action_end` to finish.
    pub fn action_begin(&mut self, name: &str, func_name: &str) {
        if self.finalized {
            tracing::warn!("debug_action_begin called after init - ignored");
            return;
        }

        if self.pending_action.is_some() {
            tracing::warn!("debug_action_begin called while another action is pending - ignored");
            return;
        }

        // Build full path from group stack
        let full_path = if self.group_stack.is_empty() {
            name.to_string()
        } else {
            format!("{}/{}", self.group_stack.join("/"), name)
        };

        self.pending_action = Some(PendingAction {
            name: name.to_string(),
            full_path,
            func_name: func_name.to_string(),
            params: Vec::new(),
        });
    }

    /// Add an i32 parameter to the pending action
    pub fn action_param_i32(&mut self, name: &str, default_value: i32) {
        if self.finalized {
            tracing::warn!("debug_action_param_i32 called after init - ignored");
            return;
        }

        if let Some(pending) = &mut self.pending_action {
            pending.params.push(ActionParam {
                name: name.to_string(),
                param_type: super::types::ActionParamType::I32,
                default_value: ActionParamValue::I32(default_value),
            });
        } else {
            tracing::warn!("debug_action_param_i32 called without action_begin - ignored");
        }
    }

    /// Add an f32 parameter to the pending action
    pub fn action_param_f32(&mut self, name: &str, default_value: f32) {
        if self.finalized {
            tracing::warn!("debug_action_param_f32 called after init - ignored");
            return;
        }

        if let Some(pending) = &mut self.pending_action {
            pending.params.push(ActionParam {
                name: name.to_string(),
                param_type: super::types::ActionParamType::F32,
                default_value: ActionParamValue::F32(default_value),
            });
        } else {
            tracing::warn!("debug_action_param_f32 called without action_begin - ignored");
        }
    }

    /// Finish building the pending action and register it
    pub fn action_end(&mut self) {
        if self.finalized {
            tracing::warn!("debug_action_end called after init - ignored");
            return;
        }

        if let Some(pending) = self.pending_action.take() {
            self.actions.push(RegisteredAction {
                name: pending.name,
                full_path: pending.full_path,
                func_name: pending.func_name,
                params: pending.params,
            });
        } else {
            tracing::warn!("debug_action_end called without action_begin - ignored");
        }
    }

    /// Register a simple action with no parameters
    ///
    /// Convenience method equivalent to `action_begin` + `action_end`.
    pub fn register_action(&mut self, name: &str, func_name: &str) {
        if self.finalized {
            tracing::warn!("debug_register_action called after init - ignored");
            return;
        }

        // Build full path from group stack
        let full_path = if self.group_stack.is_empty() {
            name.to_string()
        } else {
            format!("{}/{}", self.group_stack.join("/"), name)
        };

        self.actions.push(RegisteredAction {
            name: name.to_string(),
            full_path,
            func_name: func_name.to_string(),
            params: Vec::new(),
        });
    }

    /// Internal registration with read_only flag
    fn register_internal(
        &mut self,
        name: &str,
        wasm_ptr: u32,
        value_type: ValueType,
        constraints: Option<Constraints>,
        read_only: bool,
    ) {
        if self.finalized {
            tracing::warn!("debug registration called after init - ignored");
            return;
        }

        // Build full path from group stack
        let full_path = if self.group_stack.is_empty() {
            name.to_string()
        } else {
            format!("{}/{}", self.group_stack.join("/"), name)
        };

        self.values.push(RegisteredValue {
            name: name.to_string(),
            full_path,
            wasm_ptr,
            value_type,
            constraints,
            read_only,
        });
    }

    /// Finalize registration (called after init completes)
    ///
    /// This auto-closes any unclosed groups/actions and prevents further registration.
    pub fn finalize_registration(&mut self) {
        if !self.group_stack.is_empty() {
            tracing::warn!(
                "debug: {} unclosed groups at end of init, auto-closing",
                self.group_stack.len()
            );
            self.group_stack.clear();
        }

        if self.pending_action.is_some() {
            tracing::warn!("debug: unclosed action at end of init, discarding");
            self.pending_action = None;
        }

        self.finalized = true;

        let total = self.values.len() + self.actions.len();
        if total > 0 {
            tracing::info!(
                "Debug inspection: registered {} values, {} actions",
                self.values.len(),
                self.actions.len()
            );
        }
    }

    /// Clear the registry (for game reload)
    pub fn clear(&mut self) {
        self.values.clear();
        self.actions.clear();
        self.group_stack.clear();
        self.pending_action = None;
        self.finalized = false;
    }

    /// Get number of registered values
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if registry is empty (no values or actions)
    pub fn is_empty(&self) -> bool {
        self.values.is_empty() && self.actions.is_empty()
    }
}
