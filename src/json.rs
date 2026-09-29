use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::{DataError, DataValue, Result};

/// JSON文字列を DataValue に変換する。
pub fn from_str(input: &str) -> Result<DataValue> {
    let value: Value =
        serde_json::from_str(input).map_err(|error| DataError::Message(error.to_string()))?;

    from_json_value(value)
}

/// DataValue をJSON文字列へ変換する。
pub fn to_string(value: &DataValue) -> Result<String> {
    let json_value = to_json_value(value)?;
    serde_json::to_string(&json_value).map_err(|error| DataError::Message(error.to_string()))
}

/// DataValue を読みやすい整形済みJSONへ変換する。
pub fn to_string_pretty(value: &DataValue) -> Result<String> {
    let json_value = to_json_value(value)?;
    serde_json::to_string_pretty(&json_value)
        .map_err(|error| DataError::Message(error.to_string()))
}

fn from_json_value(value: Value) -> Result<DataValue> {
    match value {
        Value::Null => Ok(DataValue::Null),
        Value::Bool(value) => Ok(DataValue::Bool(value)),
        Value::Number(value) => value
            .as_f64()
            .map(DataValue::Number)
            .ok_or_else(|| DataError::Message("JSON numberをf64へ変換できません。".to_string())),
        Value::String(value) => Ok(DataValue::String(value)),
        Value::Array(values) => values
            .into_iter()
            .map(from_json_value)
            .collect::<Result<Vec<_>>>()
            .map(DataValue::Array),
        Value::Object(values) => values
            .into_iter()
            .map(|(key, value)| Ok((key, from_json_value(value)?)))
            .collect::<Result<BTreeMap<_, _>>>()
            .map(DataValue::Object),
    }
}

fn to_json_value(value: &DataValue) -> Result<Value> {
    match value {
        DataValue::Null => Ok(Value::Null),
        DataValue::Bool(value) => Ok(Value::Bool(*value)),
        DataValue::Number(value) => {
            let number = serde_json::Number::from_f64(*value).ok_or_else(|| {
                DataError::Message("有限でない数値はJSONへ変換できません。".to_string())
            })?;

            Ok(Value::Number(number))
        }
        DataValue::String(value) => Ok(Value::String(value.clone())),
        DataValue::Array(values) => values
            .iter()
            .map(to_json_value)
            .collect::<Result<Vec<_>>>()
            .map(Value::Array),
        DataValue::Object(values) => values
            .iter()
            .map(|(key, value)| Ok((key.clone(), to_json_value(value)?)))
            .collect::<Result<Map<_, _>>>()
            .map(Value::Object),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trip() {
        let input = r#"{
            "name": "狐",
            "hp": 100,
            "active": true,
            "items": ["薬草", "木刀"],
            "extra": null
        }"#;

        let value = from_str(input).expect("JSONの読み込みに失敗しました。");
        let output = to_string(&value).expect("JSONの書き出しに失敗しました。");
        let restored = from_str(&output).expect("JSONの再読み込みに失敗しました。");

        assert_eq!(value, restored);
    }

    #[test]
    fn pretty_json_is_readable() {
        let value = DataValue::Object(BTreeMap::from([
            ("name".to_string(), DataValue::String("狐".to_string())),
            ("hp".to_string(), DataValue::Number(100.0)),
        ]));

        let output = to_string_pretty(&value).expect("整形JSONの生成に失敗しました。");

        assert!(output.contains("\n"));
        assert!(output.contains("\"name\""));
        assert!(output.contains("\"hp\""));
    }
}
