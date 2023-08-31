use egui::{Id, Response, Ui};

use crate::{value::JsonTreeValue, DefaultExpand, JsonTree, JsonTreeResponse, JsonTreeStyle};

#[derive(Default)]
pub struct JsonTreeConfig<'a> {
    pub(crate) style: JsonTreeStyle,
    pub(crate) default_expand: DefaultExpand<'a>,
    pub(crate) response_callback: Option<Box<dyn FnMut(Response, String) + 'a>>,
}

pub struct JsonTreeBuilder<'a> {
    pub(crate) id: Id,
    pub(crate) value: JsonTreeValue,
    pub(crate) config: JsonTreeConfig<'a>,
}

impl<'a> JsonTreeBuilder<'a> {
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

    pub fn show(self, ui: &mut Ui) -> JsonTreeResponse {
        JsonTree::new(self.id, self.value).show_with_config(ui, self.config)
    }
}
