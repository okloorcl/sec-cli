use anyhow::Result;
use serde::Serialize;

use super::models::OutputMode;

pub fn print_records<T: Serialize>(records: &[T], mode: OutputMode) -> Result<()> {
    print!("{}", records_to_string(records, mode)?);
    Ok(())
}

pub(crate) fn records_to_string<T: Serialize>(records: &[T], mode: OutputMode) -> Result<String> {
    match mode {
        OutputMode::Json => json_string(records, false),
        OutputMode::PrettyJson => json_string(records, true),
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
}
