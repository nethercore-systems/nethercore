//! Debug inspection panel UI
//!
//! Provides an egui panel for viewing and editing registered debug values and actions.

use hashbrown::HashSet;

use super::export::export_as_rust_flat;
use super::frame_control::{FrameController, TIME_SCALE_OPTIONS};
use super::registry::{DebugRegistry, RegisteredValue, TreeNode};
use super::types::{ActionParamValue, DebugValue};

mod actions;
mod widgets;

use actions::ActionRenderer;
use widgets::ValueWidgetRenderer;

/// Request to invoke a debug action
#[derive(Debug, Clone)]
pub struct ActionRequest {
    /// Name of the WASM function to call
    pub func_name: String,
    /// Arguments to pass to the function
    pub args: Vec<ActionParamValue>,
}

/// Debug inspection panel state
pub struct DebugPanel {
    /// Whether the panel is visible
    pub visible: bool,
    /// Set of collapsed group paths
    collapsed_groups: HashSet<String>,
    /// Cached tree structure (rebuilt when values change)
    tree_cache: Option<Vec<TreeNode>>,
    /// Whether tree needs to be rebuilt
    tree_dirty: bool,
    /// Action renderer (handles parameter state)
    action_renderer: ActionRenderer,
    /// Value widget renderer
    value_renderer: ValueWidgetRenderer,
}

impl Default for DebugPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugPanel {
    /// Create a new debug panel
    pub fn new() -> Self {
        Self {
            visible: false,
            collapsed_groups: HashSet::new(),
            tree_cache: None,
            tree_dirty: true,
            action_renderer: ActionRenderer::new(),
            value_renderer: ValueWidgetRenderer::new(),
        }
    }

    /// Toggle panel visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Set panel visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Mark tree as needing rebuild (call after registry changes)
    pub fn invalidate_tree(&mut self) {
        self.tree_dirty = true;
    }

    /// Render the debug panel
    ///
    /// Returns (value_changed, action_request):
    /// - value_changed: true if any value was changed (caller should invoke on_debug_change)
    /// - action_request: Some if an action button was clicked (caller should invoke the action)
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        registry: &DebugRegistry,
        frame_controller: &mut FrameController,
        read_value: impl Fn(&RegisteredValue) -> Option<DebugValue>,
        write_value: impl Fn(&RegisteredValue, &DebugValue) -> bool,
    ) -> (bool, Option<ActionRequest>) {
        if !self.visible || registry.is_empty() {
            return (false, None);
        }

        // Rebuild tree if needed
        if self.tree_dirty || self.tree_cache.is_none() {
            self.tree_cache = Some(registry.build_tree());
            self.tree_dirty = false;
        }

        let mut any_changed = false;
        let mut action_request: Option<ActionRequest> = None;

        egui::Window::new("Debug Inspector")
            .id(egui::Id::new("debug_inspection_window"))
            .default_pos([10.0, 10.0])
            .default_size([320.0, 400.0])
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                // Frame controls
                if !frame_controller.is_disabled() {
                    any_changed |= self.render_frame_controls(ui, frame_controller);
                    ui.separator();
                }

                // Scrollable area for values and actions
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // Clone tree to avoid borrow conflict with render_tree(&mut self)
                        if let Some(tree) = self.tree_cache.clone() {
                            let (changed, action) = self.render_tree(
                                ui,
                                &tree,
                                registry,
                                "",
                                &read_value,
                                &write_value,
                            );
                            any_changed |= changed;
                            if action.is_some() {
                                action_request = action;
                            }
                        }
                    });

                ui.separator();

                // Export buttons
                self.render_export_buttons(ui, registry, &read_value);
            });

        (any_changed, action_request)
    }

    /// Render frame control buttons
    fn render_frame_controls(
        &self,
        ui: &mut egui::Ui,
        frame_controller: &mut FrameController,
    ) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            // Pause/Play toggle
            let pause_text = if frame_controller.is_paused() {
                "▶ Play"
            } else {
                "⏸ Pause"
            };
            if ui.button(pause_text).clicked() {
                frame_controller.toggle_pause();
                changed = true;
            }

            // Step frame (only when paused)
            ui.add_enabled_ui(frame_controller.is_paused(), |ui| {
                if ui.button("⏭ Step").clicked() {
                    frame_controller.request_step();
                    changed = true;
                }
            });
        });

        ui.horizontal(|ui| {
            ui.label("Speed:");

            // Time scale dropdown
            let current_scale = frame_controller.time_scale();
            egui::ComboBox::from_id_salt("time_scale")
                .selected_text(format!("{}x", current_scale))
                .show_ui(ui, |ui| {
                    for &scale in &TIME_SCALE_OPTIONS {
                        if ui
                            .selectable_value(
                                &mut frame_controller.time_scale_index(),
                                TIME_SCALE_OPTIONS.iter().position(|&s| s == scale).unwrap(),
                                format!("{}x", scale),
                            )
                            .clicked()
                        {
                            frame_controller.set_time_scale(scale);
                            changed = true;
                        }
                    }
                });
        });

        changed
    }

    /// Render the value tree recursively
    ///
    /// Returns (any_changed, action_request)
    fn render_tree(
        &mut self,
        ui: &mut egui::Ui,
        nodes: &[TreeNode],
        registry: &DebugRegistry,
        parent_path: &str,
        read_value: &impl Fn(&RegisteredValue) -> Option<DebugValue>,
        write_value: &impl Fn(&RegisteredValue, &DebugValue) -> bool,
    ) -> (bool, Option<ActionRequest>) {
        let mut any_changed = false;
        let mut action_request: Option<ActionRequest> = None;

        for node in nodes {
            match node {
                TreeNode::Group { name, children } => {
                    let path = if parent_path.is_empty() {
                        name.clone()
                    } else {
                        format!("{}/{}", parent_path, name)
                    };

                    let is_collapsed = self.collapsed_groups.contains(&path);
                    let header = egui::CollapsingHeader::new(name)
                        .default_open(!is_collapsed)
                        .show(ui, |ui| {
                            let (changed, action) = self.render_tree(
                                ui,
                                children,
                                registry,
                                &path,
                                read_value,
                                write_value,
                            );
                            any_changed |= changed;
                            if action.is_some() {
                                action_request = action;
                            }
                        });

                    // Track collapse state
                    if header.header_response.clicked() {
                        if is_collapsed {
                            self.collapsed_groups.remove(&path);
                        } else {
                            self.collapsed_groups.insert(path);
                        }
                    }
                }
                TreeNode::Value(idx) => {
                    if let Some(reg_value) = registry.values.get(*idx)
                        && let Some(current) = read_value(reg_value)
                        && let Some(new_val) = self
                            .value_renderer
                            .render_value_widget(ui, reg_value, current)
                        && write_value(reg_value, &new_val)
                    {
                        any_changed = true;
                    }
                }
                TreeNode::Action(idx) => {
                    if let Some(reg_action) = registry.actions.get(*idx)
                        && let Some(request) =
                            self.action_renderer.render_action(ui, *idx, reg_action)
                    {
                        action_request = Some(request);
                    }
                }
            }
        }

        (any_changed, action_request)
    }

    /// Render export buttons
    fn render_export_buttons(
        &self,
        ui: &mut egui::Ui,
        registry: &DebugRegistry,
        read_value: &impl Fn(&RegisteredValue) -> Option<DebugValue>,
    ) {
        ui.horizontal(|ui| {
            if ui.button("📋 Copy as Rust").clicked() {
                let text = export_as_rust_flat(registry, read_value);
                ui.ctx().copy_text(text);
            }
        });
    }
}
