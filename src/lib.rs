use std::{
    collections::{BTreeSet, HashSet, VecDeque},
    hash::Hash,
};

use delimiters::{ARRAY_DELIMITERS, OBJECT_DELIMITERS};
use egui::{collapsing_header::CollapsingState, Color32, Id, RichText, Ui};
use search::{is_valid_search_term, search};
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
    collapsing_state_ids: HashSet<Id>,
}

impl JsonTree {
    pub fn new(id: impl Hash) -> Self {
        Self {
            id: Id::new(id),
            default_expand: Expand::All(false),
            style: JsonTreeStyle::default(),
            key: None,
            collapsing_state_ids: HashSet::new(),
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

    pub fn show(&mut self, ui: &mut Ui, value: &Value) {
        let (default_expand, search_term) = match &self.default_expand {
            Expand::All(b) => (InnerExpand::All(*b), None),
            Expand::Levels(l) => (InnerExpand::Levels(*l), None),
            Expand::SearchResults(search_term) => (
                InnerExpand::Paths(search(value, search_term)),
                (is_valid_search_term(search_term)).then(|| search_term.to_owned()),
            ),
        };

        let mut collapsing_state_ids = HashSet::new();

        self.show_inner(
            ui,
            &mut vec![],
            value,
            None,
            default_expand,
            &search_term,
            &mut collapsing_state_ids,
        );

        self.collapsing_state_ids = collapsing_state_ids;
    }

    #[allow(clippy::too_many_arguments)]
    fn show_inner(
        &self,
        ui: &mut Ui,
        path_segments: &mut Vec<String>,
        value: &Value,
        parent: Option<Expandable>,
        default_expand: InnerExpand,
        search_term: &Option<String>,
        collapsing_state_ids: &mut HashSet<Id>,
    ) {
        let key_text = get_key_text(&self.key, parent, &self.style, search_term);

        match value {
            Value::Null => {
                show_val(
                    ui,
                    key_text,
                    "null".to_string(),
                    self.style.null_color,
                    search_term,
                    self.style.highlight_color,
                );
            }
            Value::Bool(b) => {
                show_val(
                    ui,
                    key_text,
                    b.to_string(),
                    self.style.bool_color,
                    search_term,
                    self.style.highlight_color,
                );
            }
            Value::Number(n) => {
                show_val(
                    ui,
                    key_text,
                    n.to_string(),
                    self.style.number_color,
                    search_term,
                    self.style.highlight_color,
                );
            }
            Value::String(s) => {
                show_val(
                    ui,
                    key_text,
                    format!("\"{}\"", s),
                    self.style.string_color,
                    search_term,
                    self.style.highlight_color,
                );
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
                    search_term,
                    collapsing_state_ids,
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
                    search_term,
                    collapsing_state_ids,
                );
            }
        };
    }

    #[allow(clippy::too_many_arguments)]
    fn show_expandable<'a, K, I>(
        &self,
        path_segments: &mut Vec<String>,
        ui: &mut Ui,
        elem_iter: I,
        parent: Option<Expandable>,
        expandable: Expandable,
        default_expand: InnerExpand,
        search_term: &Option<String>,
        collapsing_state_ids: &mut HashSet<Id>,
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

        collapsing_state_ids.insert(id_source);

        let state = CollapsingState::load_with_default_open(ui.ctx(), id_source, default_open);
        let is_expanded = state.is_open();

        state
            .show_header(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;

                    if let Some(key_texts) =
                        get_key_text(&self.key, parent, &self.style, search_term)
                    {
                        for key_text in key_texts {
                            ui.monospace(key_text);
                        }
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
                                search_term,
                                collapsing_state_ids,
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

    pub fn reset_expanded(&self, ui: &mut Ui) {
        for id in self.collapsing_state_ids.iter() {
            if let Some(state) = CollapsingState::load(ui.ctx(), *id) {
                state.remove(ui.ctx());
            }
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
    search_term: &Option<String>,
) -> Option<Vec<RichText>> {
    match (key, parent) {
        (Some(key), Some(Expandable::Array)) => Some(format_array_idx(key, style.array_idx_color)),
        (Some(key), Some(Expandable::Object)) => Some(format_object_key(
            key,
            style.object_key_color,
            search_term,
            style.highlight_color,
        )),
        _ => None,
    }
}

fn show_val(
    ui: &mut Ui,
    key_texts: Option<Vec<RichText>>,
    value: String,
    color: Color32,
    search_term: &Option<String>,
    highlight_color: Color32,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        if let Some(key_texts) = key_texts {
            for key_text in key_texts {
                ui.monospace(key_text);
            }
            ui.monospace(": ");
        }

        for text in get_highlighted_texts(&value, color, search_term, highlight_color) {
            ui.monospace(text);
        }
    });
}

fn format_object_key(
    key: &String,
    color: Color32,
    search_term: &Option<String>,
    highlight_color: Color32,
) -> Vec<RichText> {
    let mut texts = get_highlighted_texts(key, color, search_term, highlight_color);
    texts.push_front(RichText::new("\"").color(color));
    texts.push_back(RichText::new("\"").color(color));
    texts.into()
}

fn format_array_idx(idx: &String, color: Color32) -> Vec<RichText> {
    vec![RichText::new(idx).color(color)]
}

fn get_highlighted_texts(
    text: &String,
    text_color: Color32,
    search_term: &Option<String>,
    highlight_color: Color32,
) -> VecDeque<RichText> {
    if let Some(highlight) = search_term {
        if let Some((idx, _)) = text.match_indices(highlight).next() {
            return VecDeque::from_iter([
                RichText::new(&text[..idx]).color(text_color),
                RichText::new(&text[idx..idx + highlight.len()])
                    .color(text_color)
                    .background_color(highlight_color),
                RichText::new(&text[idx + highlight.len()..]).color(text_color),
            ]);
        }
    }
    VecDeque::from_iter([RichText::new(text).color(text_color)])
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
