use egui_json_tree::{DefaultExpand, JsonTree};
use serde_json::json;

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
            .response_callback(|_, pointer| {
                rendered_pointers.push(pointer.to_string());
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
            .response_callback(|_, pointer| {
                rendered_pointers.push(pointer.to_string());
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
          1,
          2
        ]
      }
    });

    let mut rendered_pointers = vec![];

    egui::__run_test_ui(|ui| {
        JsonTree::new("id", &obj)
            .response_callback(|_, pointer| {
                rendered_pointers.push(pointer.to_string());
            })
            .default_expand(DefaultExpand::ToLevel(1))
            .show(ui);
    });

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
          1,
          2
        ]
      }
    });

    let mut rendered_pointers = vec![];

    egui::__run_test_ui(|ui| {
        JsonTree::new("id", &obj)
            .response_callback(|_, pointer| {
                rendered_pointers.push(pointer.to_string());
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
