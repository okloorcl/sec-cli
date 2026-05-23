use anyhow::Result;
use serde::Serialize;

use super::models::OutputMode;

pub fn print_records<T: Serialize>(records: &[T], mode: OutputMode) -> Result<()> {
    match mode {
        OutputMode::Json => print_json(records, false),
        OutputMode::PrettyJson => print_json(records, true),
        OutputMode::JsonLines => {
            for record in records {
                println!("{}", serde_json::to_string(record)?);
            }
            Ok(())
        }
    }
}

fn print_json<T: Serialize + ?Sized>(value: &T, pretty: bool) -> Result<()> {
    if pretty {
        println!("{}", serde_json::to_string_pretty(value)?);
    } else {
        println!("{}", serde_json::to_string(value)?);
    }
    Ok(())
}
