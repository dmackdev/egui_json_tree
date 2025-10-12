use std::sync::LazyLock;

use egui_json_tree::{DefaultExpand, JsonTree};
use egui_kittest::Harness;

#[cfg(feature = "serde_json")]
use serde_json::{Value, json};

#[cfg(all(feature = "simd_json", not(feature = "serde_json")))]
use simd_json::{json, owned::Value};

// Keep all keys ordered within objects so rendering order is the same for both serde_json and simd_json.
static OBJECT: LazyLock<Value> = LazyLock::new(|| {
    json!({
      "bar": {
        "grep": 21,
        "qux": false,
        "thud": {
          "a/b": [
            4,
            5,
            {
              "m~n": "Greetings!"
            }
          ]
        }
      },
      "baz": null,
      "foo": [
        1,
        2,
        [
          "grep"
        ]
      ]
    })
});

#[test]
fn render_object_with_default_expand_all() {
    let mut harness = Harness::new_ui(|ui| {
        JsonTree::new("id", &*OBJECT)
            .default_expand(DefaultExpand::All)
            .show(ui);
    });
    harness.fit_contents();
    harness.snapshot("render_object_with_default_expand_all");
}

#[test]
fn render_object_with_default_expand_none() {
    let mut harness = Harness::new_ui(|ui| {
        JsonTree::new("id", &*OBJECT)
            .default_expand(DefaultExpand::None)
            .show(ui);
    });
    harness.fit_contents();
    harness.snapshot("render_object_with_default_expand_none");
}

#[test]
fn render_object_search_results() {
    // Harness::fit_contents seems to cause the tree to wrap, so set a fixed size here.
    let mut harness = Harness::builder().with_size([400., 400.]).build_ui_state(
        |ui, default_expand| {
            JsonTree::new("id", &*OBJECT)
                .default_expand(*default_expand)
                .show(ui);
        },
        DefaultExpand::SearchResults(""),
    );

    let mut snapshot_errors = vec![];

    for (idx, search_default_expand) in [
        DefaultExpand::SearchResults(""),
        DefaultExpand::SearchResults("g"),
        DefaultExpand::SearchResults("gr"),
        DefaultExpand::SearchResults("gre"),
        DefaultExpand::SearchResults("gree"),
    ]
    .into_iter()
    .enumerate()
    {
        *harness.state_mut() = search_default_expand;
        harness.run();
        let filename = format!("default_expand_search_results/{idx}_{search_default_expand:?}")
            .replace("\"", "");
        if let Err(err) = harness.try_snapshot(filename) {
            snapshot_errors.push(err);
        }
    }

    assert!(snapshot_errors.is_empty());
}

#[test]
fn render_object_with_changing_default_expand_automatically_resets_expanded() {
    // Harness::fit_contents seems to cause the tree to wrap, so set a fixed size here.
    let mut harness = Harness::builder().with_size([400., 400.]).build_ui_state(
        |ui, default_expand| {
            JsonTree::new("id", &*OBJECT)
                .default_expand(*default_expand)
                .show(ui);
        },
        DefaultExpand::None,
    );

    let mut snapshot_errors = vec![];

    for (idx, default_expand) in [
        DefaultExpand::None,
        DefaultExpand::ToLevel(2),
        DefaultExpand::SearchResults("gree"),
        DefaultExpand::All,
        DefaultExpand::SearchResultsOrAll("null"),
    ]
    .into_iter()
    .enumerate()
    {
        *harness.state_mut() = default_expand;
        harness.run();
        let filename =
            format!("changing_default_expand/{idx}_{default_expand:?}").replace("\"", "");
        if let Err(err) = harness.try_snapshot(filename) {
            snapshot_errors.push(err);
        }
    }

    assert!(snapshot_errors.is_empty());
}

#[test]
fn render_object_with_default_expand_to_levels() {
    // Harness::fit_contents seems to cause the tree to wrap, so set a fixed size here.
    let mut harness = Harness::builder().with_size([400., 400.]).build_ui_state(
        |ui, level| {
            JsonTree::new("id", &*OBJECT)
                .default_expand(DefaultExpand::ToLevel(*level))
                .show(ui);
        },
        0,
    );
    let mut snapshot_errors = vec![];

    for level in 0..=4 {
        *harness.state_mut() = level;
        harness.run();
        if let Err(err) = harness.try_snapshot(format!("default_expand_to_level/{level}")) {
            snapshot_errors.push(err);
        }
    }

    assert!(snapshot_errors.is_empty());
}

#[test]
fn render_object_with_egui_light_theme_should_style_tree_with_light_theme() {
    let mut harness = Harness::new_ui(|ui| {
        ui.ctx().set_theme(egui::Theme::Light);
        JsonTree::new("id", &*OBJECT)
            .default_expand(DefaultExpand::All)
            .show(ui);
    });
    harness.fit_contents();
    harness.snapshot("light_theme");
}
