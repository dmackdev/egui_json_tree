//! Representation of JSON values for presentation purposes.
//!
//! Write your own [`From`] or [`Into`] implementation which converts to [`JsonTreeValue`] if you wish to visualise a custom JSON type with a [`JsonTree`](crate::JsonTree),
//! and disable default features in your `Cargo.toml` if you do not need the [`serde_json`](serde_json) dependency.
//!
//! See the [`From<&serde_json::Value> for JsonTreeValue`](../../src/egui_json_tree/value.rs.html#39-65) implementation for reference.
/// Representation of JSON values for presentation purposes.
// #[derive(Debug, Clone, Hash)]
pub enum JsonTreeValue<'a> {
    /// Representation for a non-recursive JSON value:
    /// - A `String` representation of the base value, e.g. `"true"` for the boolean value `true`. The representation for a JSON `String` should be quoted.
    /// - The type of the base value.
    Base(String, BaseValueType),
    /// Representation for a recursive JSON value:
    /// - A `Vec` of key-value pairs. The order *must always* be the same.
    ///   - For arrays, the key should be the index of each element.
    ///   - For objects, the key should be the key of each object entry, without quotes.
    /// - The type of the recursive value, i.e. array or object.
    Expandable(Vec<(String, &'a dyn IntoJsonTreeValue)>, ExpandableType),
}

/// The type of a non-recursive JSON value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BaseValueType {
    Null,
    Bool,
    Number,
    String,
}

/// The type of a recursive JSON value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExpandableType {
    Array,
    Object,
}

pub trait IntoJsonTreeValue {
    fn into_json_tree_value(&self) -> JsonTreeValue;
    fn is_expandable(&self) -> bool;
}

#[cfg(feature = "serde_json")]
impl IntoJsonTreeValue for serde_json::Value {
    fn into_json_tree_value(&self) -> JsonTreeValue {
        match self {
            serde_json::Value::Null => JsonTreeValue::Base("null".to_string(), BaseValueType::Null),
            serde_json::Value::Bool(b) => JsonTreeValue::Base(b.to_string(), BaseValueType::Bool),
            serde_json::Value::Number(n) => {
                JsonTreeValue::Base(n.to_string(), BaseValueType::Number)
            }
            serde_json::Value::String(s) => {
                JsonTreeValue::Base(format!("\"{}\"", s), BaseValueType::String)
            }
            serde_json::Value::Array(arr) => JsonTreeValue::Expandable(
                arr.iter()
                    .enumerate()
                    .map(|(idx, elem)| (idx.to_string(), elem as &dyn IntoJsonTreeValue))
                    .collect(),
                ExpandableType::Array,
            ),
            serde_json::Value::Object(obj) => JsonTreeValue::Expandable(
                obj.iter()
                    .map(|(key, val)| (key.to_owned(), val as &dyn IntoJsonTreeValue))
                    .collect(),
                ExpandableType::Object,
            ),
        }
    }

    fn is_expandable(&self) -> bool {
        matches!(
            self,
            serde_json::Value::Array(_) | serde_json::Value::Object(_)
        )
    }
}
