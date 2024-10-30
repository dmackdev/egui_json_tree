use std::collections::HashSet;

use egui::{
    collapsing_header::{paint_default_icon, CollapsingState},
    Id, Ui,
};

use crate::{
    delimiters::{SpacingDelimiter, ARRAY_DELIMITERS, OBJECT_DELIMITERS},
    pointer::{JsonPointer, JsonPointerSegment},
    render::{
        JsonTreeRenderer, RenderBaseValueContext, RenderExpandableDelimiterContext,
        RenderPropertyContext, RenderSpacingDelimiterContext,
    },
    response::JsonTreeResponse,
    search::SearchTerm,
    tree::JsonTreeConfig,
    value::{ExpandableType, JsonTreeValue, ToJsonTreeValue},
    DefaultExpand, JsonTreeStyle, ToggleButtonsState,
};

pub(crate) struct JsonTreeNode<'a, T: ToJsonTreeValue> {
    id: Id,
    value: &'a T,
    parent: Option<JsonPointerSegment<'a>>,
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
            |path_segments: &[JsonPointerSegment]| persistent_id.with(tree_id.with(path_segments));

        let style = config.style.unwrap_or_default();
        let default_expand = config.default_expand.unwrap_or_default();

        let mut path_id_map = HashSet::new();

        let (default_expand, search_term) = match default_expand {
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
                        search_term.find_matching_paths_in(
                            self.value,
                            style.abbreviate_root,
                            &make_persistent_id,
                        )
                    })
                    .unwrap_or_default();
                (InnerExpand::Paths(paths), search_term)
            }
        };

        let mut renderer = config.renderer;

        let node_config = JsonTreeNodeConfig {
            default_expand,
            style,
            search_term,
        };

        // Wrap in a vertical layout in case this tree is placed directly in a horizontal layout,
        // which does not allow indent layouts as direct children.
        ui.vertical(|ui| {
            // Centres the collapsing header icon.
            ui.spacing_mut().interact_size.y = node_config.style.resolve_font_id(ui).size;

            self.show_impl(
                ui,
                &mut vec![],
                &mut path_id_map,
                &make_persistent_id,
                &node_config,
                &mut renderer,
            );
        });

        JsonTreeResponse {
            collapsing_state_ids: path_id_map,
        }
    }

    fn show_impl<'b>(
        self,
        ui: &mut Ui,
        path_segments: &'b mut Vec<JsonPointerSegment<'a>>,
        path_id_map: &'b mut PathIdMap<'a>,
        make_persistent_id: &'b dyn Fn(&[JsonPointerSegment]) -> Id,
        config: &'b JsonTreeNodeConfig,
        renderer: &'b mut JsonTreeRenderer<'a, T>,
    ) {
        match self.value.to_json_tree_value() {
            JsonTreeValue::Base(value, display_value, value_type) => {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;

                    if let Some(property) = self.parent {
                        renderer.render_property(
                            ui,
                            RenderPropertyContext {
                                property,
                                value: self.value,
                                pointer: JsonPointer(path_segments),
                                style: &config.style,
                                search_term: config.search_term.as_ref(),
                            },
                        );
                        renderer.render_spacing_delimiter(
                            ui,
                            RenderSpacingDelimiterContext {
                                delimiter: SpacingDelimiter::Colon,
                                style: &config.style,
                            },
                        );
                    }

                    renderer.render_value(
                        ui,
                        RenderBaseValueContext {
                            value,
                            display_value,
                            value_type,
                            pointer: JsonPointer(path_segments),
                            style: &config.style,
                            search_term: config.search_term.as_ref(),
                        },
                    );
                });
            }
            JsonTreeValue::Expandable(entries, expandable_type) => {
                let expandable = Expandable {
                    id: self.id,
                    value: self.value,
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
                    renderer,
                );
            }
        };
    }
}

