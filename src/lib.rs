//! An interactive JSON tree visualiser for `egui`, with search and highlight functionality.
//!
//! By default, [`serde_json::Value`](serde_json::Value) is supported for visualisation.
//! If you wish to use a different JSON type, disable default features in your `Cargo.toml` to not include the [`serde_json`](serde_json) dependency,
//! and see the [`value`](mod@value) module.

mod delimiters;
mod response;
mod search;
mod style;
mod tree;

pub use response::JsonTreeResponse;
pub use style::JsonTreeStyle;
pub use tree::*;
pub mod value;
