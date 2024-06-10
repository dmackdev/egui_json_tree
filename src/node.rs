use std::collections::{HashMap, HashSet};

use egui::{collapsing_header::CollapsingState, Id, Ui};

use crate::{
    delimiters::{ARRAY_DELIMITERS, COMMA_SPACE, EMPTY_SPACE, OBJECT_DELIMITERS},
    pointer::JsonPointer,
    render_hooks::{RenderHooks, RenderKeyContext, RenderPuncContext, RenderValueContext},
    response::JsonTreeResponse,
    search::SearchTerm,
    tree::JsonTreeConfig,
    value::{ExpandableType, JsonTreeValue, NestedProperty, ToJsonTreeValue},
    DefaultExpand,
};

pub struct JsonTreeNode<'a, T: ToJsonTreeValue> {
    id: Id,
    value: &'a T,
    parent: Option<NestedProperty<'a>>,
}

impl<'a, T: ToJsonTreeValue> JsonTreeNode<'a, T> {
    pub(crate) fn new(id: Id, value: &'a T) -> Self {
        Self {
            id,
            value,
            parent: None,
        }
    }

    pub(crate) fn show_with_config(
        self,
        ui: &mut Ui,
        config: JsonTreeConfig<'a, T>,
    ) -> JsonTreeResponse {
        let persistent_id = ui.id();
        let tree_id = self.id;
        let make_persistent_id =
            |path_segments: &Vec<NestedProperty>| persistent_id.with(tree_id.with(path_segments));

        let mut path_id_map = HashMap::new();

        let (default_expand, search_term) = match config.default_expand {
            DefaultExpand::All => (InnerExpand::All, None),
            DefaultExpand::None => (InnerExpand::None, None),
            DefaultExpand::ToLevel(l) => (InnerExpand::ToLevel(l), None),
            DefaultExpand::SearchResults(search_str) => {
                // If searching, the entire path_id_map must be populated.
                populate_path_id_map(self.value, &mut path_id_map, &make_persistent_id);
                let search_term = SearchTerm::parse(search_str);
                let paths = search_term
                    .as_ref()
                    .map(|search_term| {
                        search_term.find_matching_paths_in(self.value, config.abbreviate_root)
                    })
                    .unwrap_or_default();
                (InnerExpand::Paths(paths), search_term)
            }
        };

        let node_config = JsonTreeNodeConfig {
            default_expand,
            abbreviate_root: config.abbreviate_root,
        };

        let mut render_hooks = config.render_hooks;
        render_hooks.search_term = search_term;

        // Wrap in a vertical layout in case this tree is placed directly in a horizontal layout,
        // which does not allow indent layouts as direct children.
        ui.vertical(|ui| {
            // Centres the collapsing header icon.
            ui.spacing_mut().interact_size.y = render_hooks.style.font_id(ui).size;

            self.show_impl(
                ui,
                &mut vec![],
                &mut path_id_map,
                &make_persistent_id,
                &node_config,
                &mut render_hooks,
            );
        });

        JsonTreeResponse {
            collapsing_state_ids: path_id_map.into_values().collect(),
        }
    }

    fn show_impl<'b>(
        self,
        ui: &mut Ui,
        path_segments: &'b mut Vec<NestedProperty<'a>>,
        path_id_map: &'b mut PathIdMap<'a>,
        make_persistent_id: &'b dyn Fn(&Vec<NestedProperty>) -> Id,
        config: &'b JsonTreeNodeConfig<'a>,
        render_hooks: &'b mut RenderHooks<'a, T>,
    ) {
        match self.value.to_json_tree_value() {
            JsonTreeValue::Base(value, display_value, value_type) => {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;

                    if let Some(key) = self.parent {
                        render_hooks.render_key(
                            ui,
                            RenderKeyContext {
                                key,
                                pointer: JsonPointer(path_segments),
                            },
                        );
                    }

                    render_hooks.render_value(
                        ui,
                        RenderValueContext {
                            value,
                            display_value,
                            value_type,
                            pointer: JsonPointer(path_segments),
                        },
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
                    &make_persistent_id,
                    config,
                    render_hooks,
                );
            }
        };
    }
}

