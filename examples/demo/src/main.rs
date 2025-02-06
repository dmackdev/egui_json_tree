use apps::{
    copy_to_clipboard::CopyToClipboardExample, custom_input::CustomExample,
    editor::JsonEditorExample, search::SearchExample,
    toggle_buttons::ToggleButtonsCustomisationDemo, wrapping::WrappingExample, Example, Show,
};
use egui::Direction;
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
        let long_strings_object = json!({
          "baz": "Ullamco ipsum proident occaecat eiusmod ea aute ex non cupidatat laboris duis amet cupidatat. Ullamco sint do enim consectetur Lorem occaecat mollit. Aliquip voluptate ullamco consectetur adipisicing elit fugiat labore laboris. Occaecat non incididunt duis consectetur aliquip dolore cillum eiusmod. Qui sunt est excepteur laborum.",
          "bar": [
            "Laboris id occaecat sit quis aliqua et. Fugiat nisi nulla nostrud voluptate id enim do esse deserunt non culpa incididunt eiusmod. Minim nulla reprehenderit irure duis amet commodo commodo aliquip ut. Lorem amet ipsum excepteur consectetur qui dolore. In occaecat dolor ullamco voluptate dolore qui incididunt occaecat pariatur est qui aliquip labore non.",
            "Velit ex nisi in et enim veniam ullamco reprehenderit consectetur Lorem. Dolor commodo pariatur Lorem proident. Ad minim aliquip excepteur officia consequat nulla mollit adipisicing ut veniam Lorem. Sint mollit occaecat velit do. Nulla aute Lorem non excepteur.",
            "Officia culpa in adipisicing sunt qui culpa voluptate ad veniam adipisicing anim ex aute. Laboris ipsum id est cillum minim quis sint ex culpa dolore minim. Lorem excepteur deserunt voluptate minim consequat qui quis enim. Do non irure pariatur exercitation commodo laboris sit. Sunt magna nulla magna Lorem reprehenderit dolore et tempor Lorem esse quis exercitation tempor commodo."
          ],
          "qux": {
            "thud": "Et mollit occaecat et aliqua officia adipisicing adipisicing. Fugiat cillum dolor eu laborum cupidatat aliqua et reprehenderit do laboris velit in. Dolor voluptate Lorem pariatur voluptate enim labore in et pariatur consequat esse elit. Do qui aute proident in aliquip. Ea velit quis ex enim proident tempor laboris exercitation aute consectetur minim.",
            "fizz": {
              "buzz": "Sunt Lorem officia reprehenderit ea esse aliqua in veniam. Do irure amet dolore magna amet tempor anim sit irure tempor proident laborum dolore. Aute et ullamco eiusmod culpa et esse. Minim ut elit laboris est. Est mollit et mollit dolore ea adipisicing nostrud excepteur."
            }
          }
        });

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
                Box::new(WrappingExample::new(long_strings_object)),
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
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
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
            if let Some(example) = example {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        example.show(ui);
                    });
            } else {
                if !self.left_sidebar_expanded {
                    collapsible_sidebar_button_ui(ui, &mut self.left_sidebar_expanded);
                }
                ui.with_layout(
                    egui::Layout::centered_and_justified(Direction::LeftToRight),
                    |ui| {
                        ui.heading("Select an example.");
                    },
                );
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
