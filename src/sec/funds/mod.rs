use std::sync::LazyLock;

use anyhow::Result;
use regex::Regex;

use crate::sec::{
    client::SecClient,
    documents::{DocumentSet, SubmissionDocument, read::plain_text},
    edgar::accession_document_url,
    models::{
        FilingQuery, FilingRecord, FundDisclosureQuery, FundDisclosureRecord, FundExcerptRecord,
        FundHoldingRecord, FundProxyVoteRecord,
    },
    parsers::{
        text_helpers,
        xml::{XmlEvent, parse_f64, read_xml},
    },
};

const FUND_FORMS: &[&str] = &[
    "NPORT-P",
    "NPORT-P/A",
    "N-PORT",
    "N-PORT/A",
    "N-CSR",
    "N-CSR/A",
    "N-CSRS",
    "N-CSRS/A",
    "N-CEN",
    "N-CEN/A",
    "N-PX",
    "N-PX/A",
    "497K",
    "497K/A",
    "24F-2NT",
    "24F-2NT/A",
];

impl SecClient {
    pub async fn fund_disclosures(
        &self,
        query: FundDisclosureQuery,
    ) -> Result<Vec<FundDisclosureRecord>> {
        let filings = self.fund_filings(&query).await?;
        let mut records = Vec::new();
        for filing in filings {
            let docs = self.filing_documents(&filing).await?;
            let Some(doc) = choose_fund_document(&docs) else {
                continue;
            };
            records.push(parse_fund_disclosure(&filing, doc, &query)?);
        }
        Ok(records)
    }

    async fn fund_filings(&self, query: &FundDisclosureQuery) -> Result<Vec<FilingRecord>> {
        let requested = query
            .form
            .as_deref()
            .filter(|value| !value.eq_ignore_ascii_case("all"));
        let fetch_latest = if requested.is_some() {
            query.latest
        } else {
            query.latest.saturating_mul(30).max(80)
        };
        let filings = self
            .filings(FilingQuery {
                cik: query.cik,
                form: requested.map(str::to_string),
                latest: fetch_latest,
                from: None,
                to: None,
                include_amends: query.include_amends,
            })
            .await?;
        let mut filtered = filings
            .into_iter()
            .filter(|filing| is_fund_form(&filing.form, query.include_amends))
            .collect::<Vec<_>>();
        filtered.truncate(query.latest);
        Ok(filtered)
    }
}

pub fn parse_fund_disclosure(
    filing: &FilingRecord,
    doc: &SubmissionDocument,
    query: &FundDisclosureQuery,
) -> Result<FundDisclosureRecord> {
    let xml = doc.xml_content();
    let xml_data = parse_fund_xml(xml, query.limit_holdings)?;
    let text = plain_text(&doc.content);
    let holdings_count = xml_data.holdings.len();
    let proxy_votes_count = xml_data.proxy_votes.len();

    Ok(FundDisclosureRecord {
        accession: filing.accession.clone(),
        cik: filing.cik,
        company: filing.company.clone(),
        form: filing.form.clone(),
        filing_date: filing.filing_date.clone(),
        disclosure_type: disclosure_type(&filing.form).to_string(),
        is_amendment: filing.form.ends_with("/A"),
        registrant_name: xml_data
            .registrant_name
            .or_else(|| capture_label(&text, "Registrant")),
        series_name: xml_data
            .series_name
            .or_else(|| capture_label(&text, "Series")),
        class_name: xml_data
            .class_name
            .or_else(|| capture_label(&text, "Class")),
        period_end: xml_data.period_end.or_else(|| capture_period(&text)),
        fiscal_year_end: xml_data.fiscal_year_end,
        total_assets: xml_data.total_assets,
        total_liabilities: xml_data.total_liabilities,
        net_assets: xml_data.net_assets,
        holdings_count,
        holdings: xml_data.holdings,
        proxy_votes_count,
        proxy_votes: xml_data.proxy_votes,
        shareholder_report: excerpt(&text, "Shareholder Report", query.limit_bytes),
        portfolio_summary: excerpt(&text, "Portfolio", query.limit_bytes),
        proxy_voting_record: excerpt(&text, "Proxy Voting", query.limit_bytes)
            .or_else(|| excerpt(&text, "Proxy Voting Record", query.limit_bytes)),
        summary_prospectus: excerpt(&text, "Summary Prospectus", query.limit_bytes),
        registration_fee_notice: excerpt(&text, "Registration Fee", query.limit_bytes)
            .or_else(|| excerpt(&text, "24F-2", query.limit_bytes)),
        financial_statements: excerpt(&text, "Financial Statements", query.limit_bytes),
        controls: excerpt(&text, "Controls and Procedures", query.limit_bytes),
        document: doc.filename.clone(),
        document_sequence: doc.sequence.clone(),
        document_description: doc.description.clone(),
        document_url: doc
            .filename
            .as_deref()
            .map(|filename| accession_document_url(filing.cik, &filing.accession, filename)),
        source_url: filing.source_url.clone(),
    })
}

