use anyhow::{Result, anyhow};

use super::{
    client::SecClient,
    models::{
        EightKQuery, Form4Query, ParseQuery, ParsedRecord, ProspectusQuery, ProxyQuery,
        Schedule13Query, ThirteenFQuery,
    },
    registry::{ParserKind, parser_for_form},
};

impl SecClient {
    pub async fn parse_form(&self, query: ParseQuery) -> Result<Vec<ParsedRecord>> {
        let parser_kind = parser_kind_for_form(&query.form)?;

        let mut records: Vec<ParsedRecord> = match parser_kind {
            ParserKind::Form4 => self
                .form4_transactions(Form4Query {
                    cik: query.cik,
                    latest: query.latest,
                    include_amends: query.include_amends,
                })
                .await?
                .into_iter()
                .map(ParsedRecord::Form4Transaction)
                .collect(),
            ParserKind::EightK => self
                .eightk_events(EightKQuery {
                    cik: query.cik,
                    latest: query.latest,
                    include_amends: query.include_amends,
                    item: None,
                    limit_bytes: None,
                })
                .await?
                .into_iter()
                .map(ParsedRecord::EightKEvent)
                .collect(),
            ParserKind::Schedule13 => self
                .schedule13_reports(Schedule13Query {
                    cik: query.cik,
                    form: Some(query.form.clone()),
                    latest: query.latest,
                    include_amends: query.include_amends,
                    limit_bytes: None,
                })
                .await?
                .into_iter()
                .map(ParsedRecord::Schedule13)
                .collect(),
            ParserKind::Proxy => self
                .proxy_statements(ProxyQuery {
                    cik: query.cik,
                    latest: query.latest,
                    include_amends: query.include_amends,
                    limit_rows: None,
                })
                .await?
                .into_iter()
                .map(ParsedRecord::ProxyStatement)
                .collect(),
            ParserKind::Prospectus => self
                .prospectuses(ProspectusQuery {
                    cik: query.cik,
                    form: Some(query.form.clone()),
                    latest: query.latest,
                    include_amends: query.include_amends,
                    limit_bytes: None,
                    limit_tables: None,
                    limit_rows: None,
                })
                .await?
                .into_iter()
                .map(ParsedRecord::Prospectus)
                .collect(),
            ParserKind::ThirteenF => self
                .thirteenf_holdings(ThirteenFQuery {
                    cik: query.cik,
                    latest: query.latest,
                    include_amends: query.include_amends,
                })
                .await?
                .into_iter()
                .map(ParsedRecord::ThirteenfHolding)
                .collect(),
        };

        if let Some(limit) = query.limit {
            records.truncate(limit);
        }
        Ok(records)
    }
}

fn parser_kind_for_form(form: &str) -> Result<ParserKind> {
    parser_for_form(form)
        .map(|parser| parser.kind)
        .ok_or_else(|| anyhow!("unsupported parsed form '{}'", form))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_parser_kind_for_supported_forms() {
        assert_eq!(parser_kind_for_form("4").unwrap(), ParserKind::Form4);
        assert_eq!(parser_kind_for_form("8-K/A").unwrap(), ParserKind::EightK);
        assert_eq!(
            parser_kind_for_form("SC 13G").unwrap(),
            ParserKind::Schedule13
        );
        assert_eq!(parser_kind_for_form("DEF 14A").unwrap(), ParserKind::Proxy);
        assert_eq!(parser_kind_for_form("S-1").unwrap(), ParserKind::Prospectus);
        assert_eq!(
            parser_kind_for_form("424B4").unwrap(),
            ParserKind::Prospectus
        );
        assert_eq!(
            parser_kind_for_form("424B").unwrap(),
            ParserKind::Prospectus
        );
        assert_eq!(
            parser_kind_for_form("13F-HR").unwrap(),
            ParserKind::ThirteenF
        );
    }

    #[test]
    fn rejects_unsupported_forms() {
        assert!(parser_kind_for_form("10-K").is_err());
    }
}
