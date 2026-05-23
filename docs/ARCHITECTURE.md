# sec-cli Architecture

`sec-cli` is a Rust SEC disclosure engine with a CLI shell. The core is built as
a library so the same pipeline can later serve a CLI, HTTP API, MCP server,
batch worker, or embedded agent tool.

## Design Goal

The project should optimize for three things before adding many forms:

- Clear boundaries between fetching, storage, document discovery, parsing, and output.
- Stable machine-readable records that preserve SEC source lineage.
- Form parsers that can be added independently without changing the whole system.

## Layer Map

```text
CLI / future API / future MCP
        |
        v
query + parser pipeline
        |
        v
EDGAR client + index APIs
        |
        v
HTTP transport + cache/store
        |
        v
submission/document layer
        |
        v
form parsers / table parsers / XBRL parsers
        |
        v
domain records -> JSON / JSONL / future Arrow / Parquet
```

## Current Modules

```text
src/lib.rs                          library entry point
src/main.rs                         minimal binary entry point
src/cli/                            CLI shell split from the SEC core
src/cli/args.rs                     CLI argument schema
src/cli/runner.rs                   CLI orchestration only
src/mcp/                            stdio MCP adapter over the SEC core
src/server/                         local JSON HTTP API over the SEC core
src/sec/mod.rs                      public SEC module surface
src/sec/client/                     SEC domain client facade and cached fetch entry point
src/sec/http/                       low-level SEC HTTP transport
src/sec/storage/                    local cache/store abstraction with TTL-aware reads
src/sec/edgar/                      SEC data sources and URL builders
src/sec/edgar/filings.rs            submissions index -> FilingRecord
src/sec/edgar/facts.rs              CompanyFacts -> FactRecord
src/sec/company/                    10-K/10-Q company-report topic table parser
src/sec/llm/                        OpenAI/Anthropic-compatible model clients
src/sec/resolve/                    LLM name resolution plus SEC verification
src/sec/documents/                  submission and attachment/document selection
src/sec/documents/submission.rs     complete-submission.txt -> SubmissionDocument[]
src/sec/documents/selectors.rs      primary XML, ownership XML, 13F table selectors
src/sec/documents/records.rs        document inventory records for CLI/API use
src/sec/parsers/                    shared parser machinery and form parsers
src/sec/proxy/                      DEF 14A proxy statement parser
src/sec/prospectus/                 S-1/F-1/424B prospectus parser
src/sec/foreign/                    20-F/6-K/40-F foreign issuer parser
src/sec/funds/                      N-PORT/N-CSR/N-CEN fund disclosure parser
src/sec/metrics/                    SEC-derived financial metrics and secondary analysis
src/sec/utils.rs                    shared string, legal suffix, and truncation helpers
src/sec/parsers/xml.rs              streaming XML helpers
src/sec/parsers/forms/              form-specific parsers
src/sec/models/                     query models and output records
src/sec/pipeline/                   unified form dispatch for supported parsed forms
src/sec/registry/                   supported form parser registry
src/sec/search/                     filing text search
src/sec/output/                     JSON / JSONL rendering
```

## Form Parser Rule

Each SEC form family should have its own parser module when it has distinct
semantics. A form parser owns domain meaning; shared table/XML/HTML helpers own
mechanics.

Examples:

- `forms/form4.rs`: Forms 3/4/5 ownership XML family can eventually share an ownership parser.
- `forms/thirteenf.rs`: 13F-HR / amendments / 13F-NT family.
- `company`: deeper 10-K/10-Q topic tables for segments, geography, debt, obligations, leases, taxes, and repurchases.
- `eightk`: 8-K item extraction plus exhibit discovery and earnings-release classification.
- `prospectus`: S-1 / F-1 / 424B offering and IPO signals.
- `foreign`: 20-F / 6-K / 40-F foreign issuer annual/current reports.
- `funds`: N-PORT portfolio holdings, N-CSR shareholder reports, and N-CEN fund census data.
- `proxy`: DEF 14A compensation and governance tables.
- `schedule13`: SC 13D / SC 13G beneficial ownership.
- Future `forms/fund_votes.rs`: N-PX voting records, 497K summaries, and 24F-2NT notices.

