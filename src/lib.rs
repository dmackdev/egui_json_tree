use std::{collections::HashSet, fmt::Display, hash::Hash};

use delimiters::{Delimiters, ARRAY_DELIMITERS, OBJECT_DELIMITERS};
use egui::Id;
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

    pub fn show(mut self, ui: &mut egui::Ui, expanded_paths: &mut HashSet<String>, value: &Value) {
        self.show_inner(ui, expanded_paths, &mut vec![], value);
    }

    fn show_inner(
        &mut self,
        ui: &mut egui::Ui,
        expanded_paths: &mut HashSet<String>,
        path_segments: &mut Vec<String>,
        value: &Value,
    ) {
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
                self.show_expandable(path_segments, expanded_paths, ui, iter, &ARRAY_DELIMITERS);
            }
            Value::Object(obj) => {
                let iter = obj.iter().map(|(k, v)| (format!("\"{}\"", k), v));
                self.show_expandable(path_segments, expanded_paths, ui, iter, &OBJECT_DELIMITERS);
            }
        };
    }

    fn show_expandable<'a, K, I>(
        &self,
        path_segments: &mut Vec<String>,
        expanded_paths: &mut HashSet<String>,
        ui: &mut egui::Ui,
        elem_iter: I,
        delimiters: &Delimiters,
    ) where
        K: Display,
        I: Iterator<Item = (K, &'a Value)>,
    {
        let path = path_segments.join("/").to_string();
        let was_expanded_last_frame = expanded_paths.contains(&path);

        let header = format!(
            "{}{}",
            &self.prefix,
            if was_expanded_last_frame {
                delimiters.opening
            } else {
                delimiters.collapsed
            }
        );

        let id_source = generate_id(self.id, path_segments);
        let response = egui::CollapsingHeader::new(header)
            .id_source(id_source)
            .show(ui, |ui| {
                for (key, elem) in elem_iter {
                    path_segments.push(key.to_string());

                    let mut add_nested_tree = |ui: &mut egui::Ui| {
                        ui.visuals_mut().indent_has_left_vline = true;

                        JsonTree::new(generate_id(self.id, path_segments))
                            .prefix(format!("{key} : "))
                            .show_inner(ui, expanded_paths, path_segments, elem);
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

        if was_expanded_last_frame {
            ui.horizontal(|ui| {
                let indent = ui.spacing().icon_width / 2.0;
                ui.add_space(indent);

                ui.monospace(delimiters.closing);
            });
        }

        if response.fully_closed() {
            expanded_paths.remove(&path);
        } else {
            expanded_paths.insert(path);
        }
    }
}

fn is_expandable(value: &Value) -> bool {
    matches!(value, Value::Array(_) | Value::Object(_))
}

fn generate_id(id: Id, path: &[String]) -> Id {
    Id::new(format!("{:?}-{}", id, path.join("/")))
}
