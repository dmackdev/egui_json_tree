use crate::{
    DefaultExpand, JsonTreeResponse, JsonTreeStyle,
    node::JsonTreeNode,
    render::{JsonTreeRenderer, RenderContext},
    value::ToJsonTreeValue,
};
use egui::{Id, Ui};
use std::hash::Hash;

pub(crate) struct JsonTreeConfig<'a, T: ToJsonTreeValue> {
    pub(crate) style: Option<JsonTreeStyle>,
    pub(crate) default_expand: Option<DefaultExpand<'a>>,
    pub(crate) auto_reset_expanded: bool,
    pub(crate) renderer: JsonTreeRenderer<'a, T>,
}

impl<T: ToJsonTreeValue> Default for JsonTreeConfig<'_, T> {
    fn default() -> Self {
        Self {
            style: Default::default(),
            default_expand: Default::default(),
            auto_reset_expanded: true,
            renderer: Default::default(),
        }
    }
}

/// An interactive JSON tree visualiser.
#[must_use = "You should call .show()"]
pub struct JsonTree<'a, T: ToJsonTreeValue> {
    pub(crate) id: Id,
    pub(crate) value: &'a T,
    pub(crate) config: JsonTreeConfig<'a, T>,
}

impl<'a, T: ToJsonTreeValue> JsonTree<'a, T> {
    /// Creates a new [`JsonTree`].
    /// `id` must be a globally unique identifier.
    pub fn new(id: impl Hash, value: &'a T) -> Self {
        Self {
            id: Id::new(id),
            value,
            config: JsonTreeConfig::default(),
        }
    }

    /// Override colors for JSON syntax highlighting, and search match highlighting.
    pub fn style(mut self, style: JsonTreeStyle) -> Self {
        self.config.style = Some(style);
        self
    }

    /// Override how the [`JsonTree`] expands arrays/objects by default.
    pub fn default_expand(mut self, default_expand: DefaultExpand<'a>) -> Self {
        self.config.default_expand = Some(default_expand);
        self
    }

    /// If enabled, automatically reset expanded arrays/objects to respect the [`DefaultExpand`] setting when it changes for this tree Id.
    /// This can still be performed manually via [`JsonTreeResponse::reset_expanded`](crate::JsonTreeResponse::reset_expanded) after rendering the tree.
    /// Defaults to enabled.
    pub fn auto_reset_expanded(mut self, auto_reset_expanded: bool) -> Self {
        self.config.auto_reset_expanded = auto_reset_expanded;
        self
    }

    /// A convenience method for conditionally registering a custom rendering hook.
    /// See [`JsonTree::on_render`].
    pub fn on_render_if(
        self,
        condition: bool,
        render_hook: impl FnMut(&mut Ui, RenderContext<'a, '_, T>) + 'a,
    ) -> Self {
        if condition {
            self.on_render(render_hook)
        } else {
            self
        }
    }

    /// Customise rendering of the [`JsonTree`], and/or handle interactions.
    ///
    /// This hook can be used to enrich the visualisation with
    /// extra UI interactions by handling [`egui::Response`] values,
    /// and adding UI elements such as buttons and checkboxes within the [`JsonTree`].
    ///
    /// The provided hook will be called in order to render array indices and brackets,
    /// object keys and braces, and non-recursive JSON values, instead of the default render implementation.
    ///
    /// The [`RenderContext`] argument to the hook provides information about the render call,
    /// including the JSON value and a JSON pointer to it.
    ///
    /// You may also call [`render_ctx.render_default(ui)`](crate::render::DefaultRender) on this argument
    /// (or on any of the render contexts contained within its enum variants) to render as normal.
    ///
    /// See [`copy_to_clipboard.rs`](https://github.com/dmackdev/egui_json_tree/blob/main/demo/src/apps/copy_to_clipboard.rs)
    /// and [`editor.rs`](https://github.com/dmackdev/egui_json_tree/blob/main/demo/src/apps/editor.rs)
    /// from the demo for detailed examples and usage.
    pub fn on_render(
        mut self,
        render_hook: impl FnMut(&mut Ui, RenderContext<'a, '_, T>) + 'a,
    ) -> Self {
        self.config.renderer.render_hook = Some(Box::new(render_hook));
        self
    }

