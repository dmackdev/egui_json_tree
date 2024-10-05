//! An interactive JSON tree visualiser for `egui`, with search and highlight functionality.
//!
//! # Usage
//! ```rust
//! # use egui::{Color32};
//! # use egui_json_tree::{
//! #   render::{
//! #       DefaultRender, RenderBaseValueContext, RenderContext, RenderExpandableDelimiterContext,
//! #       RenderPropertyContext,
//! #   },
//! #   DefaultExpand, JsonTree, JsonTreeStyle, ToggleButtonsState
//! # };
//! # egui::__run_test_ui(|ui| {
//! let value = serde_json::json!({ "foo": "bar", "fizz": [1, 2, 3]});
//!
//! // Simple:
//! JsonTree::new("simple-tree", &value).show(ui);
//!
//! // Customised:
//! let response = JsonTree::new("customised-tree", &value)
//!     .style(JsonTreeStyle {
//!         bool_color: Color32::YELLOW,
//!         ..Default::default()
//!     })
//!     .default_expand(DefaultExpand::All)
//!     .abbreviate_root(true) // Show {...} when the root object is collapsed.
//!     .toggle_buttons_state(ToggleButtonsState::VisibleDisabled)
//!     .on_render(|ui, ctx| {
//!         // Customise rendering of the JsonTree, and/or handle interactions.
//!         match ctx {
//!             RenderContext::Property(ctx) => {
//!                 ctx.render_default(ui).context_menu(|ui| {
//!                     // Show a context menu when right clicking
//!                     // an array index or object key.
//!                 });
//!             }
//!             RenderContext::BaseValue(ctx) => {
//!                 // Show a button after non-recursive JSON values.
//!                 ctx.render_default(ui);
//!                 if ui.small_button("+").clicked() {
//!                     // ...
//!                 }
//!             }
//!             RenderContext::ExpandableDelimiter(ctx) => {
//!                 // Render array brackets and object braces as normal.
//!                 ctx.render_default(ui);
//!             }
//!         };
//!     })
//!     .show(ui);
//!
//! // Reset the expanded state of all arrays/objects to respect the `default_expand` setting.
//! response.reset_expanded(ui);
//! # });
//! ```
//!
//! See [`examples/demo.rs`](https://github.com/dmackdev/egui_json_tree/blob/master/examples/demo.rs)
//! and run the examples for more detailed use cases, including:
//! - Automatic expansion of arrays/objects and highlighting, based on search term matches.
//! - Copying JSON paths and values to the clipboard.
//! - A JSON editor UI.
//!
//! # Supported JSON Types
//!
//! [`JsonTree`] can visualise any type that implements [`ToJsonTreeValue`](trait@value::ToJsonTreeValue).
//! See the table of crate features below for provided implementations.
//!
//! | Feature/Dependency | JSON Type                 | Default |
//! | ------------------ | ------------------------- | ------- |
//! | `serde_json`       | `serde_json::Value`       | Yes     |
//! | `simd_json`        | `simd_json::owned::Value` | No      |
//!
//! If you wish to use a different JSON type, see the [`value`](mod@value) module,
//! and disable default features in your `Cargo.toml` if you do not need the `serde_json` dependency.
mod default_expand;
mod node;
mod response;
mod search;
mod style;
mod toggle_buttons_state;
mod tree;

pub mod delimiters;
pub mod pointer;
pub mod render;
pub mod value;

pub use default_expand::DefaultExpand;
pub use response::JsonTreeResponse;
pub use style::JsonTreeStyle;
pub use toggle_buttons_state::ToggleButtonsState;
pub use tree::JsonTree;
