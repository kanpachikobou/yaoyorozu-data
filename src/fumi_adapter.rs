use crate::{DataError, DataValue, FumiCommand, FumiData, FumiResult, Result};

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


/// 既存の詞ASTにある「命令引数（助詞 + 値）」を
/// データ命令へ変換するための中間表現。
#[derive(Debug, Clone, PartialEq)]
pub struct FumiArgument {
    pub 助詞: Option<String>,
    pub 値: DataValue,
}

impl FumiArgument {
    pub fn new(助詞: Option<String>, 値: DataValue) -> Self {
        Self { 助詞, 値 }
    }
}

/// 命令引数の助詞を使って、詞の命令をデータ命令へ変換する。
///
/// 想定する引数名:
/// - 種別 / 種類 → データカテゴリ
/// - ID / 識別子 → レコードID
/// - 項目 / フィールド → 変更対象フィールド
/// - 値 → 設定値
///
/// 詞パーサーはASTを解析した後、この関数へ
/// 命令の動詞と引数を渡すだけでよい。
pub fn command_from_arguments(
    verb: &str,
    arguments: &[FumiArgument],
) -> Result<FumiCommand> {
    let category = arguments
        .iter()
        .find(|argument| matches!(argument.助詞.as_deref(), Some("種別" | "種類")))
        .map(|argument| data_value_to_string(&argument.値))
        .transpose()?
        .ok_or_else(|| DataError::Message("データ命令には種別が必要です".into()))?;

    let id = arguments
        .iter()
        .find(|argument| matches!(argument.助詞.as_deref(), Some("ID" | "id" | "識別子")))
        .map(|argument| argument.値.clone())
        .ok_or_else(|| DataError::Message("データ命令にはIDが必要です".into()))?;

    let field = arguments
        .iter()
        .find(|argument| matches!(argument.助詞.as_deref(), Some("項目" | "フィールド")))
        .map(|argument| data_value_to_string(&argument.値))
        .transpose()?;

    let value = arguments
        .iter()
        .find(|argument| matches!(argument.助詞.as_deref(), Some("値")))
        .map(|argument| argument.値.clone());

    command_from_verb(verb, category, id, field, value)
}

fn data_value_to_string(value: &DataValue) -> Result<String> {
    match value {
        DataValue::String(value) => Ok(value.clone()),
        DataValue::Number(value) => Ok(value.to_string()),
        DataValue::Bool(value) => Ok(value.to_string()),
        DataValue::Null => Err(DataError::Message("なしは文字列として扱えません".into())),
        DataValue::Array(_) | DataValue::Object(_) => Err(DataError::Message(
            "配列やオブジェクトは文字列引数として扱えません".into(),
        )),
    }
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
    fn converts_argument_list() {
        let arguments = vec![
            FumiArgument::new(
                Some("種別".into()),
                DataValue::String("enemy".into()),
            ),
            FumiArgument::new(Some("ID".into()), DataValue::Number(1001.0)),
        ];

        let command = command_from_arguments("取得", &arguments).unwrap();

        assert_eq!(
            command,
            FumiCommand::Get {
                category: "enemy".into(),
                id: DataValue::Number(1001.0),
            }
        );
    }

    #[test]
    fn converts_set_arguments() {
        let arguments = vec![
            FumiArgument::new(
                Some("種別".into()),
                DataValue::String("enemy".into()),
            ),
            FumiArgument::new(Some("ID".into()), DataValue::Number(1001.0)),
            FumiArgument::new(
                Some("項目".into()),
                DataValue::String("hp".into()),
            ),
            FumiArgument::new(Some("値".into()), DataValue::Number(180.0)),
        ];

        let command = command_from_arguments("設定", &arguments).unwrap();

        assert_eq!(
            command,
            FumiCommand::SetField {
                category: "enemy".into(),
                id: DataValue::Number(1001.0),
                field: "hp".into(),
                value: DataValue::Number(180.0),
            }
        );
    }

    #[test]
    fn rejects_missing_category() {
        let arguments = vec![
            FumiArgument::new(Some("ID".into()), DataValue::Number(1001.0)),
        ];

        assert!(command_from_arguments("取得", &arguments).is_err());
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
