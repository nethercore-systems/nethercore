//! Tree structure for hierarchical debug UI display

use super::DebugRegistry;

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

impl DebugRegistry {
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
