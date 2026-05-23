use quick_xml::{Reader, events::Event};

pub(crate) fn read_xml<F>(xml: &str, mut handle: F) -> anyhow::Result<()>
where
    F: FnMut(XmlEvent) -> anyhow::Result<()>,
{
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    loop {
        match reader.read_event()? {
            Event::Start(event) => handle(XmlEvent::Start(local_name(event.local_name())))?,
            Event::End(event) => handle(XmlEvent::End(local_name(event.local_name())))?,
            Event::Text(event) => {
                let text = event.decode()?.trim().to_string();
                if !text.is_empty() {
                    handle(XmlEvent::Text(text))?;
                }
            }
            Event::CData(event) => {
                let text = event.decode()?.trim().to_string();
                if !text.is_empty() {
                    handle(XmlEvent::Text(text))?;
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(())
}

pub(crate) enum XmlEvent {
    Start(String),
    End(String),
    Text(String),
}

pub(crate) fn path_ends_with(path: &[String], suffix: &[&str]) -> bool {
    if path.len() < suffix.len() {
        return false;
    }
    path[path.len() - suffix.len()..]
        .iter()
        .zip(suffix)
        .all(|(left, right)| left == right)
}

pub(crate) fn parse_f64(value: &str) -> Option<f64> {
    value.replace(',', "").parse().ok()
}

pub(crate) fn parse_u64(value: &str) -> Option<u64> {
    value.replace(',', "").parse().ok()
}

fn local_name(name: impl AsRef<[u8]>) -> String {
    String::from_utf8_lossy(name.as_ref()).into_owned()
}
