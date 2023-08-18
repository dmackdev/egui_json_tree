#[derive(Debug, Clone)]
pub enum JsonTreeValue {
    BaseValue(BaseValue),
    Expandable(Vec<(String, JsonTreeValue)>, ExpandableType),
}

#[derive(Debug, Clone)]
pub struct BaseValue {
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
