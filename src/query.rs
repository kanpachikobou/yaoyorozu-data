use crate::{DataError, DataValue, Result};

/// DataValue の配列から、オブジェクトの指定フィールドを検索する。
pub fn find<'a>(value: &'a DataValue, field: &str, expected: &DataValue) -> Result<Option<&'a DataValue>> {
    let rows = match value {
        DataValue::Array(rows) => rows,
        _ => {
            return Err(DataError::Message(
                "検索対象は配列である必要があります。".to_string(),
            ));
        }
    };

    for row in rows {
        if let DataValue::Object(object) = row {
            if object.get(field) == Some(expected) {
                return Ok(Some(row));
            }
        }
    }

    Ok(None)
}

/// "id" フィールドが一致する最初のデータを取得する。
pub fn find_by_id<'a>(value: &'a DataValue, id: &DataValue) -> Result<Option<&'a DataValue>> {
    find(value, "id", id)
}

/// DataValue の配列から、指定フィールドに一致する最初のオブジェクトを削除する。
pub fn remove_first(
    value: &mut DataValue,
    field: &str,
    expected: &DataValue,
) -> Result<Option<DataValue>> {
    let rows = match value {
        DataValue::Array(rows) => rows,
        _ => {
            return Err(DataError::Message(
                "検索対象は配列である必要があります。".to_string(),
            ));
        }
    };

    let index = rows.iter().position(|row| match row {
        DataValue::Object(object) => object.get(field) == Some(expected),
        _ => false,
    });

    Ok(index.map(|index| rows.remove(index)))
}

/// "id" フィールドが一致する最初のデータを削除する。
pub fn remove_by_id(value: &mut DataValue, id: &DataValue) -> Result<Option<DataValue>> {
    remove_first(value, "id", id)
}

/// DataValue の配列から、指定フィールドに一致する最初のデータを変更可能な形で取得する。
pub fn find_mut<'a>(
    value: &'a mut DataValue,
    field: &str,
    expected: &DataValue,
) -> Result<Option<&'a mut DataValue>> {
    let rows = match value {
        DataValue::Array(rows) => rows,
        _ => {
            return Err(DataError::Message(
                "検索対象は配列である必要があります。".to_string(),
            ));
        }
    };

    for row in rows.iter_mut() {
        if let DataValue::Object(object) = row {
            if object.get(field) == Some(expected) {
                return Ok(Some(row));
            }
        }
    }

    Ok(None)
}

/// "id" フィールドが一致する最初のデータを変更可能な形で取得する。
pub fn find_by_id_mut<'a>(
    value: &'a mut DataValue,
    id: &DataValue,
) -> Result<Option<&'a mut DataValue>> {
    find_mut(value, "id", id)
}

/// DataValue の配列から、指定フィールドに一致するすべてのオブジェクトを取得する。
pub fn find_all<'a>(
    value: &'a DataValue,
    field: &str,
    expected: &DataValue,
) -> Result<Vec<&'a DataValue>> {
    let rows = match value {
        DataValue::Array(rows) => rows,
        _ => {
            return Err(DataError::Message(
                "検索対象は配列である必要があります。".to_string(),
            ));
        }
    };

    Ok(rows
        .iter()
        .filter(|row| match row {
            DataValue::Object(object) => object.get(field) == Some(expected),
            _ => false,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn enemy(id: f64, name: &str, hp: f64) -> DataValue {
        DataValue::Object(BTreeMap::from([
            ("id".to_string(), DataValue::Number(id)),
            ("name".to_string(), DataValue::String(name.to_string())),
            ("hp".to_string(), DataValue::Number(hp)),
        ]))
    }

    #[test]
    fn finds_first_matching_row() {
        let data = DataValue::Array(vec![
            enemy(1.0, "狐", 100.0),
            enemy(2.0, "鬼", 250.0),
        ]);

        let result = find(&data, "name", &DataValue::String("鬼".to_string())).unwrap();

        match &data {
            DataValue::Array(rows) => assert_eq!(result, Some(&rows[1])),
            _ => unreachable!(),
        }
    }

    #[test]
    fn finds_all_matching_rows() {
        let data = DataValue::Array(vec![
            enemy(1.0, "狐", 100.0),
            enemy(2.0, "鬼", 250.0),
            enemy(3.0, "鬼", 300.0),
        ]);

        let result = find_all(&data, "name", &DataValue::String("鬼".to_string())).unwrap();

        assert_eq!(result.len(), 2);
        match &data {
            DataValue::Array(rows) => {
                assert_eq!(result[0], &rows[1]);
                assert_eq!(result[1], &rows[2]);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn finds_by_id() {
        let data = DataValue::Array(vec![
            enemy(1001.0, "狐", 100.0),
            enemy(1002.0, "鬼", 250.0),
        ]);

        let result = find_by_id(&data, &DataValue::Number(1002.0)).unwrap();

        assert_eq!(
            result.and_then(|row| row.get("name")).and_then(DataValue::as_str),
            Some("鬼")
        );
    }

    #[test]
    fn finds_and_modifies_by_id() {
        let mut data = DataValue::Array(vec![
            enemy(1001.0, "狐", 100.0),
            enemy(1002.0, "鬼", 250.0),
        ]);

        let result = find_by_id_mut(&mut data, &DataValue::Number(1002.0)).unwrap().unwrap();
        result.set("hp", DataValue::Number(500.0)).unwrap();

        let result = find_by_id(&data, &DataValue::Number(1002.0)).unwrap().unwrap();
        assert_eq!(result.get("hp").and_then(DataValue::as_f64), Some(500.0));
    }

    #[test]
    fn removes_by_id() {
        let mut data = DataValue::Array(vec![
            enemy(1001.0, "狐", 100.0),
            enemy(1002.0, "鬼", 250.0),
        ]);

        let removed = remove_by_id(&mut data, &DataValue::Number(1001.0)).unwrap();

        assert!(removed.is_some());
        assert!(find_by_id(&data, &DataValue::Number(1001.0)).unwrap().is_none());
    }

    #[test]
    fn returns_none_when_not_found() {
        let data = DataValue::Array(vec![enemy(1.0, "狐", 100.0)]);

        let result = find(&data, "name", &DataValue::String("鬼".to_string())).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn rejects_non_array_data() {
        let data = enemy(1.0, "狐", 100.0);

        assert!(find(&data, "name", &DataValue::String("狐".to_string())).is_err());
    }
}