fn show_expandable<'a, 'b, T: ToJsonTreeValue>(
    ui: &mut Ui,
    path_segments: &'b mut Vec<NestedProperty<'a>>,
    path_id_map: &'b mut PathIdMap<'a>,
    expandable: Expandable<'a, T>,
    make_persistent_id: &'b dyn Fn(&Vec<NestedProperty>) -> Id,
    config: &'b JsonTreeNodeConfig<'a>,
    render_hooks: &'b mut RenderHooks<'a, T>,
) {
    let JsonTreeNodeConfig {
        default_expand,
        abbreviate_root,
    } = config;

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
        .or_insert_with(|| make_persistent_id(path_segments));

    let state = CollapsingState::load_with_default_open(ui.ctx(), id_source, default_open);
    let is_expanded = state.is_open();

    state
        .show_header(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;

                if path_segments.is_empty() && !is_expanded {
                    if *abbreviate_root {
                        render_hooks.render_punc(
                            ui,
                            RenderPuncContext {
                                punc: delimiters.collapsed,
                                pointer: JsonPointer(path_segments),
                            },
                        );
                        return;
                    }

                    render_hooks.render_punc(
                        ui,
                        RenderPuncContext {
                            punc: delimiters.opening,
                            pointer: JsonPointer(path_segments),
                        },
                    );
                    render_hooks.render_punc(
                        ui,
                        RenderPuncContext {
                            punc: EMPTY_SPACE,
                            pointer: JsonPointer(path_segments),
                        },
                    );

                    let entries_len = expandable.entries.len();

                    for (idx, (key, elem)) in expandable.entries.iter().enumerate() {
                        // Don't show array indices when the array is collapsed.
                        if matches!(expandable.expandable_type, ExpandableType::Object) {
                            render_hooks.render_key(
                                ui,
                                RenderKeyContext {
                                    key: *key,
                                    pointer: JsonPointer(path_segments),
                                },
                            );
                        }

                        match elem.to_json_tree_value() {
                            JsonTreeValue::Base(value, display_value, value_type) => {
                                render_hooks.render_value(
                                    ui,
                                    RenderValueContext {
                                        value,
                                        display_value,
                                        value_type,
                                        pointer: JsonPointer(path_segments),
                                    },
                                );
                            }
                            JsonTreeValue::Expandable(entries, expandable_type) => {
                                let nested_delimiters = match expandable_type {
                                    ExpandableType::Array => &ARRAY_DELIMITERS,
                                    ExpandableType::Object => &OBJECT_DELIMITERS,
                                };

                                let delimiter = if entries.is_empty() {
                                    nested_delimiters.collapsed_empty
                                } else {
                                    nested_delimiters.collapsed
                                };

                                render_hooks.render_punc(
                                    ui,
                                    RenderPuncContext {
                                        punc: delimiter,
                                        pointer: JsonPointer(path_segments),
                                    },
                                );
                            }
                        };
                        let spacing = if idx == entries_len - 1 {
                            EMPTY_SPACE
                        } else {
                            COMMA_SPACE
                        };
                        render_hooks.render_punc(
                            ui,
                            RenderPuncContext {
                                punc: spacing,
                                pointer: JsonPointer(path_segments),
                            },
                        );
                    }

                    render_hooks.render_punc(
                        ui,
                        RenderPuncContext {
                            punc: delimiters.closing,
                            pointer: JsonPointer(path_segments),
                        },
                    );
                } else {
                    if let Some(key) = expandable.parent {
                        render_hooks.render_key(
                            ui,
                            RenderKeyContext {
                                key,
                                pointer: JsonPointer(path_segments),
                            },
                        );
                    }

                    if is_expanded {
                        render_hooks.render_punc(
                            ui,
                            RenderPuncContext {
                                punc: delimiters.opening,
                                pointer: JsonPointer(path_segments),
                            },
                        );
                    } else {
                        let delimiter = if expandable.entries.is_empty() {
                            delimiters.collapsed_empty
                        } else {
                            delimiters.collapsed
                        };
                        render_hooks.render_punc(
                            ui,
                            RenderPuncContext {
                                punc: delimiter,
                                pointer: JsonPointer(path_segments),
                            },
                        );
                    }
                }
            });
        })
        .body(|ui| {
            for (key, elem) in expandable.entries {
                let is_expandable = elem.is_expandable();

                path_segments.push(key);

                let mut add_nested_tree = |ui: &mut Ui| {
                    let nested_tree = JsonTreeNode {
                        id: expandable.id,
                        value: elem,
                        parent: Some(key),
                    };

                    nested_tree.show_impl(
                        ui,
                        path_segments,
                        path_id_map,
                        make_persistent_id,
                        config,
                        render_hooks,
                    );
                };

                if is_expandable {
                    add_nested_tree(ui);
                } else {
                    ui.scope(|ui| {
                        ui.visuals_mut().indent_has_left_vline = false;
                        ui.spacing_mut().indent =
                            ui.spacing().icon_width + ui.spacing().icon_spacing;

                        ui.indent(id_source, add_nested_tree);
                    });
                }

                path_segments.pop();
            }
        });

    if is_expanded {
        ui.horizontal_wrapped(|ui| {
            let indent = ui.spacing().icon_width / 2.0;
            ui.add_space(indent);
            render_hooks.render_punc(
                ui,
                RenderPuncContext {
                    punc: delimiters.closing,
                    pointer: JsonPointer(path_segments),
                },
            );
        });
    }
}

struct JsonTreeNodeConfig<'a> {
    default_expand: InnerExpand<'a>,
    abbreviate_root: bool,
}

#[derive(Debug, Clone)]
enum InnerExpand<'a> {
    All,
    None,
    ToLevel(u8),
    Paths(HashSet<Vec<NestedProperty<'a>>>),
}

struct Expandable<'a, T> {
    id: Id,
    entries: Vec<(NestedProperty<'a>, &'a T)>,
    expandable_type: ExpandableType,
    parent: Option<NestedProperty<'a>>,
}

type PathIdMap<'a> = HashMap<Vec<NestedProperty<'a>>, Id>;

fn populate_path_id_map<'a, 'b, T: ToJsonTreeValue>(
    value: &'a T,
    path_id_map: &'b mut PathIdMap<'a>,
    make_persistent_id: &'b dyn Fn(&Vec<NestedProperty<'a>>) -> Id,
) {
    populate_path_id_map_impl(value, &mut vec![], path_id_map, make_persistent_id);
}

fn populate_path_id_map_impl<'a, 'b, T: ToJsonTreeValue>(
    value: &'a T,
    path_segments: &'b mut Vec<NestedProperty<'a>>,
    path_id_map: &'b mut PathIdMap<'a>,
    make_persistent_id: &'b dyn Fn(&Vec<NestedProperty<'a>>) -> Id,
) {
    if let JsonTreeValue::Expandable(entries, _) = value.to_json_tree_value() {
        for (key, val) in entries {
            let id = make_persistent_id(path_segments);
            path_id_map.insert(path_segments.clone(), id);
            path_segments.push(key);
            populate_path_id_map_impl(val, path_segments, path_id_map, make_persistent_id);
            path_segments.pop();
        }
    }
}
