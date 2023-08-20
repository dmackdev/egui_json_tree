use std::collections::HashMap;

use eframe::egui::{RichText, Ui};
use egui_json_tree::{Expand, JsonTree};
use serde_json::{json, Value};

trait Show {
    fn title(&self) -> &'static str;
    fn show(&mut self, ui: &mut Ui);
}

struct Example {
    title: &'static str,
    value: Value,
}

impl Example {
    fn new(title: &'static str, value: Value) -> Self {
        Self { title, value }
    }
}

impl Show for Example {
    fn title(&self) -> &'static str {
        self.title
    }

    fn show(&mut self, ui: &mut Ui) {
        JsonTree::new(self.title, &self.value).show(ui, Expand::None);
    }
}

struct CustomExample {
    title: &'static str,
    input: String,
}

impl CustomExample {
    fn new(title: &'static str) -> Self {
        Self {
            title,
            input: serde_json::to_string_pretty(&json!({"foo": "bar"})).unwrap(),
        }
    }
}

impl Show for CustomExample {
    fn title(&self) -> &'static str {
        self.title
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.label("Enter raw JSON in the text box to see the visualisation below.");

        ui.add_space(ui.spacing().item_spacing.y);
        ui.add(
            egui::TextEdit::multiline(&mut self.input)
                .code_editor()
                .desired_rows(4)
                .desired_width(f32::INFINITY),
        );

        let value = serde_json::from_str(&self.input);
        let pretty_string = value
            .as_ref()
            .ok()
            .and_then(|v| serde_json::to_string_pretty(v).ok());

        ui.add_space(ui.spacing().item_spacing.y);
        ui.add_enabled_ui(pretty_string.is_some(), |ui| {
            if ui.button("Beautify").clicked() {
                self.input = pretty_string.unwrap();
            }
        });

        ui.add_space(ui.spacing().item_spacing.y);
        ui.separator();

        match value.as_ref() {
            Ok(value) => {
                JsonTree::new(self.title, value).show(ui, Expand::None);
            }
            Err(err) => {
                ui.label(RichText::new(err.to_string()).color(ui.visuals().error_fg_color));
            }
        };

        ui.add_space(ui.spacing().item_spacing.y);
    }
}

struct SearchExample {
    title: &'static str,
    value: Value,
    search_input: String,
}

impl SearchExample {
    fn new(value: Value) -> Self {
        Self {
            title: "Search Example",
            value,
            search_input: "".to_string(),
        }
    }
}

impl Show for SearchExample {
    fn title(&self) -> &'static str {
        self.title
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.label("Search:");

        let (text_edit_response, clear_button_response) = ui
            .horizontal(|ui| {
                let text_edit_response = ui.text_edit_singleline(&mut self.search_input);
                let clear_button_response = ui.button("Clear");
                (text_edit_response, clear_button_response)
            })
            .inner;

        let tree = JsonTree::new(self.title, &self.value);
        let response = tree.show(ui, Expand::SearchResults(self.search_input.clone()));

        if text_edit_response.changed() {
            response.reset_expanded(ui);
        }

        if clear_button_response.clicked() {
            self.search_input.clear();
            response.reset_expanded(ui);
        }

        if ui.button("Reset expanded").clicked() {
            response.reset_expanded(ui);
        }
    }
}

struct DemoApp {
    examples: Vec<Box<dyn Show>>,
    open_example_titles: HashMap<&'static str, bool>,
}

impl Default for DemoApp {
    fn default() -> Self {
        let complex_object = json!({"foo": [1, 2, [3]], "bar": { "a" : false, "b": { "fizz": [4, 5, { "x": "Greetings!" }]}, "c": 21}, "baz": null});

        Self {
            examples: vec![
                Box::new(Example::new("Null", json!(null))),
                Box::new(Example::new("Bool", json!(true))),
                Box::new(Example::new("Number (int)", json!(42))),
                Box::new(Example::new("Number (neg int)", json!(-273))),
                Box::new(Example::new("Number (float)", json!(13.37))),
                Box::new(Example::new("String", json!("This is a string!"))),
                Box::new(Example::new("Array", json!([1, 2, 3]))),
                Box::new(Example::new(
                    "Nested Arrays",
                    json!([1, [2, 3, 4], [5, 6, [7], 8], [9, [[], 10]]]),
                )),
                Box::new(Example::new(
                    "Object",
                    json!({"foo": 123, "bar": "Hello world!", "baz": null}),
                )),
                Box::new(Example::new("Complex Object", complex_object.clone())),
                Box::new(CustomExample::new("Custom Input")),
                Box::new(SearchExample::new(complex_object)),
            ],
            open_example_titles: HashMap::new(),
        }
    }
}

impl eframe::App for DemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("left_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    for example in self.examples.iter() {
                        let is_open = self.open_example_titles.entry(example.title()).or_default();

                        ui.toggle_value(is_open, example.title());
                    }
                });
            });

        for example in self.examples.iter_mut() {
            let is_open = self.open_example_titles.entry(example.title()).or_default();

            egui::Window::new(example.title())
                .open(is_open)
                .show(ctx, |ui| example.show(ui));
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
