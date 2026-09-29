use std::fs;
use std::path::Path;

use crate::{csv, json, query, DataError, DataValue, Result};

/// CSV / JSON ファイルを DataValue として読み書きする共通ストア。
#[derive(Debug, Clone, PartialEq)]
pub struct DataStore {
    value: DataValue,
}

impl DataStore {
    pub fn new(value: DataValue) -> Self {
        Self { value }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let input = fs::read_to_string(path)
            .map_err(|error| DataError::Message(format!("入力ファイルを読めません: {error}")))?;

        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase)
            .ok_or_else(|| {
                DataError::Message(format!("ファイル形式を判定できません: {}", path.display()))
            })?;

        let value = match extension.as_str() {
            "csv" => csv::from_str(&input)?,
            "json" => json::from_str(&input)?,
            _ => {
                return Err(DataError::Message(format!(
                    "対応していないファイル形式です: .{extension}（csv / json のみ対応）"
                )));
            }
        };

        Ok(Self::new(value))
    }

    pub fn value(&self) -> &DataValue {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut DataValue {
        &mut self.value
    }

    pub fn find<'a>(
        &'a self,
        field: &str,
        expected: &DataValue,
    ) -> Result<Option<&'a DataValue>> {
        query::find(&self.value, field, expected)
    }

    pub fn find_by_id<'a>(
        &'a self,
        id: &DataValue,
    ) -> Result<Option<&'a DataValue>> {
        query::find_by_id(&self.value, id)
    }

    pub fn find_mut<'a>(
        &'a mut self,
        field: &str,
        expected: &DataValue,
    ) -> Result<Option<&'a mut DataValue>> {
        query::find_mut(&mut self.value, field, expected)
    }

    pub fn find_by_id_mut<'a>(
        &'a mut self,
        id: &DataValue,
    ) -> Result<Option<&'a mut DataValue>> {
        query::find_by_id_mut(&mut self.value, id)
    }

    pub fn find_all<'a>(
        &'a self,
        field: &str,
        expected: &DataValue,
    ) -> Result<Vec<&'a DataValue>> {
        query::find_all(&self.value, field, expected)
    }

    pub fn push(&mut self, value: DataValue) -> Result<()> {
        self.value.push(value)
    }

    pub fn remove_first(
        &mut self,
        field: &str,
        expected: &DataValue,
    ) -> Result<Option<DataValue>> {
        query::remove_first(&mut self.value, field, expected)
    }

    pub fn remove_by_id(&mut self, id: &DataValue) -> Result<Option<DataValue>> {
        query::remove_by_id(&mut self.value, id)
    }

    pub fn into_value(self) -> DataValue {
        self.value
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();

        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase)
            .ok_or_else(|| {
                DataError::Message(format!("ファイル形式を判定できません: {}", path.display()))
            })?;

        let output = match extension.as_str() {
            "csv" => csv::to_string(&self.value)?,
            "json" => json::to_string_pretty(&self.value)?,
            _ => {
                return Err(DataError::Message(format!(
                    "対応していないファイル形式です: .{extension}（csv / json のみ対応）"
                )));
            }
        };

        fs::write(path, output)
            .map_err(|error| DataError::Message(format!("出力ファイルを書き込めません: {error}")))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_store_from_value() {
        let value = DataValue::object();
        let store = DataStore::new(value.clone());
        assert_eq!(store.value(), &value);
    }

    #[test]
    fn load_and_save_json() {
        let directory = std::env::temp_dir();
        let input_path = directory.join("yaoyorozu-data-store-input.json");
        let output_path = directory.join("yaoyorozu-data-store-output.json");

        fs::write(
            &input_path,
            r#"{"name":"狐","hp":100,"active":true}"#,
        )
        .unwrap();

        let store = DataStore::load(&input_path).unwrap();
        store.save(&output_path).unwrap();
        let restored = DataStore::load(&output_path).unwrap();

        assert_eq!(store, restored);

        let _ = fs::remove_file(input_path);
        let _ = fs::remove_file(output_path);
    }

    #[test]
    fn find_and_modify_by_id() {
        let mut store = DataStore::new(DataValue::Array(vec![
            DataValue::Object(std::collections::BTreeMap::from([
                ("id".to_string(), DataValue::Number(1001.0)),
                ("name".to_string(), DataValue::String("狐".to_string())),
                ("hp".to_string(), DataValue::Number(100.0)),
            ])),
            DataValue::Object(std::collections::BTreeMap::from([
                ("id".to_string(), DataValue::Number(1002.0)),
                ("name".to_string(), DataValue::String("鬼".to_string())),
                ("hp".to_string(), DataValue::Number(250.0)),
            ])),
        ]));

        let id = DataValue::Number(1002.0);
        let enemy = store.find_by_id_mut(&id).unwrap().unwrap();

        enemy.set("hp", DataValue::Number(500.0)).unwrap();

        let enemy = store.find_by_id(&id).unwrap().unwrap();
        assert_eq!(enemy.get("hp").and_then(DataValue::as_f64), Some(500.0));
    }

    #[test]
    fn unsupported_extension_returns_error() {
        let result = DataStore::load("data.txt");
        assert!(result.is_err());
    }
}
