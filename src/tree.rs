use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use egui::{
    collapsing_header::CollapsingState,
    util::cache::{ComputerMut, FrameCache},
    Color32, Id, RichText, Ui,
};

use crate::{
    delimiters::{ARRAY_DELIMITERS, OBJECT_DELIMITERS},
    response::JsonTreeResponse,
    search::SearchTerm,
    style::JsonTreeStyle,
    value::{BaseValueType, ExpandableType, JsonTreeValue},
};

/// An interactive JSON tree visualiser.
///
/// ```
/// use egui_json_tree::{JsonTree, DefaultExpand};
///
/// # egui::__run_test_ui(|ui| {
/// let value = serde_json::json!({ "foo": "bar", "fizz": [1, 2, 3]});
/// let tree = JsonTree::new("globally-unique-id", &value);
///
/// // Show the JSON tree:
/// let response = tree.show(ui, DefaultExpand::All);
///
/// // Reset which arrays and objects are expanded to respect the `default_expand` argument on the next render.
/// // In this case, this will expand all arrays and objects again,
/// // if a user had collapsed any manually.
/// response.reset_expanded(ui);
/// # });
/// ```
pub struct JsonTree {
    id: Id,
    value: JsonTreeValue,
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
            style: JsonTreeStyle::default(),
            parent: None,
        }
    }

    /// Override colors for JSON syntax highlighting, and search match highlighting.
    pub fn style(mut self, style: JsonTreeStyle) -> Self {
        self.style = style;
        self
    }

    /// Show the JSON tree visualisation within the `Ui`.
    pub fn show(self, ui: &mut Ui, default_expand: DefaultExpand) -> JsonTreeResponse {
        let mut path_id_map = ui.ctx().memory_mut(|mem| {
            let cache = mem.caches.cache::<PathIdMapCache<'_>>();
            cache.get(&(self.id, &self.value))
        });

        for value in path_id_map.values_mut() {
            *value = ui.make_persistent_id(&value);
        }

        let (default_expand, search_term) = match default_expand {
            DefaultExpand::All => (InnerExpand::All, None),
            DefaultExpand::None => (InnerExpand::None, None),
            DefaultExpand::ToLevel(l) => (InnerExpand::ToLevel(l), None),
            DefaultExpand::SearchResults(search_str) => {
                let search_term = SearchTerm::parse(search_str);
                let paths = search_term
                    .as_ref()
                    .map(|search_term| {
                        ui.ctx().memory_mut(|mem| {
                            let cache = mem.caches.cache::<SearchResultsCache<'_>>();
                            cache.get(&(search_term, &self.value))
                        })
                    })
                    .unwrap_or_default();
                (InnerExpand::Paths(paths), search_term)
            }
        };

        // Wrap in a vertical layout in case this tree is placed directly in a horizontal layout,
        // which does not allow indent layouts as direct children.
        ui.vertical(|ui| {
            self.show_impl(
                ui,
                &mut vec![],
                &mut path_id_map,
                &default_expand,
                &search_term,
            );
        });

        JsonTreeResponse {
            collapsing_state_ids: path_id_map.into_values().collect(),
        }
    }

    fn show_impl(
        self,
        ui: &mut Ui,
        path_segments: &mut Vec<String>,
        path_id_map: &mut PathIdMap,
        default_expand: &InnerExpand,
        search_term: &Option<SearchTerm>,
    ) {
        match self.value {
            JsonTreeValue::Base(value_str, value_type) => {
                let key_texts = get_key_text(&self.style, &self.parent, search_term);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    show_base_value(
                        ui,
                        &self.style,
                        key_texts,
                        &value_str,
                        &value_type,
                        search_term,
                    );
                });
            }
            JsonTreeValue::Expandable(entries, expandable_type) => {
                let expandable = Expandable {
                    id: self.id,
                    entries,
                    expandable_type,
                    parent: self.parent,
                };
                show_expandable(
                    ui,
                    path_segments,
                    path_id_map,
                    expandable,
                    &self.style,
                    default_expand,
                    search_term,
                );
            }
        };
    }
}

fn show_base_value(
    ui: &mut Ui,
    style: &JsonTreeStyle,
    key_texts: Vec<RichText>,
    value_str: &str,
    value_type: &BaseValueType,
    search_term: &Option<SearchTerm>,
) {
    let mut texts = key_texts;

    add_texts_with_highlighting(
        &mut texts,
        value_str,
        style.get_color(value_type),
        search_term,
        style.highlight_color,
    );

    for text in texts {
        ui.monospace(text);
    }
}

