use egui::{Color32, FontId, TextStyle, Ui};

use crate::{value::BaseValueType, ToggleButtonsState};

/// Styling configuration to control the appearance of the [`JsonTree`](crate::JsonTree).
#[derive(Debug, Clone, Hash, Default)]
pub struct JsonTreeStyle {
    pub visuals: Option<JsonTreeVisuals>,
    pub font_id: Option<FontId>,
    pub abbreviate_root: bool,
    pub toggle_buttons_state: ToggleButtonsState,
}

impl JsonTreeStyle {
    pub fn new() -> Self {
        Self::default()
    }

    /// The colors to use. Defaults to either a dark or light color scheme, depending on [`egui::Visuals::dark_mode`].
    pub fn visuals(mut self, visuals: JsonTreeVisuals) -> Self {
        self.visuals = Some(visuals);
        self
    }

    /// The font to use. Defaults to `TextStyle::Monospace.resolve(ui.style())`.
    pub fn font_id(mut self, font_id: FontId) -> Self {
        self.font_id = Some(font_id);
        self
    }

    /// Override whether a root array/object should show direct child elements when collapsed.
    ///
    /// If `true`, a collapsed root object would render as: `{...}`.
    ///
    /// If `false`, a collapsed root object would render as: `{ "foo": "bar", "baz": {...} }`.
    ///
    /// Defaults to `false`.
    pub fn abbreviate_root(mut self, abbreviate_root: bool) -> Self {
        self.abbreviate_root = abbreviate_root;
        self
    }

    /// Override the visibility and interactivity of the toggle buttons for expanding/collapsing objects and arrays.
    /// Defaults to [`ToggleButtonsState::VisibleEnabled`].
    pub fn toggle_buttons_state(mut self, toggle_buttons_state: ToggleButtonsState) -> Self {
        self.toggle_buttons_state = toggle_buttons_state;
        self
    }

    /// Resolves the [`JsonTreeVisuals`] color scheme to use.
    pub(crate) fn resolve_visuals(&self, ui: &Ui) -> &JsonTreeVisuals {
        if let Some(visuals) = &self.visuals {
            visuals
        } else if ui.visuals().dark_mode {
            &JsonTreeVisuals::DARK
        } else {
            &JsonTreeVisuals::LIGHT
        }
    }

    /// Resolves the [`FontId`] to use.
    pub(crate) fn resolve_font_id(&self, ui: &Ui) -> FontId {
        if let Some(font_id) = &self.font_id {
            font_id.clone()
        } else {
            TextStyle::Monospace.resolve(ui.style())
        }
    }
}

/// Colors for JSON syntax highlighting, and search match highlighting.
#[derive(Debug, Clone, Hash)]
pub struct JsonTreeVisuals {
    pub object_key_color: Color32,
    pub array_idx_color: Color32,
    pub null_color: Color32,
    pub bool_color: Color32,
    pub number_color: Color32,
    pub string_color: Color32,
    pub highlight_color: Color32,
    /// The color for array brackets, object braces, colons and commas.
    pub punctuation_color: Color32,
}

impl Default for JsonTreeVisuals {
    fn default() -> Self {
        Self::DARK
    }
}

impl JsonTreeVisuals {
    pub const DARK: Self = Self {
        object_key_color: Color32::from_rgb(161, 206, 235),
        array_idx_color: Color32::from_rgb(96, 103, 168),
        null_color: Color32::from_rgb(103, 154, 209),
        bool_color: Color32::from_rgb(103, 154, 209),
        number_color: Color32::from_rgb(181, 199, 166),
        string_color: Color32::from_rgb(194, 146, 122),
        highlight_color: Color32::from_rgba_premultiplied(72, 72, 72, 50),
        punctuation_color: Color32::from_gray(140),
    };

    pub const LIGHT: Self = Self {
        object_key_color: Color32::from_rgb(23, 74, 151),
        array_idx_color: Color32::from_rgb(158, 46, 103),
        null_color: Color32::from_rgb(40, 34, 245),
        bool_color: Color32::from_rgb(40, 34, 245),
        number_color: Color32::from_rgb(1, 97, 63),
        string_color: Color32::from_rgb(149, 38, 31),
        highlight_color: Color32::from_rgba_premultiplied(181, 213, 251, 255),
        punctuation_color: Color32::from_gray(70),
    };

    pub fn get_color(&self, base_value_type: &BaseValueType) -> Color32 {
        match base_value_type {
            BaseValueType::Null => self.null_color,
            BaseValueType::Bool => self.bool_color,
            BaseValueType::Number => self.number_color,
            BaseValueType::String => self.string_color,
        }
    }
}
