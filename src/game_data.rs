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
    /// 個別のDataStoreからゲームデータを構築する。
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

    /// ディレクトリ内の標準ファイル名からゲームデータを一括ロードする。
    ///
    /// enemies.json / items.json / skills.json / quests.json / recipes.json を読み込む。
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

    /// すべてのゲームデータを対応するスキーマで検証する。
    pub fn validate(&self) -> Result<()> {
        self.enemies.validate(&crate::validate::game_enemy_schema())?;
        self.items.validate(&crate::validate::game_item_schema())?;
        Ok(())
    }

    /// 敵データをIDで取得する。
    pub fn enemy(&self, id: &DataValue) -> Result<Option<&DataValue>> {
        self.enemies.find_by_id(id)
    }

    /// アイテムデータをIDで取得する。
    pub fn item(&self, id: &DataValue) -> Result<Option<&DataValue>> {
        self.items.find_by_id(id)
    }

    /// スキルデータをIDで取得する。
    pub fn skill(&self, id: &DataValue) -> Result<Option<&DataValue>> {
        self.skills.find_by_id(id)
    }

    /// クエストデータをIDで取得する。
    pub fn quest(&self, id: &DataValue) -> Result<Option<&DataValue>> {
        self.quests.find_by_id(id)
    }

    /// レシピデータをIDで取得する。
    pub fn recipe(&self, id: &DataValue) -> Result<Option<&DataValue>> {
        self.recipes.find_by_id(id)
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

    fn empty_store() -> DataStore {
        DataStore::new(DataValue::Array(Vec::new()))
    }

    #[test]
    fn creates_game_data() {
        let data = GameData::new(
            DataStore::new(DataValue::Array(vec![enemy(1001.0)])),
            DataStore::new(DataValue::Array(vec![item(2001.0)])),
            empty_store(),
            empty_store(),
            empty_store(),
        );

        assert!(data.enemy(&DataValue::Number(1001.0)).unwrap().is_some());
        assert!(data.item(&DataValue::Number(2001.0)).unwrap().is_some());
    }

    #[test]
    fn validates_game_data() {
        let data = GameData::new(
            DataStore::new(DataValue::Array(vec![enemy(1001.0)])),
            DataStore::new(DataValue::Array(vec![item(2001.0)])),
            empty_store(),
            empty_store(),
            empty_store(),
        );

        assert!(data.validate().is_ok());
    }

    #[test]
    fn loads_all_game_data_from_directory() {
        let directory = std::env::temp_dir().join("yaoyorozu-data-game-data-test");
        let _ = fs::create_dir_all(&directory);

        fs::write(directory.join("enemies.json"), r#"[{"id":1001,"name":"狐","hp":100,"element":"火","level":1}]"#).unwrap();
        fs::write(directory.join("items.json"), r#"[{"id":2001,"name":"川越芋","category":"食材","price":100}]"#).unwrap();
        fs::write(directory.join("skills.json"), "[]").unwrap();
        fs::write(directory.join("quests.json"), "[]").unwrap();
        fs::write(directory.join("recipes.json"), "[]").unwrap();

        let data = GameData::load_from_dir(&directory).unwrap();

        assert!(data.enemy(&DataValue::Number(1001.0)).unwrap().is_some());
        assert!(data.item(&DataValue::Number(2001.0)).unwrap().is_some());

        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn missing_game_data_returns_error() {
        let directory = std::env::temp_dir().join("yaoyorozu-data-missing-game-data");
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();

        let result = GameData::load_from_dir(&directory);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("ゲームデータがありません"));

        let _ = fs::remove_dir_all(directory);
    }
}
