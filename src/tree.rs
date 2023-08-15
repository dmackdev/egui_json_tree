use std::{
    collections::{BTreeSet, HashSet, VecDeque},
    hash::Hash,
};

use egui::{collapsing_header::CollapsingState, Color32, Id, RichText, Ui};
use serde_json::Value;

use crate::{
    delimiters::{ARRAY_DELIMITERS, OBJECT_DELIMITERS},
    search::SearchTerm,
    style::JsonTreeStyle,
};

/// An interactive JSON tree visualiser which will render a provided [`serde_json::Value`].
/// ```
/// use egui_json_tree::{JsonTree, Expand};
///
/// # egui::__run_test_ui(|ui| {
/// let value = serde_json::json!({ "foo": "bar", "fizz": [1, 2, 3]});
/// let tree = JsonTree::new("globally-unique-id", &value).default_expand(Expand::All);
///
/// // Show the JSON tree:
/// let response = tree.show(ui);
///
/// // Reset which arrays and objects are expanded to respect the `default_expand` setting.
/// // In this case, this will expand all arrays and objects again,
/// // if a user had collapsed any manually.
/// response.reset_expanded(ui);
/// # });
/// ```
pub struct JsonTree<'a> {
    id: Id,
    value: &'a Value,
    default_expand: InnerExpand,
    search_term: Option<SearchTerm>,
    style: JsonTreeStyle,
    key: Option<String>,
}

impl<'a> JsonTree<'a> {
    /// Creates a new [`JsonTree`].
    /// `id` must be a globally unique identifier.
    pub fn new(id: impl Hash, value: &'a Value) -> Self {
        Self {
            id: Id::new(id),
            value,
            default_expand: InnerExpand::None,
            search_term: None,
            style: JsonTreeStyle::default(),
            key: None,
        }
    }

    /// Set how arrays/objects should be expanded by default.
    /// The default behaviour is to collapse all arrays/objects.
    pub fn default_expand(mut self, default_expand: Expand) -> Self {
        let (default_expand, search_term) = match default_expand {
            Expand::All => (InnerExpand::All, None),
            Expand::None => (InnerExpand::None, None),
            Expand::ToLevel(l) => (InnerExpand::ToLevel(l), None),
            Expand::SearchResults(search_str) => {
                let search_term = SearchTerm::parse(search_str);
                let paths = search_term
                    .as_ref()
                    .map(|search_term| search_term.find_matching_paths_in(self.value))
                    .unwrap_or_default();
                (InnerExpand::Paths(paths), search_term)
            }
        };
        self.default_expand = default_expand;
        self.search_term = search_term;
        self
    }

    /// Override colors for JSON syntax highlighting, and search match highlighting.
    pub fn style(mut self, style: JsonTreeStyle) -> Self {
        self.style = style;
        self
    }

    /// Show the JSON tree visualisation within the `Ui`.
    pub fn show(&self, ui: &mut Ui) -> JsonTreeResponse {
        let mut collapsing_state_ids = HashSet::new();

        // Wrap in a vertical layout in case this tree is placed directly in a horizontal layout,
        // which does not allow indent layouts as direct children.
        ui.vertical(|ui| {
            self.show_impl(ui, &mut vec![], None, &mut collapsing_state_ids);
        });

        JsonTreeResponse {
            collapsing_state_ids,
        }
    }

    fn show_impl(
        &self,
        ui: &mut Ui,
        path_segments: &mut Vec<String>,
        parent: Option<Expandable>,
        collapsing_state_ids: &mut HashSet<Id>,
    ) {
        let key_text = get_key_text(&self.key, parent, &self.style, &self.search_term);

        match self.value {
            Value::Null => {
                show_val(
                    ui,
                    key_text,
                    "null".to_string(),
                    self.style.null_color,
                    &self.search_term,
                    self.style.highlight_color,
                );
            }
            Value::Bool(b) => {
                show_val(
                    ui,
                    key_text,
                    b.to_string(),
                    self.style.bool_color,
                    &self.search_term,
                    self.style.highlight_color,
                );
            }
            Value::Number(n) => {
                show_val(
                    ui,
                    key_text,
                    n.to_string(),
                    self.style.number_color,
                    &self.search_term,
                    self.style.highlight_color,
                );
            }
            Value::String(s) => {
                show_val(
                    ui,
                    key_text,
                    format!("\"{}\"", s),
                    self.style.string_color,
                    &self.search_term,
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
                    collapsing_state_ids,
                );
            }
        };
    }