fn show_expandable<'a, 'b, T: ToJsonTreeValue>(
    ui: &mut Ui,
    path_segments: &'b mut Vec<JsonPointerSegment<'a>>,
    path_id_map: &'b mut PathIdMap<'a>,
    expandable: Expandable<'a, T>,
    make_persistent_id: &'b dyn Fn(&[JsonPointerSegment]) -> Id,
    config: &'b JsonTreeNodeConfig,
    renderer: &'b mut JsonTreeRenderer<'a, T>,
) {
    let JsonTreeNodeConfig {
        default_expand,
        style,
        search_term,
    } = config;

    let delimiters = match expandable.expandable_type {
        ExpandableType::Array => &ARRAY_DELIMITERS,
        ExpandableType::Object => &OBJECT_DELIMITERS,
    };

    let path_id = make_persistent_id(path_segments);
    path_id_map.insert(path_id);

    let default_open = match &default_expand {
        InnerExpand::All => true,
        InnerExpand::None => false,
        InnerExpand::ToLevel(num_levels_open) => (path_segments.len() as u8) <= *num_levels_open,
        InnerExpand::Paths(paths) => paths.contains(&path_id),
    };

    let mut state = CollapsingState::load_with_default_open(ui.ctx(), path_id, default_open);
    let is_expanded = state.is_open();

    let header_res = ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;

        if let Some(enabled) = style.toggle_buttons_state.enabled() {
            ui.add_enabled_ui(enabled, |ui| {
                state.show_toggle_button(ui, paint_default_icon)
            });
        }

        if path_segments.is_empty() && !is_expanded {
            if style.abbreviate_root {
                renderer.render_expandable_delimiter(
                    ui,
                    RenderExpandableDelimiterContext {
                        delimiter: delimiters.collapsed,
                        value: expandable.value,
                        pointer: JsonPointer(path_segments),
                        style,
                    },
                );
                return;
            }

            renderer.render_expandable_delimiter(
                ui,
                RenderExpandableDelimiterContext {
                    delimiter: delimiters.opening,
                    value: expandable.value,
                    pointer: JsonPointer(path_segments),
                    style,
                },
            );
            renderer.render_spacing_delimiter(
                ui,
                RenderSpacingDelimiterContext {
                    delimiter: SpacingDelimiter::Empty,
                    style,
                },
            );

            let entries_len = expandable.entries.len();

            for (idx, (property, elem)) in expandable.entries.iter().enumerate() {
                path_segments.push(*property);

                // Don't show array indices when the array is collapsed.
                if matches!(expandable.expandable_type, ExpandableType::Object) {
                    renderer.render_property(
                        ui,
                        RenderPropertyContext {
                            property: *property,
                            value: elem,
                            pointer: JsonPointer(path_segments),
                            style,
                            search_term: search_term.as_ref(),
                        },
                    );
                    renderer.render_spacing_delimiter(
                        ui,
                        RenderSpacingDelimiterContext {
                            delimiter: SpacingDelimiter::Colon,
                            style,
                        },
                    );
                }

                match elem.to_json_tree_value() {
                    JsonTreeValue::Base(value, display_value, value_type) => {
                        renderer.render_value(
                            ui,
                            RenderBaseValueContext {
                                value,
                                display_value,
                                value_type,
                                pointer: JsonPointer(path_segments),
                                style,
                                search_term: search_term.as_ref(),
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

                        renderer.render_expandable_delimiter(
                            ui,
                            RenderExpandableDelimiterContext {
                                delimiter,
                                value: elem,
                                pointer: JsonPointer(path_segments),
                                style,
                            },
                        );
                    }
                };

                let spacing = if idx == entries_len - 1 {
                    SpacingDelimiter::Empty
                } else {
                    SpacingDelimiter::Comma
                };

                renderer.render_spacing_delimiter(
                    ui,
                    RenderSpacingDelimiterContext {
                        delimiter: spacing,
                        style,
                    },
                );

                path_segments.pop();
            }

            renderer.render_expandable_delimiter(
                ui,
                RenderExpandableDelimiterContext {
                    delimiter: delimiters.closing,
                    value: expandable.value,
                    pointer: JsonPointer(path_segments),
                    style,
                },
            );
        } else {
            if let Some(property) = expandable.parent {
                renderer.render_property(
                    ui,
                    RenderPropertyContext {
                        property,
                        value: expandable.value,
                        pointer: JsonPointer(path_segments),
                        style,
                        search_term: config.search_term.as_ref(),
                    },
                );
                renderer.render_spacing_delimiter(
                    ui,
                    RenderSpacingDelimiterContext {
                        delimiter: SpacingDelimiter::Colon,
                        style,
                    },
                );
            }

            if is_expanded {
                renderer.render_expandable_delimiter(
                    ui,
                    RenderExpandableDelimiterContext {
                        delimiter: delimiters.opening,
                        value: expandable.value,
                        pointer: JsonPointer(path_segments),
                        style,
                    },
                );
            } else {
                let delimiter = if expandable.entries.is_empty() {
                    delimiters.collapsed_empty
                } else {
                    delimiters.collapsed
                };
                renderer.render_expandable_delimiter(
                    ui,
                    RenderExpandableDelimiterContext {
                        delimiter,
                        value: expandable.value,
                        pointer: JsonPointer(path_segments),
                        style,
                    },
                );
            }
        }
    });

    let toggle_buttons_hidden = style.toggle_buttons_state == ToggleButtonsState::Hidden;
    if toggle_buttons_hidden {
        ui.visuals_mut().indent_has_left_vline = true;
        ui.spacing_mut().indent = (ui.spacing().icon_width + ui.spacing().icon_spacing) / 2.0;
    }

    state.show_body_indented(&header_res.response, ui, |ui| {
        for (property, elem) in expandable.entries {
            let is_expandable = elem.is_expandable();

            path_segments.push(property);

            let mut add_nested_tree = |ui: &mut Ui| {
                let nested_tree = JsonTreeNode {
                    id: expandable.id,
                    value: elem,
                    parent: Some(property),
                };

                nested_tree.show_impl(
                    ui,
                    path_segments,
                    path_id_map,
                    make_persistent_id,
                    config,
                    renderer,
                );
            };

            if is_expandable && !toggle_buttons_hidden {
                add_nested_tree(ui);
            } else {
                ui.scope(|ui| {
                    ui.visuals_mut().indent_has_left_vline = false;
                    ui.spacing_mut().indent = ui.spacing().icon_width + ui.spacing().icon_spacing;

                    if toggle_buttons_hidden {
                        ui.spacing_mut().indent /= 2.0;
                    }

                    ui.indent(path_id, add_nested_tree);
                });
            }

            path_segments.pop();
        }
    });

    if is_expanded {
        ui.horizontal_wrapped(|ui| {
            if !toggle_buttons_hidden {
                let indent = ui.spacing().icon_width / 2.0;
                ui.add_space(indent);
            }
            renderer.render_expandable_delimiter(
                ui,
                RenderExpandableDelimiterContext {
                    delimiter: delimiters.closing,
                    value: expandable.value,
                    pointer: JsonPointer(path_segments),
                    style,
                },
            );
        });
    }
}

