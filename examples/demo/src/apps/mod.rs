use egui::Ui;
use egui_json_tree::JsonTree;
use serde_json::Value;

pub mod copy_to_clipboard;
pub mod custom_input;
pub mod default_expand;
pub mod editor;
pub mod search;
pub mod toggle_buttons;
pub mod wrapping;

pub trait Show {
    fn title(&self) -> &'static str;
    fn show(&mut self, ui: &mut Ui);
}

pub struct Example {
    title: &'static str,
    value: Value,
}

impl Example {
    pub fn new(title: &'static str, value: Value) -> Self {
        Self { title, value }
    }
}

impl Show for Example {
    fn title(&self) -> &'static str {
        self.title
    }

    fn show(&mut self, ui: &mut Ui) {
        JsonTree::new(self.title, &self.value).show(ui);
    }
}
