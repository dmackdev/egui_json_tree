use std::collections::HashSet;

use egui::{collapsing_header::CollapsingState, Id, Response, Ui};

/// The response from showing a [`JsonTree`](crate::JsonTree).
pub struct JsonTreeResponse {
    pub response: Option<(Response, String)>,
    pub(crate) collapsing_state_ids: HashSet<Id>,
}

impl JsonTreeResponse {
    /// For the [`JsonTree`](crate::JsonTree) that provided this response,
    /// resets the expanded state for all of its arrays/objects to respect the `default_expand` argument of [`JsonTree::show`](crate::JsonTree::show) on the next render.
    ///
    /// Call this whenever the `default_expand` argument changes, and/or you when wish to reset any manually collapsed/expanded arrays and objects to respect this argument.
    pub fn reset_expanded(&self, ui: &mut Ui) {
        for id in self.collapsing_state_ids.iter() {
            if let Some(state) = CollapsingState::load(ui.ctx(), *id) {
                state.remove(ui.ctx());
            }
        }
    }
}
