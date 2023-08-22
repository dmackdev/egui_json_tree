# egui_json_tree

An interactive JSON tree visualiser for egui, with search and highlight functionality.

## Example

```rust
let value = serde_json::json!({ "foo": "bar", "fizz": [1, 2, 3]});
let tree = JsonTree::new("globally-unique-id", &value);

// Show the JSON tree:
let response = tree.show(ui, DefaultExpand::All);

// Reset which arrays and objects are expanded to respect the `default_expand` argument on the next render.
// In this case, this will expand all arrays and objects again,
// if a user had collapsed any manually.
response.reset_expanded(ui);
```

See [demo.rs](./examples/demo.rs) for more detailed examples.

## Run Examples

```bash
cargo run --example=demo
```
