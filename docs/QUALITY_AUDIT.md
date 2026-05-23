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

## Still Intentional

- `SecClient` methods are implemented near their domain modules so each parser
  can own its query logic. The facade remains `sec::SecClient`; architecture
  docs list the module boundaries.
- CLI command dispatch still uses explicit match arms. Repeated simple command
  plumbing is being moved into `src/cli/handlers.rs` as command families become
  large enough to justify extraction.
