use std::{collections::BTreeSet, hash::Hash};

use delimiters::{ARRAY_DELIMITERS, OBJECT_DELIMITERS};
use egui::{collapsing_header::CollapsingState, Color32, Id, RichText, Ui};
use search::search;
use serde_json::Value;
use style::JsonTreeStyle;

mod delimiters;
mod search;
mod style;

pub struct JsonTree {
    id: Id,
    default_expand: Expand,
    style: JsonTreeStyle,
    key: Option<String>,
}

impl JsonTree {
    pub fn new(id: impl Hash) -> Self {
        Self {
            id: Id::new(id),
            default_expand: Expand::All(false),
            style: JsonTreeStyle::default(),
            key: None,
        }
    }

    pub fn default_expand(mut self, default_expand: Expand) -> Self {
        self.default_expand = default_expand;
        self
    }

    pub fn style(mut self, style: JsonTreeStyle) -> Self {
        self.style = style;
        self
    }

    fn key(mut self, key: String) -> Self {
        self.key = Some(key);
        self
    }

    pub fn show(mut self, ui: &mut Ui, value: &Value) {
        let default_expand = match &self.default_expand {
            Expand::All(b) => InnerExpand::All(*b),
            Expand::Levels(l) => InnerExpand::Levels(*l),
            Expand::SearchResults(search_term) => InnerExpand::Paths(search(value, search_term)),
        };

        self.show_inner(ui, &mut vec![], value, None, default_expand);
    }

    fn show_inner(
        &mut self,
        ui: &mut Ui,
        path_segments: &mut Vec<String>,
        value: &Value,
        parent: Option<Expandable>,
        default_expand: InnerExpand,
    ) {
        let key_text = get_key_text(&self.key, parent, &self.style);

        match value {
            Value::Null => {
                show_val(ui, key_text, "null".to_string(), self.style.null_color);
            }
            Value::Bool(b) => {
                show_val(ui, key_text, b.to_string(), self.style.bool_color);
            }
            Value::Number(n) => {
                show_val(ui, key_text, n.to_string(), self.style.number_color);
            }
            Value::String(s) => {
                show_val(ui, key_text, format!("\"{}\"", s), self.style.string_color);
            }
            Value::Array(arr) => {
                let iter = arr.iter().enumerate();
                self.show_expandable(
                    path_segments,
                    ui,
                    iter,
                    parent,
                    Expandable::Array,
                    default_expand,
                );
            }
            Value::Object(obj) => {
                let iter = obj.iter();
                self.show_expandable(
                    path_segments,
                    ui,
                    iter,
                    parent,
                    Expandable::Object,
                    default_expand,
                );
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
        default_expand: InnerExpand,
    ) where
        K: ToString,
        I: Iterator<Item = (K, &'a Value)>,
    {
        let delimiters = match expandable {
            Expandable::Array => &ARRAY_DELIMITERS,
            Expandable::Object => &OBJECT_DELIMITERS,
        };

        let default_open = match &default_expand {
            InnerExpand::All(b) => *b,
            InnerExpand::Levels(num_levels_open) => (path_segments.len() as u8) <= *num_levels_open,
            InnerExpand::Paths(paths) => paths.contains(&path_segments.join("/").to_string()),
        };

        let id_source =
            ui.make_persistent_id(generate_id(self.id, path_segments).with(&default_expand));
        let state = CollapsingState::load_with_default_open(ui.ctx(), id_source, default_open);
        let is_expanded = state.is_open();

        state
            .show_header(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;

                    if let Some(key_text) = get_key_text(&self.key, parent, &self.style) {
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
                            .key(key.to_string())
                            .show_inner(
                                ui,
                                path_segments,
                                elem,
                                Some(expandable),
                                default_expand.clone(),
                            );
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

fn get_key_text(
    key: &Option<String>,
    parent: Option<Expandable>,
    style: &JsonTreeStyle,
) -> Option<RichText> {
    match (key, parent) {
        (Some(key), Some(Expandable::Array)) => Some(format_array_idx(key, style.array_idx_color)),
        (Some(key), Some(Expandable::Object)) => {
            Some(format_object_key(key, style.object_key_color))
        }
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

fn format_object_key(key: &String, color: Color32) -> RichText {
    RichText::new(format!("\"{}\"", key)).color(color)
}

fn format_array_idx(idx: &String, color: Color32) -> RichText {
    RichText::new(idx).color(color)
}

#[derive(Clone, Copy)]
enum Expandable {
    Array,
    Object,
}

#[derive(Clone)]
pub enum Expand {
    /// Expand all arrays and objects according to the contained `bool`.
    All(bool),
    /// Expand arrays and objects according to how many levels deep they are nested:
    /// - `0` would expand a top-level array/object only,
    /// - `1` would expand any arrays/objects that are a direct element/value of a top-level array/object,
    /// - `2` ...
    ///
    /// And so on.
    Levels(u8),
    SearchResults(String),
}

#[derive(Clone, Hash)]
enum InnerExpand {
    All(bool),
    Levels(u8),
    Paths(BTreeSet<String>),
}
