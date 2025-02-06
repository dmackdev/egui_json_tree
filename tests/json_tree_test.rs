use std::sync::Arc;

use egui::{mutex::Mutex, CentralPanel, Context, FontDefinitions, RawInput, Style};
use egui_json_tree::{render::RenderContext, DefaultExpand, JsonTree, JsonTreeStyle};
#[cfg(feature = "serde_json")]
use serde_json::{json, Value};

#[cfg(all(feature = "simd_json", not(feature = "serde_json")))]
use simd_json::{json, owned::Value};

#[derive(Debug, PartialEq)]
struct ExpectedRender {
    value: Value,
    display_value: String,
    pointer_str: String,
}

impl<'a, 'b> From<RenderContext<'a, 'b, Value>> for ExpectedRender {
    fn from(ctx: RenderContext<'a, 'b, Value>) -> Self {
        match ctx {
            RenderContext::Property(ctx) => Self {
                value: ctx.value.clone(),
                display_value: ctx.property.to_string(),
                pointer_str: ctx.pointer.to_json_pointer_string(),
            },
            RenderContext::BaseValue(ctx) => Self {
                value: (ctx.value.clone()),
                display_value: ctx.display_value.to_string(),
                pointer_str: ctx.pointer.to_json_pointer_string(),
            },
            RenderContext::ExpandableDelimiter(ctx) => Self {
                value: ctx.value.clone(),
                display_value: ctx.delimiter.as_ref().to_owned(),
                pointer_str: ctx.pointer.to_json_pointer_string(),
            },
        }
    }
}

#[test]
fn json_tree_render_string() {
    let value = json!("Hello World!");

    let actual: Arc<Mutex<Vec<ExpectedRender>>> = Arc::new(Mutex::new(vec![]));

    egui::__run_test_ui(|ui| {
        JsonTree::new("id", &value)
            .on_render(|_, render_ctx| {
                actual.lock().push(render_ctx.into());
            })
            .show(ui);
    });

    let expected = vec![ExpectedRender {
        value: json!("Hello World!"),
        display_value: "Hello World!".to_owned(),
        pointer_str: String::new(),
    }];

    assert_eq!(actual.lock().as_slice(), expected);
}

#[test]
fn json_tree_default_expand_none() {
    let value = json!({
      "foo": {
        "bar": {
          "fizz": true
        }
      }
    });

    let actual: Arc<Mutex<Vec<ExpectedRender>>> = Arc::new(Mutex::new(vec![]));

    egui::__run_test_ui(|ui| {
        JsonTree::new("id", &value)
            .default_expand(DefaultExpand::None)
            .on_render(|_, render_ctx| {
                actual.lock().push(render_ctx.into());
            })
            .show(ui);
    });

    let expected = vec![
        ExpectedRender {
            value: value.clone(),
            display_value: "{".to_owned(),
            pointer_str: String::new(),
        },
        ExpectedRender {
            value: json!({
              "bar": {
                "fizz": true
              }
            }),
            display_value: "foo".to_owned(),
            pointer_str: "/foo".to_owned(),
        },
        ExpectedRender {
            value: json!({
              "bar": {
                "fizz": true
              }
            }),
            display_value: "{...}".to_owned(),
            pointer_str: "/foo".to_owned(),
        },
        ExpectedRender {
            value,
            display_value: "}".to_owned(),
            pointer_str: String::new(),
        },
    ];
    assert_eq!(actual.lock().as_slice(), expected);
}

