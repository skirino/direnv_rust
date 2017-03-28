use std::collections::btree_map::BTreeMap;
use serde_json;

pub type VarsMap = BTreeMap<String, Option<String>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub dir    : String,
    pub allowed: bool,
    pub before : VarsMap,
}
pub type Stack = Vec<Entry>;

pub fn decode(s: String) -> Stack {
    match serde_json::from_str(&s).unwrap() {
        serde_json::Value::Array(v) => v.into_iter().map(|e| serde_json::from_value(e).unwrap()).collect::<Stack>(),
        _                           => panic!("Unexpected JSON!"),
    }
}

pub fn encode(stack: &[Entry]) -> String {
    serde_json::to_string(&stack).unwrap()
}
