//! Representation of JSON values for presentation purposes.
//!
//! Write your own [`ToJsonTreeValue`] implementation which converts to [`JsonTreeValue`] if you wish to visualise a custom JSON type with a [`JsonTree`](crate::JsonTree),
//! and disable default features in your `Cargo.toml` if you do not need the [`serde_json`](serde_json) dependency.
//!
//! See the [`impl ToJsonTreeValue for serde_json::Value `](../../src/egui_json_tree/value.rs.html#43-77) implementation for reference.

use std::fmt::Display;

use crate::pointer::NestedProperty;
/// Representation of JSON values for presentation purposes.
pub enum JsonTreeValue<'a, T: ?Sized> {
    /// Representation for a non-recursive JSON value:
    /// - A reference to the actual JSON value itself.
    /// - A reference to a value that visually represents the JSON value.
    /// - The type of the base value.
    Base(&'a T, &'a dyn Display, BaseValueType),
    /// Representation for a recursive JSON value:
    /// - A `Vec` of key-value pairs. The order *must always* be the same.
    ///   - For arrays, the key should be the index of each element.
    ///   - For objects, the key should be the key of each object entry, without quotes.
    /// - The type of the recursive value, i.e. array or object.
    Expandable(Vec<(NestedProperty<'a>, &'a T)>, ExpandableType),
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

pub trait ToJsonTreeValue: Display {
    fn to_json_tree_value(&self) -> JsonTreeValue<Self>;
    fn is_expandable(&self) -> bool;
}

#[cfg(feature = "serde_json")]
impl ToJsonTreeValue for serde_json::Value {
    fn to_json_tree_value(&self) -> JsonTreeValue<Self> {
        match self {
            serde_json::Value::Null => JsonTreeValue::Base(self, self, BaseValueType::Null),
            serde_json::Value::Bool(b) => JsonTreeValue::Base(self, b, BaseValueType::Bool),
            serde_json::Value::Number(n) => JsonTreeValue::Base(self, n, BaseValueType::Number),
            serde_json::Value::String(s) => JsonTreeValue::Base(self, s, BaseValueType::String),
            serde_json::Value::Array(arr) => JsonTreeValue::Expandable(
                arr.iter()
                    .enumerate()
                    .map(|(idx, elem)| (NestedProperty::Index(idx), elem))
                    .collect(),
                ExpandableType::Array,
            ),
            serde_json::Value::Object(obj) => JsonTreeValue::Expandable(
                obj.iter()
                    .map(|(key, val)| (NestedProperty::Key(key), val))
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

#[cfg(feature = "simd_json")]
impl ToJsonTreeValue for simd_json::owned::Value {
    fn to_json_tree_value(&self) -> JsonTreeValue<Self> {
        match self {
            simd_json::OwnedValue::Static(s) => match s {
                simd_json::StaticNode::I64(n) => {
                    JsonTreeValue::Base(self, n, BaseValueType::Number)
                }
                simd_json::StaticNode::U64(n) => {
                    JsonTreeValue::Base(self, n, BaseValueType::Number)
                }
                simd_json::StaticNode::F64(n) => {
                    JsonTreeValue::Base(self, n, BaseValueType::Number)
                }
                simd_json::StaticNode::Bool(b) => JsonTreeValue::Base(self, b, BaseValueType::Bool),
                simd_json::StaticNode::Null => JsonTreeValue::Base(self, self, BaseValueType::Null),
            },
            simd_json::OwnedValue::String(s) => JsonTreeValue::Base(self, s, BaseValueType::String),
            simd_json::OwnedValue::Array(arr) => JsonTreeValue::Expandable(
                arr.iter()
                    .enumerate()
                    .map(|(idx, elem)| (NestedProperty::Index(idx), elem))
                    .collect(),
                ExpandableType::Array,
            ),
            simd_json::OwnedValue::Object(obj) => JsonTreeValue::Expandable(
                obj.iter()
                    .map(|(key, val)| (NestedProperty::Key(key), val))
                    .collect(),
                ExpandableType::Object,
            ),
        }
    }

    fn is_expandable(&self) -> bool {
        matches!(
            self,
            simd_json::owned::Value::Array(_) | simd_json::owned::Value::Object(_)
        )
    }
}
