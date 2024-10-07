use egui::Ui;
use egui_json_tree::{DefaultExpand, JsonTree};
use serde_json::Value;

use crate::example::Show;

pub struct SearchExample {
    title: &'static str,
    value: Value,
    search_input: String,
}

impl SearchExample {
    pub fn new(value: Value) -> Self {
        Self {
            title: "Search Example",
            value,
            search_input: "".to_string(),
        }
    }
}

impl Show for SearchExample {
    fn title(&self) -> &'static str {
        self.title
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.label("Search:");

        let (text_edit_response, clear_button_response) = ui
            .horizontal(|ui| {
                let text_edit_response = ui.text_edit_singleline(&mut self.search_input);
                let clear_button_response = ui.button("Clear");
                (text_edit_response, clear_button_response)
            })
            .inner;

        let response = JsonTree::new(self.title, &self.value)
            .default_expand(DefaultExpand::SearchResults(&self.search_input))
            .show(ui);

        if text_edit_response.changed() {
            response.reset_expanded(ui);
        }

        if clear_button_response.clicked() {
            self.search_input.clear();
            response.reset_expanded(ui);
        }

        if ui.button("Reset expanded").clicked() {
            response.reset_expanded(ui);
        }
    }
}
