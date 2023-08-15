//! An interactive JSON tree visualiser for `egui`, with search and highlight functionality.

mod delimiters;
mod search;
mod style;
mod tree;

pub use style::JsonTreeStyle;
pub use tree::*;
