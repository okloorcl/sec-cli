use serde::Serialize;

#[derive(Debug, Clone)]
pub struct MetricsQuery {
    pub cik: u64,
    pub form: Option<String>,
    pub unit: Option<String>,
    pub latest: usize,
}

#[derive(Debug, Clone)]
pub struct HealthScoreQuery {
    pub cik: u64,
    pub form: Option<String>,
    pub unit: Option<String>,
    pub latest: usize,
}

#[derive(Debug, Serialize)]
pub struct FinancialMetricRecord {
    pub cik: u64,
    pub company: Option<String>,
    pub metric: String,
    pub category: String,
    pub value: Option<f64>,
    pub display_value: Option<String>,
    pub unit: String,
    pub fiscal_year: Option<i64>,
    pub fiscal_period: Option<String>,
    pub form: Option<String>,
    pub period_end: Option<String>,
    pub calculation: String,
    pub components: Vec<MetricComponentRecord>,
    pub source_urls: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct MetricComponentRecord {
    pub line_item: String,
    pub statement: String,
    pub value: Option<f64>,
    pub unit: String,
    pub accession: Option<String>,
    pub fact_id: Option<String>,
    pub source_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HealthScoreRecord {
    pub cik: u64,
    pub company: Option<String>,
    pub score_name: String,
    pub score: Option<f64>,
    pub max_score: Option<f64>,
    pub rating: String,
    pub fiscal_year: Option<i64>,
    pub fiscal_period: Option<String>,
    pub form: Option<String>,
    pub period_end: Option<String>,
    pub calculation: String,
    pub signals: Vec<HealthScoreSignalRecord>,
    pub source_urls: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct HealthScoreSignalRecord {
    pub name: String,
    pub passed: Option<bool>,
    pub points: f64,
    pub max_points: f64,
    pub value: Option<f64>,
    pub threshold: String,
    pub calculation: String,
    pub source_urls: Vec<String>,
}
