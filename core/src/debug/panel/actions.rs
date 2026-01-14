//! Action button rendering with parameter inputs

use hashbrown::HashMap;

use super::super::registry::RegisteredAction;
use super::super::types::ActionParamValue;
use super::ActionRequest;

/// Handles rendering of action buttons with parameter state
pub(super) struct ActionRenderer {
    /// Current parameter values for each action (action_idx -> param values)
    action_params: HashMap<usize, Vec<ActionParamValue>>,
}

impl ActionRenderer {
    pub fn new() -> Self {
        Self {
            action_params: HashMap::new(),
        }
    }

    /// Render an action button with optional parameter inputs
    ///
    /// Returns Some(ActionRequest) if the button was clicked
    pub fn render_action(
        &mut self,
        ui: &mut egui::Ui,
        action_idx: usize,
        action: &RegisteredAction,
    ) -> Option<ActionRequest> {
        // Initialize parameter values from defaults if not already set
        if !self.action_params.contains_key(&action_idx) {
            let defaults: Vec<ActionParamValue> =
                action.params.iter().map(|p| p.default_value).collect();
            self.action_params.insert(action_idx, defaults);
        }

        let mut clicked = false;

        ui.horizontal(|ui| {
            // Action name as label first
            ui.label(&action.name);

            // Render parameter inputs (compact, with name prefix)
            if let Some(params) = self.action_params.get_mut(&action_idx) {
                for (i, param) in action.params.iter().enumerate() {
                    if let Some(value) = params.get_mut(i) {
                        match value {
                            ActionParamValue::I32(v) => {
                                ui.add(egui::DragValue::new(v).prefix(format!("{}: ", param.name)));
                            }
                            ActionParamValue::F32(v) => {
                                ui.add(
                                    egui::DragValue::new(v)
                                        .speed(0.1)
                                        .prefix(format!("{}: ", param.name)),
                                );
                            }
                        }
                    }
                }
            }

            // Execution button
            if ui.button("▶").clicked() {
                clicked = true;
            }
        });

        if clicked {
            let args = self
                .action_params
                .get(&action_idx)
                .cloned()
                .unwrap_or_default();
            Some(ActionRequest {
                func_name: action.func_name.clone(),
                args,
            })
        } else {
            None
        }
    }
}
