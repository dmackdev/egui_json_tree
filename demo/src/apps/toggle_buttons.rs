use egui::Ui;
use egui_json_tree::{DefaultExpand, JsonTree, JsonTreeStyle, ToggleButtonsState};
use serde_json::Value;

use super::Show;

pub struct ToggleButtonsExample {
    value: Value,
    toggle_buttons_state: ToggleButtonsState,
}

impl ToggleButtonsExample {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            toggle_buttons_state: Default::default(),
        }
    }
}

impl Show for ToggleButtonsExample {
    fn title(&self) -> &'static str {
        "Toggle Buttons Customisation"
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.hyperlink_to(
            "Source",
            "https://github.com/dmackdev/egui_json_tree/blob/main/demo/src/apps/toggle_buttons.rs",
        );
        ui.label("Use the buttons below to control the visibility and interactivity of the toggle buttons.");

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
            ui.add_space(10.0);

            JsonTree::new(self.title(), &self.value)
                .default_expand(DefaultExpand::All)
                .style(JsonTreeStyle::new().toggle_buttons_state(self.toggle_buttons_state))
                .show(ui);
        });
    }
}