#[test]
fn json_tree_default_expand_all() {
    let value = json!({
      "foo": {
        "bar": {
          "fizz": true
        }
      }
    });

    let actual: Arc<Mutex<Vec<ExpectedRender>>> = Arc::new(Mutex::new(vec![]));

    egui::__run_test_ui(|ui| {
        JsonTree::new("id", &value)
            .default_expand(DefaultExpand::All)
            .on_render(|_, render_ctx| {
                actual.lock().push(render_ctx.into());
            })
            .show(ui);
    });

    let expected = vec![
        ExpectedRender {
            value: value.clone(),
            display_value: "{".to_owned(),
            pointer_str: String::new(),
        },
        ExpectedRender {
            value: json!({
              "bar": {
                "fizz": true
              }
            }),
            display_value: "foo".to_owned(),
            pointer_str: "/foo".to_owned(),
        },
        ExpectedRender {
            value: json!({
              "bar": {
                "fizz": true
              }
            }),
            display_value: "{".to_owned(),
            pointer_str: "/foo".to_owned(),
        },
        ExpectedRender {
            value: json!({"fizz": true}),
            display_value: "bar".to_owned(),
            pointer_str: "/foo/bar".to_owned(),
        },
        ExpectedRender {
            value: json!({"fizz": true}),
            display_value: "{".to_owned(),
            pointer_str: "/foo/bar".to_owned(),
        },
        ExpectedRender {
            value: json!(true),
            display_value: "fizz".to_owned(),
            pointer_str: "/foo/bar/fizz".to_owned(),
        },
        ExpectedRender {
            value: json!(true),
            display_value: "true".to_owned(),
            pointer_str: "/foo/bar/fizz".to_owned(),
        },
        ExpectedRender {
            value: json!({"fizz": true}),
            display_value: "}".to_owned(),
            pointer_str: "/foo/bar".to_owned(),
        },
        ExpectedRender {
            value: json!({
              "bar": {
                "fizz": true
              }
            }),
            display_value: "}".to_owned(),
            pointer_str: "/foo".to_owned(),
        },
        ExpectedRender {
            value,
            display_value: "}".to_owned(),
            pointer_str: String::new(),
        },
    ];
    assert_eq!(actual.lock().as_slice(), expected);
}

#[test]
fn json_tree_default_expand_to_level_one() {
    let value = json!({
      "foo": {
        "bar": {
          "fizz": true
        },
        "buzz": [
          { "qux": 50 }
        ]
      }
    });

    let actual: Arc<Mutex<Vec<ExpectedRender>>> = Arc::new(Mutex::new(vec![]));

    egui::__run_test_ui(|ui| {
        JsonTree::new("id", &value)
            .default_expand(DefaultExpand::ToLevel(1))
            .on_render(|_, render_ctx| {
                actual.lock().push(render_ctx.into());
            })
            .show(ui);
    });

    // Level 1 would expand the top level object and "foo", so we would
    // expect to see the keys "bar" and "buzz", but not "fizz" and "qux".
    let expected = vec![
        ExpectedRender {
            value: value.clone(),
            display_value: "{".to_owned(),
            pointer_str: String::new(),
        },
        ExpectedRender {
            value: json!({
                "bar": {
                    "fizz": true
                },
                "buzz": [
                    { "qux": 50 }
                ]
            }),
            display_value: "foo".to_owned(),
            pointer_str: "/foo".to_owned(),
        },
        ExpectedRender {
            value: json!({
                "bar": {
                    "fizz": true
                },
                "buzz": [
                    { "qux": 50 }
                ]
            }),
            display_value: "{".to_owned(),
            pointer_str: "/foo".to_owned(),
        },
        ExpectedRender {
            value: json!({"fizz": true}),
            display_value: "bar".to_owned(),
            pointer_str: "/foo/bar".to_owned(),
        },
        ExpectedRender {
            value: json!({"fizz": true}),
            display_value: "{...}".to_owned(),
            pointer_str: "/foo/bar".to_owned(),
        },
        ExpectedRender {
            value: json!([{ "qux": 50 }]),
            display_value: "buzz".to_owned(),
            pointer_str: "/foo/buzz".to_owned(),
        },
        ExpectedRender {
            value: json!([{ "qux": 50 }]),
            display_value: "[...]".to_owned(),
            pointer_str: "/foo/buzz".to_owned(),
        },
        ExpectedRender {
            value: json!({
                "bar": {
                    "fizz": true
                },
                "buzz": [
                    { "qux": 50 }
                ]
            }),
            display_value: "}".to_owned(),
            pointer_str: "/foo".to_owned(),
        },
        ExpectedRender {
            value,
            display_value: "}".to_owned(),
            pointer_str: String::new(),
        },
    ];

    assert_eq!(actual.lock().as_slice(), expected);
}

