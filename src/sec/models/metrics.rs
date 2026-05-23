use serde::Serialize;

#[derive(Debug, Clone)]
pub struct MetricsQuery {
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
