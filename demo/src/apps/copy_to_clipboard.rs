use egui::{CursorIcon, Ui};
use egui_json_tree::{JsonTree, render::DefaultRender};
use serde_json::Value;

use super::Show;

pub struct CopyToClipboardExample {
    value: Value,
}

impl CopyToClipboardExample {
    pub fn new(value: Value) -> Self {
        Self { value }
    }
}

impl Show for CopyToClipboardExample {
    fn title(&self) -> &'static str {
        "Copy To Clipboard"
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.hyperlink_to("Source", "https://github.com/dmackdev/egui_json_tree/blob/main/demo/src/apps/copy_to_clipboard.rs");
        ui.label("Right click on elements within the tree to copy the JSON pointer string or contents to the clipboard.");
        ui.add_space(10.0);

        JsonTree::new(self.title(), &self.value)
            .on_render(|ui, context| {
                context
                    .render_default(ui)
                    .on_hover_cursor(CursorIcon::ContextMenu)
                    .context_menu(|ui| {
                        let pointer = context.pointer().to_json_pointer_string();
                        if !pointer.is_empty() && ui.button("Copy path").clicked() {
                            println!("{pointer}");
                            ui.ctx().copy_text(pointer);
                        }

                        if ui.button("Copy contents").clicked() {
                            if let Ok(pretty_str) = serde_json::to_string_pretty(context.value()) {
                                println!("{pretty_str}");
                                ui.ctx().copy_text(pretty_str);
                            }
                        }
                    });
            })
            .show(ui);
    }
}