#[test]
fn json_tree_default_expand_search() {
    let value = json!({
      "foo": {
        "bar": {
          "fizz": true
        },
        "baz": {
          "qux": "thud"
        },
        "buzz": [
          { "grep": 50 }
        ]
      }
    });

    let actual: Arc<Mutex<Vec<ExpectedRender>>> = Arc::new(Mutex::new(vec![]));

    egui::__run_test_ui(|ui| {
        JsonTree::new("id", &value)
            .default_expand(DefaultExpand::SearchResults("t"))
            .on_render(|_, render_ctx| {
                actual.lock().push(render_ctx.into());
            })
            .show(ui);
    });

    let expected = vec![
        ExpectedRender {
            value: value.clone(),
            display_value: "{".to_owned(),
            pointer_str: String::new(),
        },
        ExpectedRender {
            value: json!({
                "bar": {
                    "fizz": true
                },
                "baz": {
                    "qux": "thud"
                },
                "buzz": [
                    { "grep": 50 }
                ]
            }),
            display_value: "foo".to_owned(),
            pointer_str: "/foo".to_owned(),
        },
        ExpectedRender {
            value: json!({
                "bar": {
                    "fizz": true
                },
                "baz": {
                    "qux": "thud"
                },
                "buzz": [
                    { "grep": 50 }
                ]
            }),
            display_value: "{".to_owned(),
            pointer_str: "/foo".to_owned(),
        },
        ExpectedRender {
            value: json!({"fizz": true}),
            display_value: "bar".to_owned(),
            pointer_str: "/foo/bar".to_owned(),
        },
        ExpectedRender {
            value: json!({"fizz": true}),
            display_value: "{".to_owned(),
            pointer_str: "/foo/bar".to_owned(),
        },
        ExpectedRender {
            value: json!(true),
            display_value: "fizz".to_owned(),
            pointer_str: "/foo/bar/fizz".to_owned(),
        },
        ExpectedRender {
            value: json!(true),
            display_value: "true".to_owned(),
            pointer_str: "/foo/bar/fizz".to_owned(),
        },
        ExpectedRender {
            value: json!({"fizz": true}),
            display_value: "}".to_owned(),
            pointer_str: "/foo/bar".to_owned(),
        },
        ExpectedRender {
            value: json!({"qux": "thud"}),
            display_value: "baz".to_owned(),
            pointer_str: "/foo/baz".to_owned(),
        },
        ExpectedRender {
            value: json!({"qux": "thud"}),
            display_value: "{".to_owned(),
            pointer_str: "/foo/baz".to_owned(),
        },
        ExpectedRender {
            value: json!("thud"),
            display_value: "qux".to_owned(),
            pointer_str: "/foo/baz/qux".to_owned(),
        },
        ExpectedRender {
            value: json!("thud"),
            display_value: "thud".to_owned(),
            pointer_str: "/foo/baz/qux".to_owned(),
        },
        ExpectedRender {
            value: json!({"qux": "thud"}),
            display_value: "}".to_owned(),
            pointer_str: "/foo/baz".to_owned(),
        },
        ExpectedRender {
            value: json!([{ "grep": 50 }]),
            display_value: "buzz".to_owned(),
            pointer_str: "/foo/buzz".to_owned(),
        },
        ExpectedRender {
            value: json!([{ "grep": 50 }]),
            display_value: "[...]".to_owned(),
            pointer_str: "/foo/buzz".to_owned(),
        },
        ExpectedRender {
            value: json!({
                "bar": {
                    "fizz": true
                },
                "baz": {
                    "qux": "thud"
                },
                "buzz": [
                    { "grep": 50 }
                ]
            }),
            display_value: "}".to_owned(),
            pointer_str: "/foo".to_owned(),
        },
        ExpectedRender {
            value,
            display_value: "}".to_owned(),
            pointer_str: String::new(),
        },
    ];

    assert_eq!(actual.lock().as_slice(), expected);
}

