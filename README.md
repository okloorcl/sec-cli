# sec-cli

Agent-ready SEC EDGAR parser and query CLI, powered by Rust.

[![Rust](https://img.shields.io/badge/Rust-2024-orange)](https://www.rust-lang.org/)
[![CI](https://github.com/okloorcl/sec-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/okloorcl/sec-cli/actions/workflows/ci.yml)
[![Release](https://github.com/okloorcl/sec-cli/actions/workflows/release.yml/badge.svg)](https://github.com/okloorcl/sec-cli/actions/workflows/release.yml)
[![SEC EDGAR](https://img.shields.io/badge/Data-SEC%20EDGAR-blue)](https://www.sec.gov/edgar)
[![Output](https://img.shields.io/badge/Output-JSON%20%7C%20JSONL%20%7C%20Markdown-green)](#output-modes)
[![Agent Ready](https://img.shields.io/badge/Agent-ready-111827)](#agent-workflows)
[![LLM Resolver](https://img.shields.io/badge/LLM-OpenAI%20%7C%20Anthropic-7c3aed)](#llm-resolver)
[![中文](https://img.shields.io/badge/README-中文-red)](README.zh-CN.md)

| Core | What it gives you |
| --- | --- |
| Insider activity | Form 4 owner, role, transaction code, shares, price, value, footnotes, signatures |
| Institutional holdings | 13F holdings, portfolio summary, top positions, quarter-over-quarter changes |
| Company disclosure | 8-K events, 10-K/10-Q risk factors, MD&A, foreign issuer 20-F/6-K/40-F, filing search |
| Fund disclosure | N-PORT holdings, N-CSR shareholder reports, N-CEN census, N-PX votes, 497K summaries, 24F notices |
| Capital markets | S-1/F-1/424B prospectus terms, IPO signals, proceeds, risks, underwriters |
| Financial analysis | SEC-derived margins, growth, free cash flow, ROA/ROE, liquidity, leverage |
| Market monitoring | SEC daily master index scans across all new filings by date, form, and company |
| Global search | SEC EDGAR Full-Text Search (EFTS) across companies, forms, dates, and CIKs |
| Agent interface | Stable JSON/JSONL, LLM name resolution, source URLs, accession numbers |

```bash
sec filings --ticker AAPL --form 10-K
sec daily --date 2026-05-15 --form 8-K --limit 50 --pretty
sec efts --query "supply chain risk" --form 10-K --from 2024-01-01 --to 2024-12-31 --limit 10 --pretty
sec facts --ticker AAPL --concept revenue
sec statements --ticker AAPL --statement income --period annual --latest 4
sec metrics --ticker AAPL --period annual --latest 4 --pretty
sec ixbrl --ticker AAPL --form 10-K --concept RevenueFromContractWithCustomerExcludingAssessedTax
sec xbrl-links --ticker AAPL --form 10-K --linkbase presentation --concept Revenue --limit 20 --pretty
sec xbrl-tree --ticker AAPL --form 10-K --role OPERATIONS --limit 30 --pretty
sec xbrl-calc --ticker AAPL --form 10-K --role OPERATIONS --limit 20 --pretty
sec xbrl-statement --ticker AAPL --form 10-K --role OPERATIONS --values-only --limit 30 --pretty
sec tables --ticker AAPL --form 10-K --limit-tables 5 --limit-rows 10
sec company-report --ticker AAPL --form 10-K --topic segment --pretty
sec proxy --ticker AAPL --latest 1 --pretty
sec prospectus --ticker RDDT --form S-1 --include-amends --latest 1 --pretty
sec foreign --ticker TSM --form 20-F --latest 1 --pretty
sec fund --cik 0000036405 --form NPORT-P --latest 1 --limit-holdings 10 --pretty
sec fund --cik 0000036405 --form N-PX --latest 1 --limit-holdings 20 --pretty
sec search --ticker TSLA --form 10-K --query "supply chain risk"
sec section --ticker AAPL --form 10-K --item risk-factors --limit-bytes 8000
sec report --ticker AAPL --kind financial --latest 4
sec report --ticker AAPL --kind risk
sec resolve --query 段永平 --pretty
sec 13f-diff --investor 段永平 --pretty
sec report --cik 1067983 --kind portfolio --limit 10
sec docs --ticker AAPL --form 10-K --latest 1 --limit 20
sec doc --ticker AAPL --form 10-K --primary --limit-bytes 4000
sec form4 --ticker AAPL --latest 3
sec form4-summary --ticker AAPL --latest 3
sec 8k --ticker AAPL --item 2.02 --latest 5
sec 8k-exhibits --ticker AAPL --category earnings_release --latest 5 --pretty
sec 13d --ticker TSLA --form 13g --latest 2 --include-amends
sec 13f --cik 1067983 --latest 1
sec 13f-aggregate --cik 1067983 --latest 1 --limit 20
sec 13f-diff --cik 1067983 --limit 20
sec 13f-summary --cik 1067983 --latest 1
sec parse --ticker AAPL --form 4 --latest 1
sec forms --pretty
sec config set-identity "Your Name your.email@example.com"
sec completions zsh > ~/.zfunc/_sec
sec serve --host 127.0.0.1 --port 8716
sec mcp
```

`sec-cli` turns SEC filings into source-backed JSON for agents, analysts, and
data workflows. It is designed as a fast command-line tool first: stable output,
clear exit codes, local caching, and source URLs on every result.

## Status

This is an early MVP. The first implementation focuses on:

- Finding company filings from SEC submissions data
- Scanning SEC daily master indexes for all-market filing monitoring by date, form, company, and amendments
- Searching SEC EDGAR Full-Text Search across the market with optional ticker/CIK, form, and date filters
- Querying SEC CompanyFacts for source-backed XBRL facts
- Building broader standardized 10-K/10-Q income statement, balance sheet, and cash flow rows from CompanyFacts
- Calculating source-backed financial metrics such as growth, margins, free cash flow, ROA/ROE, current ratio, and leverage
- Streaming Inline XBRL facts directly from primary filing HTML
- Parsing XBRL presentation, calculation, definition, label, and schema linkbase attachments
- Extracting HTML tables from filing primary documents
- Parsing deeper 10-K/10-Q company-report topic tables such as segment revenue, geography, debt maturities, obligations, leases, taxes, and repurchases
- Parsing DEF 14A proxy statements for meeting details, voting proposals, directors, auditors, and executive compensation tables
- Parsing S-1/F-1/424B prospectuses for securities offered, ticker/exchange, price range, proceeds, risks, underwriters, and selected offering tables
- Parsing 20-F/6-K/40-F foreign issuer disclosures for annual reports, current reports, exchanges, symbols, auditors, event signals, and key excerpts
- Parsing N-PORT/N-CSR/N-CEN/N-PX/497K/24F-2NT fund disclosures for portfolio holdings, fund metadata, proxy votes, summary prospectuses, securities-sold notices, shareholder-report excerpts, and financial statement sections
- Searching filing submission text with snippets
- Extracting common 10-K/10-Q sections such as business, risk factors, and MD&A
- Generating source-backed Markdown reports for insider activity, 13F portfolios, and risk review
- Resolving public investor/fund/person names through an LLM, then validating candidates against SEC 13F filings
- Listing and reading individual SEC submission documents
- Parsing Form 4 insider ownership transactions
- Summarizing Form 4 reports, owners, signatures, footnotes, and net activity
- Parsing Form 8-K current-report events by item
- Discovering and classifying 8-K exhibits, including earnings releases, press releases, contracts, agreements, XBRL, and accountant letters
- Parsing Schedule 13D/13G beneficial ownership reports
- Parsing 13F-HR information-table holdings
- Aggregating 13F-HR holdings by CUSIP/class/put-call
- Comparing the latest two 13F-HR portfolios
- Parsing 13F-HR cover, summary, signature, and manager metadata
- Returning JSON arrays, JSONL, CSV, terminal tables, and Markdown reports
- Caching SEC responses locally
- Serving the same core queries through a local JSON HTTP API
- Serving the SEC query/parser/report surface through a stdio MCP adapter for agents

Longer term, the project aims to grow into a Rust-powered SEC disclosure engine:
Arrow/Parquet exports, optional bulk/offline archives, and deeper agent-native
query workflows on top of the current CLI/HTTP/MCP surfaces.

## What You Can Answer Accurately

These are useful, source-backed questions that work today:

| Question | Command |
| --- | --- |
| What did insiders recently buy or sell? | `sec form4 --ticker AAPL --latest 5 --pretty` |
| Which executives/directors filed Form 4s and what was net activity? | `sec form4-summary --ticker AAPL --latest 5 --pretty` |
| Which 8-K events did a company recently report? | `sec 8k --ticker AAPL --latest 5 --pretty` |
| Did a company file earnings-related 8-K events? | `sec 8k --ticker AAPL --item 2.02 --latest 5 --pretty` |
| Which 8-K exhibits include earnings releases or material contracts? | `sec 8k-exhibits --ticker AAPL --category earnings_release --latest 5 --pretty` |
| What did the whole market file on a specific day? | `sec daily --date 2026-05-15 --form 8-K --limit 50 --pretty` |
| Which companies mentioned a phrase across SEC filings? | `sec efts --query "supply chain risk" --form 10-K --from 2024-01-01 --to 2024-12-31 --pretty` |
| What are the latest standardized financial statement rows? | `sec statements --ticker AAPL --statement all --period annual --latest 1 --pretty` |
| What are the latest SEC-derived financial ratios and growth metrics? | `sec metrics --ticker AAPL --period annual --latest 4 --pretty` |
| Can I get a human-readable financial trend memo? | `sec report --ticker AAPL --kind financial --latest 4` |
| What Inline XBRL facts are embedded in the filing HTML? | `sec ixbrl --ticker AAPL --form 10-K --concept RevenueFromContractWithCustomerExcludingAssessedTax --pretty` |
| What tables are embedded in a filing? | `sec tables --ticker AAPL --form 10-K --limit-tables 5 --limit-rows 10 --pretty` |
| Which 10-K/10-Q topic tables discuss segments, geography, debt, obligations, leases, taxes, or repurchases? | `sec company-report --ticker AAPL --form 10-K --topic segment --pretty` |
| What is in the latest proxy statement? | `sec proxy --ticker AAPL --latest 1 --pretty` |
| What are the key terms in an IPO prospectus? | `sec prospectus --ticker RDDT --form S-1 --include-amends --latest 1 --pretty` |
| What did a foreign private issuer disclose in its latest annual/current report? | `sec foreign --ticker TSM --form 20-F --latest 1 --pretty` |
| What holdings did a fund disclose in N-PORT? | `sec fund --cik 0000036405 --form NPORT-P --latest 1 --limit-holdings 10 --pretty` |
| How did a fund vote proxies in N-PX? | `sec fund --cik 0000036405 --form N-PX --latest 1 --limit-holdings 20 --pretty` |
| Which 5% beneficial owners recently filed 13D/13G? | `sec 13d --ticker TSLA --form 13g --include-amends --pretty` |
| What is Berkshire Hathaway's latest 13F portfolio? | `sec 13f-aggregate --cik 1067983 --limit 20 --pretty` |
| What changed between the latest two 13F filings? | `sec 13f-diff --cik 1067983 --limit 20 --pretty` |
| What if I know the investor name but not the CIK? | `sec resolve --query 段永平 --pretty`, then `sec 13f-diff --investor 段永平 --pretty` |
| What are a company's latest 10-K risk factors? | `sec section --ticker AAPL --form 10-K --item risk-factors --pretty` |
| How can an app or local agent call sec-cli over HTTP? | `sec serve --port 8716`, then `curl "http://127.0.0.1:8716/v1/filings?ticker=AAPL&form=10-K&latest=1"` |
| How can an MCP-capable agent call sec-cli directly? | Run `sec config set-identity ...`, then configure the agent to launch `sec mcp` |
| Where did the answer come from? | Every structured result includes `source_url`; document results also include `document_url` |

## How To Choose Selectors

`sec-cli` has two broad query families: company disclosure and 13F investment
manager disclosure.

Company-disclosure commands use `--ticker` or `--cik`:

- `filings`
- `facts`
- `statements`
- `metrics`
- `company-report`
- `search`
- `section`
- `docs`
- `doc`
- `form4`
- `form4-summary`
- `8k`
- `proxy`
- `prospectus`
- `foreign`
- `fund`
- `parse`
- `report --kind risk`
- `report --kind insider`

13F investment-manager commands can use four selector styles:

- `--cik`: most precise. Use it when you already know the SEC CIK.
- `--manager`: deterministic SEC company search by legal filing-manager name; no LLM.
- `--investor`: public person/fund name. Uses verified cache first, then LLM if needed, then SEC validation.
- `--ticker`: useful when the 13F manager is also a public company and the ticker maps to the 13F filer, such as `BRK-B`.

The same subject can be queried several ways; all paths should converge on the
same verified SEC CIK:

```bash
# Duan Yongping / H&H: natural-language input for people and agents
sec resolve --query 段永平 --pretty
sec 13f-diff --investor 段永平 --limit 10 --pretty
sec report --investor 段永平 --kind portfolio --limit 10

# Duan Yongping / H&H: legal entity, deterministic, no LLM
sec resolve --manager "H&H International Investment LLC" --pretty
sec 13f-summary --manager "H&H International Investment LLC" --latest 2 --pretty
sec 13f-diff --manager "H&H International Investment LLC" --limit 10 --pretty

# Duan Yongping / H&H: CIK, best for scripts and production jobs
sec resolve --cik 1759760 --pretty
sec 13f-aggregate --cik 1759760 --latest 1 --limit 20 --pretty
sec 13f-diff --cik 1759760 --limit 10 --jsonl

# Warren Buffett / Berkshire: public name, manager, CIK, and ticker all work
sec resolve --query "Warren Buffett" --pretty
sec 13f-summary --manager "BERKSHIRE HATHAWAY INC" --latest 1 --pretty
sec 13f-diff --cik 1067983 --limit 20 --pretty
sec 13f-diff --ticker BRK-B --limit 20 --pretty
```

Practical rule:

- For a person or agent prompt, start with `--investor` or `--query`.
- For repeatable scripts, prefer `--cik`.
- For known SEC legal entities, prefer `--manager`.
- For operating-company filings and insider activity, prefer `--ticker`, then `--cik`.

## Data Sources And Output Tables

`sec-cli` uses SEC public data directly. It does not call paid market-data APIs.

| Data/source | Commands | What it contains | Main output table |
| --- | --- | --- | --- |
| SEC submissions JSON | `filings` | Filing list, dates, accession numbers, primary document names | filing records |
| SEC daily master index | `daily`, `monitor` | All-market daily filing feed: CIK, company, form, filing date, archive filename, accession, source URLs | daily filing records |
| SEC EDGAR Full-Text Search / EFTS | `efts`, `full-text`, `global-search` | All-market text-search hits with score, company, CIK, form, dates, accession, document URL | EFTS search records |
| SEC CompanyFacts JSON | `facts`, `statements`, `metrics`, `report --kind financial` | XBRL facts such as revenue, net income, assets, units, periods, standardized statement lines, derived margins/growth/returns/liquidity/leverage | fact records, financial statement rows, financial metric records, Markdown financial report |
| Inline XBRL filing HTML | `ixbrl` | Filing-embedded `ix:nonFraction` and `ix:nonNumeric` facts, context refs, units, scale, decimals, raw value | Inline XBRL fact records |
| XBRL linkbase attachments | `xbrl-links`, `linkbase`, `xbrl-tree`, `xbrl-calc`, `xbrl-statement` | EX-101.PRE/CAL/DEF/LAB/SCH relationships: presentation arcs, calculation weights, definition arcs, labels, schema elements, rendered statement rows with same-accession CompanyFacts values | XBRL linkbase relationship records, presentation tree rows, calculation checks, rendered XBRL statement rows |
| Filing HTML tables | `tables` | Table rows from primary HTML documents: compensation tables, segment tables, registration tables, contract tables | HTML table records |
| 10-K/10-Q company report primary document | `company-report`, `parse --form "10-K"` | Classified topic tables: segment revenue, geography, revenue disaggregation, debt maturities, contractual obligations, leases, taxes, share repurchases | company report records |
| DEF 14A proxy statement primary document | `proxy`, `parse --form "DEF 14A"` | Annual meeting date/site, voting proposals, board recommendations, director nominees, auditor, named executive officers, summary compensation table | proxy statement records |
| S-1/F-1/424B prospectus primary document | `prospectus`, `parse --form "S-1"` | Securities offered, IPO/prospectus type, ticker/exchange, price range, shares, proceeds, underwriters, auditor, risk/business/proceeds excerpts | prospectus records |
| 20-F/6-K/40-F foreign issuer primary document | `foreign`, `parse --form "20-F"` | Foreign private issuer annual/current reports, exchanges, symbols, auditors, event signals, risk/business/operating review/controls/financial statements excerpts | foreign issuer records |
| N-PORT/N-CSR/N-CEN/N-PX/497K/24F-2NT fund documents | `fund`, `parse --form "NPORT-P"` | Fund registrant/series/class metadata, N-PORT holdings, N-PX proxy votes, 497K summary prospectus excerpts, 24F securities-sold notices, assets/liabilities/net assets, N-CSR shareholder report excerpts, controls and financial statements | fund disclosure records |
| SEC complete submission text and archive documents | `search`, `section`, `docs`, `doc` | Original filing text, HTML/XML attachments, exhibits, source snippets | snippet, section, document records |
| Form 3/4/5 XML ownership reports | `form4`, `form4-summary`, `report --kind insider` | Insider owners, roles, transaction codes, shares, prices, footnotes, signatures | transaction and ownership-report records |
| Form 8-K primary document | `8k` | Current-report event items such as 2.02 earnings, 5.02 management changes, 8.01 other events, 9.01 exhibits | 8-K event records |
| Form 8-K exhibits | `8k-exhibits` | Attached EX documents classified as earnings release, press release, material contract, transaction agreement, charter/bylaws, security instrument, XBRL, accountant letter | 8-K exhibit records |
| Schedule 13D/13G primary document | `13d`, `13g`, `schedule13` | 5% beneficial ownership, reporting persons, ownership percentage, voting/dispositive power, activist/passive intent signal | Schedule 13 records |
| Form 13F-HR information table | `13f`, `13f-aggregate`, `13f-diff`, `report --kind portfolio` | Institutional long holdings: issuer, class, CUSIP, value, shares, voting authority | holding, aggregate holding, diff records |
| Form 13F-HR primary document | `13f-summary` | Manager identity, report period, total holdings/value, signature, included managers | 13F report summary records |
| 10-K/10-Q primary document | `section`, `report --kind risk` | Business, risk factors, cybersecurity, MD&A, financial statement sections | section records and Markdown report |
| LLM resolver plus SEC validation | `resolve`, 13F commands with `--investor` | Public name to legal SEC filing manager/CIK candidate | resolve candidate records |

Every serious output carries source metadata. For citations and audit trails,
look for `source_url`, `document_url`, `accession`, `document`, `section`, and
`fact_id` depending on the command.

Output record cheat sheet:

| Output record | Produced by | Read this first | Source fields |
| --- | --- | --- | --- |
| Filing | `filings` | `company`, `form`, `filing_date`, `report_date`, `primary_document` | `accession`, `source_url`, `text_url` |
| Daily filing | `daily`, `monitor` | `company`, `form`, `filing_date`, `filename` | `accession`, `source_url`, `text_url` |
| EFTS search hit | `efts`, `full-text`, `global-search` | `company`, `form`, `file_date`, `score`, `document` | `accession`, `source_url`, `document_url` |
| Fact | `facts` | `concept`, `label`, `value`, `unit`, `fy`, `fp`, `filed` | `accession`, `source_url`, `fact_id` |
| Financial statement row | `statements` | `statement`, `line_order`, `line_item`, `value`, `unit`, `fiscal_year`, `fiscal_period` | `accession`, `source_url`, `fact_id` |
| Financial metric | `metrics` | `metric`, `category`, `value`, `display_value`, `period_end`, `calculation`, `components` | `source_urls`, component `accession`, component `fact_id` |
| Inline XBRL fact | `ixbrl` | `name`, `context_ref`, `unit_ref`, `scale`, `raw_value`, `numeric_value` | `accession`, `document_url`, `source_url` |
| XBRL linkbase relationship | `xbrl-links` | `linkbase`, `relationship`, `role`, `parent_concept`, `child_concept`, `concept`, `label`, `order`, `weight` | `accession`, `document_url`, `source_url` |
| XBRL presentation tree row | `xbrl-tree` | `role`, `depth`, `line_order`, `concept`, `label`, `parent_concept`, `path` | `accession`, `document_url`, `source_url` |
| XBRL calculation check | `xbrl-calc` | `parent_concept`, `parent_value`, `calculated_value`, `difference`, `status`, `matched_children` | `accession`, `document_url`, `source_url` |
| Rendered XBRL statement row | `xbrl-statement`, `statement-render` | `role`, `depth`, `line_order`, `concept`, `label`, `value`, `numeric_value`, `calculation_status`, `path` | `accession`, `fact_id`, `document_url`, `source_url` |
| HTML table | `tables` | `title_hint`, `row_count`, `column_count`, `headers`, `rows`, `truncated` | `accession`, `document_url`, `source_url` |
| Company report topic table | `company-report` | `topics[].topic`, `confidence`, `headers`, `rows`, `matched_table_count`, `scanned_table_count` | `accession`, `document_url`, `source_url` |
| Proxy statement | `proxy`, `parse --form "DEF 14A"` | `meeting_date`, `proposals`, `director_nominees`, `auditor`, `named_executive_officers`, `summary_compensation_table` | `accession`, `document_url`, `source_url` |
| Prospectus | `prospectus`, `parse --form "S-1"` | `securities_offered`, `proposed_ticker`, `exchange`, `price_range`, `shares_offered`, `underwriters`, `risk_factors` | `accession`, `document_url`, `source_url` |
| Foreign issuer | `foreign`, `parse --form "20-F"` | `report_type`, `exchange`, `ticker_or_symbol`, `auditor`, `event_signals`, `risk_factors`, `operating_review` | `accession`, `document_url`, `source_url` |
| Fund disclosure | `fund`, `parse --form "NPORT-P"` | `disclosure_type`, `registrant_name`, `series_name`, `period_end`, `holdings`, `proxy_votes`, `summary_prospectus`, `registration_fee_notice`, `net_assets` | `accession`, `document_url`, `source_url` |
| Search snippet | `search` | `query`, `snippet`, `offset`, `form`, `filing_date` | `accession`, `source_url`, `document`, `section` |
| Section | `section` | `item`, `title`, `content`, `truncated` | `accession`, `document_url`, `source_url` |
| Document | `docs`, `doc` | `filename`, `document_type`, `description`, `content_type`, `content` | `accession`, `document_url`, `source_url` |
| Form 4 transaction | `form4` | `reporting_owner`, `officer_title`, `transaction_code`, `shares`, `price`, `value` | `accession`, `source_url` |
| Form 4 report summary | `form4-summary` | `owners`, `transaction_count`, `net_shares`, `total_value`, `footnotes` | `accession`, `source_url` |
| 8-K event | `8k` | `item`, `item_title`, `category`, `is_furnished_item`, `content` | `accession`, `document_url`, `source_url` |
| 8-K exhibit | `8k-exhibits` | `document_type`, `category`, `is_earnings_release`, `description`, `content` | `accession`, `document_url`, `source_url` |
| Schedule 13D/13G | `13d`, `13g`, `schedule13` | `reporting_persons`, `beneficially_owned_shares`, `percent_of_class`, `activist_intent` | `accession`, `document_url`, `source_url` |
| 13F holding | `13f` | `manager`, `issuer`, `class`, `cusip`, `value_usd`, `shares` | `accession`, `source_url` |
| 13F aggregate holding | `13f-aggregate` | `issuer`, `cusip`, `value_usd`, `shares`, `rows` | `source_url` |
| 13F diff row | `13f-diff` | `issuer`, `change_type`, `change_value_usd`, `change_shares` | `current_source_url`, `previous_source_url` |
| 13F report summary | `13f-summary` | `manager`, `report_date`, `total_holdings_reported`, `total_value_usd`, `signature_name` | `accession`, `source_url` |
| Resolve candidate | `resolve` | `investor`, `manager`, `cik`, `confidence`, `validation.status` | `validation.source_url`, `validation.latest_accession` |

`edgartools` already has Python objects, rich displays, DataFrame exports, AI
context helpers, and many filing-type parsers. `sec-cli` is deliberately
different: it is a standalone Rust CLI optimized for automation and agents. The
parity goal is to cover the useful structured outputs edgartools exposes, while
adding stable command-line schemas, precise exit behavior, source URLs on every
record, and Markdown reports that can be dropped directly into an analyst note.

## Architecture

The code is intentionally split by responsibility:

- `cli`: command arguments and CLI orchestration only
- `lib`: reusable Rust core shared by the CLI, HTTP API, MCP adapter, and future batch/export jobs
- `http`: low-level SEC HTTP
- `storage`: local byte cache/store
- `client`: SEC domain facade, ticker-to-CIK lookup
- `edgar`: SEC data sources, submissions, facts, archive URLs
- `company`: deeper 10-K/10-Q topic-table parser
- `metrics`: source-backed financial ratios and growth analysis
- `documents`: complete-submission `.txt` splitting and attachment selection
- `llm`: OpenAI-compatible and Anthropic-compatible model clients
- `resolve`: LLM candidate resolution plus SEC 13F validation
- `parsers`: shared XML helpers and form-specific parsers
- `models`: query DTOs and stable output records
- `registry`: supported parser discovery
- `pipeline`: unified parser dispatch for supported filing forms
- `search`: filing text search and snippets
- `output`: stable JSON / JSONL rendering

New SEC forms should usually be added as a new parser under `src/sec/parsers/forms/`.
The CLI should stay thin: resolve CIK, call a domain operation, print records.
See `docs/ARCHITECTURE.md` for the longer-term architecture.

## Install

For normal use, download a prebuilt binary from the latest GitHub Release. You
do not need Rust or Cargo unless you are developing sec-cli itself.

Release page: <https://github.com/okloorcl/sec-cli/releases/latest>

| Platform | Architecture | Release asset |
| --- | --- | --- |
| macOS | Apple Silicon / arm64 | `sec-cli-aarch64-apple-darwin.tar.gz` |
| Windows | amd64 / x86_64 | `sec-cli-x86_64-pc-windows-msvc.zip` |
| Linux | amd64 / x86_64 | `sec-cli-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | amd32 / i686 | `sec-cli-i686-unknown-linux-gnu.tar.gz` |
| Linux | arm64 / AArch64 | `sec-cli-aarch64-unknown-linux-gnu.tar.gz` |
| Linux | arm32 / ARMv7 hard-float | `sec-cli-armv7-unknown-linux-gnueabihf.tar.gz` |

macOS Apple Silicon:

```bash
curl -L -o sec-cli.tar.gz \
  https://github.com/okloorcl/sec-cli/releases/latest/download/sec-cli-aarch64-apple-darwin.tar.gz
tar -xzf sec-cli.tar.gz
sudo mv sec-cli-aarch64-apple-darwin/sec /usr/local/bin/sec
sec --help
```

Linux amd64:

```bash
curl -L -o sec-cli.tar.gz \
  https://github.com/okloorcl/sec-cli/releases/latest/download/sec-cli-x86_64-unknown-linux-gnu.tar.gz
tar -xzf sec-cli.tar.gz
sudo mv sec-cli-x86_64-unknown-linux-gnu/sec /usr/local/bin/sec
sec --help
```

Windows PowerShell:

```powershell
Invoke-WebRequest `
  -Uri https://github.com/okloorcl/sec-cli/releases/latest/download/sec-cli-x86_64-pc-windows-msvc.zip `
  -OutFile sec-cli.zip
Expand-Archive sec-cli.zip -DestinationPath .
.\sec-cli-x86_64-pc-windows-msvc\sec.exe --help
```

SEC requests must include a real identity. The easiest durable setup is:

```bash
sec config set-identity "Your Name your.email@example.com"
sec config show
```

You can also use environment variables:

```bash
export SEC_IDENTITY="Your Name your.email@example.com"
```

You can also pass it per command:

```bash
sec --identity "Your Name your.email@example.com" filings --ticker AAPL
```

Shell completion scripts are generated locally:

```bash
sec completions zsh > ~/.zfunc/_sec
sec completions bash > sec.bash
sec completions fish > ~/.config/fish/completions/sec.fish
```

## Development

Use Cargo only when you are developing or testing the project locally:

```bash
cargo build
cargo test

SEC_IDENTITY="Your Name your.email@example.com" \
  cargo run --bin sec -- filings --ticker AAPL --form 10-K --latest 2 --pretty
```

## CI And Release Targets

GitHub Actions checks the project on every push and pull request. Pushing a
`v*` tag builds and uploads release binaries for the same targets:

| Platform | Architecture | Rust target | CI behavior |
| --- | --- | --- | --- |
| Ubuntu Linux | amd64 / x86_64 | `x86_64-unknown-linux-gnu` | check, test, release build |
| Ubuntu Linux | amd32 / i686 | `i686-unknown-linux-gnu` | cross check, cross release build |
| Ubuntu Linux | arm64 / AArch64 | `aarch64-unknown-linux-gnu` | cross check, cross release build |
| Ubuntu Linux | arm32 / ARMv7 hard-float | `armv7-unknown-linux-gnueabihf` | cross check, cross release build |
| Windows | amd64 / x86_64 | `x86_64-pc-windows-msvc` | check, test, release build |
| macOS | arm64 / Apple Silicon | `aarch64-apple-darwin` | check, test, release build |

Native runners execute tests where GitHub provides the matching machine.
Linux non-native targets use `cross`, so CI verifies compilation for 32-bit and
ARM Linux without trying to run those binaries on the x86_64 runner.

## LLM Resolver

`sec resolve` does not use a hardcoded investor map. Resolution is layered:
standard inputs are handled by deterministic SEC lookups first, and the LLM is
only used when the input is a non-standard public name.

- `--cik` validates the CIK directly against SEC `13F-HR` filings.
- `--manager` searches SEC company records for the legal 13F filing manager.
- `--query` checks the verified local cache, asks the LLM for likely legal
  filing managers, then validates/corrects candidates against SEC filings.

The LLM is used for name understanding; SEC data remains the source of truth.

Verified resolutions are cached under the local sec-cli cache directory, so
commands such as `sec 13f-diff --investor <NAME>` can reuse the last SEC-verified
CIK instead of depending on a fresh LLM answer every time.

OpenAI-compatible config, including GLM/BigModel-compatible endpoints:

```bash
mkdir -p ~/.config/sec-cli
cat > ~/.config/sec-cli/llm.json <<'JSON'
{
  "provider": "openai",
  "base_url": "https://open.bigmodel.cn/api/coding/paas/v4",
  "model": "GLM-5.1",
  "api_key_env": "BIGMODEL_API_KEY"
}
JSON

export BIGMODEL_API_KEY="your-api-key"
sec resolve --query 段永平 --pretty
sec resolve --manager "H&H International Investment LLC" --pretty
sec resolve --cik 1759760 --pretty
```

Anthropic-compatible config:

```json
{
  "provider": "anthropic",
  "base_url": "https://open.bigmodel.cn/api/anthropic",
  "model": "GLM-5.1",
  "api_key_env": "BIGMODEL_API_KEY"
}
```

Environment overrides:

| Variable | Meaning |
| --- | --- |
| `SEC_CLI_LLM_CONFIG` | Override config file path |
| `SEC_CLI_LLM_PROVIDER` | `openai` or `anthropic` |
| `SEC_CLI_LLM_BASE_URL` | Provider base URL |
| `SEC_CLI_LLM_MODEL` | Model name |
| `SEC_CLI_LLM_API_KEY_ENV` | Name of the environment variable containing the API key |
| `SEC_CLI_LLM_API_KEY` | Direct API key fallback; prefer `api_key_env` for shells and repos |

Per-command overrides are also available:

```bash
sec resolve --query "Warren Buffett" \
  --llm-provider openai \
  --llm-base-url https://open.bigmodel.cn/api/coding/paas/v4 \
  --llm-model GLM-5.1 \
  --llm-api-key-env BIGMODEL_API_KEY \
  --pretty
```

Do not commit API keys. Keep local config files private and prefer environment
variables for secrets.

## Local Full Test

Copy this block to build, configure GLM, run unit tests, and validate live SEC
queries plus live LLM resolution. Replace the two exported values with your real
identity and API key before running.

```bash
cd /Users/w0x7ce/Downloads/AACC/sec-cli

export SEC_IDENTITY="Your Name your.email@example.com"
export BIGMODEL_API_KEY="paste-your-bigmodel-key-here"

mkdir -p ~/.config/sec-cli
cat > ~/.config/sec-cli/llm.json <<'JSON'
{
  "provider": "openai",
  "base_url": "https://open.bigmodel.cn/api/coding/paas/v4",
  "model": "GLM-5.1",
  "api_key_env": "BIGMODEL_API_KEY"
}
JSON

cargo build
cargo test
cargo check

cargo run --bin sec -- filings --ticker AAPL --form 10-K --latest 1 --pretty
cargo run --bin sec -- facts --ticker AAPL --concept revenue --form 10-K --latest 3 --pretty
cargo run --bin sec -- statements --ticker AAPL --statement income --period annual --latest 2 --pretty
cargo run --bin sec -- statements --ticker AAPL --statement cashflow --period quarterly --latest 4 --jsonl
cargo run --bin sec -- ixbrl --ticker AAPL --form 10-K --concept RevenueFromContractWithCustomerExcludingAssessedTax --latest 1 --limit 3 --pretty
cargo run --bin sec -- xbrl-links --ticker AAPL --form 10-K --linkbase presentation --concept Revenue --limit 10 --pretty
cargo run --bin sec -- xbrl-tree --ticker AAPL --form 10-K --role OPERATIONS --limit 15 --pretty
cargo run --bin sec -- xbrl-calc --ticker AAPL --form 10-K --role OPERATIONS --limit 10 --pretty
cargo run --bin sec -- xbrl-statement --ticker AAPL --form 10-K --role OPERATIONS --values-only --limit 10 --pretty
cargo run --bin sec -- tables --ticker AAPL --form 10-K --latest 1 --limit-tables 3 --limit-rows 5 --pretty
cargo run --bin sec -- foreign --ticker TSM --form 20-F --latest 1 --limit-bytes 800 --pretty
cargo run --bin sec -- fund --cik 0000036405 --form NPORT-P --latest 1 --limit-holdings 5 --pretty
cargo run --bin sec -- form4-summary --ticker AAPL --latest 2 --pretty
cargo run --bin sec -- 8k --ticker AAPL --item 2.02 --latest 5 --limit-bytes 600 --pretty
cargo run --bin sec -- 13d --ticker TSLA --form 13g --latest 2 --include-amends --pretty

cargo run --bin sec -- resolve --query 段永平 --pretty
cargo run --bin sec -- resolve --manager "H&H International Investment LLC" --pretty
cargo run --bin sec -- resolve --cik 1759760 --pretty
cargo run --bin sec -- 13f-summary --investor 段永平 --latest 2 --pretty
cargo run --bin sec -- 13f-diff --manager "H&H International Investment LLC" --limit 10 --pretty
cargo run --bin sec -- 13f-diff --investor 段永平 --limit 10 --pretty
cargo run --bin sec -- report --investor 段永平 --kind portfolio --limit 10

cargo run --bin sec -- resolve --query 巴菲特 --pretty
cargo run --bin sec -- 13f-summary --investor 巴菲特 --latest 1 --pretty

cargo run --bin sec -- resolve --query Bridgewater --pretty
cargo run --bin sec -- report --investor Bridgewater --kind portfolio --limit 5
```

Expected smoke-test signals:

- `cargo test` should pass all tests.
- `resolve --query 段永平` should end with `validation.status = verified_13f`
  and CIK `1759760`.
- `resolve --manager "H&H International Investment LLC"` and `resolve --cik
  1759760` should return the same verified CIK without using the LLM.
- `13f-diff --investor 段永平` should show H&H International Investment, LLC
  and recent changes such as Apple, Tesla, Nvidia, Berkshire, and PDD.
- `resolve --query 巴菲特` should resolve to Berkshire Hathaway Inc, CIK
  `1067983`.

## Commands

### filings

Find recent filings by ticker or CIK.

```bash
sec filings --ticker AAPL --form 10-K --latest 3 --pretty
sec filings --cik 320193 --form 10-Q --from 2023-01-01 --to 2025-12-31
sec filings --ticker TSLA --form 8-K --latest 5 --jsonl
sec filings --ticker NVDA --form 10-K --include-amends --latest 2 --pretty
```

Each result includes:

- `accession`
- `cik`
- `company`
- `form`
- `filing_date`
- `primary_document`
- `source_url`
- `text_url`

### daily

Scan the all-market SEC daily master index. This is the high-volume monitoring
entry point: it starts from a SEC filing date and filters the whole daily feed,
instead of starting from a single ticker.

```bash
sec daily --date 2026-05-15 --form 8-K --limit 50 --pretty
sec daily --date 2026-05-15 --form 13F-HR --include-amends --jsonl
sec daily --date 2026-05-15 --company apple --pretty
sec monitor --form 4 --limit 100 --jsonl
```

If `--date` is omitted, sec-cli uses the latest SEC weekday in UTC. Weekend
defaults roll back to Friday. Each record includes `cik`, `company`, `form`,
`filing_date`, `accession`, `filename`, `text_url`, and `source_url`.

### efts

Search the official SEC EDGAR Full-Text Search index across the whole market.
Use this when you do not know which company filed the phrase, or when you want
to scan a theme across many companies.

```bash
sec efts --query "supply chain risk" --form 10-K --from 2024-01-01 --to 2024-12-31 --limit 10 --pretty
sec efts --ticker AAPL --query "artificial intelligence" --form 10-K --limit 5 --pretty
sec efts --cik 320193 --query "services revenue" --form 10-K,10-Q --from 2023-01-01 --pretty
sec full-text --query "GLP-1" --form 10-K --limit 20 --jsonl
```

`--ticker` and `--cik` are optional. Without them, the search is all-market.
`--form` accepts one form or comma-separated forms. Output includes `score`,
`cik`, `company`, `form`, `file_date`, `period_ending`, `accession`, `document`,
`source_url`, and `document_url`.

### facts

Query SEC CompanyFacts by concept alias or XBRL concept name.

```bash
sec facts --ticker AAPL --concept revenue --form 10-K --latest 5 --pretty
sec facts --ticker MSFT --concept us-gaap:NetIncomeLoss --latest 10 --jsonl
sec facts --cik 320193 --concept us-gaap:RevenueFromContractWithCustomerExcludingAssessedTax --unit USD --latest 8 --pretty
```

Each fact includes:

- `concept`
- `label`
- `value`
- `unit`
- `fy`
- `fp`
- `form`
- `filed`
- `start`
- `end`
- `accession`
- `source_url`
- `fact_id`

### statements

Build standardized 10-K/10-Q financial statement rows from SEC CompanyFacts.
This is a normalized long table, not a rendered spreadsheet: every row is one
statement line, period, concept, unit, and source filing.

```bash
sec statements --ticker AAPL --statement income --period annual --latest 4 --pretty
sec statements --ticker AAPL --statement balance --period annual --latest 2 --pretty
sec statements --ticker AAPL --statement cashflow --period quarterly --latest 4 --jsonl
sec statements --cik 320193 --statement all --period annual --latest 1 --pretty
```

`--statement` accepts:

- `income`: revenue, cost of revenue, gross profit, R&D, SG&A, operating income, interest, tax, net income, EPS, shares
- `balance`: cash, securities, receivables, inventory, current assets, PP&E, goodwill, intangibles, leases, debt, liabilities, equity
- `cashflow`: net income, D&A, stock compensation, working-capital changes, operating cash flow, capex, acquisitions, dividends, repurchases, debt issuance/repayment, cash change
- `all`: income, balance, and cashflow together

`--period` accepts:

- `annual`: 10-K facts
- `quarterly`: 10-Q facts
- `all`: any available filing form

Each row includes:

- `cik`
- `company`
- `statement`
- `line_order`
- `line_item`
- `concept`
- `taxonomy`
- `label`
- `value`
- `numeric_value`
- `unit`
- `fiscal_year`
- `fiscal_period`
- `form`
- `filed`
- `start`
- `end`
- `frame`
- `accession`
- `source_url`
- `fact_id`

### metrics

Calculate source-backed financial metrics from standardized CompanyFacts
statement rows. This is the first secondary-analysis layer: every metric keeps
the SEC facts used in `components`, including accession, fact id, and source
URL.

```bash
sec metrics --ticker AAPL --period annual --latest 4 --pretty
sec metrics --ticker AAPL --period quarterly --latest 8 --jsonl
sec metrics --cik 320193 --period annual --latest 1 --pretty
```

`--period` accepts:

- `annual`: derive metrics from 10-K facts
- `quarterly`: derive metrics from 10-Q facts
- `all`: use any available filing form

Metrics currently include roughly 30 SEC-derived records when the required
facts are available:

- profitability: `gross_margin`, `operating_margin`, `net_margin`
- profitability/tax: `effective_tax_rate`
- growth: `revenue_growth`, `net_income_growth`
- cash flow: `free_cash_flow`, `free_cash_flow_margin`, `operating_cash_flow_margin`, `free_cash_flow_to_net_income`
- returns: `return_on_assets`, `return_on_equity`, `roic`
- liquidity: `working_capital`, `current_ratio`, `quick_ratio`, `cash_ratio`, `cash_to_assets`
- leverage/solvency: `total_debt`, `net_debt`, `liabilities_to_assets`, `debt_to_equity`, `debt_to_assets`, `net_debt_to_equity`, `interest_coverage`
- efficiency: `asset_turnover`
- capital intensity/return: `capex_to_revenue`, `dividend_payout_ratio`, `share_repurchases_to_revenue`, `share_repurchases_to_free_cash_flow`

Each metric includes: `metric`, `category`, `value`, `display_value`, `unit`,
`period_end`, `fiscal_year`, `fiscal_period`, `form`, `calculation`,
`components`, and `source_urls`.

### ixbrl

Stream Inline XBRL facts directly from the primary filing HTML. This is useful
when you need the exact facts embedded in a specific 10-K/10-Q document rather
than SEC's normalized CompanyFacts API.

```bash
sec ixbrl --ticker AAPL --form 10-K --concept RevenueFromContractWithCustomerExcludingAssessedTax --latest 1 --limit 3 --pretty
sec ixbrl --ticker AAPL --form 10-K --concept us-gaap:NetIncomeLoss --limit 5 --jsonl
sec ixbrl --cik 320193 --form 10-Q --latest 1 --limit 100 --pretty
```

`--concept` accepts either a full concept such as `us-gaap:NetIncomeLoss` or a
local concept name such as `NetIncomeLoss`.

Each fact includes:

- `accession`
- `fact_type`
- `name`
- `namespace`
- `local_name`
- `context_ref`
- `unit_ref`
- `decimals`
- `scale`
- `format`
- `sign`
- `id`
- `raw_value`
- `value`
- `numeric_value`
- `document_url`
- `source_url`

### xbrl-links

Parse XBRL linkbase attachments from complete SEC submissions. This command is
the low-level foundation for true filing-specific financial statement rendering:
it exposes the presentation tree, calculation weights, definition arcs, labels,
and schema elements that are not available in SEC CompanyFacts JSON.

```bash
sec xbrl-links --ticker AAPL --form 10-K --linkbase presentation --concept Revenue --limit 20 --pretty
sec xbrl-links --ticker AAPL --form 10-K --linkbase calculation --concept NetIncomeLoss --pretty
sec linkbase --cik 320193 --form 10-Q --linkbase label --concept Revenue --jsonl
```

`--linkbase` accepts `presentation`, `calculation`, `definition`, `label`, or
`schema`. `--concept` matches parent, child, or label concepts and accepts either
`us-gaap:Revenues` style names or local names such as `Revenues`.

Each relationship includes: `linkbase`, `relationship`, `role`, `arcrole`,
`parent_concept`, `child_concept`, `concept`, `label`, `label_role`, `order`,
`weight`, `preferred_label`, `document_url`, and `source_url`.

### xbrl-tree

Render filing-specific XBRL presentation arcs into preorder tree rows. This is
the human- and agent-friendly view of `EX-101.PRE`: each row has a `depth`,
`line_order`, `path`, parent concept, role URI, and source document URL. Use
`xbrl-statement` when you also want same-filing fact values.

```bash
sec xbrl-tree --ticker AAPL --form 10-K --role OPERATIONS --limit 30 --pretty
sec xbrl-tree --ticker AAPL --form 10-K --concept NetIncomeLoss --pretty
sec presentation-tree --cik 320193 --form 10-Q --limit 50 --jsonl
```

`--role` is a case-insensitive substring filter over role URIs, so short terms
such as `OPERATIONS`, `BALANCE`, `CASH`, or `Revenue` are usually enough.

Each row includes: `role`, `depth`, `line_order`, `concept`, `label`,
`parent_concept`, `order`, `preferred_label`, `path`, `document_url`, and
`source_url`.

### xbrl-calc

Validate XBRL calculation linkbase parent totals against same-accession
CompanyFacts values. It groups `EX-101.CAL` arcs by role and parent concept,
applies each child weight, and reports whether the SEC fact value matches the
calculated total within `--tolerance`.

```bash
sec xbrl-calc --ticker AAPL --form 10-K --role OPERATIONS --limit 20 --pretty
sec xbrl-calc --ticker AAPL --form 10-K --concept GrossProfit --tolerance 1 --pretty
sec calculation-checks --cik 320193 --form 10-Q --unit USD --limit 50 --jsonl
```

Each check includes: `parent_concept`, `parent_value`, `calculated_value`,
`difference`, `relative_difference`, `status`, `children_count`,
`matched_children`, `missing_children`, `document_url`, and `source_url`.

### xbrl-statement

Render filing-specific presentation-tree rows with same-accession CompanyFacts
values and calculation-check status. This is the closest CLI view to a true
SEC-native financial statement: `EX-101.PRE` controls row order and hierarchy,
CompanyFacts supplies values, CompanyFacts labels fill missing extension labels,
and `EX-101.CAL` adds `calculation_status` for totals.

```bash
sec xbrl-statement --ticker AAPL --form 10-K --role OPERATIONS --values-only --limit 30 --pretty
sec xbrl-statement --ticker AAPL --form 10-K --role BALANCE --unit USD --limit 50 --pretty
sec statement-render --cik 320193 --form 10-Q --concept NetIncomeLoss --jsonl
```

Useful flags:

- `--role`: filters role URIs by substring, such as `OPERATIONS`, `BALANCE`, or `CASH`.
- `--values-only`: hides abstract/heading rows and returns only rows with matched facts.
- `--unit`: selects the CompanyFacts unit, usually `USD` or `shares`.
- `--tolerance`: controls calculation-check tolerance for total rows.

Each row includes: `role`, `depth`, `line_order`, `concept`, `label`, `value`,
`numeric_value`, `unit`, `fact_id`, `calculation_status`,
`calculation_difference`, `calculation_relative_difference`, `path`,
`document_url`, and `source_url`.

### tables

Extract HTML tables from primary filing documents. This is intentionally generic:
the command returns rows and source metadata so agents can inspect compensation,
segment, debt, registration, exhibit, or contract tables without bespoke parsing
for every table type.

```bash
sec tables --ticker AAPL --form 10-K --latest 1 --limit-tables 5 --limit-rows 10 --pretty
sec tables --ticker TSLA --form DEF 14A --include-amends --limit-tables 20 --limit-rows 8 --jsonl
sec tables --cik 320193 --form 10-Q --latest 1 --limit-tables 10 --pretty
```

Each table includes:

- `table_index`
- `title_hint`
- `row_count`
- `column_count`
- `returned_rows`
- `truncated`
- `headers`
- `rows`
- `document_url`
- `source_url`

### company-report

Parse high-value 10-K/10-Q topic tables from the primary company report. This is
more opinionated than `tables`: it classifies likely segment revenue,
geographic revenue, revenue disaggregation, debt maturity, contractual
obligations, lease maturity, tax, and share repurchase tables.

```bash
sec company-report --ticker AAPL --form 10-K --latest 1 --pretty
sec company-report --ticker AAPL --form 10-K --topic segment --limit-tables 5 --limit-rows 12 --pretty
sec company-report --cik 320193 --form 10-Q --topic debt --jsonl
sec parse --ticker AAPL --form 10-K --limit 5 --pretty
```

Each record includes `matched_table_count`, `scanned_table_count`, and
`topics[]` with `topic`, `confidence`, `title_hint`, `headers`, `rows`, and SEC
source fields.

### proxy

Parse DEF 14A proxy statements. This command turns shareholder-meeting
materials into one structured record per filing: meeting logistics, voting
proposals, board recommendations, director nominees, auditor, named executive
officers, and the summary compensation table.

```bash
sec proxy --ticker AAPL --latest 1 --pretty
sec proxy --cik 320193 --latest 2 --include-amends --limit-rows 20 --pretty
sec parse --ticker AAPL --form "DEF 14A" --latest 1 --pretty
```

Each proxy record includes:

- `meeting_date`
- `meeting_time`
- `meeting_site`
- `record_date`
- `materials_available_date`
- `proposals`
- `director_nominees`
- `auditor`
- `named_executive_officers`
- `summary_compensation_table`
- `document_url`
- `source_url`

### prospectus

Parse S-1, F-1, and 424B prospectus filings. This command extracts capital
markets signals that are useful for IPO and offering analysis: offered
securities, IPO/prospectus type, proposed ticker, exchange, price range, shares,
offering amount, underwriters, auditor, selected tables, and source-backed
excerpts for use of proceeds, risk factors, business, and dilution.

```bash
sec prospectus --ticker RDDT --form S-1 --include-amends --latest 1 --pretty
sec prospectus --cik 1713445 --form all --latest 3 --limit-bytes 800 --limit-tables 5 --pretty
sec parse --ticker RDDT --form "424B4" --latest 1 --pretty
```

`--form` accepts `all`, `S-1`, `S-1/A`, `F-1`, `F-1/A`, `424B`, `424B1`
through `424B5`, plus `424B7`. Use `--include-amends` when you want amended
registration statements such as `S-1/A`.

Each prospectus record includes:

- `prospectus_type`
- `is_ipo_related`
- `securities_offered`
- `proposed_ticker`
- `exchange`
- `price_range`
- `shares_offered`
- `offering_amount`
- `underwriters`
- `auditor`
- `use_of_proceeds`
- `risk_factors`
- `business`
- `dilution`
- `tables`
- `document_url`
- `source_url`

### foreign

Parse 20-F, 6-K, and 40-F foreign issuer disclosures. This command is for ADRs
and foreign private issuers such as TSM, BABA, ASML, SHOP, or SONY. It extracts
the report type, exchange/symbol clues, auditor names, current-report event
signals, and source-backed excerpts for risk factors, business, operating
review, controls, and financial statements.

```bash
sec foreign --ticker TSM --form 20-F --latest 1 --pretty
sec foreign --ticker BABA --form 6-K --latest 3 --limit-bytes 800 --pretty
sec foreign --ticker SHOP --form 40-F --latest 1 --pretty
sec foreign --cik 1046179 --form all --latest 5 --include-amends --jsonl
sec parse --ticker TSM --form "20-F" --latest 1 --pretty
sec parse --ticker BABA --form "6-K" --latest 1 --pretty
```

`--form` accepts `all`, `20-F`, `20-F/A`, `6-K`, `6-K/A`, `40-F`, and
`40-F/A`. Use `--include-amends` when amended foreign annual reports or current
reports matter.

Each foreign issuer record includes:

- `report_type`
- `is_amendment`
- `exchange`
- `ticker_or_symbol`
- `auditor`
- `event_signals`
- `risk_factors`
- `business`
- `operating_review`
- `controls`
- `financial_statements`
- `document_url`
- `source_url`

### fund

Parse N-PORT, N-CSR/N-CSRS, N-CEN, N-PX, 497K, and 24F-2NT fund disclosures.
`NPORT-P` is the most structured holdings source: it contains portfolio holdings,
security identifiers, values, portfolio percentages, asset categories, issuer
categories, country, and restricted-security flags. `N-PX` exposes proxy voting
records, `497K` is a summary prospectus, `24F-2NT` is an annual notice of
securities sold, `N-CSR`/`N-CSRS` are shareholder reports, and `N-CEN` is the
annual fund census.

```bash
sec fund --cik 0000036405 --form NPORT-P --latest 1 --limit-holdings 10 --pretty
sec fund --cik 0000036405 --form N-PX --latest 1 --limit-holdings 20 --pretty
sec fund --cik 0000036405 --form 497K --latest 1 --limit-bytes 1200 --pretty
sec fund --cik 0000036405 --form 24F-2NT --latest 1 --pretty
sec fund --cik 0000036405 --form N-CSR --latest 1 --limit-bytes 1200 --pretty
sec fund --cik 0000036405 --form N-CEN --latest 1 --pretty
sec fund --cik 0000036405 --form all --latest 5 --include-amends --jsonl
sec parse --cik 0000036405 --form "NPORT-P" --latest 1 --limit 10 --pretty
```

`--form` accepts `all`, `NPORT-P`, `NPORT-P/A`, `N-PORT`, `N-PORT/A`,
`N-CSR`, `N-CSR/A`, `N-CSRS`, `N-CSRS/A`, `N-CEN`, `N-CEN/A`, `N-PX`,
`N-PX/A`, `497K`, `497K/A`, `24F-2NT`, and `24F-2NT/A`. Use
`--limit-holdings` to cap the returned N-PORT holdings or N-PX proxy-vote array.

Each fund disclosure record includes:

- `disclosure_type`
- `registrant_name`
- `series_name`
- `class_name`
- `period_end`
- `fiscal_year_end`
- `total_assets`
- `total_liabilities`
- `net_assets`
- `holdings_count`
- `holdings`
- `proxy_votes_count`
- `proxy_votes`
- `shareholder_report`
- `portfolio_summary`
- `proxy_voting_record`
- `summary_prospectus`
- `registration_fee_notice`
- `financial_statements`
- `controls`
- `document_url`
- `source_url`

### search

Search filing submission text and return source-backed snippets.

```bash
sec search --ticker TSLA --form 10-K --query "risk factors" --latest 1 --pretty
sec search --ticker NVDA --form 10-K --query "export controls" --jsonl
sec search --cik 320193 --form 10-K --query "supply chain" --context 300 --latest 2 --pretty
```

Search first tries an exact case-insensitive phrase match, then falls back to a
token-window match so agent queries are more robust against SEC HTML markup.

### section

Extract common sections from the primary 10-K or 10-Q document. The extractor
normalizes HTML/XBRL markup to text, locates item headings, chooses the largest
matching body over table-of-contents hits, and returns source-backed JSON.

```bash
sec section --ticker AAPL --form 10-K --item risk-factors --limit-bytes 8000 --pretty
sec section --ticker MSFT --form 10-K --item mda --latest 1 --pretty
sec section --cik 320193 --form 10-K --item 1A --latest 1 --jsonl
sec section --ticker TSLA --form 10-Q --item market-risk --limit-bytes 6000 --pretty
```

Supported item aliases include:

- `business` / `1`
- `risk-factors` / `1A`
- `cybersecurity` / `1C`
- `properties` / `2`
- `legal-proceedings` / `3`
- `mda` / `7`
- `market-risk` / `7A`
- `financial-statements` / `8`

Each section includes:

- `accession`
- `item`
- `title`
- `start_offset`
- `end_offset`
- `byte_length`
- `returned_bytes`
- `truncated`
- `document_url`
- `source_url`
- `content`

### report

Generate a source-backed Markdown report for a human analyst or an AI agent.
Reports reuse the same structured parsers used by the JSON commands.

```bash
sec report --ticker AAPL --kind insider --latest 5 --limit 10
sec report --investor 段永平 --kind portfolio --limit 10
sec report --manager "H&H International Investment LLC" --kind portfolio --limit 10
sec report --cik 1067983 --kind portfolio --limit 10
sec report --ticker AAPL --kind financial --latest 4 --limit 20
sec report --ticker AAPL --kind risk --limit-bytes 4000
sec report --ticker AAPL --kind risk --latest 1 --limit-bytes 12000 > aapl-risk.md
```

Report kinds:

- `financial`: SEC-derived metric table, multi-period trend snapshot, and rule-based signals
- `insider`: Form 4 summary table with owner, role, net shares, value, and SEC source
- `portfolio`: 13F summary, top holdings, visual bars, and largest position changes
- `risk`: 10-K risk factor and MD&A excerpts with source links

### resolve

Resolve an investor, fund, public person, known manager, or CIK to SEC 13F
filing manager candidates. Standard selectors are deterministic; natural
language `--query` can use the LLM and is always checked against SEC data when
verification is enabled.

```bash
sec resolve --query 段永平 --pretty
sec resolve --manager "H&H International Investment LLC" --pretty
sec resolve --cik 1759760 --pretty
sec resolve --query "Warren Buffett" --pretty
sec resolve --query Bridgewater --pretty
sec resolve --query "Seth Klarman" --no-verify --pretty
sec resolve --query 段永平 --llm-provider openai --llm-model GLM-5.1 --pretty
```

Each candidate includes:

- `query`
- `candidate_type`
- `investor`
- `manager`
- `cik`
- `confidence`
- `relationship`
- `evidence_queries`
- `notes`
- `validation`
- `next_commands`

`validation.status` is `verified_13f` only when the candidate has a SEC
`13F-HR` filing. Commands such as `sec 13f-diff --investor <NAME>` require a
verified 13F candidate.

### docs

List documents and attachments inside complete SEC submissions.

```bash
sec docs --ticker AAPL --form 10-K --latest 1 --limit 20 --pretty
sec docs --cik 320193 --form 8-K --latest 2 --limit 50 --jsonl
```

Each document includes:

- `accession`
- `document_type`
- `sequence`
- `filename`
- `description`
- `content_type`
- `byte_length`
- `is_primary`
- `document_url`
- `source_url`

### doc

Read one document from a complete submission. By default, `sec doc` returns one
JSON record with source metadata and content. Use `--raw` for exact extracted
document content or `--text` for compact plain text.

```bash
sec doc --ticker AAPL --form 10-K --primary --limit-bytes 4000 --pretty
sec doc --ticker AAPL --form 10-K --sequence 1 --text --limit-bytes 12000
sec doc --cik 320193 --accession 0000320193-25-000079 --filename aapl-20250927.htm --raw
sec doc --ticker AAPL --form 10-K --filename a10-kexhibit21109272025.htm --limit-bytes 4000 --pretty
```

Selectors:

- `--primary` selects sequence `1`
- `--sequence 2` selects a document by SEC sequence
- `--filename form4.xml` selects a document by filename
- `--accession` narrows the filing set to one accession
- `--limit-bytes` returns a UTF-8-safe prefix and sets `truncated`

Each JSON record includes:

- `accession`
- `document_type`
- `sequence`
- `filename`
- `content_type`
- `byte_length`
- `returned_bytes`
- `truncated`
- `document_url`
- `source_url`
- `content`

### form4

Parse Form 4 ownership transactions.

```bash
sec form4 --ticker AAPL --latest 3 --limit 10 --pretty
sec form4 --cik 320193 --latest 10 --jsonl
sec form4-summary --ticker TSLA --include-amends --latest 5 --pretty
```

Each transaction includes:

- `accession`
- `issuer`
- `issuer_ticker`
- `reporting_owner`
- `officer_title`
- `transaction_date`
- `transaction_form_type`
- `transaction_code`
- `equity_swap_involved`
- `transaction_type`
- `security_title`
- `shares`
- `price`
- `value`
- `shares_owned_after`
- `direct_or_indirect`
- `nature_of_ownership`
- `derivative`
- `conversion_or_exercise_price`
- `exercise_date`
- `expiration_date`
- `underlying_security_title`
- `underlying_shares`
- `source_url`

### form4-summary

Summarize each Form 4 ownership report before drilling into transaction rows.
This is useful for agents that need a compact, source-backed answer about who
filed, whether activity was net acquisition or disposition, and which footnotes
or signatures are present.

```bash
sec form4-summary --ticker AAPL --latest 3 --limit 10 --pretty
sec form4-summary --cik 320193 --include-amends --jsonl
```

Each report summary includes:

- `accession`
- `period_of_report`
- `issuer`
- `issuer_ticker`
- `owners`
- `signatures`
- `footnotes`
- `transaction_count`
- `acquisition_count`
- `disposition_count`
- `derivative_transaction_count`
- `total_shares_acquired`
- `total_shares_disposed`
- `net_shares`
- `total_value`
- `source_url`

### 8k

Parse Form 8-K current-report event items from the primary document. This turns
the usual free-form 8-K HTML into event records with official item labels and
source-backed excerpts.

```bash
sec 8k --ticker AAPL --latest 5 --pretty
sec 8k --ticker AAPL --item 2.02 --latest 5 --limit-bytes 600 --pretty
sec 8k --ticker TSLA --item 5.02 --latest 10 --jsonl
sec 8k --cik 320193 --item 9.01 --include-amends --pretty
sec 8k-exhibits --ticker AAPL --category earnings_release --latest 5 --limit-bytes 1200 --pretty
sec 8k-exhibits --ticker MSFT --category material_contract --latest 10 --jsonl
```

Common item filters:

- `1.01`: material agreement
- `2.02`: results of operations and financial condition
- `4.02`: non-reliance on previously issued financial statements
- `5.02`: director/officer departure, appointment, or compensation
- `7.01`: Regulation FD disclosure
- `8.01`: other events
- `9.01`: financial statements and exhibits

`8k-exhibits` categories include `earnings_release`, `press_release`,
`material_contract`, `transaction_agreement`, `charter_or_bylaws`,
`security_instrument`, `accountant_letter`, `xbrl`, and `other_exhibit`.

Each event includes:

- `accession`
- `item`
- `item_title`
- `category`
- `is_furnished_item`
- `start_offset`
- `end_offset`
- `byte_length`
- `returned_bytes`
- `truncated`
- `document`
- `document_url`
- `source_url`
- `content`

### 13d / 13g / schedule13

Parse Schedule 13D and Schedule 13G beneficial ownership reports. These filings
show investors or groups that report more than 5% ownership of a public company.
`13D` usually signals possible influence or activist intent; `13G` is usually
passive or exempt ownership reporting.

```bash
sec 13d --ticker TSLA --form 13g --latest 2 --include-amends --pretty
sec 13g --ticker TSLA --latest 5 --include-amends --jsonl
sec schedule13 --cik 1318605 --form all --latest 5 --include-amends --pretty
sec parse --ticker TSLA --form "SC 13G" --latest 1 --include-amends --pretty
```

`--form` accepts `13d`, `13g`, `SC 13D`, `SC 13G`, `SC 13D/A`, `SC 13G/A`,
or `all`. Use `--include-amends` when you want the current amended ownership
picture, which is usually what analysts want.

Each report includes:

- `accession`
- `form`
- `filing_type`
- `is_amendment`
- `activist_intent`
- `issuer_name`
- `security_title`
- `cusip`
- `event_date`
- `reporting_persons`
- `filing_rule`
- `beneficially_owned_shares`
- `percent_of_class`
- `sole_voting_power`
- `shared_voting_power`
- `sole_dispositive_power`
- `shared_dispositive_power`
- `purpose_of_transaction`
- `ownership_summary`
- `signatures`
- `document_url`
- `source_url`

### 13f

Parse 13F-HR information-table holdings. Values include both the SEC-reported
number and a normalized USD value because older 13F filings reported values in
thousands while modern XML filings report dollars.

```bash
sec 13f --cik 1067983 --latest 1 --limit 20 --pretty
sec 13f --ticker BRK-B --limit 50 --jsonl
sec 13f --manager "H&H International Investment LLC" --latest 1 --limit 20 --pretty
```

Each holding includes:

- `accession`
- `manager`
- `report_date`
- `issuer`
- `class`
- `cusip`
- `value_reported`
- `value_scale`
- `value_usd`
- `shares`
- `investment_discretion`
- `voting_sole`
- `voting_shared`
- `voting_none`
- `source_url`

### 13f-aggregate

Aggregate 13F information-table rows by accession, CUSIP, class, and put/call.
This is usually the portfolio view analysts want when a consolidated 13F filing
contains multiple included managers.

```bash
sec 13f-aggregate --cik 1067983 --latest 1 --limit 20 --pretty
sec 13f-aggregate --ticker BRK-B --latest 4 --jsonl
sec 13f-aggregate --investor "Warren Buffett" --latest 1 --limit 20 --pretty
```

Each aggregate holding includes:

- `issuer`
- `class`
- `cusip`
- `put_call`
- `value_reported`
- `value_scale`
- `value_usd`
- `shares`
- `voting_sole`
- `voting_shared`
- `voting_none`
- `rows`
- `source_url`

### 13f-diff

Compare the latest two 13F-HR portfolios after aggregation. This classifies
positions by share-count movement as `new`, `increased`, `reduced`,
`unchanged`, or `exited`, and sorts by the absolute USD value change.

```bash
sec 13f-diff --cik 1067983 --limit 20 --pretty
sec 13f-diff --manager "H&H International Investment LLC" --limit 20 --pretty
sec 13f-diff --investor 段永平 --pretty
sec 13f-diff --ticker BRK-B --jsonl
```

Each diff row includes:

- `current_accession`
- `previous_accession`
- `current_report_date`
- `previous_report_date`
- `issuer`
- `class`
- `cusip`
- `put_call`
- `change_type`
- `current_value_usd`
- `previous_value_usd`
- `change_value_usd`
- `current_shares`
- `previous_shares`
- `change_shares`
- `current_source_url`
- `previous_source_url`

### 13f-summary

Parse the 13F primary document: report metadata, summary totals, signature, and
included manager information.

```bash
sec 13f-summary --cik 1067983 --latest 1 --pretty
sec 13f-summary --ticker BRK-B --latest 4 --jsonl
sec 13f-summary --manager "H&H International Investment LLC" --latest 2 --pretty
```

Each report summary includes:

- `accession`
- `manager`
- `report_date`
- `report_type`
- `total_holdings_reported`
- `total_value_reported`
- `value_scale`
- `total_value_usd`
- `filing_manager_name`
- `signature_name`
- `other_managers`
- `source_url`

### parse

Use the unified parser pipeline for supported forms. This is the interface most
HTTP/MCP adapters and future batch/export jobs should call internally.

```bash
sec parse --ticker AAPL --form 4 --latest 1 --limit 5 --pretty
sec parse --cik 1067983 --form 13F-HR --latest 1 --limit 20 --jsonl
```

Each record is wrapped with a stable `kind`, such as `form4_transaction` or
`thirteenf_holding`.

### forms

List supported structured parser families.

```bash
sec forms --pretty
```

### serve

Run a local JSON HTTP API for apps, dashboards, and local agents. The server uses
the same `SecClient`, cache, parsers, and source-backed records as the CLI.

```bash
SEC_IDENTITY="Your Name your.email@example.com" sec serve --host 127.0.0.1 --port 8716

curl "http://127.0.0.1:8716/health"
curl "http://127.0.0.1:8716/v1/forms"
curl "http://127.0.0.1:8716/v1/filings?ticker=AAPL&form=10-K&latest=1"
curl "http://127.0.0.1:8716/v1/daily?date=2026-05-15&form=8-K&limit=50"
curl "http://127.0.0.1:8716/v1/efts?query=supply%20chain%20risk&form=10-K&from=2024-01-01&to=2024-12-31&limit=10"
curl "http://127.0.0.1:8716/v1/facts?ticker=AAPL&concept=revenue&latest=3"
curl "http://127.0.0.1:8716/v1/statements?ticker=AAPL&statement=income&period=annual&latest=2"
curl "http://127.0.0.1:8716/v1/metrics?ticker=AAPL&period=annual&latest=4"
curl "http://127.0.0.1:8716/v1/company-report?ticker=AAPL&form=10-K&topic=segment"
curl "http://127.0.0.1:8716/v1/8k?ticker=AAPL&item=2.02&latest=5&limit_bytes=600"
curl "http://127.0.0.1:8716/v1/8k-exhibits?ticker=AAPL&category=earnings_release&latest=5"
curl "http://127.0.0.1:8716/v1/13f?cik=1067983&latest=1&limit=20"
curl "http://127.0.0.1:8716/v1/proxy?ticker=AAPL&latest=1"
curl "http://127.0.0.1:8716/v1/prospectus?ticker=RDDT&form=S-1&include_amends=true"
curl "http://127.0.0.1:8716/v1/foreign?ticker=TSM&form=20-F&latest=1"
curl "http://127.0.0.1:8716/v1/fund?cik=0000036405&form=NPORT-P&limit_holdings=10"
curl "http://127.0.0.1:8716/v1/parse?ticker=AAPL&form=4&latest=1&limit=5"
```

Available endpoints:

| Endpoint | Equivalent CLI |
| --- | --- |
| `/health` | health check |
| `/v1/forms` | `sec forms` |
| `/v1/filings` | `sec filings` |
| `/v1/daily` | `sec daily` |
| `/v1/efts` | `sec efts` |
| `/v1/facts` | `sec facts` |
| `/v1/statements` | `sec statements` |
| `/v1/metrics` | `sec metrics` |
| `/v1/company-report` | `sec company-report` |
| `/v1/ixbrl` | `sec ixbrl` |
| `/v1/sections` | `sec section` |
| `/v1/docs` | `sec docs` |
| `/v1/form4`, `/v1/form4-summary` | `sec form4`, `sec form4-summary` |
| `/v1/8k` | `sec 8k` |
| `/v1/8k-exhibits` | `sec 8k-exhibits` |
| `/v1/schedule13` | `sec 13d` / `sec 13g` |
| `/v1/13f`, `/v1/13f-summary`, `/v1/13f-diff` | `sec 13f`, `sec 13f-summary`, `sec 13f-diff` |
| `/v1/proxy` | `sec proxy` |
| `/v1/prospectus` | `sec prospectus` |
| `/v1/foreign` | `sec foreign` |
| `/v1/fund` | `sec fund` |
| `/v1/parse` | `sec parse` |

### mcp

Run a stdio Model Context Protocol adapter for MCP-capable agents. The adapter
uses JSON-RPC over stdin/stdout and exposes source-backed SEC tools without
requiring an HTTP server.

```bash
sec config set-identity "Your Name your.email@example.com"
sec mcp
```

Available MCP tools:

| Tool | What it calls |
| --- | --- |
| `sec_forms` | parser registry |
| `sec_filings` | `sec filings` equivalent |
| `sec_daily` | `sec daily` all-market index scan |
| `sec_efts` | `sec efts` SEC full-text search |
| `sec_facts` | `sec facts` equivalent |
| `sec_statements` | `sec statements` equivalent |
| `sec_metrics` | `sec metrics` equivalent |
| `sec_ixbrl` | `sec ixbrl` equivalent |
| `sec_tables` | `sec tables` equivalent |
| `sec_company_report` | `sec company-report` equivalent |
| `sec_proxy` | `sec proxy` equivalent |
| `sec_prospectus` | `sec prospectus` equivalent |
| `sec_foreign` | `sec foreign` equivalent |
| `sec_fund` | `sec fund` equivalent |
| `sec_search` | `sec search` equivalent |
| `sec_section` | `sec section` equivalent |
| `sec_docs` | `sec docs` equivalent |
| `sec_doc` | `sec doc` equivalent |
| `sec_form4` | `sec form4` equivalent |
| `sec_form4_summary` | `sec form4-summary` equivalent |
| `sec_8k` | `sec 8k` equivalent |
| `sec_8k_exhibits` | `sec 8k-exhibits` equivalent |
| `sec_schedule13` | `sec 13d` / `sec 13g` equivalent |
| `sec_13f` | `sec 13f` equivalent |
| `sec_13f_aggregate` | `sec 13f-aggregate` equivalent |
| `sec_13f_diff` | `sec 13f-diff` equivalent for CIK/ticker selectors |
| `sec_13f_summary` | `sec 13f-summary` equivalent |
| `sec_report` | Markdown reports for `insider`, `portfolio`, and `risk` |
| `sec_parse` | unified parser pipeline for supported forms |

Example MCP tool arguments:

```json
{
  "name": "sec_filings",
  "arguments": {
    "ticker": "AAPL",
    "form": "10-K",
    "latest": 1
  }
}
```

## Options Reference

Global options:

| Option | Meaning |
| --- | --- |
| `--identity <TEXT>` | SEC request identity / user agent. Required unless local config, `SEC_IDENTITY`, or `EDGAR_IDENTITY` is set. |
| `--cache-dir <PATH>` | Override the local response cache directory. |
| `--output <MODE>` | Override structured output globally: `json`, `pretty`, `jsonl`, `csv`, or `table`. |

Command options:

| Command | Required selector | Important options |
| --- | --- | --- |
| `filings` | `--ticker` or `--cik` | `--form`, `--latest`, `--from`, `--to`, `--include-amends`, `--jsonl`, `--pretty` |
| `daily` / `monitor` | none | `--date`, `--form`, `--company`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `efts` / `full-text` / `global-search` | `--query` | `--ticker`, `--cik`, `--form`, `--from`, `--to`, `--limit`, `--jsonl`, `--pretty` |
| `facts` | `--ticker` or `--cik`, `--concept` | `--form`, `--unit`, `--latest`, `--jsonl`, `--pretty` |
| `statements` | `--ticker` or `--cik` | `--statement`, `--period`, `--unit`, `--latest`, `--jsonl`, `--pretty` |
| `metrics` | `--ticker` or `--cik` | `--period`, `--unit`, `--latest`, `--jsonl`, `--pretty` |
| `company-report` | `--ticker` or `--cik` | `--form`, `--topic`, `--latest`, `--limit-tables`, `--limit-rows`, `--include-amends`, `--jsonl`, `--pretty` |
| `ixbrl` | `--ticker` or `--cik` | `--form`, `--concept`, `--latest`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `xbrl-links` / `linkbase` | `--ticker` or `--cik` | `--form`, `--linkbase`, `--role`, `--concept`, `--latest`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `xbrl-tree` / `presentation-tree` | `--ticker` or `--cik` | `--form`, `--role`, `--concept`, `--latest`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `xbrl-calc` / `calculation-checks` | `--ticker` or `--cik` | `--form`, `--role`, `--concept`, `--unit`, `--tolerance`, `--latest`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `xbrl-statement` / `statement-render` | `--ticker` or `--cik` | `--form`, `--role`, `--concept`, `--unit`, `--tolerance`, `--values-only`, `--latest`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `tables` | `--ticker` or `--cik` | `--form`, `--latest`, `--limit-tables`, `--limit-rows`, `--include-amends`, `--jsonl`, `--pretty` |
| `proxy` | `--ticker` or `--cik` | `--latest`, `--limit-rows`, `--include-amends`, `--jsonl`, `--pretty` |
| `prospectus` | `--ticker` or `--cik` | `--form`, `--latest`, `--limit-bytes`, `--limit-tables`, `--limit-rows`, `--include-amends`, `--jsonl`, `--pretty` |
| `foreign` | `--ticker` or `--cik` | `--form`, `--latest`, `--limit-bytes`, `--include-amends`, `--jsonl`, `--pretty` |
| `fund` | `--ticker` or `--cik` | `--form`, `--latest`, `--limit-holdings`, `--limit-bytes`, `--include-amends`, `--jsonl`, `--pretty` |
| `search` | `--ticker` or `--cik`, `--query` | `--form`, `--latest`, `--context`, `--include-amends`, `--jsonl`, `--pretty` |
| `section` | `--ticker` or `--cik`, `--item` | `--form`, `--latest`, `--accession`, `--limit-bytes`, `--include-amends`, `--jsonl`, `--pretty` |
| `report` | `--ticker`, `--cik`, `--manager`, or `--investor`; `--kind` | `--latest`, `--limit`, `--limit-bytes`, `--include-amends` |
| `resolve` | `--query`, `--manager`, or `--cik` | `--no-verify`, `--llm-provider`, `--llm-base-url`, `--llm-model`, `--llm-api-key-env`, `--jsonl`, `--pretty` |
| `docs` | `--ticker` or `--cik` | `--form`, `--latest`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `doc` | `--ticker` or `--cik` | `--form`, `--latest`, `--accession`, `--filename`, `--sequence`, `--primary`, `--limit-bytes`, `--raw`, `--text`, `--jsonl`, `--pretty` |
| `form4` | `--ticker` or `--cik` | `--latest`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `form4-summary` | `--ticker` or `--cik` | `--latest`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `8k` | `--ticker` or `--cik` | `--item`, `--latest`, `--limit`, `--limit-bytes`, `--include-amends`, `--jsonl`, `--pretty` |
| `8k-exhibits` | `--ticker` or `--cik` | `--category`, `--latest`, `--limit`, `--limit-bytes`, `--include-amends`, `--jsonl`, `--pretty` |
| `13d` / `13g` / `schedule13` | `--ticker` or `--cik` | `--form`, `--latest`, `--include-amends`, `--limit-bytes`, `--jsonl`, `--pretty` |
| `13f` | `--ticker`, `--cik`, `--manager`, or `--investor` | `--latest`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `13f-aggregate` | `--ticker`, `--cik`, `--manager`, or `--investor` | `--latest`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `13f-diff` | `--ticker`, `--cik`, `--manager`, or `--investor` | `--latest`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `13f-summary` | `--ticker`, `--cik`, `--manager`, or `--investor` | `--latest`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `parse` | `--ticker` or `--cik`, `--form` | `--latest`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `forms` | none | `--jsonl`, `--pretty` |
| `config` | none | `set-identity <TEXT>`, `show`, `path` |
| `completions` | shell name | `bash`, `zsh`, `fish`, `power-shell`, `elvish` |
| `serve` | none | `--host`, `--port` |
| `mcp` | none | stdio JSON-RPC server; configure local identity or `SEC_IDENTITY` in the agent environment |

## Output Modes

Default output is compact JSON:

```bash
sec filings --ticker AAPL --form 10-K
```

Pretty JSON:

```bash
sec filings --ticker AAPL --form 10-K --pretty
```

JSONL:

```bash
sec facts --ticker AAPL --concept revenue --jsonl
```

CSV:

```bash
sec --output csv filings --ticker AAPL --form 10-K --latest 3
```

Terminal table:

```bash
sec --output table filings --ticker AAPL --form 10-K --latest 3
```

`--output` is global and can be used with any structured command. `sec report`
still prints Markdown, while `sec doc --raw` and `sec doc --text` keep printing
document content.

## Agent Workflows

For AI agents, prefer JSON/JSONL commands when the next step is computation,
filtering, or citation, and prefer `sec report` when the next step is a human
readable briefing.

Useful patterns:

```bash
sec form4-summary --ticker AAPL --latest 5 --pretty
sec 8k --ticker AAPL --item 2.02 --latest 5 --limit-bytes 600 --pretty
sec ixbrl --ticker AAPL --form 10-K --concept NetIncomeLoss --limit 5 --jsonl
sec tables --ticker AAPL --form 10-K --limit-tables 5 --limit-rows 10 --pretty
sec 13d --ticker TSLA --form 13g --latest 2 --include-amends --pretty
sec foreign --ticker TSM --form 20-F --latest 1 --pretty
sec fund --cik 0000036405 --form NPORT-P --latest 1 --limit-holdings 10 --pretty
sec 13f-diff --cik 1067983 --limit 20 --jsonl
sec resolve --query 段永平 --pretty
sec 13f-diff --investor 段永平 --pretty
sec section --ticker AAPL --form 10-K --item risk-factors --limit-bytes 12000 --pretty
sec report --ticker AAPL --kind financial > aapl-financial.md
sec report --ticker AAPL --kind risk > aapl-risk.md
```

Every result is designed to preserve traceability with fields like `accession`,
`document`, `section`, `fact_id`, `source_url`, and `document_url` where
applicable.

## Cache

By default, responses are cached under the system cache directory:

```text
~/Library/Caches/sec-cli   # macOS
~/.cache/sec-cli           # Linux
```

Override with:

```bash
sec --cache-dir ./cache filings --ticker AAPL
```

## Vision

`sec-cli` is not trying to be a Python convenience wrapper. It is a CLI-native
SEC data tool for agents:

- Stable JSON / JSONL
- Source-backed results
- Fast on-demand fetching and parsing
- Local caching by default
- Future Rust streaming parsers for SGML/XBRL
- Future Arrow/Parquet exports for data engineering
