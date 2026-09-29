use crate::{DataError, DataValue, GameData, Result};

/// 詞スクリプトからゲームデータへ接続するためのデータ橋渡し。
///
/// 詞側はCSV/JSONの形式を直接意識せず、カテゴリとIDを指定して
/// yaoyorozu-data の共通データへアクセスする。
#[derive(Debug)]
pub struct FumiData<'a> {
    game_data: &'a mut GameData,
}

impl<'a> FumiData<'a> {
    /// GameDataから詞用のデータアクセス口を作る。
    pub fn new(game_data: &'a mut GameData) -> Self {
        Self { game_data }
    }

    /// IDでゲームデータを取得する。
    pub fn get(&self, category: &str, id: &DataValue) -> Result<Option<&DataValue>> {
        match category {
            "enemy" | "enemies" => self.game_data.enemy(id),
            "item" | "items" => self.game_data.item(id),
            "skill" | "skills" => self.game_data.skill(id),
            "quest" | "quests" => self.game_data.quest(id),
            "recipe" | "recipes" => self.game_data.recipe(id),
            _ => Err(DataError::Message(format!(
                "未知のゲームデータ種別です: {category}"
            ))),
        }
    }

    /// IDでゲームデータを変更可能な形で取得する。
    pub fn get_mut(
        &mut self,
        category: &str,
        id: &DataValue,
    ) -> Result<Option<&mut DataValue>> {
        match category {
            "enemy" | "enemies" => self.game_data.enemy_mut(id),
            "item" | "items" => self.game_data.item_mut(id),
            "skill" | "skills" => self.game_data.skill_mut(id),
            "quest" | "quests" => self.game_data.quest_mut(id),
            "recipe" | "recipes" => self.game_data.recipe_mut(id),
            _ => Err(DataError::Message(format!(
                "未知のゲームデータ種別です: {category}"
            ))),
        }
    }

    /// 指定したIDのゲームデータが存在するか確認する。
    pub fn has(&self, category: &str, id: &DataValue) -> Result<bool> {
        Ok(self.get(category, id)?.is_some())
    }

    /// 詞側からフィールドを書き換える。
    ///
    /// 現段階では既存レコードのトップレベルフィールドを対象とする。
    pub fn set_field(
        &mut self,
        category: &str,
        id: &DataValue,
        field: &str,
        value: DataValue,
    ) -> Result<()> {
        let record = self
            .get_mut(category, id)?
            .ok_or_else(|| DataError::Message(format!(
                "ゲームデータが見つかりません: {category} / {id:?}"
            )))?;

        record.set(field.to_string(), value)
    }
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
            DataStore::new(DataValue::Array(vec![row(&[
                ("id", DataValue::Number(2001.0)),
                ("name", DataValue::String("川越芋".into())),
                ("category", DataValue::String("食材".into())),
                ("price", DataValue::Number(100.0)),
            ])])),
            DataStore::new(DataValue::Array(Vec::new())),
            DataStore::new(DataValue::Array(Vec::new())),
            DataStore::new(DataValue::Array(Vec::new())),
        )
    }

    #[test]
    fn gets_data_by_category_and_id() {
        let mut data = game_data();
        let fumi = FumiData::new(&mut data);

        let value = fumi
            .get("enemy", &DataValue::Number(1001.0))
            .unwrap()
            .unwrap();

        assert_eq!(
            value.get("name").and_then(DataValue::as_str),
            Some("狐")
        );
    }

    #[test]
    fn checks_existence() {
        let mut data = game_data();
        let fumi = FumiData::new(&mut data);

        assert!(fumi.has("item", &DataValue::Number(2001.0)).unwrap());
        assert!(!fumi.has("item", &DataValue::Number(9999.0)).unwrap());
    }

    #[test]
    fn changes_field() {
        let mut data = game_data();
        let mut fumi = FumiData::new(&mut data);

        fumi.set_field(
            "enemy",
            &DataValue::Number(1001.0),
            "hp",
            DataValue::Number(120.0),
        )
        .unwrap();

        let value = fumi
            .get("enemy", &DataValue::Number(1001.0))
            .unwrap()
            .unwrap();

        assert_eq!(value.get("hp").and_then(DataValue::as_f64), Some(120.0));
    }

    #[test]
    fn rejects_unknown_category() {
        let mut data = game_data();
        let fumi = FumiData::new(&mut data);

        let result = fumi.get("unknown", &DataValue::Number(1.0));

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("未知のゲームデータ種別"));
    }
}
