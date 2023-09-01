# egui_json_tree

An interactive JSON tree visualiser for `egui`, with search and highlight functionality.

<p align="center">
  <img src="./media/search_example.gif" alt="Search Example"/>
</p>

## Usage

```rust
use egui::{Color32};
use egui_json_tree::{DefaultExpand, JsonTree, JsonTreeStyle};

let value = serde_json::json!({ "foo": "bar", "fizz": [1, 2, 3]});

let response = JsonTree::new("globally-unique-id", &value)
    .style(JsonTreeStyle {
        bool_color: Color32::YELLOW,
        ..Default::default()
    })
    .default_expand(DefaultExpand::All)
    .response_callback(|response, json_pointer_str| {
      // Handle interactions within the JsonTree.
    })
    .show(ui);

// Reset the expanded state of all arrays/objects to respect the `default_expand` setting.
response.reset_expanded(ui);
```

See [demo.rs](./examples/demo.rs) and run the examples for more detailed use cases, including the search match highlight/auto expand functionality, and how to copy JSON paths and values to the clipboard.

`JsonTree` can visualise any type that implements `Into<JsonTreeValue>`. An implementation to support `serde_json::Value` is provided with this crate. If you wish to use a different JSON type, see the `value` module, and disable default features in your `Cargo.toml` if you do not need the `serde_json` dependency.

## Run Examples

```bash
cargo run --example=demo
```

## Open Docs

```bash
cargo doc --no-deps --open
```

## Notes

This crate currently depends on an unpublished version of `egui` from the master branch, in order to expose the ability to reset the expanded state of arrays/objects in a `JsonTree`. On the next `egui` release, this crate will update its `egui` dependency to the latest published version, and this crate will be published to `crates.io`.
