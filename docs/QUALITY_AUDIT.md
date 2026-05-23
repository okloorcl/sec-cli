# Quality Audit

This file tracks code-quality findings that were fixed after the parser
roadmap reached the DEF 14A milestone.

## Completed

- [x] Centralized UTF-8 truncation in `sec::utils::truncate_utf8`.
- [x] Centralized empty-string normalization in `sec::utils::nonempty`.
- [x] Reused the XML `path_ends_with` helper instead of duplicating it.
- [x] Unified legal suffix filtering through `LEGAL_SUFFIXES`.
- [x] Cached hot-path regular expressions with `LazyLock`.
- [x] Removed full-document lowercase copies from SGML document scanning.
- [x] Made search snippet tag stripping context-aware.
- [x] Added TTL-aware URL cache reads.
- [x] Deduplicated client cached-fetch logic.
- [x] Avoided mutating `ThirteenFQuery` inside diff generation.
- [x] Documented the 13F value-scale cutoff.
- [x] Tightened resolver company-name matching to avoid short-name false positives.
- [x] Preserved the original LLM parse error when repair fails.
- [x] Added focused tests for search matching, reports formatting, parser dispatch,
  output rendering, filing filters, fact aliases, and storage cache behavior.
- [x] Removed the placeholder default SEC User-Agent; commands now require
  `--identity`, `SEC_IDENTITY`, or `EDGAR_IDENTITY`.
- [x] Simplified thousands grouping and covered money/visual report formatting.
- [x] Made SGML `extract_tag` handle nested same-name tags.
- [x] Added SEC HTTP timeout, retry, exponential backoff, and 5 req/s pacing.
- [x] Concurrently fetch filing documents/text for multi-filing workflows.
- [x] Redacted LLM API keys from `Debug` output.
- [x] Split repeated disclosure parser text helpers into shared modules.
- [x] Added release profile optimization with thin LTO and symbol stripping.
- [x] Classified HTTP API errors into 400/404/502/500 responses.
- [x] Added terminal table and CSV output modes.
- [x] Cached `company_tickers.json` in memory after first parse.
- [x] Hardened LLM resolver prompts against prompt injection.
- [x] Reused one shared LLM HTTP client.
- [x] Removed repeated 13F fetches from portfolio report generation.
- [x] Restricted cache directory/file permissions on Unix.
- [x] Added local identity config and shell completion generation.
- [x] Expanded MCP tool coverage and split the MCP adapter into focused modules.
- [x] Split HTTP server error handling into a focused module.
- [x] Replaced ad hoc SEC submissions/companyfacts field access with typed DTOs.
- [x] Added `LlmResolver` and `CacheStore` traits for mockable resolver/cache tests.
- [x] Centralized auditor and underwriter recognition patterns.

## Still Intentional

- Some output records keep `serde_json::Value` for raw XBRL fact values because
  SEC facts can be numeric, string, boolean, or null. SEC response parsing now
  uses typed DTOs before records are emitted.
- Some financial ratios, percentages, scores, and share quantities remain `f64`
  where fractional math is expected. Money-like totals are being moved toward
  integer or source-backed representations as schemas evolve.
- `SecClient` methods are implemented near their domain modules so each parser
  can own its query logic. The facade remains `sec::SecClient`; architecture
  docs list the module boundaries.
- CLI command dispatch still uses explicit match arms. Repeated simple command
  plumbing is being moved into `src/cli/handlers.rs` as command families become
  large enough to justify extraction.
