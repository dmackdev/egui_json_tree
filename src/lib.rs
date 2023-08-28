//! An interactive JSON tree visualisation library for `egui`, with search and highlight functionality.
//!
//! ```
//! use egui_json_tree::{DefaultExpand, JsonTree};
//!
//! # egui::__run_test_ui(|ui| {
//! let value = serde_json::json!({ "foo": "bar", "fizz": [1, 2, 3]});
//! let tree = JsonTree::new("globally-unique-id", &value);
//!
//! // Show the JSON tree:
//! let response = tree.show(ui, DefaultExpand::All);
//! # });
//! ```
//! [`JsonTree`] can visualise any type that implements [`Into`](std::convert::Into)[`<JsonTreeValue>`](value::JsonTreeValue).
//! An implementation to support [`serde_json::Value`](serde_json::Value) is provided with this crate.
//! If you wish to use a different JSON type, see the [`value`](mod@value) module,
//! and disable default features in your `Cargo.toml` if you do not need the [`serde_json`](serde_json) dependency.
//!
//! Coloring for JSON syntax highlighting and search match highlighting may be overriden through the [`JsonTree::style`] builder method.

mod delimiters;
mod response;
mod search;
mod style;
mod tree;

pub use response::JsonTreeResponse;
pub use style::JsonTreeStyle;
pub use tree::*;
pub mod value;
