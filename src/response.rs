use egui::{Id, Ui};

use crate::node::ShouldResetExpanded;

/// The response from showing a [`JsonTree`](crate::JsonTree).
pub struct JsonTreeResponse {
    pub(crate) tree_id: Id,
}

impl JsonTreeResponse {
    /// For the [`JsonTree`](crate::JsonTree) that provided this response,
    /// resets the expanded state for all of its arrays/objects to respect the `default_expand` setting.
    ///
    /// Call this whenever the `default_expand` setting changes,
    /// and/or you when wish to reset any manually collapsed/expanded arrays and objects to respect this setting.
    pub fn reset_expanded(&self, ui: &mut Ui) {
        ui.ctx()
            .data_mut(|d| d.insert_temp(self.tree_id, ShouldResetExpanded))
    }
}
