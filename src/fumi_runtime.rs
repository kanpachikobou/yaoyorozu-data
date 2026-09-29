use crate::{FumiCommand, FumiData, FumiResult, GameData, Result};

/// 詞ランタイムのデータ実行環境。
///
/// 詞パーサーや将来の八百万駆動側は、GameDataの内部構造を直接触らず、
/// このランタイムへ命令を渡してデータ操作を行う。
#[derive(Debug)]
pub struct FumiRuntime {
    game_data: GameData,
}

impl FumiRuntime {
    /// GameDataを所有する詞ランタイムを作る。
    pub fn new(game_data: GameData) -> Self {
        Self { game_data }
    }

    /// 現在のGameDataへの読み取りアクセスを返す。
    pub fn game_data(&self) -> &GameData {
        &self.game_data
    }

    /// 現在のGameDataへの変更アクセスを返す。
    pub fn game_data_mut(&mut self) -> &mut GameData {
        &mut self.game_data
    }

    /// 詞命令を1件実行する。
    pub fn execute(&mut self, command: FumiCommand) -> Result<FumiResult> {
        let mut data = FumiData::new(&mut self.game_data);
        command.execute(&mut data)
    }

    /// 複数の詞命令を順番に実行する。
    pub fn execute_all(
        &mut self,
        commands: impl IntoIterator<Item = FumiCommand>,
    ) -> Result<Vec<FumiResult>> {
        let mut results = Vec::new();

        for command in commands {
            results.push(self.execute(command)?);
        }

        Ok(results)
    }

    /// GameDataの所有権をランタイムから取り出す。
    pub fn into_game_data(self) -> GameData {
        self.game_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DataStore, DataValue};

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
    fn executes_commands_through_runtime() {
        let mut runtime = FumiRuntime::new(game_data());

        let result = runtime
            .execute(FumiCommand::Get {
                category: "enemy".into(),
                id: DataValue::Number(1001.0),
            })
            .unwrap();

        assert!(matches!(
            result,
            FumiResult::Value(Some(ref value))
                if value.get("name").and_then(DataValue::as_str) == Some("狐")
        ));

        runtime
            .execute(FumiCommand::SetField {
                category: "enemy".into(),
                id: DataValue::Number(1001.0),
                field: "hp".into(),
                value: DataValue::Number(120.0),
            })
            .unwrap();

        let result = runtime
            .execute(FumiCommand::Get {
                category: "enemy".into(),
                id: DataValue::Number(1001.0),
            })
            .unwrap();

        assert!(matches!(
            result,
            FumiResult::Value(Some(ref value))
                if value.get("hp").and_then(DataValue::as_f64) == Some(120.0)
        ));
    }

    #[test]
    fn executes_multiple_commands_in_order() {
        let mut runtime = FumiRuntime::new(game_data());

        let results = runtime
            .execute_all([
                FumiCommand::Has {
                    category: "enemy".into(),
                    id: DataValue::Number(1001.0),
                },
                FumiCommand::SetField {
                    category: "enemy".into(),
                    id: DataValue::Number(1001.0),
                    field: "hp".into(),
                    value: DataValue::Number(150.0),
                },
            ])
            .unwrap();

        assert_eq!(
            results,
            vec![FumiResult::Bool(true), FumiResult::Unit]
        );
    }
}