struct JsonTreeNodeConfig {
    default_expand: InnerExpand,
    style: JsonTreeStyle,
    search_term: Option<SearchTerm>,
}

#[derive(Debug, Clone)]
enum InnerExpand {
    All,
    None,
    ToLevel(u8),
    Paths(HashSet<Id>),
}

struct Expandable<'a, T: ToJsonTreeValue> {
    id: Id,
    value: &'a T,
    entries: Vec<(JsonPointerSegment<'a>, &'a T)>,
    expandable_type: ExpandableType,
    parent: Option<JsonPointerSegment<'a>>,
}

type PathIdMap<'a> = HashSet<Id>;

fn populate_path_id_map<'a, 'b, T: ToJsonTreeValue>(
    value: &'a T,
    path_id_map: &'b mut PathIdMap<'a>,
    make_persistent_id: &'b dyn Fn(&[JsonPointerSegment]) -> Id,
) {
    populate_path_id_map_impl(value, &mut vec![], path_id_map, make_persistent_id);
}

fn populate_path_id_map_impl<'a, 'b, T: ToJsonTreeValue>(
    value: &'a T,
    path_segments: &'b mut Vec<JsonPointerSegment<'a>>,
    path_id_map: &'b mut PathIdMap<'a>,
    make_persistent_id: &'b dyn Fn(&[JsonPointerSegment]) -> Id,
) {
    if let JsonTreeValue::Expandable(entries, _) = value.to_json_tree_value() {
        for (property, val) in entries {
            let id = make_persistent_id(path_segments);
            path_id_map.insert(id);
            path_segments.push(property);
            populate_path_id_map_impl(val, path_segments, path_id_map, make_persistent_id);
            path_segments.pop();
        }
    }
}
