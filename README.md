# egui_json_tree

[![Latest version](https://img.shields.io/crates/v/egui_json_tree.svg)](https://crates.io/crates/egui_json_tree)
[![Documentation](https://docs.rs/egui_json_tree/badge.svg)](https://docs.rs/egui_json_tree)
[![Build Status](https://github.com/dmackdev/egui_json_tree/workflows/CI/badge.svg)](https://github.com/dmackdev/egui_json_tree/actions?workflow=CI)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/dmackdev/egui_json_tree/blob/master/LICENSE-MIT)
[![Apache](https://img.shields.io/badge/license-Apache-blue.svg)](https://github.com/dmackdev/egui_json_tree/blob/master/LICENSE-APACHE)

An interactive JSON tree visualiser for `egui`, with search and highlight functionality.

<p align="center">
  <img src="./media/search_example.gif" alt="Search Example"/>
</p>

## Usage

```rust
use egui::{Color32};
use egui_json_tree::{DefaultExpand, JsonTree, JsonTreeStyle};

let value = serde_json::json!({ "foo": "bar", "fizz": [1, 2, 3]});

// Simple:
JsonTree::new("simple-tree", &value).show(ui);

// Customised:
let response = JsonTree::new("customised-tree", &value)
    .style(JsonTreeStyle {
        bool_color: Color32::YELLOW,
        ..Default::default()
    })
    .default_expand(DefaultExpand::All)
    .response_callback(|response, json_pointer_string| {
      // Handle interactions within the JsonTree.
    })
    .abbreviate_root(true) // Show {...} when the root object is collapsed.
    .show(ui);

// Reset the expanded state of all arrays/objects to respect the `default_expand` setting.
response.reset_expanded(ui);
```

See [demo.rs](./examples/demo.rs) and run the examples for more detailed use cases, including the search match highlight/auto expand functionality, and how to copy JSON paths and values to the clipboard.

`JsonTree` can visualise any type that implements `value::ToJsonTreeValue`. An implementation to support `serde_json::Value` is provided with this crate. If you wish to use a different JSON type, see the `value` module, and disable default features in your `Cargo.toml` if you do not need the `serde_json` dependency.

## Run Examples

```bash
cargo run --example=demo
```
