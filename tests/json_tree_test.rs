use egui::{CentralPanel, Context, FontDefinitions, Style};
use egui_json_tree::{
    pointer::ToJsonPointerString, render::ResponseContext, DefaultExpand, JsonTree,
};
use serde_json::json;

#[test]
fn json_tree_response_callback_for_rendering_string() {
    let value = json!("Hello World!");

    let mut rendered_pointers = vec![];

    egui::__run_test_ui(|ui| {
        JsonTree::new("id", &value)
            .on_response(|ResponseContext { pointer, .. }| {
                rendered_pointers.push(pointer.to_json_pointer_string());
            })
            .show(ui);
    });

    let expected_pointers = vec![""];
    assert_eq!(expected_pointers, rendered_pointers);
}

#[test]
fn json_tree_default_expand_none() {
    let obj = json!({
      "foo": {
        "bar": {
          "fizz": true
        }
      }
    });

    let mut rendered_pointers = vec![];

    egui::__run_test_ui(|ui| {
        JsonTree::new("id", &obj)
            .on_response(|ResponseContext { pointer, .. }| {
                rendered_pointers.push(pointer.to_json_pointer_string());
            })
            .default_expand(DefaultExpand::None)
            .show(ui);
    });

    let expected_pointers = vec!["", ""];
    assert_eq!(expected_pointers, rendered_pointers);
}

#[test]
fn json_tree_default_expand_all() {
    let obj = json!({
      "foo": {
        "bar": {
          "fizz": true
        }
      }
    });

    let mut rendered_pointers = vec![];

    egui::__run_test_ui(|ui| {
        JsonTree::new("id", &obj)
            .on_response(|ResponseContext { pointer, .. }| {
                rendered_pointers.push(pointer.to_json_pointer_string());
            })
            .default_expand(DefaultExpand::All)
            .show(ui);
    });

    let expected_pointers = vec!["/foo", "/foo/bar", "/foo/bar/fizz", "/foo/bar/fizz"];
    assert_eq!(expected_pointers, rendered_pointers);
}

#[test]
fn json_tree_default_expand_to_level_one() {
    let obj = json!({
      "foo": {
        "bar": {
          "fizz": true
        },
        "buzz": [
          { "qux": 50 }
        ]
      }
    });

    let mut rendered_pointers = vec![];

    egui::__run_test_ui(|ui| {
        JsonTree::new("id", &obj)
            .on_response(|ResponseContext { pointer, .. }| {
                rendered_pointers.push(pointer.to_json_pointer_string());
            })
            .default_expand(DefaultExpand::ToLevel(1))
            .show(ui);
    });

    // Level 1 would expand the top level object and "foo", so we would
    // expect to see the keys "bar" and "buzz", but not "fizz" and "qux"
    let expected_pointers = vec!["/foo", "/foo/bar", "/foo/bar", "/foo/buzz", "/foo/buzz"];
    assert_eq!(expected_pointers, rendered_pointers);
}

#[test]
fn json_tree_default_expand_search() {
    let obj = json!({
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

    let mut rendered_pointers = vec![];

    egui::__run_test_ui(|ui| {
        JsonTree::new("id", &obj)
            .on_response(|ResponseContext { pointer, .. }| {
                rendered_pointers.push(pointer.to_json_pointer_string());
            })
            .default_expand(DefaultExpand::SearchResults("t"))
            .show(ui);
    });

    let expected_pointers = vec![
        "/foo",
        "/foo/bar",
        "/foo/bar/fizz",
        "/foo/bar/fizz",
        "/foo/baz",
        "/foo/baz/qux",
        "/foo/baz/qux",
        "/foo/buzz",
        "/foo/buzz",
    ];
    assert_eq!(expected_pointers, rendered_pointers);
}

#[test]
fn json_tree_reset_expanded() {
    let obj = json!({
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

    let expected_pointers_all = vec![
        "/baz", "/baz/qux", "/baz/qux", "/buzz", "/buzz/0", "/buzz/0",
    ];

    let id = "id";

    // First, render and expand everything.
    // We expect everything to be expanded as this is the first render.
    let _ = ctx.run(Default::default(), |ctx| {
        let mut rendered_pointers = vec![];

        CentralPanel::default().show(ctx, |ui| {
            JsonTree::new(id, &obj)
                .on_response(|ResponseContext { pointer, .. }| {
                    rendered_pointers.push(pointer.to_json_pointer_string());
                })
                .default_expand(DefaultExpand::All)
                .show(ui);
        });

        assert_eq!(expected_pointers_all, rendered_pointers);
    });

    // Next we render the same tree but change the `default_expand` setting.
    // Because we already rendered the tree with everything expanded,
    // we expect everything to be expanded still.
    // Note that we call `reset_expanded` after rendering the tree.
    let _ = ctx.run(Default::default(), |ctx| {
        let mut rendered_pointers = vec![];

        CentralPanel::default().show(ctx, |ui| {
            JsonTree::new(id, &obj)
                .on_response(|ResponseContext { pointer, .. }| {
                    rendered_pointers.push(pointer.to_json_pointer_string());
                })
                .default_expand(DefaultExpand::None)
                .show(ui)
                .reset_expanded(ui);
        });

        assert_eq!(expected_pointers_all, rendered_pointers);
    });

    // Now we render again with the same `default_expand` setting as the last render.
    // Because we called `reset_expanded` in the last frame, we now expect this setting to be respected,
    // and now nothing should be expanded.
    let _ = ctx.run(Default::default(), |ctx| {
        let mut rendered_pointers = vec![];

        CentralPanel::default().show(ctx, |ui| {
            JsonTree::new(id, &obj)
                .on_response(|ResponseContext { pointer, .. }| {
                    rendered_pointers.push(pointer.to_json_pointer_string());
                })
                .default_expand(DefaultExpand::None)
                .show(ui);
        });

        assert_eq!(vec!["", "", "", ""], rendered_pointers);
    });
}
