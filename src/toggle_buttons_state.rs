/// Setting for the visibility and interactivity of the toggle buttons for expanding/collapsing objects and arrays.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ToggleButtonsState {
    #[default]
    VisibleEnabled,
    VisibleDisabled,
    Hidden,
}

impl ToggleButtonsState {
    #[expect(clippy::trivially_copy_pass_by_ref, reason = "needs refactoring")]
    pub(crate) const fn enabled(&self) -> Option<bool> {
        match self {
            Self::VisibleEnabled => Some(true),
            Self::VisibleDisabled => Some(false),
            Self::Hidden => None,
        }
    }
}
