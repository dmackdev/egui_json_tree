use indexmap::IndexMap;

#[derive(Debug, Clone)]
pub enum JsonTreeValue {
    BaseValue(BaseValue),
    Array(Vec<JsonTreeValue>),
    Object(IndexMap<String, JsonTreeValue>),
}

#[derive(Debug, Clone)]
pub struct BaseValue {
    pub value_str: String,
    pub value_type: BaseValueType,
}

#[derive(Debug, Clone, Copy)]
pub enum BaseValueType {
    Null,
    Bool,
    Number,
    String,
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
            serde_json::Value::Array(arr) => {
                JsonTreeValue::Array(arr.iter().map(|elem| elem.into()).collect())
            }
            serde_json::Value::Object(obj) => JsonTreeValue::Object(IndexMap::from_iter(
                obj.iter().map(|(key, val)| (key.to_owned(), val.into())),
            )),
        }
    }
}
