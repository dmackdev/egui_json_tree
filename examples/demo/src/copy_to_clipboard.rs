use egui::{CursorIcon, Ui};
use egui_json_tree::{render::DefaultRender, JsonTree};
use serde_json::Value;

use crate::example::Show;

pub struct CopyToClipboardExample {
    title: &'static str,
    value: Value,
}

impl CopyToClipboardExample {
    pub fn new(value: Value) -> Self {
        Self {
            title: "Copy To Clipboard Example",
            value,
        }
    }
}

impl Show for CopyToClipboardExample {
    fn title(&self) -> &'static str {
        self.title
    }

    fn show(&mut self, ui: &mut Ui) {
        JsonTree::new(self.title, &self.value)
            .on_render(|ui, context| {
                context
                    .render_default(ui)
                    .on_hover_cursor(CursorIcon::ContextMenu)
                    .context_menu(|ui| {
                        let pointer = context.pointer().to_json_pointer_string();
                        if !pointer.is_empty() && ui.button("Copy path").clicked() {
                            ui.output_mut(|o| {
                                println!("{}", pointer);
                                o.copied_text = pointer;
                            });
                            ui.close_menu();
                        }

                        if ui.button("Copy contents").clicked() {
                            if let Ok(pretty_str) = serde_json::to_string_pretty(context.value()) {
                                println!("{}", pretty_str);
                                ui.output_mut(|o| o.copied_text = pretty_str);
                            }
                            ui.close_menu();
                        }
                    });
            })
            .show(ui);
    }
}
