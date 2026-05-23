use anyhow::{Result, anyhow};

use super::{
    client::SecClient,
    models::{
        EightKQuery, Form4Query, ParseQuery, ParsedRecord, ProxyQuery, Schedule13Query,
        ThirteenFQuery,
    },
    registry::{ParserKind, parser_for_form},
};

impl SecClient {
    pub async fn parse_form(&self, query: ParseQuery) -> Result<Vec<ParsedRecord>> {
        let parser = parser_for_form(&query.form)
            .ok_or_else(|| anyhow!("unsupported parsed form '{}'", query.form))?;

        let mut records: Vec<ParsedRecord> = match parser.kind {
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
