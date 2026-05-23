pub mod client;
pub mod documents;
pub mod edgar;
pub mod http;
pub mod investors;
pub mod models;
pub mod output;
pub mod parsers;
pub mod pipeline;
pub mod registry;
pub mod reports;
pub mod search;
pub mod sections;
pub mod storage;

pub use client::SecClient;
pub use edgar::accession_text_url;
pub use investors::{resolve_investor, search_investors};
pub use models::{
    DocumentQuery, DocumentReadQuery, FactQuery, FilingQuery, Form4Query, OutputMode, ParseQuery,
    ReportQuery, SearchQuery, SectionQuery, ThirteenFQuery,
};
pub use output::print_records;
pub use registry::supported_parsers;
pub use reports::ReportKind;
pub use search::find_matches;