    fn show_expandable<K, I>(
        &self,
        path_segments: &mut Vec<String>,
        ui: &mut Ui,
        elem_iter: I,
        parent: Option<Expandable>,
        expandable: Expandable,
        collapsing_state_ids: &mut HashSet<Id>,
    ) where
        K: ToString,
        I: Iterator<Item = (K, &'a Value)>,
    {
        let delimiters = match expandable {
            Expandable::Array => &ARRAY_DELIMITERS,
            Expandable::Object => &OBJECT_DELIMITERS,
        };

        let default_open = match &self.default_expand {
            InnerExpand::All => true,
            InnerExpand::None => false,
            InnerExpand::ToLevel(num_levels_open) => {
                (path_segments.len() as u8) <= *num_levels_open
            }
            InnerExpand::Paths(paths) => paths.contains(&path_segments.join("/").to_string()),
        };

        let id_source =
            ui.make_persistent_id(generate_id(self.id, path_segments).with(&self.default_expand));

        collapsing_state_ids.insert(id_source);

        let state = CollapsingState::load_with_default_open(ui.ctx(), id_source, default_open);
        let is_expanded = state.is_open();

        state
            .show_header(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;

                    if let Some(key_texts) =
                        get_key_text(&self.key, parent, &self.style, &self.search_term)
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

                        let nested_tree = JsonTree {
                            id: generate_id(self.id, path_segments),
                            value: elem,
                            default_expand: self.default_expand.clone(),
                            search_term: self.search_term.clone(),
                            style: self.style.clone(),
                            key: Some(key.to_string()),
                        };

                        nested_tree.show_impl(
                            ui,
                            path_segments,
                            Some(expandable),
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
    search_term: &Option<SearchTerm>,
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
    search_term: &Option<SearchTerm>,
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
    search_term: &Option<SearchTerm>,
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
    search_term: &Option<SearchTerm>,
    highlight_color: Color32,
) -> VecDeque<RichText> {
    if let Some(search_term) = search_term {
        if let Some(idx) = search_term.match_index(text) {
            return VecDeque::from_iter([
                RichText::new(&text[..idx]).color(text_color),
                RichText::new(&text[idx..idx + search_term.len()])
                    .color(text_color)
                    .background_color(highlight_color),
                RichText::new(&text[idx + search_term.len()..]).color(text_color),
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
/// Configuration for how a `JsonTree` should expand arrays and objects by default.
pub enum Expand {
    /// Expand all arrays and objects.
    All,
    /// Collapse all arrays and objects.
    None,
    /// Expand arrays and objects according to how many levels deep they are nested:
    /// - `0` would expand a top-level array/object only,
    /// - `1` would expand a top-level array/object and any array/object that is a direct child,
    /// - `2` ...
    ///
    /// And so on.
    ToLevel(u8),
    /// Expand arrays and objects to display object keys and values,
    /// and array elements, that match the search term. Letter case is ignored. The matches are highlighted.
    /// If the search term is empty, nothing will be expanded by default.
    SearchResults(String),
}

#[derive(Clone, Hash)]
enum InnerExpand {
    All,
    None,
    ToLevel(u8),
    Paths(BTreeSet<String>),
}

/// The response from showing a [`JsonTree`].
pub struct JsonTreeResponse {
    // TODO: Add me.
    // pub response: Response,
    collapsing_state_ids: HashSet<Id>,
}

impl JsonTreeResponse {
    /// For the [`JsonTree`] that provided this response,
    /// resets the expanded state for all of its arrays/objects to respect its `default_expand` setting.
    pub fn reset_expanded(&self, ui: &mut Ui) {
        for id in self.collapsing_state_ids.iter() {
            if let Some(state) = CollapsingState::load(ui.ctx(), *id) {
                state.remove(ui.ctx());
            }
        }
    }
}
