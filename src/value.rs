//! Representation of JSON values for presentation purposes.
//!
//! Write your own [`ToJsonTreeValue`] implementation which converts to [`JsonTreeValue`] if you wish to visualise a custom JSON type with a [`JsonTree`](crate::JsonTree),
//! and disable default features in your `Cargo.toml` if you do not need the [`serde_json`] dependency.
//!
//! For reference, see the provided [`ToJsonTreeValue`] implementations in [`value.rs`](../../src/egui_json_tree/value.rs.html) for the following JSON types:
//! - `serde_json::Value`
//! - `simd_json::owned::Value`

use std::fmt::Display;

use crate::pointer::JsonPointerSegment;
/// Representation of JSON values for presentation purposes.
pub enum JsonTreeValue<'a, T: ?Sized> {
    /// Representation for a non-recursive JSON value:
    /// - A reference to the actual JSON value itself.
    /// - A reference to a value that visually represents the JSON value.
    /// - The type of the base value.
    Base(&'a T, &'a dyn Display, BaseValueType),
    /// Representation for a recursive JSON value:
    /// - A `Vec` of property-value pairs. The order *must always* be the same.
    ///   - For arrays, the property should be the index of each element.
    ///   - For objects, the property should be the key of each object entry, without quotes.
    /// - The type of the recursive value, i.e. array or object.
    Expandable(Vec<(JsonPointerSegment<'a>, &'a T)>, ExpandableType),
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

/// A trait for types that can be converted to a [`JsonTreeValue`].
pub trait ToJsonTreeValue {
    /// Converts this JSON value to a [`JsonTreeValue`].
    fn to_json_tree_value(&self) -> JsonTreeValue<'_, Self>;
    /// Returns whether this JSON value is expandable, i.e. whether it is an object or an array.
    fn is_expandable(&self) -> bool;
}

#[cfg(feature = "serde_json")]
impl ToJsonTreeValue for serde_json::Value {
    fn to_json_tree_value(&self) -> JsonTreeValue<'_, Self> {
        match self {
            Self::Null => JsonTreeValue::Base(self, self, BaseValueType::Null),
            Self::Bool(b) => JsonTreeValue::Base(self, b, BaseValueType::Bool),
            Self::Number(n) => JsonTreeValue::Base(self, n, BaseValueType::Number),
            Self::String(s) => JsonTreeValue::Base(self, s, BaseValueType::String),
            Self::Array(arr) => JsonTreeValue::Expandable(
                arr.iter()
                    .enumerate()
                    .map(|(idx, elem)| (JsonPointerSegment::Index(idx), elem))
                    .collect(),
                ExpandableType::Array,
            ),
            Self::Object(obj) => JsonTreeValue::Expandable(
                obj.iter()
                    .map(|(key, val)| (JsonPointerSegment::Key(key), val))
                    .collect(),
                ExpandableType::Object,
            ),
        }
    }

    fn is_expandable(&self) -> bool {
        matches!(self, Self::Array(_) | Self::Object(_))
    }
}

#[cfg(feature = "simd_json")]
impl ToJsonTreeValue for simd_json::owned::Value {
    fn to_json_tree_value(&self) -> JsonTreeValue<'_, Self> {
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
                    .map(|(idx, elem)| (JsonPointerSegment::Index(idx), elem))
                    .collect(),
                ExpandableType::Array,
            ),
            simd_json::OwnedValue::Object(obj) => JsonTreeValue::Expandable(
                obj.iter()
                    .map(|(key, val)| (JsonPointerSegment::Key(key), val))
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
