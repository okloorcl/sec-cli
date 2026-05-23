# sec-cli Roadmap

This roadmap tracks implemented SEC data sources, structured parsers, and major
agent/API milestones. Completed work is checked off and should stay linked to
real commands or modules in the repository.

## Done

- [x] SEC ticker to CIK mapping.
- [x] SEC submissions JSON filing metadata.
- [x] SEC CompanyFacts XBRL fact lookup.
- [x] 10-K / 10-Q financial statements builder.
- [x] Inline XBRL streaming parser.
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
- [x] Schedule 13D / 13G parser.
- [x] DEF 14A proxy statement parser.
- [x] Shared parser utilities, cache TTL policy, and hot-path parser performance cleanup.
- [x] Investor/fund/person resolver with LLM fallback and SEC validation.
- [x] Source-backed Markdown reports for insider activity, portfolio activity, and risk review.
- [x] Cross-platform CI for Linux amd64/i686/arm64/arm32, Windows amd64, and macOS arm64.

## Planned

- [ ] S-1 / 424B IPO and prospectus parser.
- [ ] 20-F / 6-K / 40-F foreign issuer parser.
- [ ] N-PORT / N-CSR / N-CEN fund disclosure parser.
- [ ] Local HTTP API.
- [ ] MCP adapter for agent tools.

## Suggested Implementation Order

1. S-1 / 424B parser.
2. 20-F / 6-K / 40-F parser.
3. N-PORT / N-CSR / N-CEN parser.
4. Local HTTP API and MCP adapter.

## Completion Rule

For each major feature:

1. Implement the parser/query/report surface.
2. Add focused tests.
3. Update `README.md`, `README.zh-CN.md`, and this roadmap.
4. Run `cargo fmt`, `cargo test`, and `cargo check`.
5. Commit the feature separately.
