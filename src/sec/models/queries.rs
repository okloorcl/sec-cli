use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct FilingQuery {
    pub cik: u64,
    pub form: Option<String>,
    pub latest: usize,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub include_amends: bool,
}

#[derive(Debug, Clone)]
pub struct FactQuery {
    pub cik: u64,
    pub concept: String,
    pub form: Option<String>,
    pub unit: Option<String>,
    pub latest: usize,
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub query: String,
    pub context: usize,
}

#[derive(Debug, Clone)]
pub struct Form4Query {
    pub cik: u64,
    pub latest: usize,
    pub include_amends: bool,
}

#[derive(Debug, Clone)]
pub struct ThirteenFQuery {
    pub cik: u64,
    pub latest: usize,
    pub include_amends: bool,
}

#[derive(Debug, Clone)]
pub struct ParseQuery {
    pub cik: u64,
    pub form: String,
    pub latest: usize,
    pub include_amends: bool,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputMode {
    Json,
    PrettyJson,
    JsonLines,
}
