use apps::{
    copy_to_clipboard::CopyToClipboardExample, custom_input::CustomExample,
    editor::JsonEditorExample, search::SearchExample,
    toggle_buttons::ToggleButtonsCustomisationDemo, Example, Show,
};
use serde_json::json;

mod apps;

struct DemoApp {
    examples: Vec<Box<dyn Show>>,
    open_example_idx: Option<usize>,
    left_sidebar_expanded: bool,
}

impl Default for DemoApp {
    fn default() -> Self {
        let complex_object = json!({"foo": [1, 2, [3]], "bar": { "qux" : false, "thud": { "a/b": [4, 5, { "m~n": "Greetings!" }]}, "grep": 21}, "baz": null});

        Self {
            examples: vec![
                Box::new(Example::new("Null", json!(null))),
                Box::new(Example::new("Bool", json!(true))),
                Box::new(Example::new("Number (int)", json!(42))),
                Box::new(Example::new("Number (neg int)", json!(-273))),
                Box::new(Example::new("Number (float)", json!(13.37))),
                Box::new(Example::new("String", json!("This is a string!"))),
                Box::new(Example::new("Array", json!([1, 2, 3]))),
                Box::new(Example::new(
                    "Nested Arrays",
                    json!([1, [2, 3, 4], [5, 6, [7], 8], [9, [[], 10]]]),
                )),
                Box::new(Example::new(
                    "Object",
                    json!({"foo": 123, "bar": "Hello world!", "baz": null}),
                )),
                Box::new(Example::new("Complex Object", complex_object.clone())),
                Box::new(CustomExample::new()),
                Box::new(SearchExample::new(complex_object.clone())),
                Box::new(CopyToClipboardExample::new(complex_object.clone())),
                Box::new(JsonEditorExample::new(complex_object.clone())),
                Box::new(ToggleButtonsCustomisationDemo::new(complex_object)),
            ],
            open_example_idx: None,
            left_sidebar_expanded: true,
        }
    }
}

impl eframe::App for DemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("left-panel")
            .resizable(false)
            .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(10.0))
            .show_animated(ctx, self.left_sidebar_expanded, |ui| {
                collapsible_sidebar_button_ui(ui, &mut self.left_sidebar_expanded);
                ui.add_space(10.0);

                ui.label(egui::RichText::new("Theme").monospace());
                egui::global_theme_preference_buttons(ui);
                ui.add_space(10.0);

                ui.label(egui::RichText::new("Examples").monospace());
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    for (idx, example) in self.examples.iter().enumerate() {
                        let is_open = self
                            .open_example_idx
                            .is_some_and(|open_idx| open_idx == idx);

                        if ui.selectable_label(is_open, example.title()).clicked() {
                            if is_open {
                                self.open_example_idx = None;
                            } else {
                                self.open_example_idx = Some(idx);
                            }
                        }
                    }
                });
            });

        let example = self
            .open_example_idx
            .map(|open_idx| &mut self.examples[open_idx]);

        if let Some(example) = &example {
            egui::TopBottomPanel::top("top-panel")
                .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(10.0))
                .show(ctx, |ui| {
                    ui.horizontal_centered(|ui| {
                        if !self.left_sidebar_expanded {
                            collapsible_sidebar_button_ui(ui, &mut self.left_sidebar_expanded);
                        }
                        ui.heading(example.title());
                    });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            match example {
                Some(example) => {
                    example.show(ui);
                }
                None => {
                    if !self.left_sidebar_expanded {
                        collapsible_sidebar_button_ui(ui, &mut self.left_sidebar_expanded);
                    }
                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| {
                            ui.heading("Select an example.");
                        },
                    );
                }
            };
        });
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }
}

fn collapsible_sidebar_button_ui(ui: &mut egui::Ui, open: &mut bool) {
    if ui.button("☰").clicked() {
        *open = !*open;
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let _ = eframe::run_native(
        "egui_json_tree Demo",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::<DemoApp>::default())),
    );
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("canvas")
            .expect("Failed to find element with id: canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("canvas was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::<DemoApp>::default())),
            )
            .await;

        // Remove the loading text and spinner:
        let loading_text = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("loading_text"));
        if let Some(loading_text) = loading_text {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
