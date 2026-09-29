use crate::{DataError, DataValue, FumiData, Result};

/// 詞の既存命令種別から、データ操作命令へ変換するための種別。
///
/// 実際の詞ASTを直接依存させず、八百万データ側では
/// 「取得・設定・存在確認」というデータ操作の意味だけを扱う。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FumiDataVerb {
    取得,
    存在,
    設定,
}

impl FumiDataVerb {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "取得" => Some(Self::取得),
            "存在" | "確認" => Some(Self::存在),
            "設定" | "変更" => Some(Self::設定),
            _ => None,
        }
    }
}

/// 詞の命令引数からデータ操作命令を組み立てるための入力。
#[derive(Debug, Clone, PartialEq)]
pub struct FumiDataRequest {
    pub 動詞: FumiDataVerb,
    pub 種別: String,
    pub id: DataValue,
    pub 項目: Option<String>,
    pub 値: Option<DataValue>,
}

impl FumiDataRequest {
    pub fn 取得(種別: impl Into<String>, id: DataValue) -> Self {
        Self {
            動詞: FumiDataVerb::取得,
            種別: 種別.into(),
            id,
            項目: None,
            値: None,
        }
    }

    pub fn 存在(種別: impl Into<String>, id: DataValue) -> Self {
        Self {
            動詞: FumiDataVerb::存在,
            種別: 種別.into(),
            id,
            項目: None,
            値: None,
        }
    }

    pub fn 設定(
        種別: impl Into<String>,
        id: DataValue,
        項目: impl Into<String>,
        値: DataValue,
    ) -> Self {
        Self {
            動詞: FumiDataVerb::設定,
            種別: 種別.into(),
            id,
            項目: Some(項目.into()),
            値: Some(値),
        }
    }

    pub fn into_command(self) -> Result<FumiCommand> {
        match self.動詞 {
            FumiDataVerb::取得 => Ok(FumiCommand::Get {
                category: self.種別,
                id: self.id,
            }),
            FumiDataVerb::存在 => Ok(FumiCommand::Has {
                category: self.種別,
                id: self.id,
            }),
            FumiDataVerb::設定 => {
                let field = self.項目.ok_or_else(|| {
                    DataError::Message("設定命令には項目が必要です".into())
                })?;
                let value = self.値.ok_or_else(|| {
                    DataError::Message("設定命令には値が必要です".into())
                })?;

                Ok(FumiCommand::SetField {
                    category: self.種別,
                    id: self.id,
                    field,
                    value,
                })
            }
        }
    }
}

/// 既存の「詞」の動詞名をデータ操作命令へ変換する。
///
/// これにより詞AST側はCSV/JSONやGameDataを知らずに済む。
pub fn command_from_verb(
    verb: &str,
    category: impl Into<String>,
    id: DataValue,
    field: Option<String>,
    value: Option<DataValue>,
) -> Result<FumiCommand> {
    let verb = FumiDataVerb::from_str(verb).ok_or_else(|| {
        DataError::Message(format!("データ操作に対応していない詞命令です: {verb}"))
    })?;

    FumiDataRequest {
        動詞: verb,
        種別: category.into(),
        id,
        項目: field,
        値: value,
    }
    .into_command()
}

/// データ命令を直接実行する便利関数。
pub fn execute_request(
    data: &mut FumiData<'_>,
    request: FumiDataRequest,
) -> Result<FumiResult> {
    request.into_command()?.execute(data)
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
    fn converts_get_request() {
        let request = FumiDataRequest::取得("enemy", DataValue::Number(1001.0));
        let command = request.into_command().unwrap();

        assert_eq!(
            command,
            FumiCommand::Get {
                category: "enemy".into(),
                id: DataValue::Number(1001.0),
            }
        );
    }

    #[test]
    fn converts_existing_verb_names() {
        assert_eq!(
            command_from_verb("取得", "enemy", DataValue::Number(1001.0), None, None)
                .unwrap(),
            FumiCommand::Get {
                category: "enemy".into(),
                id: DataValue::Number(1001.0),
            }
        );

        assert_eq!(
            command_from_verb("確認", "enemy", DataValue::Number(1001.0), None, None)
                .unwrap(),
            FumiCommand::Has {
                category: "enemy".into(),
                id: DataValue::Number(1001.0),
            }
        );
    }

    #[test]
    fn converts_and_executes_set_request() {
        let mut data = game_data();
        let mut fumi = FumiData::new(&mut data);

        let result = execute_request(
            &mut fumi,
            FumiDataRequest::設定(
                "enemy",
                DataValue::Number(1001.0),
                "hp",
                DataValue::Number(150.0),
            ),
        )
        .unwrap();

        assert_eq!(result, FumiResult::Unit);

        let enemy = fumi
            .get("enemy", &DataValue::Number(1001.0))
            .unwrap()
            .unwrap();

        assert_eq!(
            enemy.get("hp").and_then(DataValue::as_f64),
            Some(150.0)
        );
    }

    #[test]
    fn rejects_missing_set_arguments() {
        let request = FumiDataRequest {
            動詞: FumiDataVerb::設定,
            種別: "enemy".into(),
            id: DataValue::Number(1001.0),
            項目: None,
            値: Some(DataValue::Number(150.0)),
        };

        assert!(request.into_command().is_err());
    }

    #[test]
    fn rejects_unknown_verb() {
        let result =
            command_from_verb("攻撃", "enemy", DataValue::Number(1001.0), None, None);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("データ操作に対応していない詞命令"));
    }
}
