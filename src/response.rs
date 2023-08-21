use std::collections::HashSet;

use egui::{collapsing_header::CollapsingState, Id, Ui};

/// The response from showing a [`JsonTree`](crate::JsonTree).
pub struct JsonTreeResponse {
    // TODO: Add me.
    // pub response: Response,
    pub(crate) collapsing_state_ids: HashSet<Id>,
}

impl JsonTreeResponse {
    /// For the [`JsonTree`](crate::JsonTree) that provided this response,
    /// resets the expanded state for all of its arrays/objects to respect its `default_expand` setting.
    ///
    /// You should call this anytime the `default_expand` value changes,
    /// including if the search string in the [`DefaultExpand::SearchResults(String)`](crate::DefaultExpand::SearchResults) variant changes.
    pub fn reset_expanded(&self, ui: &mut Ui) {
        for id in self.collapsing_state_ids.iter() {
            if let Some(state) = CollapsingState::load(ui.ctx(), *id) {
                state.remove(ui.ctx());
            }
        }
    }
}
