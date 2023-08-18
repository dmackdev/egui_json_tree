use std::{
    collections::{BTreeSet, HashSet, VecDeque},
    hash::Hash,
};

use egui::{collapsing_header::CollapsingState, Color32, Id, RichText, Ui};

use crate::{
    delimiters::{ARRAY_DELIMITERS, OBJECT_DELIMITERS},
    response::JsonTreeResponse,
    search::SearchTerm,
    style::JsonTreeStyle,
    value::JsonTreeValue,
    BaseValue, ExpandableType,
};

/// An interactive JSON tree visualiser.
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
pub struct JsonTree {
    id: Id,
    value: JsonTreeValue,
    default_expand: InnerExpand,
    search_term: Option<SearchTerm>,
    style: JsonTreeStyle,
    parent: Option<Parent>,
}

impl JsonTree {
    /// Creates a new [`JsonTree`].
    /// `id` must be a globally unique identifier.
    pub fn new(id: impl Hash, value: impl Into<JsonTreeValue>) -> Self {
        Self {
            id: Id::new(id),
            value: value.into(),
            default_expand: InnerExpand::None,
            search_term: None,
            style: JsonTreeStyle::default(),
            parent: None,
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
                    .map(|search_term| search_term.find_matching_paths_in(&self.value))
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
    pub fn show(self, ui: &mut Ui) -> JsonTreeResponse {
        let mut collapsing_state_ids = HashSet::new();

        // Wrap in a vertical layout in case this tree is placed directly in a horizontal layout,
        // which does not allow indent layouts as direct children.
        ui.vertical(|ui| {
            self.show_impl(ui, &mut vec![], &mut collapsing_state_ids);
        });

        JsonTreeResponse {
            collapsing_state_ids,
        }
    }

    fn show_impl(
        self,
        ui: &mut Ui,
        path_segments: &mut Vec<String>,
        collapsing_state_ids: &mut HashSet<Id>,
    ) {
        match self.value {
            JsonTreeValue::BaseValue(base_value) => {
                let key_texts = get_key_text(&self.style, &self.parent, &self.search_term);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    show_base_value(ui, &self.style, key_texts, &base_value, &self.search_term);
                });
            }
            JsonTreeValue::Expandable(entries, expandable_type) => {
                let expandable = Expandable {
                    id: self.id,
                    default_expand: self.default_expand,
                    entries,
                    expandable_type,
                    search_term: self.search_term,
                    parent: self.parent,
                };
                show_expandable(
                    ui,
                    path_segments,
                    collapsing_state_ids,
                    expandable,
                    &self.style,
                );
            }
        };
    }
}

fn generate_id(id: Id, path: &[String]) -> Id {
    // TODO: Use .with(path)
    Id::new(format!("{:?}-{}", id, path.join("/")))
}

fn show_base_value(
    ui: &mut Ui,
    style: &JsonTreeStyle,
    key_texts: Vec<RichText>,
    base_value: &BaseValue,
    search_term: &Option<SearchTerm>,
) {
    for key_text in key_texts {
        ui.monospace(key_text);
    }

    for text in get_highlighted_texts(
        &base_value.value_str,
        style.get_color(base_value.value_type),
        search_term,
        style.highlight_color,
    ) {
        ui.monospace(text);
    }
}

