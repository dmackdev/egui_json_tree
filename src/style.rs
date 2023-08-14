use egui::Color32;

/// Contains coloring parameters for JSON syntax highlighting, and search match highlighting.
pub struct JsonTreeStyle {
    pub object_key_color: Color32,
    pub array_idx_color: Color32,
    pub null_color: Color32,
    pub bool_color: Color32,
    pub number_color: Color32,
    pub string_color: Color32,
    pub highlight_color: Color32,
}

impl Default for JsonTreeStyle {
    fn default() -> Self {
        Self {
            object_key_color: Color32::from_rgb(161, 206, 235),
            array_idx_color: Color32::from_rgb(96, 103, 168),
            null_color: Color32::from_rgb(103, 154, 209),
            bool_color: Color32::from_rgb(103, 154, 209),
            number_color: Color32::from_rgb(181, 199, 166),
            string_color: Color32::from_rgb(194, 146, 122),
            highlight_color: Color32::from_rgba_premultiplied(72, 72, 72, 50),
        }
    }
}
