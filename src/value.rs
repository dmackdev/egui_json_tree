//! Representation of JSON values for presentation purposes.
//!
//! Implement your own `From` or `Into` conversion to [`JsonTreeValue`] if you wish to visualise a custom JSON type with a [`JsonTree`](crate::JsonTree).
//! See the [`From<&serde_json::Value> for JsonTreeValue`](../../src/egui_json_tree/value.rs.html#37-63) implementation for reference.
/// Representation of JSON values for presentation purposes.
#[derive(Debug, Clone)]
pub enum JsonTreeValue {
    /// Representation for a non-recursive JSON value:
    /// - A `String` representation of the base value.
    /// - The type of the base value.
    Base(String, BaseValueType),
    /// Representation for a recursive JSON value:
    /// - A `Vec` of key-value pairs. The order *must* always be the same.
    ///   - For arrays, the key should be the index of each element.
    ///   - For objects, the key should be the key of each object entry, in quotes.
    /// - The type of the recursive value, i.e. array or object.
    Expandable(Vec<(String, JsonTreeValue)>, ExpandableType),
}

/// The type of a non-recursive JSON value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseValueType {
    Null,
    Bool,
    Number,
    String,
}

/// The type of a recursive JSON value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpandableType {
    Array,
    Object,
}

#[cfg(feature = "serde_json")]
impl From<&serde_json::Value> for JsonTreeValue {
    fn from(value: &serde_json::Value) -> Self {
        match value {
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
                    .map(|(idx, elem)| (idx.to_string(), elem.into()))
                    .collect(),
                ExpandableType::Array,
            ),
            serde_json::Value::Object(obj) => JsonTreeValue::Expandable(
                obj.iter()
                    .map(|(key, val)| (key.to_owned(), val.into()))
                    .collect(),
                ExpandableType::Object,
            ),
        }
    }
}
