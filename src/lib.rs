//! An interactive JSON tree visualiser for `egui`, with search and highlight functionality.
//!
//! ```
//! use egui::{Color32};
//! use egui_json_tree::{DefaultExpand, JsonTree, JsonTreeStyle};
//!
//! # egui::__run_test_ui(|ui| {
//! let value = serde_json::json!({ "foo": "bar", "fizz": [1, 2, 3]});
//!
//! let response = JsonTree::new("globally-unique-id", &value)
//!     .style(JsonTreeStyle {
//!         bool_color: Color32::YELLOW,
//!         ..Default::default()
//!     })
//!     .default_expand(DefaultExpand::All)
//!     .response_callback(|response, json_pointer_str| {
//!       // Handle interactions within the JsonTree.
//!     })
//!     .show(ui);
//!
//! // Reset the expanded state of all arrays/objects to respect the `default_expand` setting.
//! response.reset_expanded(ui);
//! # });
//! ```
//! [`JsonTree`] can visualise any type that implements [`Into`](std::convert::Into)[`<JsonTreeValue>`](value::JsonTreeValue).
//! An implementation to support [`serde_json::Value`](serde_json::Value) is provided with this crate.
//! If you wish to use a different JSON type, see the [`value`](mod@value) module,
//! and disable default features in your `Cargo.toml` if you do not need the [`serde_json`](serde_json) dependency.
mod default_expand;
mod delimiters;
mod node;
mod response;
mod search;
mod style;
mod tree;

pub use response::JsonTreeResponse;
pub use style::JsonTreeStyle;
pub mod value;
pub use default_expand::DefaultExpand;
pub use tree::JsonTree;
