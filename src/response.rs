use std::collections::HashSet;

use egui::{collapsing_header::CollapsingState, Id, Response, Ui};

/// The response from showing a [`JsonTree`](crate::JsonTree).
pub struct JsonTreeResponse {
    /// If any object key, array index, or value was hovered, this `Option` will contain the [`Response`](egui::Response)
    /// and JSON pointer string.
    ///
    /// The JSON pointer is an identifier composed of each subsequent object key or array index, e.g. `"/foo/bar/0"`.
    ///
    /// For anything hovered within a collapsed top-level array/object, the JSON pointer string will refer to the entire JSON document, i.e. `""`.
    pub inner: Option<(Response, String)>,
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
