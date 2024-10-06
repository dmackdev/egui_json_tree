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
    .abbreviate_root(true) // Show {...} when the root object is collapsed.
    .toggle_buttons_state(ToggleButtonsState::VisibleDisabled)
    .on_render(|ui, ctx| {
        // Customise rendering of the JsonTree, and/or handle interactions.
        match ctx {
            RenderContext::Property(ctx) => {
                ctx.render_default(ui).context_menu(|ui| {
                    // Show a context menu when right clicking
                    // an array index or object key.
                });
            }
            RenderContext::BaseValue(ctx) => {
                // Show a button after non-recursive JSON values.
                ctx.render_default(ui);
                if ui.small_button("+").clicked() {
                    // ...
                }
            }
            RenderContext::ExpandableDelimiter(ctx) => {
                // Render array brackets and object braces as normal.
                ctx.render_default(ui);
            }
        };
    })
    .show(ui);

// Reset the expanded state of all arrays/objects to respect the `default_expand` setting.
response.reset_expanded(ui);
```

See [demo.rs](./examples/demo.rs) and run the examples for more detailed use cases, including:

- Automatic expansion of arrays/objects and highlighting, based on search term matches.
- Copying JSON paths and values to the clipboard.
- A JSON editor UI.

## Supported JSON Types

`JsonTree` can visualise any type that implements `value::ToJsonTreeValue`.

See the table of crate features below for provided implementations.

| Feature/Dependency | JSON Type                 | Default |
| ------------------ | ------------------------- | ------- |
| `serde_json`       | `serde_json::Value`       | Yes     |
| `simd_json`        | `simd_json::owned::Value` | No      |

If you wish to use a different JSON type, see the `value` module, and disable default features in your `Cargo.toml` if you do not need the `serde_json` dependency.

## Run Examples

```bash
cargo run --example=demo
```
