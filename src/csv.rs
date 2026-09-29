use std::collections::BTreeMap;

use crate::{DataError, DataValue, Result};

/// CSV文字列を DataValue::Array に変換する。
///
/// 先頭行を列名として扱い、各データ行を Object に変換する。
/// 値は可能な範囲で null / bool / number として解釈し、
/// それ以外は文字列として保持する。
pub fn from_str(input: &str) -> Result<DataValue> {
    let mut reader = csv::Reader::from_reader(input.as_bytes());

    let headers = reader
        .headers()
        .map_err(|error| DataError::Message(error.to_string()))?
        .clone();

    let mut rows = Vec::new();

    for record in reader.records() {
        let record = record.map_err(|error| DataError::Message(error.to_string()))?;

        if record.len() != headers.len() {
            return Err(DataError::Message(format!(
                "CSVの列数が一致しません。ヘッダー: {}列、データ: {}列",
                headers.len(),
                record.len()
            )));
        }

        let mut object = BTreeMap::new();

        for (header, value) in headers.iter().zip(record.iter()) {
            object.insert(header.to_string(), parse_value(value));
        }

        rows.push(DataValue::Object(object));
    }

    Ok(DataValue::Array(rows))
}

/// DataValue::Array をCSV文字列へ変換する。
///
/// 配列内の Object のキーを列名として使用する。
/// 列順は最初のObjectに現れるキーの順序ではなく、
/// BTreeMapによる安定した辞書順になる。
pub fn to_string(value: &DataValue) -> Result<String> {
    let rows = match value {
        DataValue::Array(rows) => rows,
        _ => {
            return Err(DataError::Message(
                "CSVへ変換するデータは配列である必要があります。".to_string(),
            ));
        }
    };

    if rows.is_empty() {
        return Ok(String::new());
    }

    let objects = rows
        .iter()
        .map(|row| match row {
            DataValue::Object(object) => Ok(object),
            _ => Err(DataError::Message(
                "CSVの各行はオブジェクトである必要があります。".to_string(),
            )),
        })
        .collect::<Result<Vec<_>>>()?;

    let mut headers = BTreeMap::new();

    for object in &objects {
        for key in object.keys() {
            headers.insert(key.clone(), ());
        }
    }

    let headers: Vec<String> = headers.into_keys().collect();

    let mut writer = csv::Writer::from_writer(Vec::new());

    writer
        .write_record(&headers)
        .map_err(|error| DataError::Message(error.to_string()))?;

    for object in objects {
        let values = headers
            .iter()
            .map(|header| value_to_csv_string(object.get(header)))
            .collect::<Vec<_>>();

        writer
            .write_record(values)
            .map_err(|error| DataError::Message(error.to_string()))?;
    }

    let bytes = writer
        .into_inner()
        .map_err(|error| DataError::Message(error.to_string()))?;

    String::from_utf8(bytes).map_err(|error| DataError::Message(error.to_string()))
}

fn parse_value(value: &str) -> DataValue {
    if value.is_empty() {
        return DataValue::Null;
    }

    match value {
        "true" => DataValue::Bool(true),
        "false" => DataValue::Bool(false),
        _ => match value.parse::<f64>() {
            Ok(number) => DataValue::Number(number),
            Err(_) => DataValue::String(value.to_string()),
        },
    }
}

fn value_to_csv_string(value: Option<&DataValue>) -> String {
    match value {
        None | Some(DataValue::Null) => String::new(),
        Some(DataValue::Bool(value)) => value.to_string(),
        Some(DataValue::Number(value)) => value.to_string(),
        Some(DataValue::String(value)) => value.clone(),
        Some(DataValue::Array(value)) => format!("{value:?}"),
        Some(DataValue::Object(value)) => format!("{value:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_round_trip() {
        let input = "id,name,hp,element\n1,狐,100,木\n2,鬼,250,火\n";
        let value = from_str(input).expect("CSVの読み込みに失敗しました。");

        let output = to_string(&value).expect("CSVの書き出しに失敗しました。");
        let restored = from_str(&output).expect("CSVの再読み込みに失敗しました。");

        assert_eq!(value, restored);
    }

    #[test]
    fn csv_types_are_detected() {
        let input = "name,hp,active,note\n狐,100,true,妖怪\n";
        let value = from_str(input).expect("CSVの読み込みに失敗しました。");

        let DataValue::Array(rows) = value else {
            panic!("配列ではありません。");
        };

        let DataValue::Object(row) = &rows[0] else {
            panic!("オブジェクトではありません。");
        };

        assert_eq!(row.get("name"), Some(&DataValue::String("狐".to_string())));
        assert_eq!(row.get("hp"), Some(&DataValue::Number(100.0)));
        assert_eq!(row.get("active"), Some(&DataValue::Bool(true)));
        assert_eq!(row.get("note"), Some(&DataValue::String("妖怪".to_string())));
    }
}
