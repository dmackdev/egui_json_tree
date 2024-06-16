use std::{collections::HashMap, str::FromStr};

use eframe::egui::{Frame, RichText, Ui};
use egui::{vec2, Align, Button, Color32, Layout, Margin, Rounding, Stroke, TextEdit};
use egui_json_tree::{
    delimiters::ExpandableDelimiter,
    pointer::{JsonPointerSegment, ToJsonPointerString},
    render::{DefaultRender, RenderContext, RenderValueContext},
    DefaultExpand, JsonTree,
};
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
        JsonTree::new(self.title, &self.value).show(ui);
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

        let value: serde_json::Result<Value> = serde_json::from_str(&self.input);
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
                JsonTree::new(self.title, value).show(ui);
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

        let response = JsonTree::new(self.title, &self.value)
            .default_expand(DefaultExpand::SearchResults(&self.search_input))
            .show(ui);

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

struct CopyToClipboardExample {
    title: &'static str,
    value: Value,
}

impl CopyToClipboardExample {
    fn new(value: Value) -> Self {
        Self {
            title: "Copy To Clipboard Example",
            value,
        }
    }
}

impl Show for CopyToClipboardExample {
    fn title(&self) -> &'static str {
        self.title
    }

    fn show(&mut self, ui: &mut Ui) {
        JsonTree::new(self.title, &self.value)
            .on_render(|ui, context| {
                context.render_default(ui).context_menu(|ui| {
                    ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                        ui.set_width(150.0);

                        if ui
                            .add(Button::new("Copy property path").frame(false))
                            .clicked()
                        {
                            ui.output_mut(|o| {
                                let pointer_str = context.pointer().to_json_pointer_string();
                                println!("{}", pointer_str);
                                o.copied_text = pointer_str;
                            });
                            ui.close_menu();
                        }

                        if ui.add(Button::new("Copy contents").frame(false)).clicked() {
                            if let Ok(pretty_str) = serde_json::to_string_pretty(context.value()) {
                                println!("{}", pretty_str);
                                ui.output_mut(|o| o.copied_text = pretty_str);
                            }
                            ui.close_menu();
                        }
                    });
                });
            })
            .show(ui);
    }
}

struct RenderHooksExample {
    value: Value,
    edit: bool,
    edit_state: Edit,
}

impl RenderHooksExample {
    fn new(value: Value) -> Self {
        Self {
            value,
            edit: false,
            edit_state: Default::default(),
        }
    }
}

#[derive(Default)]
struct Edit {
    object_key_edits: HashMap<String, EditObjectKeyState>,
    value_edits: HashMap<String, EditState>,
    mutations: Vec<JsonValueMutationEvent>,
}

impl Edit {
    fn edit_key_ui(
        &mut self,
        ui: &mut Ui,
        parent_pointer_str: String,
        pointer_str: String,
        key: &str,
    ) {
        let edit_state =
            self.object_key_edits
                .entry(pointer_str)
                .or_insert_with(|| EditObjectKeyState {
                    parent_pointer_str,
                    original_key: key.to_string(),
                    new_key: key.to_string(),
                });

        TextEdit::singleline(&mut edit_state.new_key)
            .code_editor()
            .margin(Margin::symmetric(2.0, 0.0))
            .clip_text(false)
            .desired_width(0.0)
            .min_size(vec2(10.0, 2.0))
            .show(ui);
    }

    fn edit_value_ui(&mut self, ui: &mut Ui, context: &RenderValueContext<Value>) {
        let edit_state = self
            .value_edits
            .entry(context.pointer.to_json_pointer_string())
            .or_insert_with(|| EditState {
                input: context.value.to_string(),
                error: None,
            });

        let mut frame = Frame::default().rounding(Rounding::same(2.0));

        if edit_state.error.is_some() {
            frame = frame.stroke(Stroke::new(1.0, Color32::RED));
        }

        let edit_response = frame
            .show(ui, |ui| {
                TextEdit::singleline(&mut edit_state.input)
                    .code_editor()
                    .margin(Margin::symmetric(2.0, 0.0))
                    .clip_text(false)
                    .desired_width(0.0)
                    .min_size(vec2(10.0, 2.0))
                    .show(ui)
                    .response
            })
            .inner;

        if edit_response.changed() {
            edit_state.error.take();
        }

        if let Some(error) = &edit_state.error {
            edit_response.on_hover_text(error);
        }
    }

