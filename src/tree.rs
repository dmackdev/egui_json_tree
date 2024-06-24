use crate::{
    node::JsonTreeNode,
    render::{JsonTreeRenderer, RenderContext},
    value::ToJsonTreeValue,
    DefaultExpand, JsonTreeResponse, JsonTreeStyle,
};
use egui::{Id, Ui};
use std::hash::Hash;

pub(crate) struct JsonTreeConfig<'a, T: ToJsonTreeValue> {
    pub(crate) style: JsonTreeStyle,
    pub(crate) default_expand: DefaultExpand<'a>,
    pub(crate) abbreviate_root: bool,
    pub(crate) renderer: JsonTreeRenderer<'a, T>,
}

impl<'a, T: ToJsonTreeValue> Default for JsonTreeConfig<'a, T> {
    fn default() -> Self {
        Self {
            style: Default::default(),
            default_expand: Default::default(),
            abbreviate_root: Default::default(),
            renderer: Default::default(),
        }
    }
}

/// An interactive JSON tree visualiser.
#[must_use = "You should call .show()"]
pub struct JsonTree<'a, T: ToJsonTreeValue> {
    id: Id,
    value: &'a T,
    config: JsonTreeConfig<'a, T>,
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
        self.config.style = style;
        self
    }

    /// Override how the [`JsonTree`] expands arrays/objects by default.
    pub fn default_expand(mut self, default_expand: DefaultExpand<'a>) -> Self {
        self.config.default_expand = default_expand;
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
    /// See [`examples/demo.rs`](https://github.com/dmackdev/egui_json_tree/blob/master/examples/demo.rs)
    /// for detailed examples and use cases.
    pub fn on_render(
        mut self,
        render_hook: impl FnMut(&mut Ui, RenderContext<'a, '_, T>) + 'a,
    ) -> Self {
        self.config.renderer.render_hook = Some(Box::new(render_hook));
        self
    }

    /// Override whether a root array/object should show direct child elements when collapsed.
    ///
    /// If called with `true`, a collapsed root object would render as: `{...}`.
    ///
    /// Otherwise, a collapsed root object would render as: `{ "foo": "bar", "baz": {...} }`.
    pub fn abbreviate_root(mut self, abbreviate_root: bool) -> Self {
        self.config.abbreviate_root = abbreviate_root;
        self
    }

    /// Show the JSON tree visualisation within the `Ui`.
    pub fn show(self, ui: &mut Ui) -> JsonTreeResponse {
        JsonTreeNode::new(self.id, self.value).show_with_config(ui, self.config)
    }
}
