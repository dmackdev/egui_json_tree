use egui::{Slider, Ui};
use egui_json_tree::{DefaultExpand, JsonTree};
use serde_json::Value;

use super::Show;

#[derive(Default)]
enum StateDefaultExpand {
    All,
    #[default]
    None,
    ToLevel(u8),
    SearchResults(String),
    SearchResultsOrAll(String),
}

impl<'a> From<&'a StateDefaultExpand> for DefaultExpand<'a> {
    fn from(value: &'a StateDefaultExpand) -> Self {
        match value {
            StateDefaultExpand::All => DefaultExpand::All,
            StateDefaultExpand::None => DefaultExpand::None,
            StateDefaultExpand::ToLevel(l) => DefaultExpand::ToLevel(*l),
            StateDefaultExpand::SearchResults(search_term) => {
                DefaultExpand::SearchResults(search_term)
            }
            StateDefaultExpand::SearchResultsOrAll(search_term) => {
                DefaultExpand::SearchResultsOrAll(search_term)
            }
        }
    }
}

pub struct DefaultExpandExample {
    value: Value,
    state_default_expand: StateDefaultExpand,
}

impl DefaultExpandExample {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            state_default_expand: Default::default(),
        }
    }
}

impl Show for DefaultExpandExample {
    fn title(&self) -> &'static str {
        "Default Expand settings"
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.hyperlink_to("Source", "https://github.com/dmackdev/egui_json_tree/blob/main/examples/demo/src/apps/default_expand.rs");
        ui.label("A showcase of the different options to configure how the tree expands arrays and objects by default.");
        ui.add_space(10.0);

        if ui
            .radio(
                matches!(self.state_default_expand, StateDefaultExpand::None),
                "None",
            )
            .clicked()
        {
            self.state_default_expand = StateDefaultExpand::None;
        }
        if ui
            .radio(
                matches!(self.state_default_expand, StateDefaultExpand::All),
                "All",
            )
            .clicked()
        {
            self.state_default_expand = StateDefaultExpand::All;
        }
        if ui
            .radio(
                matches!(self.state_default_expand, StateDefaultExpand::ToLevel(_)),
                "To level",
            )
            .clicked()
        {
            self.state_default_expand = StateDefaultExpand::ToLevel(0);
        }
        if ui
            .radio(
                matches!(
                    self.state_default_expand,
                    StateDefaultExpand::SearchResults(_)
                ),
                "Search results",
            )
            .clicked()
        {
            self.state_default_expand = StateDefaultExpand::SearchResults("".to_string());
        }
        if ui
            .radio(
                matches!(
                    self.state_default_expand,
                    StateDefaultExpand::SearchResultsOrAll(_)
                ),
                "Search results or all",
            )
            .clicked()
        {
            self.state_default_expand = StateDefaultExpand::SearchResultsOrAll("".to_string());
        }

        match &mut self.state_default_expand {
            StateDefaultExpand::All => {}
            StateDefaultExpand::None => {}
            StateDefaultExpand::ToLevel(level) => {
                ui.add(Slider::new(level, 0..=4));
            }
            StateDefaultExpand::SearchResults(search_term)
            | StateDefaultExpand::SearchResultsOrAll(search_term) => {
                ui.label("Search:");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(search_term);
                    if ui.button("Clear").clicked() {
                        search_term.clear();
                    }
                });
            }
        };

        let response = JsonTree::new(self.title(), &self.value)
            .default_expand((&self.state_default_expand).into())
            .show(ui);

        if ui.button("Reset expanded").clicked() {
            response.reset_expanded(ui);
        }
    }
}