#[test]
fn json_tree_reset_expanded() {
    let value = json!({
      "baz": {
        "qux": 1
      },
      "buzz": [
        1,
      ]
    });

    // Reusing the same Context so the memory persists between multiple frames.
    let ctx = Context::default();
    ctx.set_fonts(FontDefinitions::empty());
    ctx.set_style(Style {
        animation_time: 0.0,
        ..Default::default()
    });

    let id = "id";

    let expected_all_expanded = vec![
        ExpectedRender {
            value: value.clone(),
            display_value: "{".to_owned(),
            pointer_str: String::new(),
        },
        ExpectedRender {
            value: json!({"qux": 1}),
            display_value: "baz".to_owned(),
            pointer_str: "/baz".to_owned(),
        },
        ExpectedRender {
            value: json!({"qux": 1}),
            display_value: "{".to_owned(),
            pointer_str: "/baz".to_owned(),
        },
        ExpectedRender {
            value: json!(1),
            display_value: "qux".to_owned(),
            pointer_str: "/baz/qux".to_owned(),
        },
        ExpectedRender {
            value: json!(1),
            display_value: "1".to_owned(),
            pointer_str: "/baz/qux".to_owned(),
        },
        ExpectedRender {
            value: json!({"qux": 1}),
            display_value: "}".to_owned(),
            pointer_str: "/baz".to_owned(),
        },
        ExpectedRender {
            value: json!([1]),
            display_value: "buzz".to_owned(),
            pointer_str: "/buzz".to_owned(),
        },
        ExpectedRender {
            value: json!([1]),
            display_value: "[".to_owned(),
            pointer_str: "/buzz".to_owned(),
        },
        ExpectedRender {
            value: json!(1),
            display_value: "0".to_owned(),
            pointer_str: "/buzz/0".to_owned(),
        },
        ExpectedRender {
            value: json!(1),
            display_value: "1".to_owned(),
            pointer_str: "/buzz/0".to_owned(),
        },
        ExpectedRender {
            value: json!([1]),
            display_value: "]".to_owned(),
            pointer_str: "/buzz".to_owned(),
        },
        ExpectedRender {
            value: value.clone(),
            display_value: "}".to_owned(),
            pointer_str: String::new(),
        },
    ];

    // First, render and expand everything.
    // We call `abbreviate_root` to only show "{...}" when the root object is collapsed.
    // We expect everything to be expanded as this is the first render.
    let _ = ctx.run(RawInput::default(), |ctx| {
        let mut actual: Vec<ExpectedRender> = vec![];

        CentralPanel::default().show(ctx, |ui| {
            JsonTree::new(id, &value)
                .default_expand(DefaultExpand::All)
                .style(JsonTreeStyle::new().abbreviate_root(true))
                .on_render(|_, render_ctx| {
                    actual.push(render_ctx.into());
                })
                .show(ui);
        });

        assert_eq!(actual, expected_all_expanded);
    });

    // Next we render the same tree but change the `default_expand` setting.
    // Because we already rendered the tree with everything expanded,
    // we expect everything to be expanded still.
    // Note that we call `reset_expanded` after rendering the tree.
    let _ = ctx.run(RawInput::default(), |ctx| {
        let mut actual: Vec<ExpectedRender> = vec![];

        CentralPanel::default().show(ctx, |ui| {
            JsonTree::new(id, &value)
                .default_expand(DefaultExpand::None)
                .style(JsonTreeStyle::new().abbreviate_root(true))
                .on_render(|_, render_ctx| {
                    actual.push(render_ctx.into());
                })
                .show(ui)
                .reset_expanded(ui);
        });

        assert_eq!(actual, expected_all_expanded);
    });

    // Now we render again with the same `default_expand` setting as the last render.
    // Because we called `reset_expanded` in the last frame, we now expect this setting to be respected,
    // and now nothing should be expanded.
    let _ = ctx.run(RawInput::default(), |ctx| {
        let mut actual: Vec<ExpectedRender> = vec![];

        CentralPanel::default().show(ctx, |ui| {
            JsonTree::new(id, &value)
                .default_expand(DefaultExpand::None)
                .style(JsonTreeStyle::new().abbreviate_root(true))
                .on_render(|_, render_ctx| {
                    actual.push(render_ctx.into());
                })
                .show(ui);
        });

        let expected_nothing_expanded = vec![ExpectedRender {
            value: value.clone(),
            display_value: "{...}".to_owned(),
            pointer_str: String::new(),
        }];

        assert_eq!(actual, expected_nothing_expanded);
    });
}
