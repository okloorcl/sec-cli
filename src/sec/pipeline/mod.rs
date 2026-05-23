use anyhow::{Result, anyhow};

use super::{
    client::SecClient,
    models::{
        CompanyReportQuery, EightKQuery, ForeignIssuerQuery, Form4Query, FundDisclosureQuery,
        ParseQuery, ParsedRecord, ProspectusQuery, ProxyQuery, Schedule13Query, ThirteenFQuery,
    },
    registry::{ParserKind, parser_for_form},
};

impl SecClient {
    pub async fn parse_form(&self, query: ParseQuery) -> Result<Vec<ParsedRecord>> {
        let parser_kind = parser_kind_for_form(&query.form)?;

        let mut records: Vec<ParsedRecord> = match parser_kind {
            ParserKind::CompanyReport => self
                .company_reports(CompanyReportQuery {
                    cik: query.cik,
                    form: Some(query.form.clone()),
                    latest: query.latest,
                    include_amends: query.include_amends,
                    topic: None,
                    limit_tables: query.limit,
                    limit_rows: Some(25),
                })
                .await?
                .into_iter()
                .map(ParsedRecord::CompanyReport)
                .collect(),
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
            ParserKind::ForeignIssuer => self
                .foreign_issuer_reports(ForeignIssuerQuery {
                    cik: query.cik,
                    form: Some(query.form.clone()),
                    latest: query.latest,
                    include_amends: query.include_amends,
                    limit_bytes: None,
                })
                .await?
                .into_iter()
                .map(ParsedRecord::ForeignIssuer)
                .collect(),
            ParserKind::FundDisclosure => self
                .fund_disclosures(FundDisclosureQuery {
                    cik: query.cik,
                    form: Some(query.form.clone()),
                    latest: query.latest,
                    include_amends: query.include_amends,
                    limit_holdings: query.limit,
                    limit_bytes: None,
                })
                .await?
                .into_iter()
                .map(ParsedRecord::FundDisclosure)
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
        assert_eq!(
            parser_kind_for_form("10-K").unwrap(),
            ParserKind::CompanyReport
        );
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
            parser_kind_for_form("20-F").unwrap(),
            ParserKind::ForeignIssuer
        );
        assert_eq!(
            parser_kind_for_form("6-K").unwrap(),
            ParserKind::ForeignIssuer
        );
        assert_eq!(
            parser_kind_for_form("NPORT-P").unwrap(),
            ParserKind::FundDisclosure
        );
        assert_eq!(
            parser_kind_for_form("N-CSR").unwrap(),
            ParserKind::FundDisclosure
        );
        assert_eq!(
            parser_kind_for_form("13F-HR").unwrap(),
            ParserKind::ThirteenF
        );
    }

    #[test]
    fn rejects_unsupported_forms() {
        assert!(parser_kind_for_form("SD").is_err());
    }
}
