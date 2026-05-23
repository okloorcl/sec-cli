# sec-cli Roadmap

This roadmap tracks implemented SEC data sources, structured parsers, and major
agent/API milestones. Completed work is checked off and should stay linked to
real commands or modules in the repository.

## Done

- [x] SEC ticker to CIK mapping.
- [x] SEC submissions JSON filing metadata.
- [x] SEC CompanyFacts XBRL fact lookup.
- [x] 10-K / 10-Q financial statements builder.
- [x] 10-K / 10-Q company-report topic table parser.
- [x] Source-backed financial metrics and secondary analysis from CompanyFacts.
- [x] Financial trend Markdown report over SEC-derived metrics.
- [x] Inline XBRL streaming parser.
- [x] XBRL linkbase parser for presentation, calculation, definition, label, and schema attachments.
- [x] HTML table extraction.
- [x] SEC complete submission document listing and document reading.
- [x] SEC filing text search with source snippets.
- [x] 10-K / 10-Q section extraction for common items.
- [x] Form 3/4/5 transaction parser.
- [x] Form 4 report summary parser for owners, signatures, footnotes, and net activity.
- [x] Form 13F-HR information table parser.
- [x] Form 13F-HR aggregate holdings.
- [x] Form 13F-HR quarter-over-quarter diff.
- [x] Form 13F-HR primary document summary parser.
- [x] 8-K event parser.
- [x] 8-K exhibit discovery and earnings-release classification.
- [x] Schedule 13D / 13G parser.
- [x] DEF 14A proxy statement parser.
- [x] S-1 / F-1 / 424B IPO and prospectus parser.
- [x] 20-F / 6-K / 40-F foreign issuer parser.
- [x] N-PORT / N-CSR / N-CEN fund disclosure parser.
- [x] N-PX / 497K / 24F-2NT fund voting, summary prospectus, and securities-sold notice parser.
- [x] SEC daily master index scanner for all-market filing monitoring.
- [x] SEC EDGAR Full-Text Search / EFTS global search CLI and HTTP endpoint.
- [x] Shared parser utilities, cache TTL policy, and hot-path parser performance cleanup.
- [x] Core utility test coverage for search, reports, output, filings, facts, storage, and parser dispatch.
- [x] Investor/fund/person resolver with LLM fallback and SEC validation.
- [x] Source-backed Markdown reports for insider activity, portfolio activity, and risk review.
- [x] Local JSON HTTP API for core SEC queries and parser endpoints.
- [x] Stdio MCP adapter for core SEC agent tools.
- [x] Expanded MCP tools for daily index, EFTS, documents, sections, iXBRL, tables, proxy, prospectus, foreign issuer, funds, Form 4, 8-K, Schedule 13D/G, 13F, and Markdown reports.
- [x] Local SEC identity config and shell completion generation.
- [x] Terminal table and CSV output modes.
- [x] Typed SEC submissions/companyfacts response DTOs.
- [x] Mockable LLM resolver and cache-store abstractions.
- [x] Centralized disclosure name patterns for auditors and underwriters.
- [x] Cross-platform CI for Linux amd64/i686/arm64/arm32, Windows amd64, and macOS arm64.

## Planned

- [ ] Presentation-tree financial statement rendering from XBRL linkbases and filing facts.
- [ ] Calculation-linkbase validation for rendered statements.
- [ ] Expanded standard concept mapping for 100+ financial statement concepts.
- [ ] Financial metrics expansion toward 50+ ratios and financial-health scores.
- [ ] Cross-form 10-K / 10-Q financial statement stitching.
- [ ] Arrow / Parquet export layer.
- [ ] Optional bulk archive / offline mode.

## Suggested Implementation Order

1. Presentation-tree statement renderer over `xbrl-links` output.
2. Calculation-linkbase validation.
3. Expanded standard concept mapping.
4. Metrics and financial-health score expansion.
5. Cross-form statement stitching.
6. Arrow / Parquet export layer.
7. Optional bulk archive / offline mode.

## Completion Rule

For each major feature:

1. Implement the parser/query/report surface.
2. Add focused tests.
3. Update `README.md`, `README.zh-CN.md`, and this roadmap.
4. Run `cargo fmt`, `cargo test`, and `cargo check`.
5. Commit the feature separately.
