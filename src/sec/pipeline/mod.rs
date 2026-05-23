use anyhow::{Result, anyhow};

use super::{
    client::SecClient,
    models::{Form4Query, ParseQuery, ParsedRecord, ThirteenFQuery},
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
