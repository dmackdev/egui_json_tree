use egui::{
    Id, Ui,
    collapsing_header::{CollapsingState, paint_default_icon},
};

use crate::{
    DefaultExpand, JsonTree, JsonTreeStyle, ToggleButtonsState,
    default_expand::InnerDefaultExpand,
    delimiters::{ARRAY_DELIMITERS, OBJECT_DELIMITERS, SpacingDelimiter},
    pointer::{JsonPointer, JsonPointerSegment},
    render::{
        JsonTreeRenderer, ParentStatus, RenderBaseValueContext, RenderExpandableDelimiterContext,
        RenderPropertyContext, RenderSpacingDelimiterContext,
    },
    response::JsonTreeResponse,
    search::SearchTerm,
    value::{ExpandableType, JsonTreeValue, ToJsonTreeValue},
};

pub(crate) struct JsonTreeNode<'a, 'b, T: ToJsonTreeValue> {
    /// The Id of the entire tree that this node is part of.
    tree_id: Id,
    value: &'a T,
    parent: Option<JsonPointerSegment<'a>>,
    make_persistent_id: &'b dyn Fn(&[JsonPointerSegment]) -> Id,
    config: &'b JsonTreeNodeConfig,
}

impl<'a, 'b, T: ToJsonTreeValue> JsonTreeNode<'a, 'b, T> {
    pub(crate) fn show(tree: JsonTree<'a, T>, ui: &mut Ui) -> JsonTreeResponse {
        let persistent_id = ui.id();
        let tree_id = tree.id;
        let make_persistent_id =
            |path_segments: &[JsonPointerSegment]| persistent_id.with(tree_id.with(path_segments));

        let style = tree.config.style.unwrap_or_default();
        let default_expand = tree.config.default_expand.unwrap_or_default();

        let (inner_default_expand, search_term) = match default_expand {
            DefaultExpand::All => (InnerDefaultExpand::All, None),
            DefaultExpand::None => (InnerDefaultExpand::None, None),
            DefaultExpand::ToLevel(l) => (InnerDefaultExpand::ToLevel(l), None),
            DefaultExpand::SearchResults("") => (InnerDefaultExpand::None, None),
            DefaultExpand::SearchResultsOrAll("") => (InnerDefaultExpand::All, None),
            DefaultExpand::SearchResults(search_str)
            | DefaultExpand::SearchResultsOrAll(search_str) => {
                let search_term = SearchTerm::new(search_str);
                let search_match_path_ids = search_term.find_matching_paths_in(
                    tree.value,
                    style.abbreviate_root,
                    &make_persistent_id,
                );
                (
                    InnerDefaultExpand::Paths(search_match_path_ids),
                    Some(search_term),
                )
            }
        };

        let node = JsonTreeNode {
            tree_id,
            value: tree.value,
            parent: None,
            make_persistent_id: &make_persistent_id,
            config: &JsonTreeNodeConfig {
                inner_default_expand,
                style,
                search_term,
            },
        };

        let should_reset_expanded = ui.ctx().data_mut(|d| {
            if tree.config.auto_reset_expanded {
                let default_expand_hash_id = ResetExpandedHashId(egui::util::hash(default_expand));
                let default_expand_changed =
                    d.get_temp::<ResetExpandedHashId>(tree.id) != Some(default_expand_hash_id);
                if default_expand_changed {
                    d.insert_temp(tree.id, default_expand_hash_id);
                    d.insert_temp(tree.id, ShouldResetExpanded);
                }
            }
            d.remove_temp::<ShouldResetExpanded>(tree_id).is_some()
        });

        let mut renderer = tree.config.renderer;

        // Wrap in a vertical layout in case this tree is placed directly in a horizontal layout,
        // which does not allow indent layouts as direct children.
        ui.vertical(|ui| {
            // Centres the collapsing header icon.
            ui.spacing_mut().interact_size.y = node.config.style.resolve_font_id(ui).size;

            node.show_impl(ui, &mut vec![], &mut renderer, should_reset_expanded);
        });

        JsonTreeResponse { tree_id }
    }

