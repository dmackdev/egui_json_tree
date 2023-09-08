use std::collections::{HashMap, HashSet};

use egui::{
    collapsing_header::CollapsingState,
    text::LayoutJob,
    util::cache::{ComputerMut, FrameCache},
    Color32, FontId, Id, Label, Response, Sense, TextFormat, Ui,
};

use crate::{
    delimiters::{ARRAY_DELIMITERS, OBJECT_DELIMITERS},
    response::JsonTreeResponse,
    search::SearchTerm,
    style::JsonTreeStyle,
    tree::JsonTreeConfig,
    value::{BaseValueType, ExpandableType, JsonTreeValue},
    DefaultExpand,
};

pub struct JsonTreeNode {
    id: Id,
    value: JsonTreeValue,
    parent: Option<Parent>,
}

impl JsonTreeNode {
    pub(crate) fn new(id: Id, value: JsonTreeValue) -> Self {
        Self {
            id,
            value,
            parent: None,
        }
    }

    pub(crate) fn show_with_config(self, ui: &mut Ui, config: JsonTreeConfig) -> JsonTreeResponse {
        let mut path_id_map = ui.ctx().memory_mut(|mem| {
            let cache = mem.caches.cache::<PathIdMapCache<'_>>();
            cache.get(&(self.id, &self.value))
        });

        for value in path_id_map.values_mut() {
            *value = ui.make_persistent_id(&value);
        }

        let (default_expand, search_term) = match config.default_expand {
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

        let response_callback = &mut config
            .response_callback
            .unwrap_or_else(|| Box::new(|_, _| {}));

        // Wrap in a vertical layout in case this tree is placed directly in a horizontal layout,
        // which does not allow indent layouts as direct children.
        ui.vertical(|ui| {
            // Centres the collapsing header icon.
            ui.spacing_mut().interact_size.y = config.style.font_id(ui).size;

            self.show_impl(
                ui,
                &mut vec![],
                &mut path_id_map,
                &config.style,
                &default_expand,
                search_term.as_ref(),
                response_callback,
            );
        });

        JsonTreeResponse {
            collapsing_state_ids: path_id_map.into_values().collect(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn show_impl(
        self,
        ui: &mut Ui,
        path_segments: &mut Vec<String>,
        path_id_map: &mut PathIdMap,
        style: &JsonTreeStyle,
        default_expand: &InnerExpand,
        search_term: Option<&SearchTerm>,
        response_callback: &mut dyn FnMut(Response, &String),
    ) {
        let pointer_string = &get_pointer_string(path_segments);
        match self.value {
            JsonTreeValue::Base(value_str, value_type) => {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;

                    if let Some(parent) = &self.parent {
                        let key_response = render_key(ui, style, parent, search_term);
                        response_callback(key_response, pointer_string);
                    }

                    let value_response =
                        render_value(ui, style, &value_str, &value_type, search_term);
                    response_callback(value_response, pointer_string);
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
                    style,
                    default_expand,
                    search_term,
                    response_callback,
                );
            }
        };
    }
}

fn render_value(
    ui: &mut Ui,
    style: &JsonTreeStyle,
    value_str: &str,
    value_type: &BaseValueType,
    search_term: Option<&SearchTerm>,
) -> Response {
    let mut job = LayoutJob::default();
    add_text_with_highlighting(
        &mut job,
        value_str,
        style.get_color(value_type),
        search_term,
        style.highlight_color,
        &style.font_id(ui),
    );
    render_job(ui, job)
}

#[allow(clippy::too_many_arguments)]
fn show_expandable(
    ui: &mut Ui,
    path_segments: &mut Vec<String>,
    path_id_map: &mut PathIdMap,
    expandable: Expandable,
    style: &JsonTreeStyle,
    default_expand: &InnerExpand,
    search_term: Option<&SearchTerm>,
    response_callback: &mut dyn FnMut(Response, &String),
) {
    let pointer_string = &get_pointer_string(path_segments);

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

    let font_id = style.font_id(ui);

    state
        .show_header(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;

                if path_segments.is_empty() && !is_expanded {
                    render_punc(
                        ui,
                        delimiters.opening,
                        style.punctuation_color,
                        None,
                        &font_id,
                    );
                    render_punc(ui, " ", style.punctuation_color, None, &font_id);

                    let entries_len = expandable.entries.len();

                    for (idx, (key, elem)) in expandable.entries.iter().enumerate() {
                        // Don't show array indices when the array is collapsed.
                        if matches!(expandable.expandable_type, ExpandableType::Object) {
                            let key_response = render_key(
                                ui,
                                style,
                                &Parent::new(key.to_owned(), expandable.expandable_type),
                                search_term,
                            );
                            response_callback(key_response, pointer_string);
                        }

                        match elem {
                            JsonTreeValue::Base(value_str, value_type) => {
                                let value_response =
                                    render_value(ui, style, value_str, value_type, search_term);
                                response_callback(value_response, pointer_string);
                            }
                            JsonTreeValue::Expandable(values, expandable_type) => {
                                let nested_delimiters = match expandable_type {
                                    ExpandableType::Array => &ARRAY_DELIMITERS,
                                    ExpandableType::Object => &OBJECT_DELIMITERS,
                                };

                                let delimiter = if values.is_empty() {
                                    nested_delimiters.collapsed_empty
                                } else {
                                    nested_delimiters.collapsed
                                };

                                let collapsed_expandable_response = render_punc(
                                    ui,
                                    delimiter,
                                    style.punctuation_color,
                                    None,
                                    &font_id,
                                );
                                response_callback(collapsed_expandable_response, pointer_string);
                            }
                        };
                        let spacing_str = if idx == entries_len - 1 { " " } else { ", " };
                        render_punc(ui, spacing_str, style.punctuation_color, None, &font_id);
                    }

                    render_punc(
                        ui,
                        delimiters.closing,
                        style.punctuation_color,
                        None,
                        &font_id,
                    );
                } else {
                    if let Some(parent) = &expandable.parent {
                        let key_response = render_key(ui, style, parent, search_term);
                        response_callback(key_response, pointer_string);
                    }

                    if is_expanded {
                        render_punc(
                            ui,
                            delimiters.opening,
                            style.punctuation_color,
                            None,
                            &font_id,
                        );
                    } else {
                        let delimiter = if expandable.entries.is_empty() {
                            delimiters.collapsed_empty
                        } else {
                            delimiters.collapsed
                        };
                        let collapsed_expandable_response =
                            render_punc(ui, delimiter, style.punctuation_color, None, &font_id);
                        response_callback(collapsed_expandable_response, pointer_string);
                    }
                }
            });
        })
        .body(|ui| {
            for (key, elem) in expandable.entries {
                let is_expandable = matches!(elem, JsonTreeValue::Expandable(_, _));

                path_segments.push(key.clone());

                let add_nested_tree = |ui: &mut Ui| {
                    let nested_tree = JsonTreeNode {
                        id: expandable.id,
                        value: elem,
                        parent: Some(Parent::new(key, expandable.expandable_type)),
                    };

                    nested_tree.show_impl(
                        ui,
                        path_segments,
                        path_id_map,
                        style,
                        default_expand,
                        search_term,
                        response_callback,
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
            render_punc(
                ui,
                delimiters.closing,
                style.punctuation_color,
                None,
                &font_id,
            );
        });
    }
}

fn render_key(
    ui: &mut Ui,
    style: &JsonTreeStyle,
    parent: &Parent,
    search_term: Option<&SearchTerm>,
) -> Response {
    let mut job = LayoutJob::default();
    match parent {
        Parent {
            key,
            expandable_type: ExpandableType::Array,
        } => add_array_idx(
            &mut job,
            key,
            style.array_idx_color,
            style.punctuation_color,
            &style.font_id(ui),
        ),
        Parent {
            key,
            expandable_type: ExpandableType::Object,
        } => add_object_key(
            &mut job,
            key,
            style.object_key_color,
            style.punctuation_color,
            search_term,
            style.highlight_color,
            &style.font_id(ui),
        ),
    };
    render_job(ui, job)
}

fn add_object_key(
    job: &mut LayoutJob,
    key_str: &str,
    color: Color32,
    punctuation_color: Color32,
    search_term: Option<&SearchTerm>,
    highlight_color: Color32,
    font_id: &FontId,
) {
    append(job, "\"", color, None, font_id);
    add_text_with_highlighting(job, key_str, color, search_term, highlight_color, font_id);
    append(job, "\"", color, None, font_id);
    append(job, ": ", punctuation_color, None, font_id);
}

fn add_array_idx(
    job: &mut LayoutJob,
    idx_str: &str,
    color: Color32,
    punctuation_color: Color32,
    font_id: &FontId,
) {
    append(job, idx_str, color, None, font_id);
    append(job, ": ", punctuation_color, None, font_id);
}

fn add_text_with_highlighting(
    job: &mut LayoutJob,
    text_str: &str,
    text_color: Color32,
    search_term: Option<&SearchTerm>,
    highlight_color: Color32,
    font_id: &FontId,
) {
    if let Some(search_term) = search_term {
        let matches = search_term.find_match_indices_in(text_str);
        if !matches.is_empty() {
            let mut start = 0;
            for match_idx in matches {
                append(job, &text_str[start..match_idx], text_color, None, font_id);

                let highlight_end_idx = match_idx + search_term.len();

                append(
                    job,
                    &text_str[match_idx..highlight_end_idx],
                    text_color,
                    Some(highlight_color),
                    font_id,
                );

                start = highlight_end_idx;
            }
            append(job, &text_str[start..], text_color, None, font_id);
            return;
        }
    }
    append(job, text_str, text_color, None, font_id);
}

fn append(
    job: &mut LayoutJob,
    text_str: &str,
    color: Color32,
    background_color: Option<Color32>,
    font_id: &FontId,
) {
    let mut text_format = TextFormat {
        color,
        font_id: font_id.clone(),
        ..Default::default()
    };

    if let Some(background_color) = background_color {
        text_format.background = background_color;
    }

    job.append(text_str, 0.0, text_format);
}

fn render_punc(
    ui: &mut Ui,
    punc_str: &str,
    color: Color32,
    background_color: Option<Color32>,
    font_id: &FontId,
) -> Response {
    let mut job = LayoutJob::default();
    append(&mut job, punc_str, color, background_color, font_id);
    render_job(ui, job)
}

fn render_job(ui: &mut Ui, job: LayoutJob) -> Response {
    ui.add(Label::new(job).sense(Sense::click_and_drag()))
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

fn get_pointer_string(path_segments: &[String]) -> String {
    if path_segments.is_empty() {
        "".to_string()
    } else {
        format!("/{}", path_segments.join("/"))
    }
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
