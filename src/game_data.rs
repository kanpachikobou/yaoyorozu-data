use std::path::Path;

use crate::{DataError, DataStore, DataValue, Result};

/// ゲームで利用する共通データセット。
#[derive(Debug, Clone, PartialEq)]
pub struct GameData {
    pub enemies: DataStore,
    pub items: DataStore,
    pub skills: DataStore,
    pub quests: DataStore,
    pub recipes: DataStore,
}

impl GameData {
    pub fn new(
        enemies: DataStore,
        items: DataStore,
        skills: DataStore,
        quests: DataStore,
        recipes: DataStore,
    ) -> Self {
        Self {
            enemies,
            items,
            skills,
            quests,
            recipes,
        }
    }

    pub fn load_from_dir<P: AsRef<Path>>(directory: P) -> Result<Self> {
        let directory = directory.as_ref();

        Ok(Self::new(
            load_required(directory, "enemies.json")?,
            load_required(directory, "items.json")?,
            load_required(directory, "skills.json")?,
            load_required(directory, "quests.json")?,
            load_required(directory, "recipes.json")?,
        ))
    }

    /// ゲームデータをロードして、すぐに標準スキーマで検証する。
    pub fn load_and_validate_from_dir<P: AsRef<Path>>(directory: P) -> Result<Self> {
        let data = Self::load_from_dir(directory)?;
        data.validate()?;
        Ok(data)
    }

    pub fn validate(&self) -> Result<()> {
        self.enemies.validate(&crate::validate::game_enemy_schema())?;
        self.items.validate(&crate::validate::game_item_schema())?;
        self.skills.validate(&crate::validate::game_skill_schema())?;
        self.quests.validate(&crate::validate::game_quest_schema())?;
        self.recipes.validate(&crate::validate::game_recipe_schema())?;
        Ok(())
    }

    pub fn enemy(&self, id: &DataValue) -> Result<Option<&DataValue>> {
        self.enemies.find_by_id(id)
    }

    pub fn item(&self, id: &DataValue) -> Result<Option<&DataValue>> {
        self.items.find_by_id(id)
    }

    pub fn enemy_mut(&mut self, id: &DataValue) -> Result<Option<&mut DataValue>> {
        self.enemies.find_by_id_mut(id)
    }

    pub fn item_mut(&mut self, id: &DataValue) -> Result<Option<&mut DataValue>> {
        self.items.find_by_id_mut(id)
    }

    pub fn skill(&self, id: &DataValue) -> Result<Option<&DataValue>> {
        self.skills.find_by_id(id)
    }

    pub fn skill_mut(&mut self, id: &DataValue) -> Result<Option<&mut DataValue>> {
        self.skills.find_by_id_mut(id)
    }

    pub fn quest(&self, id: &DataValue) -> Result<Option<&DataValue>> {
        self.quests.find_by_id(id)
    }

    pub fn quest_mut(&mut self, id: &DataValue) -> Result<Option<&mut DataValue>> {
        self.quests.find_by_id_mut(id)
    }

    pub fn recipe(&self, id: &DataValue) -> Result<Option<&DataValue>> {
        self.recipes.find_by_id(id)
    }

    pub fn recipe_mut(&mut self, id: &DataValue) -> Result<Option<&mut DataValue>> {
        self.recipes.find_by_id_mut(id)
    }

    pub fn has_enemy(&self, id: &DataValue) -> Result<bool> {
        Ok(self.enemy(id)?.is_some())
    }

    pub fn has_item(&self, id: &DataValue) -> Result<bool> {
        Ok(self.item(id)?.is_some())
    }

    pub fn has_skill(&self, id: &DataValue) -> Result<bool> {
        Ok(self.skill(id)?.is_some())
    }

    pub fn has_quest(&self, id: &DataValue) -> Result<bool> {
        Ok(self.quest(id)?.is_some())
    }

    pub fn has_recipe(&self, id: &DataValue) -> Result<bool> {
        Ok(self.recipe(id)?.is_some())
    }
}