fn show_expandable(
    ui: &mut Ui,
    path_segments: &mut Vec<String>,
    path_id_map: &mut PathIdMap,
    expandable: Expandable,
    style: &JsonTreeStyle,
    default_expand: &InnerExpand,
    search_term: &Option<SearchTerm>,
) {
    let delimiters = match expandable.expandable_type {
        ExpandableType::Array => &ARRAY_DELIMITERS,
        ExpandableType::Object => &OBJECT_DELIMITERS,
    };

    let default_open = match &default_expand {
        InnerExpand::All => true,
        InnerExpand::None => false,
        InnerExpand::ToLevel(num_levels_open) => (path_segments.len() as u8) <= *num_levels_open,
        InnerExpand::Paths(paths) => paths.contains(path_segments),
    };

    let id_source = *path_id_map
        .entry(path_segments.to_vec())
        .or_insert_with(|| ui.make_persistent_id(generate_id(expandable.id, path_segments)));

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
                                    search_term,
                                )
                            };

                        match elem {
                            JsonTreeValue::Base(value_str, value_type) => {
                                show_base_value(
                                    ui,
                                    style,
                                    key_texts,
                                    value_str,
                                    value_type,
                                    search_term,
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
                    for key_text in get_key_text(style, &expandable.parent, search_term) {
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
                        id: expandable.id,
                        value: elem,
                        style: style.clone(),
                        parent: Some(Parent::new(key, expandable.expandable_type)),
                    };

                    nested_tree.show_impl(
                        ui,
                        path_segments,
                        path_id_map,
                        default_expand,
                        search_term,
                    );
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
    key_str: &str,
    color: Color32,
    search_term: &Option<SearchTerm>,
    highlight_color: Color32,
) -> Vec<RichText> {
    let mut texts = vec![RichText::new("\"").color(color)];

    add_texts_with_highlighting(&mut texts, key_str, color, search_term, highlight_color);

    texts.push(RichText::new("\"").color(color));
    texts.push(RichText::new(": ").monospace());

    texts
}

fn format_array_idx(idx_str: &str, color: Color32) -> Vec<RichText> {
    vec![
        RichText::new(idx_str).color(color),
        RichText::new(": ").monospace(),
    ]
}

fn add_texts_with_highlighting(
    texts: &mut Vec<RichText>,
    text_str: &str,
    text_color: Color32,
    search_term: &Option<SearchTerm>,
    highlight_color: Color32,
) {
    if let Some(search_term) = search_term {
        let matches = search_term.find_match_indices_in(text_str);
        if !matches.is_empty() {
            let mut start = 0;
            for match_idx in matches {
                texts.push(RichText::new(&text_str[start..match_idx]).color(text_color));

                let highlight_end_idx = match_idx + search_term.len();

                texts.push(
                    RichText::new(&text_str[match_idx..highlight_end_idx])
                        .color(text_color)
                        .background_color(highlight_color),
                );

                start = highlight_end_idx;
            }
            texts.push(RichText::new(&text_str[start..]).color(text_color));
            return;
        }
    }
    texts.push(RichText::new(text_str).color(text_color));
}

#[derive(Debug, Clone)]
/// Configuration for how a `JsonTree` should expand arrays and objects by default.
pub enum DefaultExpand<'a> {
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
    SearchResults(&'a str),
}

#[derive(Debug, Clone)]
enum InnerExpand {
    All,
    None,
    ToLevel(u8),
    Paths(HashSet<Vec<String>>),
}

struct Expandable {
    id: Id,
    entries: Vec<(String, JsonTreeValue)>,
    expandable_type: ExpandableType,
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

fn generate_id(base_id: Id, path_segments: &Vec<String>) -> Id {
    Id::new(base_id).with(path_segments)
}

type PathIdMap = HashMap<Vec<String>, Id>;

fn get_path_id_map(base_id: Id, value: &JsonTreeValue) -> PathIdMap {
    let mut path_id_map = HashMap::new();
    get_path_id_map_impl(base_id, value, &mut vec![], &mut path_id_map);
    path_id_map
}

fn get_path_id_map_impl(
    base_id: Id,
    value: &JsonTreeValue,
    path_segments: &mut Vec<String>,
    path_id_map: &mut PathIdMap,
) {
    if let JsonTreeValue::Expandable(entries, _) = value {
        for (key, val) in entries {
            let id = generate_id(base_id, path_segments);
            path_id_map.insert(path_segments.clone(), id);
            path_segments.push(key.to_owned());
            get_path_id_map_impl(base_id, val, path_segments, path_id_map);
            path_segments.pop();
        }
    }
}

#[derive(Default)]
struct PathIdMapComputer;
impl ComputerMut<&(Id, &JsonTreeValue), PathIdMap> for PathIdMapComputer {
    fn compute(&mut self, (base_id, value): &(Id, &JsonTreeValue)) -> PathIdMap {
        get_path_id_map(*base_id, value)
    }
}
type PathIdMapCache<'a> = FrameCache<PathIdMap, PathIdMapComputer>;

#[derive(Default)]
struct SearchResultsComputer;
impl ComputerMut<&(&SearchTerm, &JsonTreeValue), HashSet<Vec<String>>> for SearchResultsComputer {
    fn compute(
        &mut self,
        (search_term, value): &(&SearchTerm, &JsonTreeValue),
    ) -> HashSet<Vec<String>> {
        search_term.find_matching_paths_in(value)
    }
}
type SearchResultsCache<'a> = FrameCache<HashSet<Vec<String>>, SearchResultsComputer>;