    fn delete_value_ui(&mut self, ui: &mut Ui, context: &RenderValueContext<Value>) {
        ui.add_space(5.0);
        if let (Some(parent_pointer), Some(seg)) =
            (context.pointer.parent(), context.pointer.last())
        {
            if ui.small_button("x").clicked() {
                let pointer = context.pointer.to_json_pointer_string();
                let parent_pointer = parent_pointer.to_json_pointer_string();
                let mutation = match seg {
                    JsonPointerSegment::Key(key) => JsonValueMutationEvent::DeleteFromObject {
                        pointer,
                        parent_pointer,
                        key: key.to_string(),
                    },
                    JsonPointerSegment::Index(idx) => JsonValueMutationEvent::DeleteFromArray {
                        pointer,
                        parent_pointer,
                        idx: *idx,
                    },
                };
                self.mutations.push(mutation);
            }
        }
    }
}

struct EditState {
    input: String,
    error: Option<String>,
}

struct EditObjectKeyState {
    original_key: String,
    parent_pointer_str: String,
    new_key: String,
}

enum JsonValueMutationEvent {
    DeleteFromObject {
        pointer: String,
        parent_pointer: String,
        key: String,
    },
    DeleteFromArray {
        pointer: String,
        parent_pointer: String,
        idx: usize,
    },
    AddToObject {
        pointer: String,
    },
    AddToArray {
        pointer: String,
    },
}

