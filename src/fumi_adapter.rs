use crate::{DataError, DataValue, FumiCommand, FumiData, FumiResult, Result};


/// 詞側の値をデータ層へ渡すための中間値。
#[derive(Debug, Clone, PartialEq)]
pub enum FumiValue {
    文字列(String),
    数値(f64),
    真偽(bool),
    配列(Vec<FumiValue>),
    なし,
}

impl FumiValue {
    pub fn from_data_value(value: DataValue) -> Self {
        match value {
            DataValue::String(value) => Self::文字列(value),
            DataValue::Number(value) => Self::数値(value),
            DataValue::Bool(value) => Self::真偽(value),
            DataValue::Array(values) => Self::配列(
                values.into_iter().map(Self::from_data_value).collect(),
            ),
            DataValue::Null => Self::なし,
            DataValue::Object(_) => Self::なし,
        }
    }

    pub fn into_data_value(self) -> DataValue {
        match self {
            Self::文字列(value) => DataValue::String(value),
            Self::数値(value) => DataValue::Number(value),
            Self::真偽(value) => DataValue::Bool(value),
            Self::配列(values) => DataValue::Array(
                values.into_iter().map(Self::into_data_value).collect(),
            ),
            Self::なし => DataValue::Null,
        }
    }
}

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

    pub fn 文字列(助詞: impl Into<String>, 値: impl Into<String>) -> Self {
        Self::new(
            Some(助詞.into()),
            DataValue::String(値.into()),
        )
    }

    pub fn 数値(助詞: impl Into<String>, 値: f64) -> Self {
        Self::new(Some(助詞.into()), DataValue::Number(値))
    }

    pub fn 真偽(助詞: impl Into<String>, 値: bool) -> Self {
        Self::new(Some(助詞.into()), DataValue::Bool(値))
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

/// 引数列からコマンドを作成し、そのまま詞データへ実行する。
pub fn execute_arguments(
    data: &mut FumiData<'_>,
    verb: &str,
    arguments: &[FumiArgument],
) -> Result<FumiResult> {
    let command = command_from_arguments(verb, arguments)?;
    command.execute(data)
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


#[cfg(test)]
mod fumi_value_tests {
    use super::FumiValue;
    use crate::DataValue;

    #[test]
    fn converts_scalar_values() {
        assert_eq!(
            FumiValue::文字列("狐".into()).into_data_value(),
            DataValue::String("狐".into())
        );
        assert_eq!(FumiValue::数値(100.0).into_data_value(), DataValue::Number(100.0));
        assert_eq!(FumiValue::真偽(true).into_data_value(), DataValue::Bool(true));
        assert_eq!(FumiValue::なし.into_data_value(), DataValue::Null);
    }

    #[test]
    fn converts_nested_array() {
        let value = FumiValue::配列(vec![
            FumiValue::文字列("川越芋".into()),
            FumiValue::数値(2001.0),
            FumiValue::真偽(true),
        ]);

        assert_eq!(
            value.into_data_value(),
            DataValue::Array(vec![
                DataValue::String("川越芋".into()),
                DataValue::Number(2001.0),
                DataValue::Bool(true),
            ])
        );
    }

    #[test]
    fn converts_data_value_back_to_fumi_value() {
        let value = DataValue::Array(vec![
            DataValue::String("薬草".into()),
            DataValue::Number(50.0),
            DataValue::Bool(false),
            DataValue::Null,
        ]);

        assert_eq!(
            FumiValue::from_data_value(value),
            FumiValue::配列(vec![
                FumiValue::文字列("薬草".into()),
                FumiValue::数値(50.0),
                FumiValue::真偽(false),
                FumiValue::なし,
            ])
        );
    }
}


/// ASTのリテラル値をFumiValueへ渡すための変換。
///
/// 変数や二項式の評価は詞ランタイム側の責務とし、
/// データ層では具体値だけを扱う。
pub fn literal_to_fumi_value(value: &str) -> Result<FumiValue> {
    if value == "真" {
        return Ok(FumiValue::真偽(true));
    }
    if value == "偽" {
        return Ok(FumiValue::真偽(false));
    }
    if value == "なし" {
        return Ok(FumiValue::なし);
    }

    if let Ok(number) = value.parse::<f64>() {
        return Ok(FumiValue::数値(number));
    }

    if value.starts_with('「') && value.ends_with('」') {
        return Ok(FumiValue::文字列(
            value[3..value.len() - 3].to_string(),
        ));
    }

    Ok(FumiValue::文字列(value.to_string()))
}


/// 式のうち、すでに具体値になっているものだけをFumiValueへ変換する。
///
/// 変数と二項式は実行時評価が必要なため、この層では扱わない。
pub fn expression_to_fumi_value(value: &str) -> Result<FumiValue> {
    literal_to_fumi_value(value)
}


/// 詞の実行時変数を保持する軽量なコンテキスト。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FumiContext {
    variables: std::collections::BTreeMap<String, FumiValue>,
}

impl FumiContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, name: impl Into<String>, value: FumiValue) {
        self.variables.insert(name.into(), value);
    }

    pub fn get(&self, name: &str) -> Option<&FumiValue> {
        self.variables.get(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<FumiValue> {
        self.variables.remove(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    pub fn resolve(&self, name: &str) -> Result<FumiValue> {
        self.get(name)
            .cloned()
            .ok_or_else(|| DataError::Message(format!("変数が見つかりません: {name}")))
    }
}


/// 変数名から実行時コンテキストの値を解決する。
pub fn resolve_fumi_variable(context: &FumiContext, name: &str) -> Result<FumiValue> {
    context.resolve(name)
}


/// 詞側で評価可能な二項演算子。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FumiBinaryOperator {
    加算,
    減算,
    乗算,
    除算,
}

impl FumiBinaryOperator {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "+" | "加算" => Some(Self::加算),
            "-" | "減算" => Some(Self::減算),
            "*" | "乗算" => Some(Self::乗算),
            "/" | "除算" => Some(Self::除算),
            _ => None,
        }
    }
}