    /// Show the JSON tree visualisation within the `Ui`.
    pub fn show(self, ui: &mut Ui) -> JsonTreeResponse {
        JsonTreeNode::show(self, ui)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use egui::accesskit::Role;
    use egui_kittest::Node;
    use egui_kittest::kittest::NodeT;
    use egui_kittest::{Harness, kittest::Queryable};
    use serde_json::{Value, json};

    use crate::{DefaultExpand, JsonTree, JsonTreeStyle, ToggleButtonsState};

    static OBJECT: LazyLock<Value> = LazyLock::new(|| {
        json!({
          "bar": {
            "grep": 21,
            "qux": false,
          },
          "baz": null,
          "foo": [
            1,
            "two"
          ]
        })
    });

    #[test]
    fn render_object_with_toggle_buttons_visible_disabled() {
        let harness = Harness::new_ui(|ui| {
            JsonTree::new("id", &*OBJECT)
                .default_expand(DefaultExpand::All)
                .style(
                    JsonTreeStyle::new().toggle_buttons_state(ToggleButtonsState::VisibleDisabled),
                )
                .show(ui);
        });

        assert_eq!(query_all_collapsing_headers(&harness).count(), 3);
        assert!(
            query_all_collapsing_headers(&harness).all(|node| node.accesskit_node().is_disabled())
        )
    }

    #[test]
    fn render_object_with_toggle_buttons_visible_enabled() {
        let harness = Harness::new_ui(|ui| {
            JsonTree::new("id", &*OBJECT)
                .default_expand(DefaultExpand::All)
                .style(
                    JsonTreeStyle::new().toggle_buttons_state(ToggleButtonsState::VisibleEnabled),
                )
                .show(ui);
        });

        assert_eq!(query_all_collapsing_headers(&harness).count(), 3);
        assert!(
            query_all_collapsing_headers(&harness).all(|node| !node.accesskit_node().is_disabled())
        )
    }

    #[test]
    fn render_object_with_toggle_buttons_hidden() {
        let harness = Harness::new_ui(|ui| {
            JsonTree::new("id", &*OBJECT)
                .default_expand(DefaultExpand::All)
                .style(JsonTreeStyle::new().toggle_buttons_state(ToggleButtonsState::Hidden))
                .show(ui);
        });

        assert_eq!(query_all_collapsing_headers(&harness).count(), 0);
    }

    #[test]
    fn render_object_with_interaction_and_manual_reset_expanded() {
        let mut harness = Harness::new_ui_state(
            |ui, should_reset_expanded| {
                let response = JsonTree::new("id", &*OBJECT)
                    .default_expand(DefaultExpand::None)
                    .style(JsonTreeStyle::new().abbreviate_root(true))
                    .show(ui);

                if *should_reset_expanded {
                    response.reset_expanded(ui);
                }
            },
            false,
        );

        assert_eq!(query_all_collapsing_headers(&harness).count(), 1);
        assert_eq!(harness.query_all_by_role(Role::Label).count(), 1);

        get_collapsing_header_node(&harness, "").click();
        harness.run();
        assert_eq!(query_all_collapsing_headers(&harness).count(), 3);
        assert_eq!(harness.query_all_by_role(Role::Label).count(), 11);

        get_collapsing_header_node(&harness, "/bar").click();
        harness.run();
        assert_eq!(harness.query_all_by_role(Role::Label).count(), 18);
        assert!(harness.query_by_label("\"grep\"").is_some());
        assert!(harness.query_by_label("21").is_some());
        assert!(harness.query_by_label("\"qux\"").is_some());
        assert!(harness.query_by_label("false").is_some());

        *harness.state_mut() = true;
        // Resetting expanded manually has a one frame delay, since the reset call happens after the tree renders, hence two runs.
        harness.run();
        harness.run();
        assert_eq!(query_all_collapsing_headers(&harness).count(), 1);
        assert_eq!(harness.query_all_by_role(Role::Label).count(), 1);
    }

    #[test]
    fn render_object_with_default_expand_none_and_abbreviated_root() {
        let harness = Harness::new_ui(|ui| {
            JsonTree::new("id", &*OBJECT)
                .default_expand(DefaultExpand::None)
                .style(JsonTreeStyle::new().abbreviate_root(true))
                .show(ui);
        });
        assert_eq!(query_all_collapsing_headers(&harness).count(), 1);
        assert_eq!(harness.get_by_role(Role::Label).value().unwrap(), "{...}");
    }

    #[test]
    fn render_array_with_default_expand_none_and_abbreviated_root() {
        let harness = Harness::new_ui(|ui| {
            JsonTree::new("id", &json!([1, 2, 3]))
                .default_expand(DefaultExpand::None)
                .style(JsonTreeStyle::new().abbreviate_root(true))
                .show(ui);
        });
        assert_eq!(query_all_collapsing_headers(&harness).count(), 1);
        assert_eq!(harness.get_by_role(Role::Label).value().unwrap(), "[...]");
    }

    fn query_all_collapsing_headers<'a, S>(
        harness: &'a Harness<'_, S>,
    ) -> impl Iterator<Item = Node<'a>> {
        harness.query_all_by_role(Role::Button)
    }

    fn get_collapsing_header_node<'a, S>(
        harness: &'a Harness<'_, S>,
        pointer: &'a str,
    ) -> Node<'a> {
        harness.get_by_role_and_label(Role::Button, pointer)
    }
}
