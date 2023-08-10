use std::hash::Hash;

use color::{BOOL_COLOR, KEY_COLOR, NULL_COLOR, NUMBER_COLOR, STRING_COLOR};
use delimiters::{Delimiters, ARRAY_DELIMITERS, OBJECT_DELIMITERS};
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
        self.show_inner(ui, &mut vec![], value);
    }

    fn show_inner(&mut self, ui: &mut Ui, path_segments: &mut Vec<String>, value: &Value) {
        match value {
            Value::Null => {
                show_val(ui, &self.key, "null".to_string(), NULL_COLOR);
            }
            Value::Bool(b) => {
                show_val(ui, &self.key, b.to_string(), BOOL_COLOR);
            }
            Value::Number(n) => {
                show_val(ui, &self.key, n.to_string(), NUMBER_COLOR);
            }
            Value::String(s) => {
                show_val(ui, &self.key, format!("\"{}\"", s), STRING_COLOR);
            }
            Value::Array(arr) => {
                let iter = arr.iter().enumerate();
                self.show_expandable(path_segments, ui, iter, &ARRAY_DELIMITERS, |key| {
                    key.to_string()
                });
            }
            Value::Object(obj) => {
                let iter = obj.iter();
                self.show_expandable(path_segments, ui, iter, &OBJECT_DELIMITERS, |key| {
                    format!("\"{key}\"")
                });
            }
        };
    }

    fn show_expandable<'a, K, I>(
        &self,
        path_segments: &mut Vec<String>,
        ui: &mut Ui,
        elem_iter: I,
        delimiters: &Delimiters,
        format_key: impl Fn(&K) -> String,
    ) where
        K: ToString,
        I: Iterator<Item = (K, &'a Value)>,
    {
        let id_source = ui.make_persistent_id(generate_id(self.id, path_segments));
        let state = CollapsingState::load_with_default_open(ui.ctx(), id_source, self.default_open);
        let is_expanded = state.is_open();

        state
            .show_header(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    if let Some(key) = &self.key {
                        ui.monospace(RichText::new(key).color(KEY_COLOR));
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
                            .key(format_key(&key))
                            .show_inner(ui, path_segments, elem);
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

fn show_val(ui: &mut Ui, key: &Option<String>, value: String, color: Color32) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        if let Some(key) = key {
            ui.monospace(RichText::new(key).color(KEY_COLOR));
            ui.monospace(": ");
        }
        ui.monospace(RichText::new(value).color(color));
    });
}