/// 具体値2つを使った二項演算を評価する。
pub fn evaluate_binary(
    left: FumiValue,
    operator: FumiBinaryOperator,
    right: FumiValue,
) -> Result<FumiValue> {
    let (left, right) = match (left, right) {
        (FumiValue::数値(left), FumiValue::数値(right)) => (left, right),
        _ => {
            return Err(DataError::Message(
                "二項演算には数値が必要です".to_string(),
            ));
        }
    };

    let value = match operator {
        FumiBinaryOperator::加算 => left + right,
        FumiBinaryOperator::減算 => left - right,
        FumiBinaryOperator::乗算 => left * right,
        FumiBinaryOperator::除算 => {
            if right == 0.0 {
                return Err(DataError::Message("0では除算できません".to_string()));
            }
            left / right
        }
    };

    Ok(FumiValue::数値(value))
}


/// 実行時に評価できる式。
#[derive(Debug, Clone, PartialEq)]
pub enum FumiExpression {
    値(FumiValue),
    変数(String),
    二項 {
        左: Box<FumiExpression>,
        演算子: FumiBinaryOperator,
        右: Box<FumiExpression>,
    },
}

/// 式をFumiValueまで評価する。
pub fn evaluate_expression(
    expression: &FumiExpression,
    context: &FumiContext,
) -> Result<FumiValue> {
    match expression {
        FumiExpression::値(value) => Ok(value.clone()),
        FumiExpression::変数(name) => resolve_fumi_variable(context, name),
        FumiExpression::二項 { 左, 演算子, 右 } => {
            let left = evaluate_expression(左, context)?;
            let right = evaluate_expression(右, context)?;
            evaluate_binary(left, *演算子, right)
        }
    }
}


/// 外部の詞ASTを直接依存させずに、ASTの構造を
/// FumiExpressionへ渡すための中間表現。
#[derive(Debug, Clone, PartialEq)]
pub enum FumiAstExpression {
    値(FumiValue),
    変数(String),
    二項 {
        左: Box<FumiAstExpression>,
        演算子: FumiBinaryOperator,
        右: Box<FumiAstExpression>,
    },
}

impl FumiAstExpression {
    pub fn into_expression(self) -> FumiExpression {
        match self {
            Self::値(value) => FumiExpression::値(value),
            Self::変数(name) => FumiExpression::変数(name),
            Self::二項 { 左, 演算子, 右 } => FumiExpression::二項 {
                左: Box::new(左.into_expression()),
                演算子,
                右: Box::new(右.into_expression()),
            },
        }
    }
}


#[cfg(test)]
mod ast_bridge_tests {
    use super::{
        evaluate_expression, FumiAstExpression, FumiBinaryOperator, FumiContext, FumiValue,
    };

    #[test]
    fn ast_expression_converts_and_evaluates() {
        let mut context = FumiContext::new();
        context.set("敵ID", FumiValue::数値(1001.0));

        let ast = FumiAstExpression::二項 {
            左: Box::new(FumiAstExpression::変数("敵ID".into())),
            演算子: FumiBinaryOperator::加算,
            右: Box::new(FumiAstExpression::値(FumiValue::数値(1.0))),
        };

        let expression = ast.into_expression();
        assert_eq!(
            evaluate_expression(&expression, &context).unwrap(),
            FumiValue::数値(1002.0)
        );
    }

    #[test]
    fn ast_literal_expression_evaluates() {
        let ast = FumiAstExpression::値(FumiValue::文字列("川越芋".into()));
        let expression = ast.into_expression();
        let context = FumiContext::new();

        assert_eq!(
            evaluate_expression(&expression, &context).unwrap(),
            FumiValue::文字列("川越芋".into())
        );
    }
}
