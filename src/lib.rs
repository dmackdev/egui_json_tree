use std::{fmt::Display, hash::Hash};

use delimiters::{Delimiters, ARRAY_DELIMITERS, OBJECT_DELIMITERS};
use egui::{collapsing_header::CollapsingState, Id};
use serde_json::Value;

mod delimiters;

pub struct JsonTree {
    id: Id,
    prefix: String,
}

impl JsonTree {
    pub fn new(id: impl Hash) -> Self {
        Self {
            id: Id::new(id),
            prefix: "".to_string(),
        }
    }

    fn prefix(mut self, prefix: String) -> Self {
        self.prefix = prefix;
        self
    }

    pub fn show(mut self, ui: &mut egui::Ui, value: &Value) {
        self.show_inner(ui, &mut vec![], value);
    }

    fn show_inner(&mut self, ui: &mut egui::Ui, path_segments: &mut Vec<String>, value: &Value) {
        match value {
            Value::Null => {
                ui.monospace(format!("{}null", self.prefix));
            }
            Value::Bool(b) => {
                ui.monospace(format!("{}{}", self.prefix, b));
            }
            Value::Number(n) => {
                ui.monospace(format!("{}{}", self.prefix, n));
            }
            Value::String(s) => {
                ui.monospace(format!("{}\"{}\"", self.prefix, s));
            }
            Value::Array(arr) => {
                let iter = arr.iter().enumerate();
                self.show_expandable(path_segments, ui, iter, &ARRAY_DELIMITERS);
            }
            Value::Object(obj) => {
                let iter = obj.iter().map(|(k, v)| (format!("\"{}\"", k), v));
                self.show_expandable(path_segments, ui, iter, &OBJECT_DELIMITERS);
            }
        };
    }

    fn show_expandable<'a, K, I>(
        &self,
        path_segments: &mut Vec<String>,
        ui: &mut egui::Ui,
        elem_iter: I,
        delimiters: &Delimiters,
    ) where
        K: Display,
        I: Iterator<Item = (K, &'a Value)>,
    {
        let id_source = ui.make_persistent_id(generate_id(self.id, path_segments));
        let state = CollapsingState::load_with_default_open(ui.ctx(), id_source, false);
        let is_expanded = state.is_open();

        let header = format!(
            "{}{}",
            &self.prefix,
            if is_expanded {
                delimiters.opening
            } else {
                delimiters.collapsed
            }
        );

        state
            .show_header(ui, |ui| {
                ui.label(header);
            })
            .body(|ui| {
                for (key, elem) in elem_iter {
                    path_segments.push(key.to_string());

                    let mut add_nested_tree = |ui: &mut egui::Ui| {
                        ui.visuals_mut().indent_has_left_vline = true;

                        JsonTree::new(generate_id(self.id, path_segments))
                            .prefix(format!("{key} : "))
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
