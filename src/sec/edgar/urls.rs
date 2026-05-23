pub(crate) const SEC_BASE_URL: &str = "https://www.sec.gov";
pub(crate) const SEC_DATA_URL: &str = "https://data.sec.gov";

pub fn accession_index_url(cik: u64, accession: &str) -> String {
    format!(
        "{SEC_BASE_URL}/Archives/edgar/data/{}/{}/{}-index.html",
        cik,
        accession.replace('-', ""),
        accession
    )
}

pub fn accession_text_url(cik: u64, accession: &str) -> String {
    format!(
        "{SEC_BASE_URL}/Archives/edgar/data/{}/{}/{}.txt",
        cik,
        accession.replace('-', ""),
        accession
    )
}

pub fn accession_document_url(cik: u64, accession: &str, filename: &str) -> String {
    format!(
        "{SEC_BASE_URL}/Archives/edgar/data/{}/{}/{}",
        cik,
        accession.replace('-', ""),
        filename.trim_start_matches('/')
    )
}

pub(crate) fn company_tickers_url() -> String {
    format!("{SEC_BASE_URL}/files/company_tickers.json")
}

pub(crate) fn submissions_url(cik: u64) -> String {
    format!("{SEC_DATA_URL}/submissions/CIK{cik:010}.json")
}

pub(crate) fn company_facts_url(cik: u64) -> String {
    format!("{SEC_DATA_URL}/api/xbrl/companyfacts/CIK{cik:010}.json")
}
