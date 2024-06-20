use std::str::FromStr;

use eframe::egui::{RichText, Ui};
use egui::{
    text::{CCursor, CCursorRange},
    vec2, Align, Button, Layout, Margin, TextEdit,
};
use egui_json_tree::{
    delimiters::ExpandableDelimiter,
    pointer::{JsonPointerSegment, ToJsonPointerString},
    render::{
        DefaultRender, RenderContext, RenderExpandableDelimiterContext, RenderPropertyContext,
        RenderValueContext,
    },
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

                        let pointer = context.pointer().to_json_pointer_string();
                        if !pointer.is_empty()
                            && ui.add(Button::new("Copy path").frame(false)).clicked()
                        {
                            ui.output_mut(|o| {
                                println!("{}", pointer);
                                o.copied_text = pointer;
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

struct JsonEditorExample {
    value: Value,
    editor: Editor,
}

impl JsonEditorExample {
    fn new(value: Value) -> Self {
        Self {
            value,
            editor: Default::default(),
        }
    }
}

#[derive(Default)]
struct Editor {
    edit_events: Vec<EditEvent>,
    state: Option<EditState>,
}

impl Editor {
    fn show(&mut self, ui: &mut Ui, document: &Value, context: RenderContext<'_, '_, Value>) {
        match self.state.as_mut() {
            Some(EditState::EditObjectKey(state)) => {
                Self::show_edit_object_key(ui, document, context, state, &mut self.edit_events)
            }
            Some(EditState::EditValue(state)) => {
                Self::show_edit_value(ui, context, state, &mut self.edit_events);
            }
            None => {
                self.show_with_context_menus(ui, context);
            }
        };
    }

    fn show_edit_object_key(
        ui: &mut Ui,
        document: &Value,
        context: RenderContext<Value>,
        state: &mut EditObjectKeyState,
        edit_events: &mut Vec<EditEvent>,
    ) {
        if let RenderContext::Property(context) = &context {
            if let JsonPointerSegment::Key(key) = context.property {
                if key == state.key
                    && context
                        .pointer
                        .parent()
                        .map(|parent| parent.to_json_pointer_string())
                        .is_some_and(|object_pointer| object_pointer == state.object_pointer)
                {
                    Self::show_text_edit_with_focus(
                        ui,
                        &mut state.new_key_input,
                        &mut state.request_focus,
                    );

                    ui.add_space(5.0);

                    let valid_key = state.key == state.new_key_input
                        || document
                            .pointer(&state.object_pointer)
                            .and_then(|v| v.as_object())
                            .is_some_and(|obj| !obj.contains_key(&state.new_key_input));

                    ui.add_enabled_ui(valid_key, |ui| {
                        if ui.small_button("✅").clicked() {
                            edit_events.push(EditEvent::SaveObjectKeyEdit);
                        }
                    });

                    ui.add_space(5.0);

                    if ui.small_button("❌").clicked() {
                        if state.is_new_key {
                            edit_events.push(EditEvent::DeleteFromObject {
                                object_pointer: state.object_pointer.to_string(),
                                key: key.to_string(),
                            });
                        }
                        edit_events.push(EditEvent::CloseObjectKeyEdit);
                    }
                    return;
                }
            }
        }
        context.render_default(ui);
    }

    fn show_edit_value(
        ui: &mut Ui,
        context: RenderContext<Value>,
        state: &mut EditValueState,
        edit_events: &mut Vec<EditEvent>,
    ) {
        if let RenderContext::Value(context) = &context {
            if state.pointer == context.pointer.to_json_pointer_string() {
                Self::show_text_edit_with_focus(
                    ui,
                    &mut state.new_value_input,
                    &mut state.request_focus,
                );

                ui.add_space(5.0);

                if ui.small_button("✅").clicked() {
                    edit_events.push(EditEvent::SaveValueEdit);
                }

                ui.add_space(5.0);

                if ui.small_button("❌").clicked() {
                    edit_events.push(EditEvent::CloseValueEdit);
                }
                return;
            }
        }
        context.render_default(ui);
    }

    fn show_with_context_menus(&mut self, ui: &mut Ui, context: RenderContext<Value>) {
        match context {
            RenderContext::Property(context) => {
                self.show_property_context_menu(ui, context);
            }
            RenderContext::Value(context) => {
                self.show_value_context_menu(ui, context);
            }
            RenderContext::ExpandableDelimiter(context) => {
                self.show_expandable_delimiter_context_menu(ui, context);
            }
        };
    }

    fn show_property_context_menu(
        &mut self,
        ui: &mut Ui,
        context: RenderPropertyContext<'_, '_, Value>,
    ) {
        context.render_default(ui).context_menu(|ui| {
            if context.value.is_object() && ui.button("Add to object").clicked() {
                self.edit_events.push(EditEvent::AddToObject {
                    pointer: context.pointer.to_json_pointer_string(),
                });
                ui.close_menu();
            }

            if context.value.is_array() && ui.button("Add to array").clicked() {
                self.edit_events.push(EditEvent::AddToArray {
                    pointer: context.pointer.to_json_pointer_string(),
                });
                ui.close_menu();
            }

            if let Some(parent) = context.pointer.parent() {
                if let JsonPointerSegment::Key(key) = &context.property {
                    if ui.button("Edit key").clicked() {
                        self.state = Some(EditState::EditObjectKey(EditObjectKeyState {
                            key: key.to_string(),
                            object_pointer: parent.to_json_pointer_string(),
                            new_key_input: key.to_string(),
                            request_focus: true,
                            is_new_key: false,
                        }));
                        ui.close_menu()
                    }
                }

                if ui.button("Delete").clicked() {
                    let event = match context.property {
                        JsonPointerSegment::Key(key) => EditEvent::DeleteFromObject {
                            object_pointer: parent.to_json_pointer_string(),
                            key: key.to_string(),
                        },
                        JsonPointerSegment::Index(idx) => EditEvent::DeleteFromArray {
                            array_pointer: parent.to_json_pointer_string(),
                            idx,
                        },
                    };
                    self.edit_events.push(event);
                    ui.close_menu();
                }
            }
        });
    }

    fn show_value_context_menu(&mut self, ui: &mut Ui, context: RenderValueContext<'_, '_, Value>) {
        context.render_default(ui).context_menu(|ui| {
            if ui.button("Edit value").clicked() {
                self.state = Some(EditState::EditValue(EditValueState {
                    pointer: context.pointer.to_json_pointer_string(),
                    new_value_input: context.value.to_string(),
                    request_focus: true,
                }));
                ui.close_menu();
            }

            match (context.pointer.parent(), context.pointer.last()) {
                (Some(parent), Some(JsonPointerSegment::Key(key))) => {
                    if ui.button("Delete").clicked() {
                        self.edit_events.push(EditEvent::DeleteFromObject {
                            object_pointer: parent.to_json_pointer_string(),
                            key: key.to_string(),
                        });
                        ui.close_menu();
                    }
                }
                (Some(parent), Some(JsonPointerSegment::Index(idx))) => {
                    if ui.button("Delete").clicked() {
                        self.edit_events.push(EditEvent::DeleteFromArray {
                            array_pointer: parent.to_json_pointer_string(),
                            idx: *idx,
                        });
                        ui.close_menu();
                    }
                }
                _ => {}
            };
        });
    }

    fn show_expandable_delimiter_context_menu(
        &mut self,
        ui: &mut Ui,
        context: RenderExpandableDelimiterContext<'_, '_, Value>,
    ) {
        match context.delimiter {
            ExpandableDelimiter::OpeningArray => {
                context.render_default(ui).context_menu(|ui| {
                    if ui.button("Add to array").clicked() {
                        self.edit_events.push(EditEvent::AddToArray {
                            pointer: context.pointer.to_json_pointer_string(),
                        });
                        ui.close_menu();
                    }
                });
            }
            ExpandableDelimiter::OpeningObject => {
                context.render_default(ui).context_menu(|ui| {
                    if ui.button("Add to object").clicked() {
                        self.edit_events.push(EditEvent::AddToObject {
                            pointer: context.pointer.to_json_pointer_string(),
                        });
                        ui.close_menu();
                    }
                });
            }
            _ => {
                context.render_default(ui);
            }
        };
    }

    fn show_text_edit_with_focus(ui: &mut Ui, input: &mut String, request_focus: &mut bool) {
        let text_edit_output = TextEdit::singleline(input)
            .code_editor()
            .margin(Margin::symmetric(2.0, 0.0))
            .clip_text(false)
            .desired_width(0.0)
            .min_size(vec2(10.0, 2.0))
            .show(ui);

        if *request_focus {
            *request_focus = false;
            let text_edit_id = text_edit_output.response.id;
            if let Some(mut text_edit_state) = TextEdit::load_state(ui.ctx(), text_edit_id) {
                text_edit_state
                    .cursor
                    .set_char_range(Some(CCursorRange::two(
                        CCursor::new(0),
                        CCursor::new(input.len()),
                    )));
                text_edit_state.store(ui.ctx(), text_edit_id);
                ui.ctx().memory_mut(|mem| mem.request_focus(text_edit_id));
            }
        }
    }

    fn apply_events(&mut self, document: &mut Value) {
        for event in self.edit_events.drain(..) {
            match event {
                EditEvent::DeleteFromArray { array_pointer, idx } => {
                    if let Some(arr) = document
                        .pointer_mut(&array_pointer)
                        .and_then(|value| value.as_array_mut())
                    {
                        arr.remove(idx);
                    }
                }
                EditEvent::DeleteFromObject {
                    object_pointer,
                    key,
                } => {
                    if let Some(obj) = document
                        .pointer_mut(&object_pointer)
                        .and_then(|value| value.as_object_mut())
                    {
                        obj.remove(&key);
                    }
                }
                EditEvent::AddToObject { pointer } => {
                    if let Some(obj) = document
                        .pointer_mut(&pointer)
                        .and_then(|value| value.as_object_mut())
                    {
                        let mut counter = 0;
                        let mut new_key = "new_key".to_string();

                        while obj.contains_key(&new_key) {
                            counter += 1;
                            new_key = format!("new_key_{counter}");
                        }

                        obj.insert(new_key.clone(), Value::Null);

                        self.state = Some(EditState::EditObjectKey(EditObjectKeyState {
                            key: new_key.clone(),
                            object_pointer: pointer,
                            new_key_input: new_key,
                            request_focus: true,
                            is_new_key: true,
                        }));
                    }
                }
                EditEvent::AddToArray { pointer } => {
                    if let Some(arr) = document
                        .pointer_mut(&pointer)
                        .and_then(|value| value.as_array_mut())
                    {
                        arr.push(Value::Null);
                    }
                }
                EditEvent::SaveValueEdit => {
                    if let Some(EditState::EditValue(value_edit)) = self.state.take() {
                        if let Some(value) = document.pointer_mut(&value_edit.pointer) {
                            match Value::from_str(&value_edit.new_value_input) {
                                Ok(new_value) => *value = new_value,
                                Err(_) => *value = Value::String(value_edit.new_value_input),
                            }
                        }
                    }
                }
                EditEvent::SaveObjectKeyEdit => {
                    if let Some(EditState::EditObjectKey(object_key_edit)) = self.state.take() {
                        let obj = document
                            .pointer_mut(&object_key_edit.object_pointer)
                            .and_then(|value| value.as_object_mut());

                        if let Some(obj) = obj {
                            if let Some(value) = obj.remove(&object_key_edit.key) {
                                obj.insert(object_key_edit.new_key_input, value);
                            }
                        }
                    }
                }
                EditEvent::CloseObjectKeyEdit | EditEvent::CloseValueEdit => {
                    self.state.take();
                }
            }
        }
    }
}

enum EditState {
    EditObjectKey(EditObjectKeyState),
    EditValue(EditValueState),
}

struct EditObjectKeyState {
    key: String,
    object_pointer: String,
    new_key_input: String,
    request_focus: bool,
    is_new_key: bool,
}

struct EditValueState {
    pointer: String,
    new_value_input: String,
    request_focus: bool,
}

enum EditEvent {
    DeleteFromObject { object_pointer: String, key: String },
    DeleteFromArray { array_pointer: String, idx: usize },
    AddToObject { pointer: String },
    AddToArray { pointer: String },
    SaveValueEdit,
    SaveObjectKeyEdit,
    CloseObjectKeyEdit,
    CloseValueEdit,
}

impl Show for JsonEditorExample {
    fn title(&self) -> &'static str {
        "JSON Editor Example"
    }

    fn show(&mut self, ui: &mut Ui) {
        JsonTree::new(self.title(), &self.value)
            .abbreviate_root(true)
            .default_expand(DefaultExpand::All)
            .on_render(|ui, context| self.editor.show(ui, &self.value, context))
            .show(ui);

        self.editor.apply_events(&mut self.value);
    }
}

struct DemoApp {
    examples: Vec<Box<dyn Show>>,
    open_example_idx: Option<usize>,
}

impl Default for DemoApp {
    fn default() -> Self {
        let complex_object = json!({"foo": [1, 2, [3]], "bar": { "qux" : false, "thud": { "a/b": [4, 5, { "m~n": "Greetings!" }]}, "grep": 21}, "baz": null});

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
                Box::new(JsonEditorExample::new(complex_object)),
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