fn choose_fund_document(docs: &[SubmissionDocument]) -> Option<&SubmissionDocument> {
    let set = DocumentSet::new(docs);
    set.primary_documents()
        .find(|doc| doc.content_type() == "xml")
        .or_else(|| docs.iter().find(|doc| doc.content_type() == "xml"))
        .or_else(|| set.primary_documents().next())
        .or_else(|| docs.first())
}

fn is_fund_form(form: &str, include_amends: bool) -> bool {
    FUND_FORMS
        .iter()
        .any(|candidate| form.eq_ignore_ascii_case(candidate))
        || include_amends
            && [
                "NPORT-P", "N-PORT", "N-CSR", "N-CSRS", "N-CEN", "N-PX", "497K", "24F-2NT",
            ]
            .iter()
            .any(|base| form.eq_ignore_ascii_case(&format!("{base}/A")))
}

fn disclosure_type(form: &str) -> &'static str {
    let normalized = form.to_ascii_uppercase();
    if normalized.starts_with("NPORT") || normalized.starts_with("N-PORT") {
        "portfolio_holdings"
    } else if normalized.starts_with("N-CEN") {
        "annual_fund_census"
    } else if normalized.starts_with("N-PX") {
        "proxy_voting_record"
    } else if normalized.starts_with("497K") {
        "summary_prospectus"
    } else if normalized.starts_with("24F-2NT") {
        "annual_notice_of_securities_sold"
    } else {
        "shareholder_report"
    }
}

#[derive(Default)]
struct FundXmlData {
    registrant_name: Option<String>,
    series_name: Option<String>,
    class_name: Option<String>,
    period_end: Option<String>,
    fiscal_year_end: Option<String>,
    total_assets: Option<f64>,
    total_liabilities: Option<f64>,
    net_assets: Option<f64>,
    holdings: Vec<FundHoldingRecord>,
    proxy_votes: Vec<FundProxyVoteRecord>,
}

fn parse_fund_xml(xml: &str, limit_holdings: Option<usize>) -> Result<FundXmlData> {
    let mut data = FundXmlData::default();
    let mut path = Vec::<String>::new();
    let mut current = None::<FundHoldingRecord>;
    let mut current_vote = None::<FundProxyVoteRecord>;

    read_xml(xml, |event| {
        match event {
            XmlEvent::Start(name) => {
                if name.eq_ignore_ascii_case("invstOrSec") {
                    current = Some(FundHoldingRecord::default());
                } else if is_vote_record_tag(&name) {
                    current_vote = Some(FundProxyVoteRecord::default());
                }
                path.push(name);
            }
            XmlEvent::Text(value) => {
                let Some(name) = path.last().map(|value| value.as_str()) else {
                    return Ok(());
                };
                if let Some(holding) = current.as_mut() {
                    assign_holding_field(holding, name, &value);
                } else if let Some(vote) = current_vote.as_mut() {
                    assign_vote_field(vote, name, &value);
                } else {
                    assign_fund_field(&mut data, name, &value);
                }
            }
            XmlEvent::End(name) => {
                if name.eq_ignore_ascii_case("invstOrSec")
                    && limit_holdings.is_none_or(|limit| data.holdings.len() < limit)
                    && let Some(holding) = current.take()
                {
                    data.holdings.push(holding);
                }
                if is_vote_record_tag(&name)
                    && limit_holdings.is_none_or(|limit| data.proxy_votes.len() < limit)
                    && let Some(vote) = current_vote.take()
                {
                    data.proxy_votes.push(vote);
                }
                path.pop();
            }
        }
        Ok(())
    })?;

    Ok(data)
}

