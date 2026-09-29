use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum DataValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<DataValue>),
    Object(BTreeMap<String, DataValue>),
}
