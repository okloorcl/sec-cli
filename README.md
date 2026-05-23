# sec-cli

Agent-ready SEC EDGAR parser and query CLI, powered by Rust.

[![Rust](https://img.shields.io/badge/Rust-2024-orange)](https://www.rust-lang.org/)
[![SEC EDGAR](https://img.shields.io/badge/Data-SEC%20EDGAR-blue)](https://www.sec.gov/edgar)
[![Output](https://img.shields.io/badge/Output-JSON%20%7C%20JSONL%20%7C%20Markdown-green)](#output-modes)
[![Agent Ready](https://img.shields.io/badge/Agent-ready-111827)](#agent-workflows)
[![LLM Resolver](https://img.shields.io/badge/LLM-OpenAI%20%7C%20Anthropic-7c3aed)](#llm-resolver)
[![中文](https://img.shields.io/badge/README-中文-red)](README.zh-CN.md)

| Core | What it gives you |
| --- | --- |
| Insider activity | Form 4 owner, role, transaction code, shares, price, value, footnotes, signatures |
| Institutional holdings | 13F holdings, portfolio summary, top positions, quarter-over-quarter changes |
| Company disclosure | 10-K/10-Q risk factors, MD&A, filing search, exact source snippets |
| Agent interface | Stable JSON/JSONL, LLM name resolution, source URLs, accession numbers |

```bash
sec filings --ticker AAPL --form 10-K
sec facts --ticker AAPL --concept revenue
sec search --ticker TSLA --form 10-K --query "supply chain risk"
sec section --ticker AAPL --form 10-K --item risk-factors --limit-bytes 8000
sec report --ticker AAPL --kind risk
sec resolve --query 段永平 --pretty
sec 13f-diff --investor 段永平 --pretty
sec report --cik 1067983 --kind portfolio --limit 10
sec docs --ticker AAPL --form 10-K --latest 1 --limit 20
sec doc --ticker AAPL --form 10-K --primary --limit-bytes 4000
sec form4 --ticker AAPL --latest 3
sec form4-summary --ticker AAPL --latest 3
sec 13f --cik 1067983 --latest 1
sec 13f-aggregate --cik 1067983 --latest 1 --limit 20
sec 13f-diff --cik 1067983 --limit 20
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
- Extracting common 10-K/10-Q sections such as business, risk factors, and MD&A
- Generating source-backed Markdown reports for insider activity, 13F portfolios, and risk review
- Resolving public investor/fund/person names through an LLM, then validating candidates against SEC 13F filings
- Listing and reading individual SEC submission documents
- Parsing Form 4 insider ownership transactions
- Summarizing Form 4 reports, owners, signatures, footnotes, and net activity
- Parsing 13F-HR information-table holdings
- Comparing the latest two 13F-HR portfolios
- Parsing 13F-HR cover, summary, signature, and manager metadata
- Returning JSON arrays or JSONL records
- Caching SEC responses locally

Longer term, the project aims to grow into a Rust-powered SEC disclosure engine:
more form-specific parsers, XBRL streaming parsing, table extraction,
Parquet/Arrow exports, and agent-native query workflows.

## What You Can Answer Accurately

These are useful, source-backed questions that work today:

| Question | Command |
| --- | --- |
| What did insiders recently buy or sell? | `sec form4 --ticker AAPL --latest 5 --pretty` |
| Which executives/directors filed Form 4s and what was net activity? | `sec form4-summary --ticker AAPL --latest 5 --pretty` |
| What is Berkshire Hathaway's latest 13F portfolio? | `sec 13f-aggregate --cik 1067983 --limit 20 --pretty` |
| What changed between the latest two 13F filings? | `sec 13f-diff --cik 1067983 --limit 20 --pretty` |
| What if I know the investor name but not the CIK? | `sec resolve --query 段永平 --pretty`, then `sec 13f-diff --investor 段永平 --pretty` |
| What are a company's latest 10-K risk factors? | `sec section --ticker AAPL --form 10-K --item risk-factors --pretty` |
| Where did the answer come from? | Every structured result includes `source_url`; document results also include `document_url` |

`edgartools` already has Python objects, rich displays, DataFrame exports, AI
context helpers, and many filing-type parsers. `sec-cli` is deliberately
different: it is a standalone Rust CLI optimized for automation and agents. The
parity goal is to cover the useful structured outputs edgartools exposes, while
adding stable command-line schemas, precise exit behavior, source URLs on every
record, and Markdown reports that can be dropped directly into an analyst note.

## Architecture

The code is intentionally split by responsibility:

- `cli`: command arguments and CLI orchestration only
- `lib`: reusable Rust core for CLI, future API, and future MCP adapters
- `http`: low-level SEC HTTP
- `storage`: local byte cache/store
- `client`: SEC domain facade, ticker-to-CIK lookup
- `edgar`: SEC data sources, submissions, facts, archive URLs
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

## LLM Resolver

`sec resolve` does not use a hardcoded investor map. It asks an LLM for candidate
SEC 13F filing managers, then verifies each candidate by checking SEC
`13F-HR` filings. The LLM is used for name understanding; SEC data remains the
source of truth.

OpenAI-compatible config, including GLM/BigModel-compatible endpoints:

```bash
mkdir -p ~/.config/sec-cli
cat > ~/.config/sec-cli/llm.json <<'JSON'
{
  "provider": "openai",
  "base_url": "https://open.bigmodel.cn/api/coding/paas/v4",
  "model": "GLM-4.7",
  "api_key_env": "BIGMODEL_API_KEY"
}
JSON

export BIGMODEL_API_KEY="your-api-key"
sec resolve --query 段永平 --pretty
```

Anthropic-compatible config:

```json
{
  "provider": "anthropic",
  "base_url": "https://open.bigmodel.cn/api/anthropic",
  "model": "GLM-4.7",
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
  --llm-model GLM-4.7 \
  --llm-api-key-env BIGMODEL_API_KEY \
  --pretty
```

Do not commit API keys. Keep local config files private and prefer environment
variables for secrets.

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

### section

Extract common sections from the primary 10-K or 10-Q document. The extractor
normalizes HTML/XBRL markup to text, locates item headings, chooses the largest
matching body over table-of-contents hits, and returns source-backed JSON.

```bash
sec section --ticker AAPL --form 10-K --item risk-factors --limit-bytes 8000 --pretty
sec section --ticker MSFT --form 10-K --item mda --latest 1 --pretty
sec section --cik 320193 --form 10-K --item 7 --jsonl
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
sec report --cik 1067983 --kind portfolio --limit 10
sec report --ticker AAPL --kind risk --limit-bytes 4000
```

Report kinds:

- `insider`: Form 4 summary table with owner, role, net shares, value, and SEC source
- `portfolio`: 13F summary, top holdings, visual bars, and largest position changes
- `risk`: 10-K risk factor and MD&A excerpts with source links

### resolve

Resolve an investor, fund, or public person name to SEC 13F filing manager
candidates. SEC filings are filed by legal entities, so a public name often
needs model-assisted interpretation before SEC validation.

```bash
sec resolve --query 段永平 --pretty
sec resolve --query "Warren Buffett" --pretty
sec resolve --query "Seth Klarman" --no-verify --pretty
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

### 13f-diff

Compare the latest two 13F-HR portfolios after aggregation. This classifies
positions by share-count movement as `new`, `increased`, `reduced`,
`unchanged`, or `exited`, and sorts by the absolute USD value change.

```bash
sec 13f-diff --cik 1067983 --limit 20 --pretty
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

## Options Reference

Global options:

| Option | Meaning |
| --- | --- |
| `--identity <TEXT>` | SEC request identity / user agent. Prefer a real name and email. |
| `--cache-dir <PATH>` | Override the local response cache directory. |

Command options:

| Command | Required selector | Important options |
| --- | --- | --- |
| `filings` | `--ticker` or `--cik` | `--form`, `--latest`, `--from`, `--to`, `--include-amends`, `--jsonl`, `--pretty` |
| `facts` | `--ticker` or `--cik`, `--concept` | `--form`, `--unit`, `--latest`, `--jsonl`, `--pretty` |
| `search` | `--ticker` or `--cik`, `--query` | `--form`, `--latest`, `--context`, `--include-amends`, `--jsonl`, `--pretty` |
| `section` | `--ticker` or `--cik`, `--item` | `--form`, `--latest`, `--accession`, `--limit-bytes`, `--include-amends`, `--jsonl`, `--pretty` |
| `report` | `--ticker`, `--cik`, or `--investor`; `--kind` | `--latest`, `--limit`, `--limit-bytes`, `--include-amends` |
| `resolve` | `--query` | `--no-verify`, `--llm-provider`, `--llm-base-url`, `--llm-model`, `--llm-api-key-env`, `--jsonl`, `--pretty` |
| `docs` | `--ticker` or `--cik` | `--form`, `--latest`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `doc` | `--ticker` or `--cik` | `--form`, `--latest`, `--accession`, `--filename`, `--sequence`, `--primary`, `--limit-bytes`, `--raw`, `--text`, `--jsonl`, `--pretty` |
| `form4` | `--ticker` or `--cik` | `--latest`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `form4-summary` | `--ticker` or `--cik` | `--latest`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `13f` | `--ticker`, `--cik`, or `--investor` | `--latest`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `13f-aggregate` | `--ticker`, `--cik`, or `--investor` | `--latest`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `13f-diff` | `--ticker`, `--cik`, or `--investor` | `--latest`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `13f-summary` | `--ticker`, `--cik`, or `--investor` | `--latest`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `parse` | `--ticker` or `--cik`, `--form` | `--latest`, `--limit`, `--include-amends`, `--jsonl`, `--pretty` |
| `forms` | none | `--jsonl`, `--pretty` |

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

## Agent Workflows

For AI agents, prefer JSON/JSONL commands when the next step is computation,
filtering, or citation, and prefer `sec report` when the next step is a human
readable briefing.

Useful patterns:

```bash
sec form4-summary --ticker AAPL --latest 5 --pretty
sec 13f-diff --cik 1067983 --limit 20 --jsonl
sec resolve --query 段永平 --pretty
sec 13f-diff --investor 段永平 --pretty
sec section --ticker AAPL --form 10-K --item risk-factors --limit-bytes 12000 --pretty
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
