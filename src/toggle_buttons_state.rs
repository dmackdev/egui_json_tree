/// Setting for the visibility and interactivity of the toggle buttons for expanding/collapsing objects and arrays.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ToggleButtonsState {
    #[default]
    VisibleEnabled,
    VisibleDisabled,
    Hidden,
}

impl ToggleButtonsState {
    pub(crate) fn enabled(&self) -> Option<bool> {
        match self {
            ToggleButtonsState::VisibleEnabled => Some(true),
            ToggleButtonsState::VisibleDisabled => Some(false),
            ToggleButtonsState::Hidden => None,
        }
    }
}