    fn show_impl(
        self,
        ui: &mut Ui,
        path_segments: &'b mut Vec<JsonPointerSegment<'a>>,
        renderer: &'b mut JsonTreeRenderer<'a, T>,
        should_reset_expanded: bool,
    ) {
        match self.value.to_json_tree_value() {
            JsonTreeValue::Base(value, display_value, value_type) => {
                // Use horizontal instead of horizontal_wrapped so that the
                // base value always starts inline with the property and not below it.
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;

                    if let Some(property) = self.parent {
                        renderer.render_property(
                            ui,
                            RenderPropertyContext {
                                property,
                                value: self.value,
                                pointer: JsonPointer(path_segments),
                                style: &self.config.style,
                                search_term: self.config.search_term.as_ref(),
                                collapsing_state: None,
                            },
                        );
                        renderer.render_spacing_delimiter(
                            ui,
                            RenderSpacingDelimiterContext {
                                delimiter: SpacingDelimiter::Colon,
                                style: &self.config.style,
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
                            style: &self.config.style,
                            search_term: self.config.search_term.as_ref(),
                            parent_status: if self.parent.is_some() {
                                ParentStatus::ExpandedParent
                            } else {
                                ParentStatus::NoParent
                            },
                        },
                    );
                });
            }
            JsonTreeValue::Expandable(entries, expandable_type) => {
                self.show_expandable(
                    ui,
                    path_segments,
                    renderer,
                    entries,
                    expandable_type,
                    should_reset_expanded,
                );
            }
        };
    }

    fn show_expandable(
        self,
        ui: &mut Ui,
        path_segments: &'b mut Vec<JsonPointerSegment<'a>>,
        renderer: &'b mut JsonTreeRenderer<'a, T>,
        entries: Vec<(JsonPointerSegment<'a>, &'a T)>,
        expandable_type: ExpandableType,
        should_reset_expanded: bool,
    ) {
        let JsonTreeNodeConfig {
            inner_default_expand,
            style,
            search_term,
        } = self.config;

        let delimiters = match expandable_type {
            ExpandableType::Array => &ARRAY_DELIMITERS,
            ExpandableType::Object => &OBJECT_DELIMITERS,
        };

        let path_id = (self.make_persistent_id)(path_segments);

        let default_open = match &inner_default_expand {
            InnerDefaultExpand::All => true,
            InnerDefaultExpand::None => false,
            InnerDefaultExpand::ToLevel(num_levels_open) => {
                (path_segments.len() as u8) <= *num_levels_open
            }
            InnerDefaultExpand::Paths(search_match_path_ids) => {
                search_match_path_ids.contains(&path_id)
            }
        };

        let mut state = CollapsingState::load_with_default_open(ui.ctx(), path_id, default_open);
        if should_reset_expanded {
            state.set_open(default_open);
        }
        let is_expanded = state.is_open();

        let header_res = ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            if let Some(enabled) = style.toggle_buttons_state.enabled() {
                ui.add_enabled_ui(enabled, |ui| {
                    let _response = state.show_toggle_button(ui, paint_default_icon);
                    #[cfg(test)]
                    _response.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::WidgetType::CollapsingHeader,
                            ui.is_enabled(),
                            JsonPointer(path_segments).to_json_pointer_string(),
                        )
                    });
                });
            }

            if path_segments.is_empty() && !is_expanded {
                if style.abbreviate_root {
                    renderer.render_expandable_delimiter(
                        ui,
                        RenderExpandableDelimiterContext {
                            delimiter: delimiters.collapsed,
                            value: self.value,
                            pointer: JsonPointer(path_segments),
                            style,
                            collapsing_state: &mut state,
                        },
                    );
                    return;
                }

                renderer.render_expandable_delimiter(
                    ui,
                    RenderExpandableDelimiterContext {
                        delimiter: delimiters.opening,
                        value: self.value,
                        pointer: JsonPointer(path_segments),
                        style,
                        collapsing_state: &mut state,
                    },
                );
                renderer.render_spacing_delimiter(
                    ui,
                    RenderSpacingDelimiterContext {
                        delimiter: SpacingDelimiter::Empty,
                        style,
                    },
                );

                let entries_len = entries.len();

                for (idx, (property, elem)) in entries.iter().enumerate() {
                    path_segments.push(*property);

                    // Don't show array indices when the array is collapsed.
                    if matches!(expandable_type, ExpandableType::Object) {
                        renderer.render_property(
                            ui,
                            RenderPropertyContext {
                                property: *property,
                                value: elem,
                                pointer: JsonPointer(path_segments),
                                style,
                                search_term: search_term.as_ref(),
                                collapsing_state: Some(&mut state),
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
                                    parent_status: ParentStatus::CollapsedRoot,
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
                                    collapsing_state: &mut state,
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
                        value: self.value,
                        pointer: JsonPointer(path_segments),
                        style,
                        collapsing_state: &mut state,
                    },
                );
            } else {
                if let Some(property) = self.parent {
                    renderer.render_property(
                        ui,
                        RenderPropertyContext {
                            property,
                            value: self.value,
                            pointer: JsonPointer(path_segments),
                            style,
                            search_term: self.config.search_term.as_ref(),
                            collapsing_state: Some(&mut state),
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
                            value: self.value,
                            pointer: JsonPointer(path_segments),
                            style,
                            collapsing_state: &mut state,
                        },
                    );
                } else {
                    let delimiter = if entries.is_empty() {
                        delimiters.collapsed_empty
                    } else {
                        delimiters.collapsed
                    };
                    renderer.render_expandable_delimiter(
                        ui,
                        RenderExpandableDelimiterContext {
                            delimiter,
                            value: self.value,
                            pointer: JsonPointer(path_segments),
                            style,
                            collapsing_state: &mut state,
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
            for (property, elem) in entries {
                let is_expandable = elem.is_expandable();

                path_segments.push(property);

                let mut add_nested_tree = |ui: &mut Ui| {
                    let nested_tree = JsonTreeNode {
                        tree_id: self.tree_id,
                        value: elem,
                        parent: Some(property),
                        make_persistent_id: self.make_persistent_id,
                        config: self.config,
                    };

                    nested_tree.show_impl(ui, path_segments, renderer, should_reset_expanded);
                };

                if is_expandable && !toggle_buttons_hidden {
                    add_nested_tree(ui);
                } else {
                    ui.scope(|ui| {
                        ui.visuals_mut().indent_has_left_vline = false;
                        ui.spacing_mut().indent =
                            ui.spacing().icon_width + ui.spacing().icon_spacing;

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
                        value: self.value,
                        pointer: JsonPointer(path_segments),
                        style,
                        collapsing_state: &mut state,
                    },
                );
            });
        }
        // Ensure we store any change to the state if we reset expanded or if the render hook mutated it.
        state.store(ui.ctx());
    }
}

struct JsonTreeNodeConfig {
    inner_default_expand: InnerDefaultExpand,
    style: JsonTreeStyle,
    search_term: Option<SearchTerm>,
}

/// Stored in `egui`'s `IdTypeMap` to represent a hashed value to indicate whether to reset expanded arrays/objects when this changes for a particular tree Id.
/// Avoids potential conflicts in case a `u64` happened to be stored against the same tree Id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ResetExpandedHashId(u64);

/// Stored in `egui`'s `IdTypeMap` to indicate that the tree should reset its expanded arrays/objects before rendering on a given frame.
/// Avoids potential conflicts in case a `bool` happened to be stored against the same tree Id.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct ShouldResetExpanded;
