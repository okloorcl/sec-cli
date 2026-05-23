use quick_xml::{Reader, XmlVersion, events::Event};

pub(crate) fn read_xml<F>(xml: &str, mut handle: F) -> anyhow::Result<()>
where
    F: FnMut(XmlEvent) -> anyhow::Result<()>,
{
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut text_buffer = String::new();

    loop {
        match reader.read_event()? {
            Event::Start(event) => {
                flush_text(&mut text_buffer, &mut handle)?;
                handle(XmlEvent::Start(local_name(event.local_name())))?;
            }
            Event::End(event) => {
                flush_text(&mut text_buffer, &mut handle)?;
                handle(XmlEvent::End(local_name(event.local_name())))?;
            }
            Event::Text(event) => {
                text_buffer.push_str(event.decode()?.as_ref());
            }
            Event::CData(event) => {
                text_buffer.push_str(event.decode()?.as_ref());
            }
            Event::GeneralRef(event) => {
                text_buffer.push_str(&resolve_ref(&event)?);
            }
            Event::Eof => break,
            _ => {}
        }
    }
    flush_text(&mut text_buffer, &mut handle)?;

    Ok(())
}

pub(crate) enum XmlEvent {
    Start(String),
    End(String),
    Text(String),
}

pub(crate) fn read_xml_with_attrs<F>(xml: &str, mut handle: F) -> anyhow::Result<()>
where
    F: FnMut(XmlEventWithAttrs) -> anyhow::Result<()>,
{
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut text_buffer = String::new();

    loop {
        match reader.read_event()? {
            Event::Start(event) => {
                flush_text_with_attrs(&mut text_buffer, &mut handle)?;
                let attributes = event_attributes(&event)?;
                handle(XmlEventWithAttrs::Start {
                    name: local_name(event.local_name()),
                    attributes,
                })?;
            }
            Event::Empty(event) => {
                flush_text_with_attrs(&mut text_buffer, &mut handle)?;
                let name = local_name(event.local_name());
                let attributes = event_attributes(&event)?;
                handle(XmlEventWithAttrs::Start {
                    name: name.clone(),
                    attributes,
                })?;
                handle(XmlEventWithAttrs::End(name))?;
            }
            Event::End(event) => {
                flush_text_with_attrs(&mut text_buffer, &mut handle)?;
                handle(XmlEventWithAttrs::End(local_name(event.local_name())))?;
            }
            Event::Text(event) => {
                text_buffer.push_str(event.decode()?.as_ref());
            }
            Event::CData(event) => {
                text_buffer.push_str(event.decode()?.as_ref());
            }
            Event::GeneralRef(event) => {
                text_buffer.push_str(&resolve_ref(&event)?);
            }
            Event::Eof => break,
            _ => {}
        }
    }
    flush_text_with_attrs(&mut text_buffer, &mut handle)?;

    Ok(())
}

pub(crate) enum XmlEventWithAttrs {
    Start {
        name: String,
        attributes: Vec<XmlAttribute>,
    },
    End(String),
    Text(String),
}

pub(crate) struct XmlAttribute {
    pub(crate) name: String,
    pub(crate) value: String,
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

fn event_attributes(
    event: &quick_xml::events::BytesStart<'_>,
) -> anyhow::Result<Vec<XmlAttribute>> {
    let mut attributes = Vec::new();
    for attr in event.attributes().with_checks(false) {
        let attr = attr?;
        attributes.push(XmlAttribute {
            name: local_name(attr.key.local_name()),
            value: attr
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, event.decoder())?
                .into_owned(),
        });
    }
    Ok(attributes)
}

fn flush_text<F>(buffer: &mut String, handle: &mut F) -> anyhow::Result<()>
where
    F: FnMut(XmlEvent) -> anyhow::Result<()>,
{
    let text = buffer.trim();
    if !text.is_empty() {
        handle(XmlEvent::Text(text.to_string()))?;
    }
    buffer.clear();
    Ok(())
}

fn flush_text_with_attrs<F>(buffer: &mut String, handle: &mut F) -> anyhow::Result<()>
where
    F: FnMut(XmlEventWithAttrs) -> anyhow::Result<()>,
{
    let text = buffer.trim();
    if !text.is_empty() {
        handle(XmlEventWithAttrs::Text(text.to_string()))?;
    }
    buffer.clear();
    Ok(())
}

fn resolve_ref(event: &quick_xml::events::BytesRef<'_>) -> anyhow::Result<String> {
    if let Some(ch) = event.resolve_char_ref()? {
        return Ok(ch.to_string());
    }
    Ok(match event.decode()?.as_ref() {
        "amp" => "&".to_string(),
        "lt" => "<".to_string(),
        "gt" => ">".to_string(),
        "apos" => "'".to_string(),
        "quot" => "\"".to_string(),
        other => format!("&{other};"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_text_split_by_entity_references() {
        let mut texts = Vec::new();
        read_xml("<root><name>H&amp;H</name></root>", |event| {
            if let XmlEvent::Text(text) = event {
                texts.push(text);
            }
            Ok(())
        })
        .unwrap();

        assert_eq!(texts, vec!["H&H"]);
    }
}
