use std::{collections::HashSet, fmt::Display};

use delimiters::{Delimiters, ARRAY_DELIMITERS, OBJECT_DELIMITERS};
use serde_json::Value;

mod delimiters;

pub struct JsonTree<'a> {
    value: &'a Value,
    prefix: String,
}

impl<'a> JsonTree<'a> {
    pub fn new(value: &'a Value) -> Self {
        Self {
            value,
            prefix: "".to_string(),
        }
    }

    fn prefix(mut self, prefix: String) -> Self {
        self.prefix = prefix;
        self
    }

    pub fn show(mut self, ui: &mut egui::Ui, expanded_paths: &mut HashSet<String>) {
        self.show_inner(ui, expanded_paths, &mut vec![]);
    }

    fn show_inner(
        &mut self,
        ui: &mut egui::Ui,
        expanded_paths: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) {
        match self.value {
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
                show_expandable(
                    path,
                    expanded_paths,
                    ui,
                    &self.prefix,
                    iter,
                    &ARRAY_DELIMITERS,
                );
            }
            Value::Object(obj) => {
                let iter = obj.iter().map(|(k, v)| (format!("\"{}\"", k), v));
                show_expandable(
                    path,
                    expanded_paths,
                    ui,
                    &self.prefix,
                    iter,
                    &OBJECT_DELIMITERS,
                );
            }
        };
    }
}

fn show_expandable<'a, K, I>(
    path: &mut Vec<String>,
    expanded_paths: &mut HashSet<String>,
    ui: &mut egui::Ui,
    prefix: &str,
    elem_iter: I,
    delimiters: &Delimiters,
) where
    K: Display,
    I: Iterator<Item = (K, &'a Value)>,
{
    let path_id = path.join("/").to_string();
    let was_expanded_last_frame = expanded_paths.contains(&path_id);

    let header = format!(
        "{}{}",
        prefix,
        if was_expanded_last_frame {
            delimiters.opening
        } else {
            delimiters.collapsed
        }
    );

    let response = egui::CollapsingHeader::new(header)
        .id_source(&path_id)
        .show(ui, |ui| {
            for (key, elem) in elem_iter {
                path.push(key.to_string());

                let mut add_nested_tree = |ui: &mut egui::Ui, path: &mut Vec<String>| {
                    ui.visuals_mut().indent_has_left_vline = true;
                    JsonTree::new(elem).prefix(format!("{key} : ")).show_inner(
                        ui,
                        expanded_paths,
                        path,
                    );
                };

                ui.visuals_mut().indent_has_left_vline = false;

                if is_expandable(elem) {
                    add_nested_tree(ui, path);
                } else {
                    let original_indent = ui.spacing().indent;

                    ui.spacing_mut().indent = ui.spacing().icon_width + ui.spacing().icon_spacing;

                    ui.indent(path.clone(), |ui| add_nested_tree(ui, path));

                    ui.spacing_mut().indent = original_indent;
                }

                path.pop();
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
        expanded_paths.remove(&path_id);
    } else {
        expanded_paths.insert(path_id);
    }
}

fn is_expandable(value: &Value) -> bool {
    matches!(value, Value::Array(_) | Value::Object(_))
}
