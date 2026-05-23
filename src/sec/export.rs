use std::{fs::File, path::Path, sync::Arc};

use anyhow::{Context, Result};
use arrow_array::{ArrayRef, RecordBatch, StringArray};
use arrow_ipc::writer::FileWriter;
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use parquet::arrow::ArrowWriter;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Arrow,
    Parquet,
}

pub fn export_records<T: Serialize>(
    records: &[T],
    format: ExportFormat,
    path: &Path,
) -> Result<usize> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create export directory {}", parent.display()))?;
    }

    let (schema, batch) = batch_from_records(records)?;
    match format {
        ExportFormat::Arrow => write_arrow(path, &schema, &batch)?,
        ExportFormat::Parquet => write_parquet(path, schema, &batch)?,
    }
    Ok(records.len())
}

fn write_arrow(path: &Path, schema: &SchemaRef, batch: &RecordBatch) -> Result<()> {
    let file =
        File::create(path).with_context(|| format!("failed to create {}", path.display()))?;
    let mut writer = FileWriter::try_new(file, schema)
        .with_context(|| format!("failed to start Arrow writer for {}", path.display()))?;
    writer
        .write(batch)
        .with_context(|| format!("failed to write Arrow batch to {}", path.display()))?;
    writer
        .finish()
        .with_context(|| format!("failed to finish Arrow file {}", path.display()))?;
    Ok(())
}

fn write_parquet(path: &Path, schema: SchemaRef, batch: &RecordBatch) -> Result<()> {
    let file =
        File::create(path).with_context(|| format!("failed to create {}", path.display()))?;
    let mut writer = ArrowWriter::try_new(file, schema, None)
        .with_context(|| format!("failed to start Parquet writer for {}", path.display()))?;
    writer
        .write(batch)
        .with_context(|| format!("failed to write Parquet batch to {}", path.display()))?;
    writer
        .close()
        .with_context(|| format!("failed to finish Parquet file {}", path.display()))?;
    Ok(())
}

fn batch_from_records<T: Serialize>(records: &[T]) -> Result<(SchemaRef, RecordBatch)> {
    let rows = value_rows(records)?;
    let headers = headers(&rows);
    if headers.is_empty() {
        return empty_batch();
    }

    let schema = Arc::new(Schema::new(
        headers
            .iter()
            .map(|header| Field::new(header, DataType::Utf8, true))
            .collect::<Vec<_>>(),
    ));
    let arrays = headers
        .iter()
        .map(|header| {
            let values = rows
                .iter()
                .map(|row| row.get(header).and_then(export_value))
                .collect::<Vec<_>>();
            Arc::new(StringArray::from(values)) as ArrayRef
        })
        .collect::<Vec<_>>();
    let batch = RecordBatch::try_new(schema.clone(), arrays)?;
    Ok((schema, batch))
}

fn empty_batch() -> Result<(SchemaRef, RecordBatch)> {
    let schema = Arc::new(Schema::new(vec![Field::new(
        "_empty",
        DataType::Utf8,
        true,
    )]));
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![Arc::new(StringArray::from(Vec::<Option<String>>::new()))],
    )?;
    Ok((schema, batch))
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

fn export_value(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(value) => Some(value.clone()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(value).ok(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde::Serialize;

    use super::*;

    #[derive(Serialize)]
    struct Row {
        cik: u64,
        label: &'static str,
    }

    #[test]
    fn writes_arrow_ipc_file() {
        let path = temp_path("arrow");
        export_records(
            &[Row {
                cik: 320193,
                label: "Apple",
            }],
            ExportFormat::Arrow,
            &path,
        )
        .unwrap();
        assert!(std::fs::metadata(&path).unwrap().len() > 0);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn writes_parquet_file() {
        let path = temp_path("parquet");
        export_records(
            &[Row {
                cik: 320193,
                label: "Apple",
            }],
            ExportFormat::Parquet,
            &path,
        )
        .unwrap();
        assert!(std::fs::metadata(&path).unwrap().len() > 0);
        std::fs::remove_file(path).unwrap();
    }

    fn temp_path(ext: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "sec-cli-export-{}-{}.{}",
            std::process::id(),
            ext,
            ext
        ))
    }
}
