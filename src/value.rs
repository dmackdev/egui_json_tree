//! Internal representation of JSON values for presentation purposes.
//!
//! Implement your own `From` or `Into` conversion to [`JsonTreeValue`] if you wish to visualise a custom JSON type with a [`JsonTree`](crate::JsonTree).
//! See the [`From<&serde_json::Value> for JsonTreeValue`](../../src/egui_json_tree/value.rs.html#34-68) implementation for reference.
#[derive(Debug, Clone)]
pub enum JsonTreeValue {
    BaseValue(BaseValue),
    Expandable(Vec<(String, JsonTreeValue)>, ExpandableType),
}

/// A representation for a non-recursive JSON value.
#[derive(Debug, Clone)]
pub struct BaseValue {
    /// The string representation for this base value.
    pub value_str: String,
    pub value_type: BaseValueType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseValueType {
    Null,
    Bool,
    Number,
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpandableType {
    Array,
    Object,
}

#[cfg(feature = "serde_json")]
impl From<&serde_json::Value> for JsonTreeValue {
    fn from(value: &serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => JsonTreeValue::BaseValue(BaseValue {
                value_str: "null".to_string(),
                value_type: BaseValueType::Null,
            }),
            serde_json::Value::Bool(b) => JsonTreeValue::BaseValue(BaseValue {
                value_str: b.to_string(),
                value_type: BaseValueType::Bool,
            }),
            serde_json::Value::Number(n) => JsonTreeValue::BaseValue(BaseValue {
                value_str: n.to_string(),
                value_type: BaseValueType::Number,
            }),
            serde_json::Value::String(s) => JsonTreeValue::BaseValue(BaseValue {
                value_str: format!("\"{}\"", s),
                value_type: BaseValueType::String,
            }),
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
