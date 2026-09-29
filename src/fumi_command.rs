use crate::{DataError, DataValue, FumiData, Result};

/// 詞ランタイムから実行するデータ操作命令。
///
/// 実際の詞パーサーとは分離し、パーサーは最終的にこの命令へ変換する。
#[derive(Debug, Clone, PartialEq)]
pub enum FumiCommand {
    Get {
        category: String,
        id: DataValue,
    },
    Has {
        category: String,
        id: DataValue,
    },
    SetField {
        category: String,
        id: DataValue,
        field: String,
        value: DataValue,
    },
}

/// 詞のデータ操作命令の実行結果。
///
/// スクリプト境界では所有権を持つ値を返し、借用関係をランタイム側へ漏らさない。
#[derive(Debug, Clone, PartialEq)]
pub enum FumiResult {
    Value(Option<DataValue>),
    Bool(bool),
    Unit,
}

impl FumiCommand {
    /// 命令をFumiDataへ渡して実行する。
    pub fn execute(self, data: &mut FumiData<'_>) -> Result<FumiResult> {
        match self {
            Self::Get { category, id } => {
                let value = data.get(&category, &id)?.cloned();
                Ok(FumiResult::Value(value))
            }
            Self::Has { category, id } => {
                Ok(FumiResult::Bool(data.has(&category, &id)?))
            }
            Self::SetField {
                category,
                id,
                field,
                value,
            } => {
                data.set_field(&category, &id, &field, value)?;
                Ok(FumiResult::Unit)
            }
        }
    }
}

/// 文字列から詞のデータ命令種別を解釈するための共通エラー。
///
/// 現時点では実際の詞文法パーサーを実装せず、
/// 命令実行層だけを独立して利用できるようにしている。
pub fn unsupported_command(command: &str) -> Result<FumiResult> {
    Err(DataError::Message(format!(
        "未対応の詞命令です: {command}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DataStore, GameData};

    fn row(fields: &[(&str, DataValue)]) -> DataValue {
        DataValue::Object(
            fields
                .iter()
                .map(|(key, value)| ((*key).to_string(), value.clone()))
                .collect(),
        )
    }

    fn game_data() -> GameData {
        GameData::new(
            DataStore::new(DataValue::Array(vec![row(&[
                ("id", DataValue::Number(1001.0)),
                ("name", DataValue::String("狐".into())),
                ("hp", DataValue::Number(100.0)),
                ("element", DataValue::String("火".into())),
                ("level", DataValue::Number(1.0)),
            ])])),
            DataStore::new(DataValue::Array(Vec::new())),
            DataStore::new(DataValue::Array(Vec::new())),
            DataStore::new(DataValue::Array(Vec::new())),
            DataStore::new(DataValue::Array(Vec::new())),
        )
    }

    #[test]
    fn executes_get() {
        let mut data = game_data();
        let mut fumi = FumiData::new(&mut data);

        let result = FumiCommand::Get {
            category: "enemy".into(),
            id: DataValue::Number(1001.0),
        }
        .execute(&mut fumi)
        .unwrap();

        assert!(matches!(
            result,
            FumiResult::Value(Some(ref value))
                if value.get("name").and_then(DataValue::as_str) == Some("狐")
        ));
    }

    #[test]
    fn executes_has() {
        let mut data = game_data();
        let mut fumi = FumiData::new(&mut data);

        let result = FumiCommand::Has {
            category: "enemy".into(),
            id: DataValue::Number(1001.0),
        }
        .execute(&mut fumi)
        .unwrap();

        assert_eq!(result, FumiResult::Bool(true));
    }

    #[test]
    fn executes_set_field() {
        let mut data = game_data();
        let mut fumi = FumiData::new(&mut data);

        let result = FumiCommand::SetField {
            category: "enemy".into(),
            id: DataValue::Number(1001.0),
            field: "hp".into(),
            value: DataValue::Number(120.0),
        }
        .execute(&mut fumi)
        .unwrap();

        assert_eq!(result, FumiResult::Unit);

        let value = fumi
            .get("enemy", &DataValue::Number(1001.0))
            .unwrap()
            .unwrap();

        assert_eq!(value.get("hp").and_then(DataValue::as_f64), Some(120.0));
    }

    #[test]
    fn rejects_unsupported_command() {
        let result = unsupported_command("spawn_enemy");

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("未対応の詞命令"));
    }
}
