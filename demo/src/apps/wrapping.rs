use egui::{DragValue, Ui};
use egui_json_tree::{
    DefaultExpand, JsonTree, JsonTreeMaxWidth, JsonTreeStyle, JsonTreeWrapping,
    JsonTreeWrappingConfig,
};
use serde_json::Value;

use super::Show;

pub struct WrappingExample {
    value: Value,
    wrap: JsonTreeWrapping,
    use_custom_max_rows: bool,
}

impl WrappingExample {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            wrap: JsonTreeWrapping {
                max_rows: 1,
                max_width: JsonTreeMaxWidth::UiAvailableWidth,
                break_anywhere: true,
            },
            use_custom_max_rows: true,
        }
    }
}

impl Show for WrappingExample {
    fn title(&self) -> &'static str {
        "Wrapping"
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.hyperlink_to("Source", "https://github.com/dmackdev/egui_json_tree/blob/main/examples/demo/src/apps/wrapping.rs");
        ui.label("Use the controls below to configure the text wrapping options for primitive JSON values within the visualisation.");
        ui.label("Text is truncated when it cannot fit in the specified width and number of rows.");
        ui.add_space(10.0);

        self.show_max_rows_controls(ui);
        ui.add_space(10.0);

        self.show_max_width_controls(ui);
        ui.add_space(10.0);

        ui.checkbox(&mut self.wrap.break_anywhere, "Break anywhere");
        ui.separator();

        let wrapping_config = JsonTreeWrappingConfig {
            value_when_root: self.wrap,
            value_with_expanded_parent: self.wrap,
            value_in_collapsed_root: self.wrap,
        };
        JsonTree::new(self.title(), &self.value)
            .style(JsonTreeStyle::new().wrapping_config(wrapping_config))
            .default_expand(DefaultExpand::All)
            .show(ui);
    }
}

impl WrappingExample {
    fn show_max_rows_controls(&mut self, ui: &mut Ui) {
        ui.label(egui::RichText::new("Max Rows:").monospace());
        ui.horizontal(|ui| {
            if ui
                .radio_value(&mut self.use_custom_max_rows, true, "Custom")
                .changed()
            {
                self.wrap.max_rows = 1;
            }

            if self.use_custom_max_rows {
                ui.add(
                    DragValue::new(&mut self.wrap.max_rows)
                        .speed(0.1)
                        .range(1..=10),
                );
            }
        });

        if ui
            .radio_value(&mut self.use_custom_max_rows, false, "usize::MAX")
            .clicked()
        {
            self.wrap.max_rows = usize::MAX;
        }
    }

    fn show_max_width_controls(&mut self, ui: &mut Ui) {
        ui.label(egui::RichText::new("Max Width:").monospace());
        ui.horizontal(|ui| {
            if ui
                .radio(
                    matches!(self.wrap.max_width, JsonTreeMaxWidth::Points(_)),
                    "Points",
                )
                .clicked()
                && !matches!(self.wrap.max_width, JsonTreeMaxWidth::Points(_))
            {
                self.wrap.max_width = JsonTreeMaxWidth::Points(100.0);
            }
            if let JsonTreeMaxWidth::Points(pts) = &mut self.wrap.max_width {
                ui.add(DragValue::new(pts).speed(10.0).range(100.0..=10000.0));
            }
        });

        if ui
            .radio(
                matches!(self.wrap.max_width, JsonTreeMaxWidth::UiAvailableWidth),
                "Available Width",
            )
            .clicked()
        {
            self.wrap.max_width = JsonTreeMaxWidth::UiAvailableWidth;
        }
    }
}
