use std::fs;
use std::path::Path;

use crate::{csv, json, DataError, DataValue, Result};

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
    fn unsupported_extension_returns_error() {
        let result = DataStore::load("data.txt");
        assert!(result.is_err());
    }
}
