use jsonc_parser::{
    cst::{CstInputValue, CstRootNode},
    ParseOptions,
};

pub fn set_key(text: &str, key: &str, value: Option<&serde_json::Value>) -> Result<String, String> {
    let root = CstRootNode::parse(text, &ParseOptions::default()).map_err(|e| e.to_string())?;
    let object = root
        .object_value()
        .ok_or("settings root must be an object")?;
    match (object.get(key), value) {
        (Some(prop), Some(value)) => prop.set_value(to_cst(value)),
        (None, Some(value)) => {
            object.append(key, to_cst(value));
        }
        (Some(prop), None) => prop.remove(),
        (None, None) => {}
    }
    Ok(root.to_string())
}

fn to_cst(value: &serde_json::Value) -> CstInputValue {
    match value {
        serde_json::Value::Null => CstInputValue::Null,
        serde_json::Value::Bool(v) => CstInputValue::Bool(*v),
        serde_json::Value::Number(v) => CstInputValue::Number(v.to_string()),
        serde_json::Value::String(v) => CstInputValue::String(v.clone()),
        serde_json::Value::Array(v) => CstInputValue::Array(v.iter().map(to_cst).collect()),
        serde_json::Value::Object(v) => {
            CstInputValue::Object(v.iter().map(|(k, v)| (k.clone(), to_cst(v))).collect())
        }
    }
}