fn load_required(directory: &Path, filename: &str) -> Result<DataStore> {
    let path = directory.join(filename);

    if !path.exists() {
        return Err(DataError::Message(format!(
            "ゲームデータがありません: {}",
            path.display()
        )));
    }

    DataStore::load(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn row(fields: &[(&str, DataValue)]) -> DataValue {
        DataValue::Object(
            fields
                .iter()
                .map(|(key, value)| ((*key).to_string(), value.clone()))
                .collect(),
        )
    }

    fn enemy(id: f64) -> DataValue {
        row(&[
            ("id", DataValue::Number(id)),
            ("name", DataValue::String("狐".into())),
            ("hp", DataValue::Number(100.0)),
            ("element", DataValue::String("火".into())),
            ("level", DataValue::Number(1.0)),
        ])
    }

    fn item(id: f64) -> DataValue {
        row(&[
            ("id", DataValue::Number(id)),
            ("name", DataValue::String("川越芋".into())),
            ("category", DataValue::String("食材".into())),
            ("price", DataValue::Number(100.0)),
        ])
    }

    fn skill(id: f64) -> DataValue {
        row(&[
            ("id", DataValue::Number(id)),
            ("name", DataValue::String("火炎斬".into())),
            ("element", DataValue::String("火".into())),
            ("power", DataValue::Number(50.0)),
            ("cost", DataValue::Number(10.0)),
        ])
    }

    fn quest(id: f64) -> DataValue {
        row(&[
            ("id", DataValue::Number(id)),
            ("name", DataValue::String("川越のはじまり".into())),
            ("type", DataValue::String("main".into())),
            ("level", DataValue::Number(1.0)),
        ])
    }

    fn recipe(id: f64) -> DataValue {
        row(&[
            ("id", DataValue::Number(id)),
            ("name", DataValue::String("焼き芋".into())),
            ("category", DataValue::String("料理".into())),
            ("result_item_id", DataValue::Number(2001.0)),
            ("quantity", DataValue::Number(1.0)),
        ])
    }

    fn valid_game_data() -> GameData {
        GameData::new(
            DataStore::new(DataValue::Array(vec![enemy(1001.0)])),
            DataStore::new(DataValue::Array(vec![item(2001.0)])),
            DataStore::new(DataValue::Array(vec![skill(3001.0)])),
            DataStore::new(DataValue::Array(vec![quest(4001.0)])),
            DataStore::new(DataValue::Array(vec![recipe(5001.0)])),
        )
    }

    #[test]
    fn creates_game_data() {
        let data = valid_game_data();

        assert!(data.enemy(&DataValue::Number(1001.0)).unwrap().is_some());
        assert!(data.item(&DataValue::Number(2001.0)).unwrap().is_some());
    }

    #[test]
    fn validates_game_data() {
        assert!(valid_game_data().validate().is_ok());
    }

    #[test]
    fn loads_and_validates_all_game_data_from_directory() {
        let directory = std::env::temp_dir().join("yaoyorozu-data-game-data-valid-test");
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();

        fs::write(
            directory.join("enemies.json"),
            r#"[{"id":1001,"name":"狐","hp":100,"element":"火","level":1}]"#,
        )
        .unwrap();
        fs::write(
            directory.join("items.json"),
            r#"[{"id":2001,"name":"川越芋","category":"食材","price":100}]"#,
        )
        .unwrap();
        fs::write(
            directory.join("skills.json"),
            r#"[{"id":3001,"name":"火炎斬","element":"火","power":50,"cost":10}]"#,
        )
        .unwrap();
        fs::write(
            directory.join("quests.json"),
            r#"[{"id":4001,"name":"川越のはじまり","type":"main","level":1}]"#,
        )
        .unwrap();
        fs::write(
            directory.join("recipes.json"),
            r#"[{"id":5001,"name":"焼き芋","category":"料理","result_item_id":2001,"quantity":1}]"#,
        )
        .unwrap();

        let data = GameData::load_and_validate_from_dir(&directory).unwrap();

        assert!(data.enemy(&DataValue::Number(1001.0)).unwrap().is_some());
        assert!(data.skill(&DataValue::Number(3001.0)).unwrap().is_some());

        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn invalid_game_data_is_rejected_during_load() {
        let directory = std::env::temp_dir().join("yaoyorozu-data-game-data-invalid-test");
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();

        fs::write(
            directory.join("enemies.json"),
            r#"[{"id":1001,"name":"狐","element":"火","level":1}]"#,
        )
        .unwrap();
        fs::write(directory.join("items.json"), "[]").unwrap();
        fs::write(directory.join("skills.json"), "[]").unwrap();
        fs::write(directory.join("quests.json"), "[]").unwrap();
        fs::write(directory.join("recipes.json"), "[]").unwrap();

        let result = GameData::load_and_validate_from_dir(&directory);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("必須項目"));

        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn missing_game_data_returns_error() {
        let directory = std::env::temp_dir().join("yaoyorozu-data-missing-game-data");
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();

        let result = GameData::load_from_dir(&directory);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("ゲームデータがありません"));

        let _ = fs::remove_dir_all(directory);
    }
}
