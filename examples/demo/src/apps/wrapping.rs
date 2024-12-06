use egui::{DragValue, Ui};
use egui_json_tree::{
    DefaultExpand, JsonTree, JsonTreeMaxWidth, JsonTreeStyle, JsonTreeWrapping,
    JsonTreeWrappingParams,
};
use serde_json::Value;

use super::Show;

pub struct WrappingExample {
    value: Value,
    state: JsonTreeWrapping,
}

impl WrappingExample {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            state: JsonTreeWrapping {
                max_rows: 1,
                max_width: JsonTreeMaxWidth::UiAvailableWidth,
            },
        }
    }
}

impl Show for WrappingExample {
    fn title(&self) -> &'static str {
        "Wrapping"
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.hyperlink_to("Source", "https://github.com/dmackdev/egui_json_tree/blob/master/examples/demo/src/apps/wrapping.rs");
        ui.add_space(10.0);

        ui.label(egui::RichText::new("Max Rows:").monospace());
        ui.add(
            DragValue::new(&mut self.state.max_rows)
                .speed(0.1)
                .range(1..=5),
        );

        ui.label(egui::RichText::new("Max Width:").monospace());
        ui.horizontal(|ui| {
            if ui
                .radio(
                    matches!(self.state.max_width, JsonTreeMaxWidth::Pt(_)),
                    "Points",
                )
                .clicked()
            {
                self.state.max_width = JsonTreeMaxWidth::Pt(100.0);
            }
            if let JsonTreeMaxWidth::Pt(ref mut pts) = &mut self.state.max_width {
                if pts.is_finite() {
                    ui.add(DragValue::new(pts).speed(10.0).range(100.0..=10000.0));
                }
            }
        });

        if ui
            .radio(
                matches!(self.state.max_width, JsonTreeMaxWidth::UiAvailableWidth),
                "Available Width",
            )
            .clicked()
        {
            self.state.max_width = JsonTreeMaxWidth::UiAvailableWidth;
        }

        JsonTree::new(self.title(), &self.value)
            .style(JsonTreeStyle::new().wrap(JsonTreeWrappingParams {
                value_no_parent: self.state,
                value_expanded_parent: self.state,
                value_collapsed_root: self.state,
            }))
            .default_expand(DefaultExpand::All)
            .show(ui);
    }
}
