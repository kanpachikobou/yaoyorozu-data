use std::collections::BTreeMap;

use crate::{DataError, DataValue, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    Any,
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

impl FieldType {
    fn matches(&self, value: &DataValue) -> bool {
        matches!(
            (self, value),
            (Self::Any, _)
                | (Self::Null, DataValue::Null)
                | (Self::Bool, DataValue::Bool(_))
                | (Self::Number, DataValue::Number(_))
                | (Self::String, DataValue::String(_))
                | (Self::Array, DataValue::Array(_))
                | (Self::Object, DataValue::Object(_))
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldRule {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
}

impl FieldRule {
    pub fn required(name: impl Into<String>, field_type: FieldType) -> Self {
        Self {
            name: name.into(),
            field_type,
            required: true,
        }
    }

    pub fn optional(name: impl Into<String>, field_type: FieldType) -> Self {
        Self {
            name: name.into(),
            field_type,
            required: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    name: String,
    fields: Vec<FieldRule>,
}

impl Schema {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
        }
    }

    pub fn field(mut self, rule: FieldRule) -> Self {
        self.fields.push(rule);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fields(&self) -> &[FieldRule] {
        &self.fields
    }

    pub fn validate(&self, value: &DataValue) -> Result<()> {
        let rows = match value {
            DataValue::Array(rows) => rows,
            _ => {
                return Err(DataError::Message(format!(
                    "スキーマ「{}」の検証対象は配列である必要があります。",
                    self.name
                )));
            }
        };

        for (index, row) in rows.iter().enumerate() {
            self.validate_row(index, row)?;
        }

        Ok(())
    }

    fn validate_row(&self, index: usize, row: &DataValue) -> Result<()> {
        let object = match row {
            DataValue::Object(object) => object,
            _ => {
                return Err(DataError::Message(format!(
                    "スキーマ「{}」の{}行目はオブジェクトである必要があります。",
                    self.name,
                    index + 1
                )));
            }
        };

        for rule in &self.fields {
            match object.get(&rule.name) {
                None if rule.required => {
                    return Err(DataError::Message(format!(
                        "スキーマ「{}」の{}行目に必須項目「{}」がありません。",
                        self.name,
                        index + 1,
                        rule.name
                    )));
                }
                None => {}
                Some(value) if !rule.field_type.matches(value) => {
                    return Err(DataError::Message(format!(
                        "スキーマ「{}」の{}行目の「{}」の型が不正です。",
                        self.name,
                        index + 1,
                        rule.name
                    )));
                }
                Some(_) => {}
            }
        }

        Ok(())
    }
}

pub fn validate(value: &DataValue, schema: &Schema) -> Result<()> {
    schema.validate(value)
}

pub fn game_enemy_schema() -> Schema {
    Schema::new("敵データ")
        .field(FieldRule::required("id", FieldType::Number))
        .field(FieldRule::required("name", FieldType::String))
        .field(FieldRule::required("hp", FieldType::Number))
        .field(FieldRule::required("element", FieldType::String))
        .field(FieldRule::required("level", FieldType::Number))
}

pub fn game_item_schema() -> Schema {
    Schema::new("アイテムデータ")
        .field(FieldRule::required("id", FieldType::Number))
        .field(FieldRule::required("name", FieldType::String))
        .field(FieldRule::required("category", FieldType::String))
        .field(FieldRule::required("price", FieldType::Number))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object(fields: BTreeMap<String, DataValue>) -> DataValue {
        DataValue::Object(fields)
    }

    fn enemy() -> DataValue {
        object(BTreeMap::from([
            ("id".into(), DataValue::Number(1001.0)),
            ("name".into(), DataValue::String("狐".into())),
            ("hp".into(), DataValue::Number(100.0)),
            ("element".into(), DataValue::String("火".into())),
            ("level".into(), DataValue::Number(1.0)),
        ]))
    }

    #[test]
    fn validates_game_enemy_schema() {
        let data = DataValue::Array(vec![enemy()]);
        assert!(game_enemy_schema().validate(&data).is_ok());
    }

    #[test]
    fn rejects_missing_required_field() {
        let mut row = enemy();
        row.remove("hp").unwrap();

        let data = DataValue::Array(vec![row]);
        let result = game_enemy_schema().validate(&data);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("hp"));
    }

    #[test]
    fn rejects_wrong_field_type() {
        let mut row = enemy();
        row.set("hp", DataValue::String("100".into())).unwrap();

        let data = DataValue::Array(vec![row]);
        let result = game_enemy_schema().validate(&data);

        assert!(result.is_err());
    }

    #[test]
    fn optional_field_is_allowed() {
        let schema = Schema::new("テスト")
            .field(FieldRule::required("id", FieldType::Number))
            .field(FieldRule::optional("note", FieldType::String));

        let data = DataValue::Array(vec![object(BTreeMap::from([(
            "id".into(),
            DataValue::Number(1.0),
        )]))]);

        assert!(schema.validate(&data).is_ok());
    }
}
