use std::collections::{HashMap, HashSet};

use egui_json_tree::JsonTree;
use serde_json::{json, Value};

struct DemoApp {
    examples: Vec<(String, Value)>,
    open_example_titles: HashMap<String, bool>,
    expanded_paths: HashMap<String, HashSet<String>>,
}

impl Default for DemoApp {
    fn default() -> Self {
        Self {
            examples: vec![
                ("Null".to_string(), json!(null)),
                ("Bool".to_string(), json!(true)),
                ("Number (int)".to_string(), json!(42)),
                ("Number (neg int)".to_string(), json!(-273)),
                ("Number (float)".to_string(), json!(13.37)),
                ("String".to_string(), json!("This is a string!")),
                ("Array".to_string(), json!([1, 2, 3])),
                (
                    "Nested Arrays".to_string(),
                    json!([1, [2, 3, 4], [5, 6, 7, 8]]),
                ),
                (
                    "Object".to_string(),
                    json!({"foo": 123, "bar": "Hello world!", "baz": null}),
                ),
            ],
            open_example_titles: HashMap::new(),
            expanded_paths: HashMap::new(),
        }
    }
}

impl eframe::App for DemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("left_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    for (title, _) in self.examples.iter() {
                        let is_open = self
                            .open_example_titles
                            .entry(title.to_string())
                            .or_default();

                        ui.toggle_value(is_open, title);
                    }
                });
            });

        for (title, value) in self.examples.iter() {
            let is_open = self
                .open_example_titles
                .entry(title.to_string())
                .or_default();

            let expanded_paths = self.expanded_paths.entry(title.clone()).or_default();

            egui::Window::new(title).open(is_open).show(ctx, |ui| {
                JsonTree::new(value).show(ui, expanded_paths);
            });
        }
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }
}

fn main() {
    let _ = eframe::run_native(
        "egui-modal example",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::<DemoApp>::default()),
    );
}