Not every HTML table needs a separate top-level parser. The right split is:

- Form parser: knows regulatory structure and output schema.
- Document parser: knows SGML attachments and primary documents.
- Table parser: knows HTML/XML table extraction.
- XBRL parser: knows facts, contexts, units, dimensions, and statement links.

## Data Sources

Primary SEC sources:

- `company_tickers.json`: ticker/name to CIK reference.
- `submissions/CIK##########.json`: recent and historical filing metadata.
- `companyfacts/CIK##########.json`: normalized XBRL facts by concept.
- Financial metrics are derived locally from CompanyFacts statement rows; no paid market-data API is used.
- `Archives/edgar/data/.../*.txt`: complete submission text with all documents.
- Filing index pages and archive attachments: primary documents, exhibits, XBRL files.
- Future daily index feeds: high-volume discovery and monitoring.
- Future bulk archives: companyfacts/submissions zip for offline mode.
- Future EFTS/full-text search: global text search.

Optional model sources:

- OpenAI-compatible chat completions endpoint for name resolution.
- Anthropic-compatible messages endpoint for name resolution.

The model is never the final authority. It proposes candidate filing managers;
SEC submissions and 13F filings validate whether the candidate is usable.

## Pipeline Contracts

Every parsed record should include enough provenance for agents and audits:

- CIK and company/manager/issuer identity where available.
- Accession number.
- Filing date and report period where available.
- Document filename, sequence, and description.
- Source URL.
- Stable record kind and field names.

## Why This Beats A Monolith

This structure keeps high-churn parsing logic isolated. Adding a new form should
usually mean:

1. Add one parser module under `forms/`.
2. Add output records to `models.rs` or a future `records/` module.
3. Register the parser in `pipeline.rs`.
4. Add a thin CLI command only if the form deserves first-class ergonomics.
5. Add fixtures and parser tests.

The CLI stays boring. The parser layer does the domain work. Backends and MCP
adapters should call the same library entry points that the CLI calls.

## Implemented Core Boundaries

The current codebase already has these boundaries in place:

- `http`: HTTP only. It knows about user-agent headers and SEC responses.
- `storage`: cache only. It persists bytes by URL-derived keys and supports TTL-aware reads.
- `client`: EDGAR domain facade. It combines HTTP/storage, owns cached fetch behavior, and exposes operations.
- `edgar`: source-specific API handling for submissions, facts, and archive URLs.
- `company`: deeper 10-K/10-Q topic-table classification over primary filing tables.
- `documents`: SEC SGML container scanning and attachment selection.
- `llm`: protocol adapters for OpenAI-compatible and Anthropic-compatible models.
- `resolve`: public-name resolution that turns model candidates into SEC-verified records.
- `metrics`: secondary analysis over CompanyFacts-derived statement rows.
- `reports`: Markdown reports for financial trends, insider activity, 13F portfolios, and risk review.
- `parsers`: reusable XML parser helpers and form-specific regulatory parsers.
- `models`: query DTOs and stable output record schemas.
- `registry`: parser discovery and supported form families.
- `pipeline`: runtime dispatch from `form` to parser.
- `mcp`: JSON-RPC stdio adapter exposing a small, stable tool surface for agents.
- `server`: local HTTP API that reuses the same client and parser records as the CLI.

This means future CLI commands, HTTP handlers, MCP tools, and batch jobs should
not talk directly to `reqwest`, file caches, or parser internals.

## Near-Term Build Order

1. Split storage into a trait-backed cache/store layer if an alternate backend is needed.
2. Add export layer for Arrow/Parquet.