fn show_expandable(
    ui: &mut Ui,
    path_segments: &mut Vec<String>,
    collapsing_state_ids: &mut HashSet<Id>,
    expandable: Expandable,
    style: &JsonTreeStyle,
) {
    let delimiters = match expandable.expandable_type {
        ExpandableType::Array => &ARRAY_DELIMITERS,
        ExpandableType::Object => &OBJECT_DELIMITERS,
    };

    let default_open = match &expandable.default_expand {
        InnerExpand::All => true,
        InnerExpand::None => false,
        InnerExpand::ToLevel(num_levels_open) => (path_segments.len() as u8) <= *num_levels_open,
        InnerExpand::Paths(paths) => paths.contains(&path_segments.join("/").to_string()),
    };

    let id_source = ui.make_persistent_id(
        generate_id(expandable.id, path_segments).with(&expandable.default_expand),
    );

    collapsing_state_ids.insert(id_source);

    let state = CollapsingState::load_with_default_open(ui.ctx(), id_source, default_open);
    let is_expanded = state.is_open();

    state
        .show_header(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;

                if path_segments.is_empty() && !is_expanded {
                    ui.label(delimiters.opening);
                    ui.monospace(" ");

                    let entries_len = expandable.entries.len();

                    for (idx, (key, elem)) in expandable.entries.iter().enumerate() {
                        let key_texts =
                            if matches!(expandable.expandable_type, ExpandableType::Array) {
                                // Don't show array indices when the array is collapsed.
                                vec![]
                            } else {
                                get_key_text(
                                    style,
                                    &Some(Parent::new(key.to_owned(), expandable.expandable_type)),
                                    &expandable.search_term,
                                )
                            };

                        match elem {
                            JsonTreeValue::BaseValue(base_value) => {
                                show_base_value(
                                    ui,
                                    style,
                                    key_texts,
                                    base_value,
                                    &expandable.search_term,
                                );
                            }
                            JsonTreeValue::Expandable(_, expandable_type) => {
                                for key_text in key_texts {
                                    ui.monospace(key_text);
                                }
                                let nested_delimiters = match expandable_type {
                                    ExpandableType::Array => &ARRAY_DELIMITERS,
                                    ExpandableType::Object => &OBJECT_DELIMITERS,
                                };
                                ui.label(nested_delimiters.collapsed);
                            }
                        };

                        if idx == entries_len - 1 {
                            ui.monospace(" ");
                        } else {
                            ui.monospace(", ");
                        }
                    }

                    ui.label(delimiters.closing);
                } else {
                    for key_text in get_key_text(style, &expandable.parent, &expandable.search_term)
                    {
                        ui.monospace(key_text);
                    }

                    ui.label(if is_expanded {
                        delimiters.opening
                    } else {
                        delimiters.collapsed
                    });
                }
            });
        })
        .body(|ui| {
            for (key, elem) in expandable.entries {
                let is_expandable = matches!(elem, JsonTreeValue::Expandable(_, _));

                path_segments.push(key.clone());

                let add_nested_tree = |ui: &mut Ui| {
                    let nested_tree = JsonTree {
                        id: generate_id(expandable.id, path_segments),
                        value: elem,
                        default_expand: expandable.default_expand.clone(),
                        search_term: expandable.search_term.clone(),
                        style: style.clone(),
                        parent: Some(Parent::new(key, expandable.expandable_type)),
                    };

                    nested_tree.show_impl(ui, path_segments, collapsing_state_ids);
                };

                if is_expandable {
                    add_nested_tree(ui);
                } else {
                    let original_indent_has_left_vline = ui.visuals_mut().indent_has_left_vline;
                    let original_indent = ui.spacing().indent;

                    ui.visuals_mut().indent_has_left_vline = false;
                    ui.spacing_mut().indent = ui.spacing().icon_width + ui.spacing().icon_spacing;

                    ui.indent(id_source, |ui| add_nested_tree(ui));

                    ui.visuals_mut().indent_has_left_vline = original_indent_has_left_vline;
                    ui.spacing_mut().indent = original_indent;
                }

                path_segments.pop();
            }
        });

    if is_expanded {
        ui.horizontal_wrapped(|ui| {
            let indent = ui.spacing().icon_width / 2.0;
            ui.add_space(indent);

            ui.monospace(delimiters.closing);
        });
    }
}

fn get_key_text(
    style: &JsonTreeStyle,
    parent: &Option<Parent>,
    search_term: &Option<SearchTerm>,
) -> Vec<RichText> {
    match parent {
        Some(Parent {
            key,
            expandable_type: ExpandableType::Array,
        }) => format_array_idx(key, style.array_idx_color),
        Some(Parent {
            key,
            expandable_type: ExpandableType::Object,
        }) => format_object_key(
            key,
            style.object_key_color,
            search_term,
            style.highlight_color,
        ),
        _ => vec![],
    }
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
    texts.push_back(RichText::new(": ").monospace());
    texts.into()
}

fn format_array_idx(idx: &String, color: Color32) -> Vec<RichText> {
    vec![
        RichText::new(idx).color(color),
        RichText::new(": ").monospace(),
    ]
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

struct Expandable {
    id: Id,
    default_expand: InnerExpand,
    entries: Vec<(String, JsonTreeValue)>,
    expandable_type: ExpandableType,
    search_term: Option<SearchTerm>,
    parent: Option<Parent>,
}

struct Parent {
    key: String,
    expandable_type: ExpandableType,
}

impl Parent {
    fn new(key: String, expandable_type: ExpandableType) -> Self {
        Self {
            key,
            expandable_type,
        }
    }
}
