use std::str::FromStr;

use egui::{
    CursorIcon, Key, Margin, TextEdit, Ui,
    text::{CCursor, CCursorRange},
    vec2,
};
use egui_json_tree::{
    DefaultExpand, JsonTree, JsonTreeStyle, ToggleButtonsState,
    delimiters::ExpandableDelimiter,
    pointer::JsonPointerSegment,
    render::{
        DefaultRender, RenderBaseValueContext, RenderContext, RenderExpandableDelimiterContext,
        RenderPropertyContext,
    },
};
use serde_json::Value;

use super::Show;

pub struct JsonEditorExample {
    value: Value,
    editor: Editor,
}

impl JsonEditorExample {
    pub fn new(value: Value) -> Self {
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
                    let enter_was_pressed_with_focus = Self::show_text_edit(
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
                        if ui.small_button("✅").clicked() || enter_was_pressed_with_focus {
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
        if let RenderContext::BaseValue(context) = &context {
            if state.pointer == context.pointer.to_json_pointer_string() {
                let enter_was_pressed_with_focus =
                    Self::show_text_edit(ui, &mut state.new_value_input, &mut state.request_focus);

                ui.add_space(5.0);

                if ui.small_button("✅").clicked() || enter_was_pressed_with_focus {
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
            RenderContext::BaseValue(context) => {
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
        mut context: RenderPropertyContext<'_, '_, Value>,
    ) {
        context
            .render_default(ui)
            .on_hover_cursor(CursorIcon::ContextMenu)
            .context_menu(|ui| {
                if context.value.is_object() && ui.button("Add to object").clicked() {
                    self.edit_events.push(EditEvent::AddToObject {
                        pointer: context.pointer.to_json_pointer_string(),
                    });
                    if let Some(state) = context.collapsing_state.as_mut() {
                        state.set_open(true);
                    }
                }

                if context.value.is_array() && ui.button("Add to array").clicked() {
                    self.edit_events.push(EditEvent::AddToArray {
                        pointer: context.pointer.to_json_pointer_string(),
                    });
                    if let Some(state) = context.collapsing_state.as_mut() {
                        state.set_open(true);
                    }
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
                    }
                }
            });
    }

    fn show_value_context_menu(
        &mut self,
        ui: &mut Ui,
        context: RenderBaseValueContext<'_, '_, Value>,
    ) {
        context
            .render_default(ui)
            .on_hover_cursor(CursorIcon::ContextMenu)
            .context_menu(|ui| {
                if ui.button("Edit value").clicked() {
                    self.state = Some(EditState::EditValue(EditValueState {
                        pointer: context.pointer.to_json_pointer_string(),
                        new_value_input: context.value.to_string(),
                        request_focus: true,
                    }));
                }

                match (context.pointer.parent(), context.pointer.last()) {
                    (Some(parent), Some(JsonPointerSegment::Key(key))) => {
                        if ui.button("Delete").clicked() {
                            self.edit_events.push(EditEvent::DeleteFromObject {
                                object_pointer: parent.to_json_pointer_string(),
                                key: key.to_string(),
                            });
                        }
                    }
                    (Some(parent), Some(JsonPointerSegment::Index(idx))) => {
                        if ui.button("Delete").clicked() {
                            self.edit_events.push(EditEvent::DeleteFromArray {
                                array_pointer: parent.to_json_pointer_string(),
                                idx: *idx,
                            });
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
            ExpandableDelimiter::OpeningArray
            | ExpandableDelimiter::CollapsedArray
            | ExpandableDelimiter::CollapsedEmptyArray
            | ExpandableDelimiter::ClosingArray => {
                context
                    .render_default(ui)
                    .on_hover_cursor(CursorIcon::ContextMenu)
                    .context_menu(|ui| {
                        if ui.button("Add to array").clicked() {
                            self.edit_events.push(EditEvent::AddToArray {
                                pointer: context.pointer.to_json_pointer_string(),
                            });
                            context.collapsing_state.set_open(true);
                        }
                    });
            }
            ExpandableDelimiter::OpeningObject
            | ExpandableDelimiter::CollapsedObject
            | ExpandableDelimiter::CollapsedEmptyObject
            | ExpandableDelimiter::ClosingObject => {
                context
                    .render_default(ui)
                    .on_hover_cursor(CursorIcon::ContextMenu)
                    .context_menu(|ui| {
                        if ui.button("Add to object").clicked() {
                            self.edit_events.push(EditEvent::AddToObject {
                                pointer: context.pointer.to_json_pointer_string(),
                            });
                            context.collapsing_state.set_open(true);
                        }
                    });
            }
        };
    }

    /// Returns `bool` indicating whether the Enter key was pressed whilst the text edit had focus.
    fn show_text_edit(ui: &mut Ui, input: &mut String, request_focus: &mut bool) -> bool {
        // Wrap in horizontal to prevent jitters when typing when children are expanded (due to use of horizontal_wrapped when rendering properties).
        let text_edit_output = ui
            .horizontal(|ui| {
                TextEdit::singleline(input)
                    .code_editor()
                    .margin(Margin::symmetric(2, 0))
                    .clip_text(false)
                    .desired_width(0.0)
                    .min_size(vec2(10.0, 2.0))
                    .return_key(None) // Disable return key so we can capture Enter key press for submission.
                    .show(ui)
            })
            .inner;

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

        text_edit_output.response.has_focus() && ui.input(|i| i.key_pressed(Key::Enter))
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
        "JSON Editor"
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.hyperlink_to(
            "Source",
            "https://github.com/dmackdev/egui_json_tree/blob/main/demo/src/apps/editor.rs",
        );
        ui.label("Right click on elements within the tree to edit values and object keys, and add/remove values.");
        ui.add_space(10.0);

        let toggle_buttons_state = match self.editor.state {
            Some(_) => ToggleButtonsState::VisibleDisabled,
            None => ToggleButtonsState::VisibleEnabled,
        };

        let style = JsonTreeStyle::new()
            .abbreviate_root(true)
            .toggle_buttons_state(toggle_buttons_state);

        JsonTree::new(self.title(), &self.value)
            .default_expand(DefaultExpand::All)
            .style(style)
            .on_render(|ui, context| self.editor.show(ui, &self.value, context))
            .show(ui);

        self.editor.apply_events(&mut self.value);
    }
}