impl Show for RenderHooksExample {
    fn title(&self) -> &'static str {
        "Render Hooks Example"
    }

    fn show(&mut self, ui: &mut Ui) {
        let (edit_toggle_response, save_button_response) = ui
            .horizontal(|ui| {
                let edit_button_text = if self.edit { "Cancel Edit" } else { "Edit" };
                (
                    ui.toggle_value(&mut self.edit, edit_button_text),
                    ui.add_enabled(self.edit, Button::new("Save")),
                )
            })
            .inner;

        JsonTree::new(self.title(), &self.value)
            .default_expand(DefaultExpand::All)
            .on_render_if(self.edit, |ui, context| {
                match context {
                    RenderContext::Property(context) => {
                        if let JsonPointerSegment::Key(key) = context.property {
                            self.edit_state.edit_key_ui(
                                ui,
                                context.pointer.parent().unwrap().to_json_pointer_string(),
                                context.pointer.to_json_pointer_string(),
                                key,
                            );
                        } else {
                            context.render_default(ui);
                        }
                    }
                    RenderContext::Value(context) => {
                        self.edit_state.edit_value_ui(ui, &context);
                        self.edit_state.delete_value_ui(ui, &context);
                    }
                    RenderContext::ExpandableDelimiter(context) => {
                        context.render_default(ui);
                        match context.delimiter {
                            ExpandableDelimiter::ClosingObject => {
                                if ui.small_button("+").clicked() {
                                    let pointer = context.pointer.to_json_pointer_string();
                                    self.edit_state
                                        .mutations
                                        .push(JsonValueMutationEvent::AddToObject { pointer })
                                }
                            }
                            ExpandableDelimiter::ClosingArray => {
                                if ui.small_button("+").clicked() {
                                    let pointer = context.pointer.to_json_pointer_string();
                                    self.edit_state
                                        .mutations
                                        .push(JsonValueMutationEvent::AddToArray { pointer })
                                }
                            }
                            _ => {}
                        }
                    }
                };
            })
            .show(ui);

        for mutation in self.edit_state.mutations.drain(..) {
            match mutation {
                JsonValueMutationEvent::DeleteFromArray {
                    pointer,
                    parent_pointer,
                    idx,
                } => {
                    let arr = self
                        .value
                        .pointer_mut(&parent_pointer)
                        .unwrap()
                        .as_array_mut()
                        .unwrap();

                    self.edit_state.value_edits.remove(&pointer);
                    arr.remove(idx);
                }
                JsonValueMutationEvent::DeleteFromObject {
                    pointer,
                    parent_pointer,
                    key,
                } => {
                    self.value
                        .pointer_mut(&parent_pointer)
                        .unwrap()
                        .as_object_mut()
                        .unwrap()
                        .remove(&key);
                    self.edit_state.value_edits.remove(&pointer);
                    self.edit_state.object_key_edits.remove(&pointer);
                }
                JsonValueMutationEvent::AddToObject { pointer } => {
                    let obj = self
                        .value
                        .pointer_mut(&pointer)
                        .unwrap()
                        .as_object_mut()
                        .unwrap();
                    obj.insert("new".to_string(), Value::Null);
                }
                JsonValueMutationEvent::AddToArray { pointer } => {
                    let arr = self
                        .value
                        .pointer_mut(&pointer)
                        .unwrap()
                        .as_array_mut()
                        .unwrap();
                    arr.push(Value::Null);
                }
            }
        }

        if save_button_response.clicked() {
            let mut save_error = false;
            for (pointer, edit_state) in self.edit_state.value_edits.iter_mut() {
                if let Some(value) = self.value.pointer_mut(pointer) {
                    match Value::from_str(&edit_state.input) {
                        Ok(new_value) => *value = new_value,
                        Err(e) => {
                            edit_state.error = Some(e.to_string());
                            save_error = true;
                        }
                    }
                }
            }

            for (_, edit_state) in self.edit_state.object_key_edits.drain() {
                if let Some(obj) = self
                    .value
                    .pointer_mut(&edit_state.parent_pointer_str)
                    .and_then(|value| value.as_object_mut())
                {
                    let value = obj.remove(&edit_state.original_key).unwrap();
                    obj.insert(edit_state.new_key, value);
                }
            }

            if !save_error {
                self.edit_state.value_edits.drain();
                self.edit = false;
            }
        }

        if edit_toggle_response.clicked() && !self.edit {
            self.edit_state.value_edits.drain();
        }
    }
}

struct DemoApp {
    examples: Vec<Box<dyn Show>>,
    open_example_idx: Option<usize>,
}

impl Default for DemoApp {
    fn default() -> Self {
        let complex_object = json!({"foo": [1, 2, [3]], "bar": { " " : false, "b": { "a/b": [4, 5, { "m~n": "Greetings!" }]}, "": 21}, "baz": null});

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
                Box::new(SearchExample::new(complex_object.clone())),
                Box::new(CopyToClipboardExample::new(complex_object.clone())),
                Box::new(RenderHooksExample::new(complex_object)),
            ],
            open_example_idx: None,
        }
    }
}

impl eframe::App for DemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("left_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    for (idx, example) in self.examples.iter().enumerate() {
                        let is_open = self
                            .open_example_idx
                            .is_some_and(|open_idx| open_idx == idx);

                        if ui.selectable_label(is_open, example.title()).clicked() {
                            if is_open {
                                self.open_example_idx = None;
                            } else {
                                self.open_example_idx = Some(idx);
                            }
                        }
                    }
                });
            });

        match self.open_example_idx {
            Some(open_idx) => {
                let example = &mut self.examples[open_idx];
                egui::TopBottomPanel::top("top-panel")
                    .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(10.0))
                    .show(ctx, |ui| {
                        ui.heading(example.title());
                    });
                egui::CentralPanel::default().show(ctx, |ui| {
                    example.show(ui);
                });
            }
            None => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| {
                            ui.heading("Select an example.");
                        },
                    );
                });
            }
        }
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }
}

fn main() {
    let _ = eframe::run_native(
        "egui-json-tree example",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::<DemoApp>::default()),
    );
}
