# sec-cli

Agent-ready SEC EDGAR parser and query CLI, powered by Rust.

```bash
sec filings --ticker AAPL --form 10-K
sec facts --ticker AAPL --concept revenue
sec search --ticker TSLA --form 10-K --query "supply chain risk"
sec docs --ticker AAPL --form 10-K --latest 1 --limit 20
sec form4 --ticker AAPL --latest 3
sec 13f --cik 1067983 --latest 1
sec 13f-aggregate --cik 1067983 --latest 1 --limit 20
sec 13f-summary --cik 1067983 --latest 1
sec parse --ticker AAPL --form 4 --latest 1
sec forms --pretty
```

`sec-cli` turns SEC filings into source-backed JSON for agents, analysts, and
data workflows. It is designed as a fast command-line tool first: stable output,
clear exit codes, local caching, and source URLs on every result.

## Status

This is an early MVP. The first implementation focuses on:

- Finding company filings from SEC submissions data
- Querying SEC CompanyFacts for source-backed XBRL facts
- Searching filing submission text with snippets
- Parsing Form 4 insider ownership transactions
- Parsing 13F-HR information-table holdings
- Parsing 13F-HR cover, summary, signature, and manager metadata
- Returning JSON arrays or JSONL records
- Caching SEC responses locally

Longer term, the project aims to grow into a Rust-powered SEC disclosure engine:
more form-specific parsers, XBRL streaming parsing, table extraction,
Parquet/Arrow exports, and agent-native query workflows.

## Architecture

The code is intentionally split by responsibility:

- `cli`: command arguments and CLI orchestration only
- `lib`: reusable Rust core for CLI, future API, and future MCP adapters
- `http`: low-level SEC HTTP
- `storage`: local byte cache/store
- `client`: SEC domain facade, ticker-to-CIK lookup
- `edgar`: SEC data sources, submissions, facts, archive URLs
- `documents`: complete-submission `.txt` splitting and attachment selection
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

From this repository:

```bash
cargo install --path .
```

During development:

```bash
cargo run --bin sec -- filings --ticker AAPL --form 10-K --latest 2 --pretty
```

SEC requests should include a real identity:

```bash
export SEC_IDENTITY="Your Name your.email@example.com"
```

You can also pass it per command:

```bash
sec --identity "Your Name your.email@example.com" filings --ticker AAPL
```

## Commands

### filings

Find recent filings by ticker or CIK.

```bash
sec filings --ticker AAPL --form 10-K --latest 3 --pretty
sec filings --cik 320193 --form 10-Q --from 2023-01-01 --to 2025-12-31
sec filings --ticker TSLA --form 10-K --include-amends --jsonl
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

### facts

Query SEC CompanyFacts by concept alias or XBRL concept name.

```bash
sec facts --ticker AAPL --concept revenue --form 10-K --latest 5 --pretty
sec facts --ticker MSFT --concept us-gaap:NetIncomeLoss --latest 10 --jsonl
sec facts --cik 320193 --concept assets --unit USD
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

### search

Search filing submission text and return source-backed snippets.

```bash
sec search --ticker TSLA --form 10-K --query "risk factors" --latest 1 --pretty
sec search --ticker AAPL --form 10-K --query "supply chain risk" --context 300
sec search --ticker NVDA --form 10-K --query "export controls" --jsonl
```

Search first tries an exact case-insensitive phrase match, then falls back to a
token-window match so agent queries are more robust against SEC HTML markup.

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

### form4

Parse Form 4 ownership transactions.

```bash
sec form4 --ticker AAPL --latest 3 --limit 10 --pretty
sec form4 --cik 320193 --include-amends --jsonl
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

### 13f

Parse 13F-HR information-table holdings. Values include both the SEC-reported
number and a normalized USD value because older 13F filings reported values in
thousands while modern XML filings report dollars.

```bash
sec 13f --cik 1067983 --latest 1 --limit 20 --pretty
sec 13f --ticker BRK-B --limit 50 --jsonl
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

### 13f-summary

Parse the 13F primary document: report metadata, summary totals, signature, and
included manager information.

```bash
sec 13f-summary --cik 1067983 --latest 1 --pretty
sec 13f-summary --ticker BRK-B --latest 4 --jsonl
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
future API/MCP adapters should call internally.

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
