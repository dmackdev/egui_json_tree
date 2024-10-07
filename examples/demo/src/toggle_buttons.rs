use egui::Ui;
use egui_json_tree::{DefaultExpand, JsonTree, ToggleButtonsState};
use serde_json::Value;

use crate::example::Show;

pub struct ToggleButtonsCustomisationDemo {
    value: Value,
    toggle_buttons_state: ToggleButtonsState,
}

impl ToggleButtonsCustomisationDemo {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            toggle_buttons_state: Default::default(),
        }
    }
}

impl Show for ToggleButtonsCustomisationDemo {
    fn title(&self) -> &'static str {
        "Toggle Buttons Customisation"
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.toggle_buttons_state,
                    ToggleButtonsState::VisibleEnabled,
                    "Visible and enabled",
                );
                ui.selectable_value(
                    &mut self.toggle_buttons_state,
                    ToggleButtonsState::VisibleDisabled,
                    "Visible and disabled",
                );
                ui.selectable_value(
                    &mut self.toggle_buttons_state,
                    ToggleButtonsState::Hidden,
                    "Hidden",
                );
            });

            JsonTree::new(self.title(), &self.value)
                .default_expand(DefaultExpand::All)
                .toggle_buttons_state(self.toggle_buttons_state)
                .show(ui);
        });
    }
}
