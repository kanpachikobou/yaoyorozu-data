use std::collections::BTreeMap;

use crate::{DataError, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum DataValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<DataValue>),
    Object(BTreeMap<String, DataValue>),
}

impl DataValue {
    pub fn object() -> Self {
        Self::Object(BTreeMap::new())
    }

    pub fn array() -> Self {
        Self::Array(Vec::new())
    }

    pub fn get(&self, key: &str) -> Option<&DataValue> {
        match self {
            Self::Object(object) => object.get(key),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut DataValue> {
        match self {
            Self::Object(object) => object.get_mut(key),
            _ => None,
        }
    }

    pub fn set(&mut self, key: impl Into<String>, value: DataValue) -> Result<()> {
        match self {
            Self::Object(object) => {
                object.insert(key.into(), value);
                Ok(())
            }
            _ => Err(DataError::Message(
                "値を設定できるのはオブジェクトです。".to_string(),
            )),
        }
    }

    pub fn remove(&mut self, key: &str) -> Option<DataValue> {
        match self {
            Self::Object(object) => object.remove(key),
            _ => None,
        }
    }

    pub fn push(&mut self, value: DataValue) -> Result<()> {
        match self {
            Self::Array(array) => {
                array.push(value);
                Ok(())
            }
            _ => Err(DataError::Message(
                "値を追加できるのは配列です。".to_string(),
            )),
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_values_can_be_read_and_changed() {
        let mut data = DataValue::object();

        data.set("name", DataValue::String("狐".to_string())).unwrap();
        data.set("hp", DataValue::Number(100.0)).unwrap();

        assert_eq!(data.get("name").and_then(DataValue::as_str), Some("狐"));
        assert_eq!(data.get("hp").and_then(DataValue::as_f64), Some(100.0));

        data.remove("hp");
        assert!(data.get("hp").is_none());
    }

    #[test]
    fn array_values_can_be_appended() {
        let mut data = DataValue::array();

        data.push(DataValue::String("薬草".to_string())).unwrap();
        data.push(DataValue::String("木刀".to_string())).unwrap();

        assert_eq!(
            data,
            DataValue::Array(vec![
                DataValue::String("薬草".to_string()),
                DataValue::String("木刀".to_string())
            ])
        );
    }

    #[test]
    fn wrong_container_returns_error() {
        let mut data = DataValue::String("狐".to_string());

        assert!(data.set("name", DataValue::Null).is_err());
        assert!(data.push(DataValue::Null).is_err());
    }
}
