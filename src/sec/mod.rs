pub mod client;
pub mod documents;
pub mod edgar;
pub mod http;
pub mod models;
pub mod output;
pub mod parsers;
pub mod pipeline;
pub mod registry;
pub mod search;
pub mod storage;

pub use client::SecClient;
pub use edgar::accession_text_url;
pub use models::{
    DocumentQuery, FactQuery, FilingQuery, Form4Query, OutputMode, ParseQuery, SearchQuery,
    ThirteenFQuery,
};
pub use output::print_records;
pub use registry::supported_parsers;
pub use search::find_matches;
