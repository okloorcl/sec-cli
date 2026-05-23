use anyhow::Result;

use crate::sec::{SecClient, parsers::forms::XmlEvent, parsers::forms::read_xml};

pub async fn find_13f_manager_cik(client: &SecClient, name: &str) -> Result<Option<u64>> {
    for query in search_variants(name) {
        if let Some(cik) = find_exact_13f_manager_cik(client, &query).await? {
            return Ok(Some(cik));
        }
    }
    Ok(None)
}

async fn find_exact_13f_manager_cik(client: &SecClient, name: &str) -> Result<Option<u64>> {
    let url = format!(
        "https://www.sec.gov/cgi-bin/browse-edgar?action=getcompany&company={}&type=13F-HR&owner=exclude&output=atom",
        form_encode(name)
    );
    let xml = client.get_text(&url).await?;
    Ok(parse_atom_13f_cik(&xml))
}

fn search_variants(name: &str) -> Vec<String> {
    let compact_ampersand = compact_ampersand(name);
    let mut variants = vec![name.trim().to_string()];
    if compact_ampersand != name.trim() {
        variants.push(compact_ampersand.clone());
    }
    let suffix_stripped = strip_legal_suffixes(&compact_ampersand);
    if !suffix_stripped.is_empty() && !variants.iter().any(|value| value == &suffix_stripped) {
        variants.push(suffix_stripped);
    }
    let simplified = simplify_company_name(&compact_ampersand);
    if !simplified.is_empty() && !variants.iter().any(|value| value == &simplified) {
        variants.push(simplified);
    }
    variants
}

fn compact_ampersand(name: &str) -> String {
    name.split('&').map(str::trim).collect::<Vec<_>>().join("&")
}

fn strip_legal_suffixes(name: &str) -> String {
    let stop_words = [
        "inc",
        "incorporated",
        "corp",
        "corporation",
        "co",
        "company",
        "llc",
        "ltd",
        "limited",
        "lp",
        "l.p",
        "group",
    ];
    name.split(|ch: char| ch == ',' || ch == '.')
        .flat_map(|part| part.split_whitespace())
        .filter(|part| {
            let normalized = part
                .trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '&')
                .to_ascii_lowercase();
            !normalized.is_empty() && !stop_words.contains(&normalized.as_str())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn simplify_company_name(name: &str) -> String {
    let stop_words = [
        "inc",
        "incorporated",
        "corp",
        "corporation",
        "co",
        "company",
        "llc",
        "ltd",
        "limited",
        "lp",
        "l.p",
        "group",
    ];
    name.replace('&', " ")
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|part| {
            let lowered = part.to_ascii_lowercase();
            !part.is_empty() && !stop_words.contains(&lowered.as_str())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_atom_13f_cik(xml: &str) -> Option<u64> {
    let mut path = Vec::new();
    let mut company_cik = None;
    read_xml(xml, |event| {
        match event {
            XmlEvent::Start(tag) => path.push(tag),
            XmlEvent::End(_) => {
                path.pop();
            }
            XmlEvent::Text(text) => {
                if path_ends_with(&path, &["company-info", "cik"]) {
                    company_cik = text.parse::<u64>().ok();
                }
            }
        }
        Ok(())
    })
    .ok()?;
    company_cik
}

fn form_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(*byte as char)
            }
            b' ' => encoded.push('+'),
            other => encoded.push_str(&format!("%{other:02X}")),
        }
    }
    encoded
}

fn path_ends_with(path: &[String], suffix: &[&str]) -> bool {
    if path.len() < suffix.len() {
        return false;
    }
    path[path.len() - suffix.len()..]
        .iter()
        .zip(suffix)
        .all(|(left, right)| left == right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_company_cik_when_feed_has_13f() {
        let xml = r#"
        <feed xmlns="http://www.w3.org/2005/Atom">
          <company-info><cik>0001759760</cik></company-info>
          <entry><content><filing-type>13F-HR</filing-type></content></entry>
        </feed>
        "#;
        assert_eq!(parse_atom_13f_cik(xml), Some(1759760));
    }

    #[test]
    fn encodes_company_names_for_browse_edgar() {
        assert_eq!(
            form_encode("H&H International Investment, LLC"),
            "H%26H+International+Investment%2C+LLC"
        );
    }

    #[test]
    fn creates_simplified_search_variant() {
        assert_eq!(
            simplify_company_name("H&H INTERNATIONAL INVESTMENT GROUP, LTD."),
            "H H INTERNATIONAL INVESTMENT"
        );
        assert_eq!(
            strip_legal_suffixes("H&H INTERNATIONAL INVESTMENT GROUP, LTD."),
            "H&H INTERNATIONAL INVESTMENT"
        );
        assert_eq!(
            search_variants("H & H International Investment LLC")[1],
            "H&H International Investment LLC"
        );
    }
}
