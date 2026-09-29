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

/// DataValue の配列から、指定フィールドに一致するすべてのオブジェクトを取得する。
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

    fn enemy(name: &str, hp: f64) -> DataValue {
        DataValue::Object(BTreeMap::from([
            ("name".to_string(), DataValue::String(name.to_string())),
            ("hp".to_string(), DataValue::Number(hp)),
        ]))
    }

    #[test]
    fn finds_first_matching_row() {
        let data = DataValue::Array(vec![
            enemy("狐", 100.0),
            enemy("鬼", 250.0),
        ]);

        let result = find(
            &data,
            "name",
            &DataValue::String("鬼".to_string()),
        )
        .unwrap();

        assert_eq!(result, Some(match &data { DataValue::Array(rows) => &rows[1], _ => unreachable!() }));
    }

    #[test]
    fn finds_all_matching_rows() {
        let data = DataValue::Array(vec![
            enemy("狐", 100.0),
            enemy("鬼", 250.0),
            enemy("鬼", 300.0),
        ]);

        let result = find_all(
            &data,
            "name",
            &DataValue::String("鬼".to_string()),
        )
        .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], match &data { DataValue::Array(rows) => &rows[1], _ => unreachable!() });
        assert_eq!(result[1], match &data { DataValue::Array(rows) => &rows[2], _ => unreachable!() });
    }

    #[test]
    fn returns_none_when_not_found() {
        let data = DataValue::Array(vec![enemy("狐", 100.0)]);

        let result = find(
            &data,
            "name",
            &DataValue::String("鬼".to_string()),
        )
        .unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn rejects_non_array_data() {
        let data = enemy("狐", 100.0);

        assert!(find(
            &data,
            "name",
            &DataValue::String("狐".to_string()),
        )
        .is_err());
    }
}
