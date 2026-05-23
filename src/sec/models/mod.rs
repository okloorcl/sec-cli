pub mod prospectus;
pub mod proxy;
pub mod queries;
pub mod records;
pub mod tables;

pub use prospectus::{ProspectusExcerptRecord, ProspectusRecord, ProspectusTableRecord};
pub use proxy::{ProxyProposalRecord, ProxyStatementRecord, ProxyTableRecord};
pub use queries::{
    DocumentQuery, DocumentReadQuery, EightKQuery, FactQuery, FilingQuery, Form4Query,
    HtmlTableQuery, InlineXbrlQuery, OutputMode, ParseQuery, ProspectusQuery, ProxyQuery,
    ReportQuery, Schedule13Query, SearchQuery, SectionQuery, StatementQuery, ThirteenFQuery,
};
pub use records::{
    DocumentContentRecord, DocumentRecord, EightKEventRecord, FactRecord, FilingRecord,
    FinancialStatementRecord, Form4FootnoteRecord, Form4OwnerRecord, Form4ReportRecord,
    Form4SignatureRecord, Form4TransactionRecord, InlineXbrlFactRecord, ParsedRecord,
    ResolveCandidateRecord, ResolveValidationRecord, Schedule13Record, SearchMatch, SectionRecord,
    ThirteenFAggregateHoldingRecord, ThirteenFDiffRecord, ThirteenFHoldingRecord,
    ThirteenFOtherManagerRecord, ThirteenFReportRecord,
};
pub use tables::HtmlTableRecord;