fn is_vote_record_tag(name: &str) -> bool {
    matches!(
        name,
        "proxyVote" | "proxyVotingRecord" | "votingRecord" | "proxyVoteRecord"
    )
}

fn assign_fund_field(data: &mut FundXmlData, name: &str, value: &str) {
    match name {
        "regName" | "registrantName" => data.registrant_name = clean(value),
        "seriesName" => data.series_name = clean(value),
        "className" => data.class_name = clean(value),
        "repPdEnd" | "periodEnd" => data.period_end = clean(value),
        "fiscalYrEnd" | "fiscalYearEnd" => data.fiscal_year_end = clean(value),
        "totAssets" => data.total_assets = parse_f64(value),
        "totLiabs" => data.total_liabilities = parse_f64(value),
        "netAssets" => data.net_assets = parse_f64(value),
        _ => {}
    }
}

fn assign_holding_field(holding: &mut FundHoldingRecord, name: &str, value: &str) {
    match name {
        "name" => holding.name = clean(value),
        "title" => holding.title = clean(value),
        "cusip" => holding.cusip = clean(value),
        "lei" => holding.lei = clean(value),
        "balance" => holding.balance = parse_f64(value),
        "units" => holding.units = clean(value),
        "curCd" => holding.currency = clean(value),
        "valUSD" => holding.value_usd = parse_f64(value),
        "pctVal" => holding.pct_value = parse_f64(value),
        "assetCat" => holding.asset_category = clean(value),
        "issuerCat" => holding.issuer_category = clean(value),
        "invCountry" => holding.country = clean(value),
        "isRestrictedSec" => holding.is_restricted = parse_bool(value),
        "liquidityCat" => holding.liquidity_category = clean(value),
        _ => {}
    }
}

fn assign_vote_field(vote: &mut FundProxyVoteRecord, name: &str, value: &str) {
    match name {
        "issuerName" | "nameOfIssuer" | "issuer" => vote.issuer_name = clean(value),
        "cusip" => vote.cusip = clean(value),
        "meetingDate" | "voteDate" => vote.meeting_date = clean(value),
        "matter" | "voteDescription" | "proposal" => vote.matter = clean(value),
        "vote" | "howVoted" | "voteCast" => vote.vote_cast = clean(value),
        "managementRecommendation" | "mgmtRecommendation" => {
            vote.management_recommendation = clean(value)
        }
        "sharesVoted" | "shares" => vote.shares_voted = parse_f64(value),
        _ => {}
    }
}

fn capture_label(text: &str, label: &str) -> Option<String> {
    text_helpers::capture_label(text, label)
}

fn capture_period(text: &str) -> Option<String> {
    static PERIOD_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)(?:period|year)\s+ended\s+([A-Z][a-z]+\s+\d{1,2},\s+\d{4})")
            .expect("valid fund period regex")
    });
    PERIOD_RE
        .captures(text)
        .and_then(|capture| capture.get(1))
        .and_then(|m| clean(m.as_str()))
}

