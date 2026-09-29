use std::fs;
use std::path::Path;

use crate::{csv, json, query, DataError, DataValue, Result};

/// CSV / JSON ファイルを DataValue として読み書きする共通ストア。
#[derive(Debug, Clone, PartialEq)]
pub struct DataStore {
    value: DataValue,
}

impl DataStore {
    /// DataValue からストアを作成する。
    pub fn new(value: DataValue) -> Self {
        Self { value }
    }

    /// CSV / JSON ファイルからストアを読み込む。
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

    /// 保持しているデータを取得する。
    pub fn value(&self) -> &DataValue {
        &self.value
    }

    /// 保持しているデータを変更可能な形で取得する。
    pub fn value_mut(&mut self) -> &mut DataValue {
        &mut self.value
    }

    /// 指定フィールドに一致する最初のデータを取得する。
    pub fn find<'a>(
        &'a self,
        field: &str,
        expected: &DataValue,
    ) -> Result<Option<&'a DataValue>> {
        query::find(&self.value, field, expected)
    }

    /// 指定フィールドに一致する最初のデータを変更可能な形で取得する。
    pub fn find_mut<'a>(
        &'a mut self,
        field: &str,
        expected: &DataValue,
    ) -> Result<Option<&'a mut DataValue>> {
        query::find_mut(&mut self.value, field, expected)
    }

    /// 指定フィールドに一致するすべてのデータを取得する。
    pub fn find_all<'a>(
        &'a self,
        field: &str,
        expected: &DataValue,
    ) -> Result<Vec<&'a DataValue>> {
        query::find_all(&self.value, field, expected)
    }

    /// 配列データへ新しい値を追加する。
    pub fn push(&mut self, value: DataValue) -> Result<()> {
        self.value.push(value)
    }

    /// 指定フィールドに一致する最初のデータを削除する。
    pub fn remove_first(
        &mut self,
        field: &str,
        expected: &DataValue,
    ) -> Result<Option<DataValue>> {
        query::remove_first(&mut self.value, field, expected)
    }

    /// DataValueを取り出す。
    pub fn into_value(self) -> DataValue {
        self.value
    }

    /// CSV / JSON ファイルへ保存する。
    ///
    /// 出力形式はファイル拡張子から決定する。
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
    fn find_and_modify_data() {
        let mut store = DataStore::new(DataValue::Array(vec![
            DataValue::Object(std::collections::BTreeMap::from([
                ("id".to_string(), DataValue::Number(1.0)),
                ("name".to_string(), DataValue::String("狐".to_string())),
                ("hp".to_string(), DataValue::Number(100.0)),
            ])),
            DataValue::Object(std::collections::BTreeMap::from([
                ("id".to_string(), DataValue::Number(2.0)),
                ("name".to_string(), DataValue::String("鬼".to_string())),
                ("hp".to_string(), DataValue::Number(250.0)),
            ])),
        ]));

        let expected = DataValue::String("狐".to_string());
        let enemy = store.find_mut("name", &expected).unwrap().unwrap();

        enemy
            .set("hp", DataValue::Number(150.0))
            .expect("HPの更新に失敗しました。");

        let enemy = store.find("name", &expected).unwrap().unwrap();
        assert_eq!(
            enemy.get("hp").and_then(DataValue::as_f64),
            Some(150.0)
        );
    }

    #[test]
    fn unsupported_extension_returns_error() {
        let result = DataStore::load("data.txt");
        assert!(result.is_err());
    }
}
