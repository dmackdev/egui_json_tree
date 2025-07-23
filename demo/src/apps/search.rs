use egui::Ui;
use egui_json_tree::{DefaultExpand, JsonTree};
use serde_json::Value;

use super::Show;

pub struct SearchExample {
    value: Value,
    search_input: String,
}

impl SearchExample {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            search_input: "".to_string(),
        }
    }
}

impl Show for SearchExample {
    fn title(&self) -> &'static str {
        "Search"
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.hyperlink_to(
            "Source",
            "https://github.com/dmackdev/egui_json_tree/blob/main/demo/src/apps/search.rs",
        );
        ui.label("Enter a search term to automatically expand the tree to reveal and highlight the matches.");
        ui.add_space(10.0);

        ui.label("Search:");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.search_input);
            if ui.button("Clear").clicked() {
                self.search_input.clear();
            }
        });

        let response = JsonTree::new(self.title(), &self.value)
            .default_expand(DefaultExpand::SearchResults(&self.search_input))
            .show(ui);

        if ui.button("Reset expanded").clicked() {
            response.reset_expanded(ui);
        }
    }
}