fn excerpt(text: &str, title: &str, limit_bytes: Option<usize>) -> Option<FundExcerptRecord> {
    let start = text_helpers::section_start(text, title, 0)?;
    let excerpt = text_helpers::excerpt_from_range(
        text,
        title,
        start,
        next_section_start(text, start + title.len()),
        limit_bytes,
    )?;
    Some(FundExcerptRecord {
        title: title.to_string(),
        byte_length: excerpt.byte_length,
        returned_bytes: excerpt.returned_bytes,
        truncated: excerpt.truncated,
        content: excerpt.content,
    })
}

fn next_section_start(text: &str, from: usize) -> Option<usize> {
    static SECTION_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"(?i)\b(Shareholder Report|Portfolio|Financial Statements|Controls and Procedures|Management Discussion|Item\s+\d+)\b",
        )
        .expect("valid fund section regex")
    });
    SECTION_RE.find(&text[from..]).map(|m| from + m.start())
}

fn clean(value: &str) -> Option<String> {
    text_helpers::clean_option(value)
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "yes" | "y" | "1" => Some(true),
        "false" | "no" | "n" | "0" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nport_xml_holdings() {
        let xml = r#"
        <edgarSubmission>
          <regName>Example Trust</regName>
          <seriesName>Example Growth Fund</seriesName>
          <repPdEnd>2026-03-31</repPdEnd>
          <totAssets>1000000</totAssets>
          <netAssets>900000</netAssets>
          <invstOrSec>
            <name>Apple Inc.</name><title>Common Stock</title><cusip>037833100</cusip>
            <balance>1000</balance><units>NS</units><curCd>USD</curCd><valUSD>200000</valUSD>
            <pctVal>22.2</pctVal><assetCat>EC</assetCat><issuerCat>CORP</issuerCat>
            <invCountry>US</invCountry><isRestrictedSec>false</isRestrictedSec>
          </invstOrSec>
        </edgarSubmission>"#;

        let data = parse_fund_xml(xml, Some(10)).unwrap();

        assert_eq!(data.registrant_name.as_deref(), Some("Example Trust"));
        assert_eq!(data.series_name.as_deref(), Some("Example Growth Fund"));
        assert_eq!(data.net_assets, Some(900000.0));
        assert_eq!(data.holdings.len(), 1);
        assert_eq!(data.holdings[0].cusip.as_deref(), Some("037833100"));
        assert_eq!(data.holdings[0].value_usd, Some(200000.0));
    }

    #[test]
    fn classifies_fund_disclosure_forms() {
        assert_eq!(disclosure_type("NPORT-P"), "portfolio_holdings");
        assert_eq!(disclosure_type("N-CEN"), "annual_fund_census");
        assert_eq!(disclosure_type("N-PX"), "proxy_voting_record");
        assert_eq!(disclosure_type("497K"), "summary_prospectus");
        assert_eq!(
            disclosure_type("24F-2NT"),
            "annual_notice_of_securities_sold"
        );
        assert_eq!(disclosure_type("N-CSR"), "shareholder_report");
    }

    #[test]
    fn parses_npx_proxy_vote_records() {
        let xml = r#"
        <edgarSubmission>
          <registrantName>Example Trust</registrantName>
          <proxyVote>
            <issuerName>Apple Inc.</issuerName>
            <cusip>037833100</cusip>
            <meetingDate>2026-02-01</meetingDate>
            <matter>Elect director</matter>
            <voteCast>For</voteCast>
            <managementRecommendation>For</managementRecommendation>
            <sharesVoted>1000</sharesVoted>
          </proxyVote>
        </edgarSubmission>"#;

        let data = parse_fund_xml(xml, Some(10)).unwrap();

        assert_eq!(data.registrant_name.as_deref(), Some("Example Trust"));
        assert_eq!(data.proxy_votes.len(), 1);
        assert_eq!(
            data.proxy_votes[0].issuer_name.as_deref(),
            Some("Apple Inc.")
        );
        assert_eq!(data.proxy_votes[0].vote_cast.as_deref(), Some("For"));
        assert_eq!(data.proxy_votes[0].shares_voted, Some(1000.0));
    }
}
