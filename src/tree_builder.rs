use crate::{
    tree::{DefaultExpand, JsonTree},
    value::JsonTreeValue,
    JsonTreeResponse, JsonTreeStyle,
};
use egui::{Id, Response, Ui};
use std::hash::Hash;

#[derive(Default)]
pub struct JsonTreeConfig<'a> {
    pub(crate) style: JsonTreeStyle,
    pub(crate) default_expand: DefaultExpand<'a>,
    pub(crate) response_callback: Option<Box<dyn FnMut(Response, String) + 'a>>,
}

#[must_use = "You should call .show()"]
pub struct JsonTreeBuilder<'a> {
    pub(crate) id: Id,
    pub(crate) value: JsonTreeValue,
    pub(crate) config: JsonTreeConfig<'a>,
}

impl<'a> JsonTreeBuilder<'a> {
    /// Creates a new [`JsonTreeBuilder`].
    /// `id` must be a globally unique identifier.
    pub fn new(id: impl Hash, value: impl Into<JsonTreeValue>) -> Self {
        Self {
            id: Id::new(id),
            value: value.into(),
            config: JsonTreeConfig::default(),
        }
    }

    /// Override colors for JSON syntax highlighting, and search match highlighting.
    pub fn style(mut self, style: JsonTreeStyle) -> Self {
        self.config.style = style;
        self
    }

    pub fn default_expand(mut self, default_expand: DefaultExpand<'a>) -> Self {
        self.config.default_expand = default_expand;
        self
    }

    pub fn response_callback(
        mut self,
        response_callback: impl FnMut(Response, String) + 'a,
    ) -> Self {
        self.config.response_callback = Some(Box::new(response_callback));
        self
    }

    /// Show the JSON tree visualisation within the `Ui`.
    pub fn show(self, ui: &mut Ui) -> JsonTreeResponse {
        JsonTree::new(self.id, self.value).show_with_config(ui, self.config)
    }
}
