use std::env;
use std::fs;

use yaoyorozu_data::{csv, json, DataError, Result};

fn main() {
    if let Err(error) = run() {
        eprintln!("エラー: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        print_usage();
        return Ok(());
    }

    let input_path = &args[1];
    let output_path = &args[2];

    let input = fs::read_to_string(input_path)
        .map_err(|error| DataError::Message(format!("入力ファイルを読めません: {error}")))?;

    let input_format = detect_format(input_path)?;
    let output_format = detect_format(output_path)?;

    let data = match input_format.as_str() {
        "csv" => csv::from_str(&input)?,
        "json" => json::from_str(&input)?,
        _ => unreachable!(),
    };

    let output = match output_format.as_str() {
        "csv" => csv::to_string(&data)?,
        "json" => json::to_string_pretty(&data)?,
        _ => unreachable!(),
    };

    fs::write(output_path, output)
        .map_err(|error| DataError::Message(format!("出力ファイルを書き込めません: {error}")))?;

    println!("{input_path} → {output_path}");
    Ok(())
}

fn detect_format(path: &str) -> Result<String> {
    let extension = std::path::Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| {
            DataError::Message(format!("ファイル形式を判定できません: {path}"))
        })?;

    match extension.as_str() {
        "csv" | "json" => Ok(extension),
        _ => Err(DataError::Message(format!(
            "対応していないファイル形式です: .{extension}（csv / json のみ対応）"
        ))),
    }
}

fn print_usage() {
    println!("yaoyorozu-data");
    println!();
    println!("CSV / JSON を相互変換します。");
    println!();
    println!("使い方:");
    println!("  yaoyorozu-data <入力ファイル> <出力ファイル>");
    println!();
    println!("例:");
    println!("  yaoyorozu-data input.csv output.json");
    println!("  yaoyorozu-data input.json output.csv");
}

#[cfg(test)]
mod tests {
    use super::*;
    use yaoyorozu_data::DataValue;

    #[test]
    fn detects_csv_and_json() {
        assert_eq!(detect_format("data.csv").unwrap(), "csv");
        assert_eq!(detect_format("data.JSON").unwrap(), "json");
    }

    #[test]
    fn rejects_unknown_format() {
        assert!(detect_format("data.txt").is_err());
    }

    #[test]
    fn data_value_is_available_to_binary() {
        let value = DataValue::String("八百万".to_string());
        assert_eq!(value, DataValue::String("八百万".to_string()));
    }
}
