pub mod company;
pub mod foreign;
pub mod funds;
pub mod metrics;
pub mod prospectus;
pub mod proxy;
pub mod queries;
pub mod records;
pub mod tables;

pub use company::{CompanyReportRecord, CompanyTopicTableRecord};
pub use foreign::{ForeignExcerptRecord, ForeignIssuerRecord};
pub use funds::{FundDisclosureRecord, FundExcerptRecord, FundHoldingRecord, FundProxyVoteRecord};
pub use metrics::{FinancialMetricRecord, MetricComponentRecord, MetricsQuery};
pub use prospectus::{ProspectusExcerptRecord, ProspectusRecord, ProspectusTableRecord};
pub use proxy::{ProxyProposalRecord, ProxyStatementRecord, ProxyTableRecord};
pub use queries::{
    CompanyReportQuery, DocumentQuery, DocumentReadQuery, EightKExhibitQuery, EightKQuery,
    FactQuery, FilingQuery, ForeignIssuerQuery, Form4Query, FundDisclosureQuery, HtmlTableQuery,
    InlineXbrlQuery, OutputMode, ParseQuery, ProspectusQuery, ProxyQuery, ReportQuery,
    Schedule13Query, SearchQuery, SectionQuery, StatementQuery, ThirteenFQuery,
};
pub use records::{
    DocumentContentRecord, DocumentRecord, EightKEventRecord, EightKExhibitRecord, FactRecord,
    FilingRecord, FinancialStatementRecord, Form4FootnoteRecord, Form4OwnerRecord,
    Form4ReportRecord, Form4SignatureRecord, Form4TransactionRecord, InlineXbrlFactRecord,
    ParsedRecord, ResolveCandidateRecord, ResolveValidationRecord, Schedule13Record, SearchMatch,
    SectionRecord, ThirteenFAggregateHoldingRecord, ThirteenFDiffRecord, ThirteenFHoldingRecord,
    ThirteenFOtherManagerRecord, ThirteenFReportRecord,
};
pub use tables::HtmlTableRecord;
