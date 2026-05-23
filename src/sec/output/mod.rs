use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

use super::models::OutputMode;

pub fn print_records<T: Serialize>(records: &[T], mode: OutputMode) -> Result<()> {
    print!("{}", records_to_string(records, mode)?);
    Ok(())
}

pub(crate) fn records_to_string<T: Serialize>(records: &[T], mode: OutputMode) -> Result<String> {
    match mode {
        OutputMode::Json => json_string(records, false),
        OutputMode::PrettyJson => json_string(records, true),
        OutputMode::Csv => csv_string(records),
        OutputMode::Table => table_string(records),
        OutputMode::JsonLines => {
            let mut out = String::new();
            for record in records {
                out.push_str(&serde_json::to_string(record)?);
                out.push('\n');
            }
            Ok(out)
        }
    }
}

fn csv_string<T: Serialize>(records: &[T]) -> Result<String> {
    let rows = value_rows(records)?;
    let headers = headers(&rows);
    let mut out = String::new();
    out.push_str(
        &headers
            .iter()
            .map(|header| csv_cell(header))
            .collect::<Vec<_>>()
            .join(","),
    );
    out.push('\n');
    for row in rows {
        out.push_str(
            &headers
                .iter()
                .map(|header| csv_cell(&cell_value(&row, header)))
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push('\n');
    }
    Ok(out)
}

fn table_string<T: Serialize>(records: &[T]) -> Result<String> {
    let rows = value_rows(records)?;
    let headers = headers(&rows);
    if headers.is_empty() {
        return Ok("\n".to_string());
    }
    let table_rows = rows
        .iter()
        .map(|row| {
            headers
                .iter()
                .map(|header| cell_value(row, header))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let widths = column_widths(&headers, &table_rows);
    let mut out = String::new();
    push_table_row(&mut out, &headers, &widths);
    push_separator(&mut out, &widths);
    for row in table_rows {
        push_table_row(&mut out, &row, &widths);
    }
    Ok(out)
}

fn value_rows<T: Serialize>(records: &[T]) -> Result<Vec<serde_json::Map<String, Value>>> {
    let mut rows = Vec::with_capacity(records.len());
    for record in records {
        rows.push(match serde_json::to_value(record)? {
            Value::Object(map) => map,
            value => {
                let mut map = serde_json::Map::new();
                map.insert("value".to_string(), value);
                map
            }
        });
    }
    Ok(rows)
}

fn headers(rows: &[serde_json::Map<String, Value>]) -> Vec<String> {
    let mut headers = Vec::new();
    for row in rows {
        for key in row.keys() {
            if !headers.iter().any(|existing| existing == key) {
                headers.push(key.clone());
            }
        }
    }
    headers
}

fn cell_value(row: &serde_json::Map<String, Value>, key: &str) -> String {
    row.get(key).map(format_value).unwrap_or_default()
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(value) => value.clone(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(value).unwrap_or_default(),
    }
}

fn csv_cell(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn column_widths(headers: &[String], rows: &[Vec<String>]) -> Vec<usize> {
    headers
        .iter()
        .enumerate()
        .map(|(idx, header)| {
            rows.iter()
                .filter_map(|row| row.get(idx))
                .map(|value| display_width(value).min(80))
                .chain([display_width(header)])
                .max()
                .unwrap_or(0)
        })
        .collect()
}

fn push_table_row(out: &mut String, row: &[String], widths: &[usize]) {
    out.push('|');
    for (idx, width) in widths.iter().enumerate() {
        let value = row.get(idx).map(String::as_str).unwrap_or("");
        out.push(' ');
        out.push_str(&truncate_cell(value, *width));
        out.push_str(&" ".repeat(width.saturating_sub(display_width(value).min(*width))));
        out.push(' ');
        out.push('|');
    }
    out.push('\n');
}

fn push_separator(out: &mut String, widths: &[usize]) {
    out.push('|');
    for width in widths {
        out.push_str(&"-".repeat(*width + 2));
        out.push('|');
    }
    out.push('\n');
}

fn truncate_cell(value: &str, width: usize) -> String {
    if display_width(value) <= width {
        return value.to_string();
    }
    let mut out = value
        .chars()
        .take(width.saturating_sub(1))
        .collect::<String>();
    out.push('…');
    out
}

fn display_width(value: &str) -> usize {
    value.chars().count()
}

fn json_string<T: Serialize + ?Sized>(value: &T, pretty: bool) -> Result<String> {
    if pretty {
        Ok(format!("{}\n", serde_json::to_string_pretty(value)?))
    } else {
        Ok(format!("{}\n", serde_json::to_string(value)?))
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::*;

    #[derive(Serialize)]
    struct Row {
        value: u64,
    }

    #[test]
    fn renders_json_array() {
        let out = records_to_string(&[Row { value: 7 }], OutputMode::Json).unwrap();
        assert_eq!(out, r#"[{"value":7}]"#.to_string() + "\n");
    }

    #[test]
    fn renders_json_lines() {
        let out = records_to_string(&[Row { value: 1 }, Row { value: 2 }], OutputMode::JsonLines)
            .unwrap();
        assert_eq!(out, "{\"value\":1}\n{\"value\":2}\n");
    }

    #[test]
    fn renders_csv() {
        let out = records_to_string(&[Row { value: 7 }], OutputMode::Csv).unwrap();
        assert_eq!(out, "value\n7\n");
    }

    #[test]
    fn renders_table() {
        let out = records_to_string(&[Row { value: 7 }], OutputMode::Table).unwrap();
        assert!(out.contains("| value |"));
        assert!(out.contains("| 7     |"));
    }
}
