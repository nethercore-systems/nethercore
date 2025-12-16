//! Debug value registry
//!
//! Stores registered debug values and provides read/write access to WASM memory.

use super::types::{ActionParam, ActionParamValue, Constraints, DebugValue, ValueType};
use wasmtime::{Caller, Memory};

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
            log::warn!("debug_group_begin called after init - ignored");
            return;
        }
        self.group_stack.push(name.to_string());
    }

    /// End the current group
    pub fn group_end(&mut self) {
        if self.finalized {
            log::warn!("debug_group_end called after init - ignored");
            return;
        }
        if self.group_stack.pop().is_none() {
            log::warn!("debug_group_end called without matching begin");
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
            log::warn!("debug_action_begin called after init - ignored");
            return;
        }

        if self.pending_action.is_some() {
            log::warn!("debug_action_begin called while another action is pending - ignored");
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
            log::warn!("debug_action_param_i32 called after init - ignored");
            return;
        }

        if let Some(pending) = &mut self.pending_action {
            pending.params.push(ActionParam {
                name: name.to_string(),
                param_type: super::types::ActionParamType::I32,
                default_value: ActionParamValue::I32(default_value),
            });
        } else {
            log::warn!("debug_action_param_i32 called without action_begin - ignored");
        }
    }

    /// Add an f32 parameter to the pending action
    pub fn action_param_f32(&mut self, name: &str, default_value: f32) {
        if self.finalized {
            log::warn!("debug_action_param_f32 called after init - ignored");
            return;
        }

        if let Some(pending) = &mut self.pending_action {
            pending.params.push(ActionParam {
                name: name.to_string(),
                param_type: super::types::ActionParamType::F32,
                default_value: ActionParamValue::F32(default_value),
            });
        } else {
            log::warn!("debug_action_param_f32 called without action_begin - ignored");
        }
    }

    /// Finish building the pending action and register it
    pub fn action_end(&mut self) {
        if self.finalized {
            log::warn!("debug_action_end called after init - ignored");
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
            log::warn!("debug_action_end called without action_begin - ignored");
        }
    }

    /// Register a simple action with no parameters
    ///
    /// Convenience method equivalent to `action_begin` + `action_end`.
    pub fn register_action(&mut self, name: &str, func_name: &str) {
        if self.finalized {
            log::warn!("debug_register_action called after init - ignored");
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
            log::warn!("debug registration called after init - ignored");
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
            log::warn!(
                "debug: {} unclosed groups at end of init, auto-closing",
                self.group_stack.len()
            );
            self.group_stack.clear();
        }

        if self.pending_action.is_some() {
            log::warn!("debug: unclosed action at end of init, discarding");
            self.pending_action = None;
        }

        self.finalized = true;

        let total = self.values.len() + self.actions.len();
        if total > 0 {
            log::info!(
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

    /// Read a value from WASM memory
    pub fn read_value<T>(
        &self,
        caller: &mut Caller<'_, T>,
        value: &RegisteredValue,
    ) -> Option<DebugValue> {
        let memory = caller.get_export("memory").and_then(|e| e.into_memory())?;
        let data = memory.data(caller);
        let ptr = value.wasm_ptr as usize;
        let size = value.value_type.byte_size();

        // Bounds check
        if ptr + size > data.len() {
            log::warn!(
                "debug: out of bounds read for {} at ptr {}",
                value.name,
                ptr
            );
            return None;
        }

        Some(self.read_value_from_slice(&data[ptr..ptr + size], value.value_type))
    }

    /// Read a value from WASM memory using a Memory handle
    pub fn read_value_with_memory<T>(
        &self,
        memory: &Memory,
        caller: &Caller<'_, T>,
        value: &RegisteredValue,
    ) -> Option<DebugValue> {
        let data = memory.data(caller);
        let ptr = value.wasm_ptr as usize;
        let size = value.value_type.byte_size();

        // Bounds check
        if ptr + size > data.len() {
            return None;
        }

        Some(self.read_value_from_slice(&data[ptr..ptr + size], value.value_type))
    }

    /// Read a value from a byte slice
    pub fn read_value_from_slice(&self, data: &[u8], value_type: ValueType) -> DebugValue {
        match value_type {
            ValueType::I8 => DebugValue::I8(data[0] as i8),
            ValueType::U8 => DebugValue::U8(data[0]),
            ValueType::Bool => DebugValue::Bool(data[0] != 0),
            ValueType::I16 => {
                let bytes: [u8; 2] = data[..2].try_into().unwrap();
                DebugValue::I16(i16::from_le_bytes(bytes))
            }
            ValueType::U16 => {
                let bytes: [u8; 2] = data[..2].try_into().unwrap();
                DebugValue::U16(u16::from_le_bytes(bytes))
            }
            ValueType::I32 => {
                let bytes: [u8; 4] = data[..4].try_into().unwrap();
                DebugValue::I32(i32::from_le_bytes(bytes))
            }
            ValueType::U32 => {
                let bytes: [u8; 4] = data[..4].try_into().unwrap();
                DebugValue::U32(u32::from_le_bytes(bytes))
            }
            ValueType::F32 => {
                let bytes: [u8; 4] = data[..4].try_into().unwrap();
                DebugValue::F32(f32::from_le_bytes(bytes))
            }
            ValueType::Vec2 => {
                let x = f32::from_le_bytes(data[0..4].try_into().unwrap());
                let y = f32::from_le_bytes(data[4..8].try_into().unwrap());
                DebugValue::Vec2 { x, y }
            }
            ValueType::Vec3 => {
                let x = f32::from_le_bytes(data[0..4].try_into().unwrap());
                let y = f32::from_le_bytes(data[4..8].try_into().unwrap());
                let z = f32::from_le_bytes(data[8..12].try_into().unwrap());
                DebugValue::Vec3 { x, y, z }
            }
            ValueType::Rect => {
                let x = i16::from_le_bytes(data[0..2].try_into().unwrap());
                let y = i16::from_le_bytes(data[2..4].try_into().unwrap());
                let w = i16::from_le_bytes(data[4..6].try_into().unwrap());
                let h = i16::from_le_bytes(data[6..8].try_into().unwrap());
                DebugValue::Rect { x, y, w, h }
            }
            ValueType::Color => {
                // Colors stored as u32 in 0xRRGGBBAA format
                DebugValue::Color(u32::from_le_bytes(data[0..4].try_into().unwrap()))
            }
            ValueType::FixedI16Q8 => {
                let bytes: [u8; 2] = data[..2].try_into().unwrap();
                DebugValue::FixedI16Q8(i16::from_le_bytes(bytes))
            }
            ValueType::FixedI32Q16 => {
                let bytes: [u8; 4] = data[..4].try_into().unwrap();
                DebugValue::FixedI32Q16(i32::from_le_bytes(bytes))
            }
            ValueType::FixedI32Q8 => {
                let bytes: [u8; 4] = data[..4].try_into().unwrap();
                DebugValue::FixedI32Q8(i32::from_le_bytes(bytes))
            }
            ValueType::FixedI32Q24 => {
                let bytes: [u8; 4] = data[..4].try_into().unwrap();
                DebugValue::FixedI32Q24(i32::from_le_bytes(bytes))
            }
        }
    }

    /// Write a value to WASM memory
    ///
    /// Returns true if the value was written successfully and differed from the previous value.
    pub fn write_value<T>(
        &self,
        caller: &mut Caller<'_, T>,
        value: &RegisteredValue,
        new_val: &DebugValue,
    ) -> bool {
        let Some(memory) = caller.get_export("memory").and_then(|e| e.into_memory()) else {
            return false;
        };

        let data = memory.data_mut(caller);
        let ptr = value.wasm_ptr as usize;
        let size = value.value_type.byte_size();

        // Bounds check
        if ptr + size > data.len() {
            log::warn!(
                "debug: out of bounds write for {} at ptr {}",
                value.name,
                ptr
            );
            return false;
        }

        self.write_value_to_slice(&mut data[ptr..ptr + size], new_val)
    }

    /// Write a value to WASM memory using a Memory handle
    ///
    /// Returns true if the value was written successfully.
    pub fn write_value_with_memory<T>(
        &self,
        memory: &Memory,
        caller: &mut Caller<'_, T>,
        value: &RegisteredValue,
        new_val: &DebugValue,
    ) -> bool {
        let data = memory.data_mut(caller);
        let ptr = value.wasm_ptr as usize;
        let size = value.value_type.byte_size();

        // Bounds check
        if ptr + size > data.len() {
            return false;
        }

        self.write_value_to_slice(&mut data[ptr..ptr + size], new_val)
    }

    /// Write a value to a byte slice, returns true if written
    pub fn write_value_to_slice(&self, data: &mut [u8], new_val: &DebugValue) -> bool {
        match new_val {
            DebugValue::I8(v) => {
                data[0] = *v as u8;
            }
            DebugValue::U8(v) => {
                data[0] = *v;
            }
            DebugValue::Bool(v) => {
                data[0] = if *v { 1 } else { 0 };
            }
            DebugValue::I16(v) => {
                data[..2].copy_from_slice(&v.to_le_bytes());
            }
            DebugValue::U16(v) => {
                data[..2].copy_from_slice(&v.to_le_bytes());
            }
            DebugValue::I32(v) => {
                data[..4].copy_from_slice(&v.to_le_bytes());
            }
            DebugValue::U32(v) => {
                data[..4].copy_from_slice(&v.to_le_bytes());
            }
            DebugValue::F32(v) => {
                data[..4].copy_from_slice(&v.to_le_bytes());
            }
            DebugValue::Vec2 { x, y } => {
                data[0..4].copy_from_slice(&x.to_le_bytes());
                data[4..8].copy_from_slice(&y.to_le_bytes());
            }
            DebugValue::Vec3 { x, y, z } => {
                data[0..4].copy_from_slice(&x.to_le_bytes());
                data[4..8].copy_from_slice(&y.to_le_bytes());
                data[8..12].copy_from_slice(&z.to_le_bytes());
            }
            DebugValue::Rect { x, y, w, h } => {
                data[0..2].copy_from_slice(&x.to_le_bytes());
                data[2..4].copy_from_slice(&y.to_le_bytes());
                data[4..6].copy_from_slice(&w.to_le_bytes());
                data[6..8].copy_from_slice(&h.to_le_bytes());
            }
            DebugValue::Color(packed) => {
                data[..4].copy_from_slice(&packed.to_le_bytes());
            }
            DebugValue::FixedI16Q8(v) => {
                data[..2].copy_from_slice(&v.to_le_bytes());
            }
            DebugValue::FixedI32Q16(v) => {
                data[..4].copy_from_slice(&v.to_le_bytes());
            }
            DebugValue::FixedI32Q8(v) => {
                data[..4].copy_from_slice(&v.to_le_bytes());
            }
            DebugValue::FixedI32Q24(v) => {
                data[..4].copy_from_slice(&v.to_le_bytes());
            }
        }
        true
    }

    /// Build a tree structure from the flat list for UI rendering
    pub fn build_tree(&self) -> Vec<TreeNode> {
        let mut root_nodes: Vec<TreeNode> = Vec::new();

        // Insert values
        for (idx, value) in self.values.iter().enumerate() {
            let path_parts: Vec<&str> = value.full_path.split('/').collect();
            insert_value_into_tree(&mut root_nodes, &path_parts, idx);
        }

        // Insert actions
        for (idx, action) in self.actions.iter().enumerate() {
            let path_parts: Vec<&str> = action.full_path.split('/').collect();
            insert_action_into_tree(&mut root_nodes, &path_parts, idx);
        }

        root_nodes
    }
}

/// Tree node for hierarchical display
#[derive(Debug, Clone)]
pub enum TreeNode {
    /// A group containing child nodes
    Group {
        name: String,
        children: Vec<TreeNode>,
    },
    /// A leaf value (index into registry.values)
    Value(usize),
    /// A leaf action (index into registry.actions)
    Action(usize),
}

/// Helper function to insert a value into the tree
fn insert_value_into_tree(nodes: &mut Vec<TreeNode>, path_parts: &[&str], value_idx: usize) {
    if path_parts.is_empty() {
        return;
    }

    if path_parts.len() == 1 {
        // Leaf node - add the value
        nodes.push(TreeNode::Value(value_idx));
        return;
    }

    // Find or create the group for this path segment
    let group_name = path_parts[0];
    let group_idx = nodes
        .iter()
        .position(|n| matches!(n, TreeNode::Group { name, .. } if name == group_name));

    match group_idx {
        Some(idx) => {
            // Group exists, recurse into it
            if let TreeNode::Group { children, .. } = &mut nodes[idx] {
                insert_value_into_tree(children, &path_parts[1..], value_idx);
            }
        }
        None => {
            // Create new group
            let mut children = Vec::new();
            insert_value_into_tree(&mut children, &path_parts[1..], value_idx);
            nodes.push(TreeNode::Group {
                name: group_name.to_string(),
                children,
            });
        }
    }
}

/// Helper function to insert an action into the tree
fn insert_action_into_tree(nodes: &mut Vec<TreeNode>, path_parts: &[&str], action_idx: usize) {
    if path_parts.is_empty() {
        return;
    }

    if path_parts.len() == 1 {
        // Leaf node - add the action
        nodes.push(TreeNode::Action(action_idx));
        return;
    }

    // Find or create the group for this path segment
    let group_name = path_parts[0];
    let group_idx = nodes
        .iter()
        .position(|n| matches!(n, TreeNode::Group { name, .. } if name == group_name));

    match group_idx {
        Some(idx) => {
            // Group exists, recurse into it
            if let TreeNode::Group { children, .. } = &mut nodes[idx] {
                insert_action_into_tree(children, &path_parts[1..], action_idx);
            }
        }
        None => {
            // Create new group
            let mut children = Vec::new();
            insert_action_into_tree(&mut children, &path_parts[1..], action_idx);
            nodes.push(TreeNode::Group {
                name: group_name.to_string(),
                children,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_basic() {
        let mut registry = DebugRegistry::new();
        assert!(registry.is_empty());

        registry.register("test_value", 0x100, ValueType::F32, None);
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.values[0].name, "test_value");
        assert_eq!(registry.values[0].full_path, "test_value");
    }

    #[test]
    fn test_registry_groups() {
        let mut registry = DebugRegistry::new();

        registry.group_begin("player");
        registry.register("speed", 0x100, ValueType::F32, None);
        registry.register("health", 0x104, ValueType::I32, None);

        registry.group_begin("attacks");
        registry.register("damage", 0x108, ValueType::U8, None);
        registry.group_end();

        registry.group_end();

        assert_eq!(registry.len(), 3);
        assert_eq!(registry.values[0].full_path, "player/speed");
        assert_eq!(registry.values[1].full_path, "player/health");
        assert_eq!(registry.values[2].full_path, "player/attacks/damage");
    }

    #[test]
    fn test_registry_finalize() {
        let mut registry = DebugRegistry::new();
        registry.group_begin("unclosed");
        registry.register("value", 0x100, ValueType::I32, None);

        // Finalize should auto-close groups
        registry.finalize_registration();
        assert!(registry.finalized);
        assert!(registry.group_stack.is_empty());

        // Further registrations should be ignored
        registry.register("ignored", 0x200, ValueType::I32, None);
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_build_tree() {
        let mut registry = DebugRegistry::new();

        registry.group_begin("player");
        registry.register("speed", 0x100, ValueType::F32, None);
        registry.group_begin("attacks");
        registry.register("damage", 0x104, ValueType::U8, None);
        registry.group_end();
        registry.group_end();

        registry.register("global_value", 0x200, ValueType::I32, None);

        let tree = registry.build_tree();
        assert_eq!(tree.len(), 2); // player group + global_value

        // Check structure
        match &tree[0] {
            TreeNode::Group { name, children } => {
                assert_eq!(name, "player");
                assert_eq!(children.len(), 2); // speed + attacks group
            }
            _ => panic!("Expected group"),
        }
        match &tree[1] {
            TreeNode::Value(idx) => {
                assert_eq!(*idx, 2); // global_value is at index 2
            }
            _ => panic!("Expected value"),
        }
    }

    #[test]
    fn test_constraints() {
        let mut registry = DebugRegistry::new();
        let constraints = Some(Constraints::new(0.0, 100.0));
        registry.register("clamped", 0x100, ValueType::F32, constraints);

        assert!(registry.values[0].constraints.is_some());
        let c = registry.values[0].constraints.unwrap();
        assert_eq!(c.min, 0.0);
        assert_eq!(c.max, 100.0);
    }

    #[test]
    fn test_read_write_value_slice() {
        let registry = DebugRegistry::new();

        // Test f32
        let mut data = [0u8; 4];
        registry.write_value_to_slice(&mut data, &DebugValue::F32(3.14));
        let read = registry.read_value_from_slice(&data, ValueType::F32);
        assert_eq!(read, DebugValue::F32(3.14));

        // Test i32
        let mut data = [0u8; 4];
        registry.write_value_to_slice(&mut data, &DebugValue::I32(-12345));
        let read = registry.read_value_from_slice(&data, ValueType::I32);
        assert_eq!(read, DebugValue::I32(-12345));

        // Test Vec2
        let mut data = [0u8; 8];
        registry.write_value_to_slice(&mut data, &DebugValue::Vec2 { x: 1.5, y: 2.5 });
        let read = registry.read_value_from_slice(&data, ValueType::Vec2);
        assert_eq!(read, DebugValue::Vec2 { x: 1.5, y: 2.5 });

        // Test Rect
        let mut data = [0u8; 8];
        registry.write_value_to_slice(
            &mut data,
            &DebugValue::Rect {
                x: 10,
                y: 20,
                w: 30,
                h: 40,
            },
        );
        let read = registry.read_value_from_slice(&data, ValueType::Rect);
        assert_eq!(
            read,
            DebugValue::Rect {
                x: 10,
                y: 20,
                w: 30,
                h: 40
            }
        );

        // Test Color - verify byte layout matches u32 0xRRGGBBAA format
        // A game with `static COLOR: u32 = 0xFF8040FF` (R=255, G=128, B=64, A=255)
        // On little-endian, this is stored as bytes [0xFF, 0x40, 0x80, 0xFF]
        let game_bytes: [u8; 4] = 0xFF8040FFu32.to_le_bytes();
        assert_eq!(
            game_bytes,
            [0xFF, 0x40, 0x80, 0xFF],
            "Sanity check: u32 LE byte order"
        );

        // Reading from game memory should give correct RGBA
        let read = registry.read_value_from_slice(&game_bytes, ValueType::Color);
        assert_eq!(
            read,
            DebugValue::Color(0xFF8040FF),
            "Reading u32 0xFF8040FF should give Color(0xFF8040FF)"
        );

        // Writing should produce bytes that match the u32 format
        let mut data = [0u8; 4];
        registry.write_value_to_slice(&mut data, &DebugValue::Color(0xFF8040FF));
        assert_eq!(
            data, game_bytes,
            "Written bytes should match u32 0xFF8040FF layout"
        );

        // Test with different alpha to catch byte-swap bugs
        // R=255, G=0, B=0, A=1 should produce u32 = 0xFF000001
        let red_low_alpha: [u8; 4] = 0xFF000001u32.to_le_bytes();
        let mut data = [0u8; 4];
        registry.write_value_to_slice(&mut data, &DebugValue::Color(0xFF000001));
        assert_eq!(
            data, red_low_alpha,
            "Color(0xFF000001) should produce bytes for u32 0xFF000001"
        );

        // Verify round-trip
        let read = registry.read_value_from_slice(&data, ValueType::Color);
        assert_eq!(read, DebugValue::Color(0xFF000001));

        // Test Bool
        let mut data = [0u8; 1];
        registry.write_value_to_slice(&mut data, &DebugValue::Bool(true));
        let read = registry.read_value_from_slice(&data, ValueType::Bool);
        assert_eq!(read, DebugValue::Bool(true));
    }
}
