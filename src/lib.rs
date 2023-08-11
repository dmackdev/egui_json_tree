use std::hash::Hash;

use color::{
    ARRAY_IDX_COLOR, BOOL_COLOR, NULL_COLOR, NUMBER_COLOR, OBJECT_KEY_COLOR, STRING_COLOR,
};
use delimiters::{ARRAY_DELIMITERS, OBJECT_DELIMITERS};
use egui::{collapsing_header::CollapsingState, Color32, Id, RichText, Ui};
use serde_json::Value;

mod color;
mod delimiters;

pub struct JsonTree {
    id: Id,
    key: Option<String>,
    default_open: bool,
}

impl JsonTree {
    pub fn new(id: impl Hash) -> Self {
        Self {
            id: Id::new(id),
            key: None,
            default_open: false,
        }
    }

    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    fn key(mut self, key: String) -> Self {
        self.key = Some(key);
        self
    }

    pub fn show(mut self, ui: &mut Ui, value: &Value) {
        self.show_inner(ui, &mut vec![], value, None);
    }

    fn show_inner(
        &mut self,
        ui: &mut Ui,
        path_segments: &mut Vec<String>,
        value: &Value,
        parent: Option<Expandable>,
    ) {
        let key_text = get_key_text(&self.key, parent);

        match value {
            Value::Null => {
                show_val(ui, key_text, "null".to_string(), NULL_COLOR);
            }
            Value::Bool(b) => {
                show_val(ui, key_text, b.to_string(), BOOL_COLOR);
            }
            Value::Number(n) => {
                show_val(ui, key_text, n.to_string(), NUMBER_COLOR);
            }
            Value::String(s) => {
                show_val(ui, key_text, format!("\"{}\"", s), STRING_COLOR);
            }
            Value::Array(arr) => {
                let iter = arr.iter().enumerate();
                self.show_expandable(path_segments, ui, iter, parent, Expandable::Array);
            }
            Value::Object(obj) => {
                let iter = obj.iter();
                self.show_expandable(path_segments, ui, iter, parent, Expandable::Object);
            }
        };
    }

    fn show_expandable<'a, K, I>(
        &self,
        path_segments: &mut Vec<String>,
        ui: &mut Ui,
        elem_iter: I,
        parent: Option<Expandable>,
        expandable: Expandable,
    ) where
        K: ToString,
        I: Iterator<Item = (K, &'a Value)>,
    {
        let delimiters = match expandable {
            Expandable::Array => &ARRAY_DELIMITERS,
            Expandable::Object => &OBJECT_DELIMITERS,
        };

        let id_source = ui.make_persistent_id(generate_id(self.id, path_segments));
        let state = CollapsingState::load_with_default_open(ui.ctx(), id_source, self.default_open);
        let is_expanded = state.is_open();

        state
            .show_header(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;

                    if let Some(key_text) = get_key_text(&self.key, parent) {
                        ui.monospace(key_text);
                        ui.monospace(": ");
                    }

                    ui.label(if is_expanded {
                        delimiters.opening
                    } else {
                        delimiters.collapsed
                    });
                });
            })
            .body(|ui| {
                for (key, elem) in elem_iter {
                    path_segments.push(key.to_string());

                    let mut add_nested_tree = |ui: &mut Ui| {
                        ui.visuals_mut().indent_has_left_vline = true;

                        JsonTree::new(generate_id(self.id, path_segments))
                            .default_open(self.default_open)
                            .key(key.to_string())
                            .show_inner(ui, path_segments, elem, Some(expandable));
                    };

                    ui.visuals_mut().indent_has_left_vline = false;

                    if is_expandable(elem) {
                        add_nested_tree(ui);
                    } else {
                        let original_indent = ui.spacing().indent;

                        ui.spacing_mut().indent =
                            ui.spacing().icon_width + ui.spacing().icon_spacing;

                        ui.indent(id_source, |ui| add_nested_tree(ui));

                        ui.spacing_mut().indent = original_indent;
                    }

                    path_segments.pop();
                }
            });

        if is_expanded {
            ui.horizontal(|ui| {
                let indent = ui.spacing().icon_width / 2.0;
                ui.add_space(indent);

                ui.monospace(delimiters.closing);
            });
        }
    }
}

fn is_expandable(value: &Value) -> bool {
    matches!(value, Value::Array(_) | Value::Object(_))
}

fn generate_id(id: Id, path: &[String]) -> Id {
    Id::new(format!("{:?}-{}", id, path.join("/")))
}

fn get_key_text(key: &Option<String>, parent: Option<Expandable>) -> Option<RichText> {
    match (key, parent) {
        (Some(key), Some(Expandable::Array)) => Some(format_array_idx(key)),
        (Some(key), Some(Expandable::Object)) => Some(format_object_key(key)),
        _ => None,
    }
}

fn show_val(ui: &mut Ui, key_text: Option<RichText>, value: String, color: Color32) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        if let Some(key_text) = key_text {
            ui.monospace(key_text);
            ui.monospace(": ");
        }
        ui.monospace(RichText::new(value).color(color));
    });
}

fn format_object_key(key: &String) -> RichText {
    RichText::new(format!("\"{}\"", key)).color(OBJECT_KEY_COLOR)
}

fn format_array_idx(idx: &String) -> RichText {
    RichText::new(idx).color(ARRAY_IDX_COLOR)
}

#[derive(Clone, Copy)]
enum Expandable {
    Array,
    Object,
}
