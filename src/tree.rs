use crate::{
    node::JsonTreeNode, render_hooks::RenderHooks, value::ToJsonTreeValue, DefaultExpand,
    JsonTreeResponse, JsonTreeStyle,
};
use egui::{Id, Response, Ui};
use std::hash::Hash;

#[derive(Default)]
pub struct JsonTreeConfig<'a> {
    pub(crate) default_expand: DefaultExpand<'a>,
    pub(crate) abbreviate_root: bool,
    pub(crate) render_hooks: RenderHooks<'a>,
}

/// An interactive JSON tree visualiser.
#[must_use = "You should call .show()"]
pub struct JsonTree<'a, T: ToJsonTreeValue> {
    id: Id,
    value: &'a T,
    config: JsonTreeConfig<'a>,
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
        self.config.render_hooks.style = style;
        self
    }

    /// Override how the [`JsonTree`] expands arrays/objects by default.
    pub fn default_expand(mut self, default_expand: DefaultExpand<'a>) -> Self {
        self.config.default_expand = default_expand;
        self
    }

    /// Register a callback to handle interactions within a [`JsonTree`].
    /// - `Response`: The `Response` from rendering an array index, object key or value.
    /// - `&String`: A JSON pointer string.
    pub fn response_callback(mut self, response_callback: impl FnMut(Response, &str) + 'a) -> Self {
        self.config.render_hooks.response_callback = Some(Box::new(response_callback));
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
