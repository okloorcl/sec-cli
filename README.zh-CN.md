# sec-cli

面向 AI Agent 和金融分析自动化的 SEC EDGAR 高速解析 CLI，Rust 实现。

[![Rust](https://img.shields.io/badge/Rust-2024-orange)](https://www.rust-lang.org/)
[![SEC EDGAR](https://img.shields.io/badge/Data-SEC%20EDGAR-blue)](https://www.sec.gov/edgar)
[![Output](https://img.shields.io/badge/Output-JSON%20%7C%20JSONL%20%7C%20Markdown-green)](#输出模式)
[![Agent Ready](https://img.shields.io/badge/Agent-ready-111827)](#agent-工作流)
[![English](https://img.shields.io/badge/README-English-blue)](README.md)

| 核心能力 | 能拿到什么 |
| --- | --- |
| 高管/董事交易 | Form 4 owner、职位、交易代码、股数、价格、金额、脚注、签名 |
| 机构持仓 | 13F 持仓、组合摘要、Top holdings、季度变化 |
| 公司披露 | 10-K/10-Q 风险因素、MD&A、全文搜索、精确来源片段 |
| Agent 接口 | 稳定 JSON/JSONL、source URL、accession、document 元数据 |

```bash
sec filings --ticker AAPL --form 10-K
sec facts --ticker AAPL --concept revenue
sec search --ticker TSLA --form 10-K --query "supply chain risk"
sec section --ticker AAPL --form 10-K --item risk-factors --limit-bytes 8000
sec report --ticker AAPL --kind risk
sec investor --query 段永平 --pretty
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

`sec-cli` 把 SEC filing 转成可追溯、结构稳定的 JSON/JSONL/Markdown。它不是 Python 包的外壳，而是独立 Rust CLI：适合 shell、数据流水线、本地服务、MCP/Agent 调用。

## 当前状态

当前 MVP 已支持：

- 查询 company filings
- 查询 SEC CompanyFacts
- 搜索 filing 原文并返回 snippet
- 抽取 10-K/10-Q 常用 section：Business、Risk Factors、MD&A 等
- 生成 Markdown 专业汇报：insider、portfolio、risk
- 列出和读取 complete submission 内的 documents
- 解析 Form 4 交易明细
- 汇总 Form 4 报告、owner、签名、脚注、净买卖
- 解析 13F-HR information table
- 聚合 13F 持仓
- 比较最近两期 13F 组合变化
- 解析 13F cover、summary、signature、other managers
- 本地缓存 SEC 响应

长期目标：逐步覆盖 edgartools 里有价值的结构化输出，同时保持 CLI/Agent 原生体验：稳定 schema、明确 exit code、来源链接、Markdown 汇报、未来 Arrow/Parquet 和本地 HTTP/MCP。

## 能准确回答什么

| 问题 | 命令 |
| --- | --- |
| 最近高管/董事买卖了什么？ | `sec form4 --ticker AAPL --latest 5 --pretty` |
| 哪些 owner 提交了 Form 4，净买卖是多少？ | `sec form4-summary --ticker AAPL --latest 5 --pretty` |
| Berkshire 最新 13F 持仓是什么？ | `sec 13f-aggregate --cik 1067983 --limit 20 --pretty` |
| 最近两期 13F 哪些仓位变化最大？ | `sec 13f-diff --cik 1067983 --limit 20 --pretty` |
| 我只知道投资人名字，不知道 CIK？ | `sec investor --query 段永平 --pretty`，然后 `sec 13f-diff --investor 段永平 --pretty` |
| 公司最新 10-K 风险因素是什么？ | `sec section --ticker AAPL --form 10-K --item risk-factors --pretty` |
| 生成能直接给人看的分析摘要？ | `sec report --ticker AAPL --kind risk` |
| 答案来源在哪里？ | 结构化结果包含 `source_url`，document 结果还包含 `document_url` |

## 架构

| 模块 | 职责 |
| --- | --- |
| `cli` | 命令参数和 orchestration |
| `client` | SEC domain facade、ticker 到 CIK |
| `http` | SEC HTTP 请求 |
| `storage` | 本地缓存 |
| `edgar` | SEC submissions、facts、archive URL |
| `documents` | complete submission 拆分、document 选择、读取 |
| `parsers` | XML helper、form-specific parsers |
| `sections` | 10-K/10-Q section 抽取 |
| `reports` | Markdown 汇报生成 |
| `models` | query DTO 和稳定输出 record |
| `registry` | parser discovery |
| `pipeline` | 统一 form parser 分发 |
| `search` | filing text search |
| `output` | JSON / JSONL 输出 |

## 安装

```bash
cargo install --path .
```

开发时：

```bash
cargo run --bin sec -- filings --ticker AAPL --form 10-K --latest 2 --pretty
```

建议设置 SEC identity：

```bash
export SEC_IDENTITY="Your Name your.email@example.com"
```

或单次命令传入：

```bash
sec --identity "Your Name your.email@example.com" filings --ticker AAPL
```

## 命令

### filings

查询某公司近期 filing。

```bash
sec filings --ticker AAPL --form 10-K --latest 3 --pretty
sec filings --cik 320193 --form 10-Q --from 2023-01-01 --to 2025-12-31
```

输出字段：`accession`、`cik`、`company`、`form`、`filing_date`、`report_date`、`primary_document`、`source_url`、`text_url`。

### facts

查询 SEC CompanyFacts。

```bash
sec facts --ticker AAPL --concept revenue --form 10-K --latest 5 --pretty
sec facts --ticker MSFT --concept us-gaap:NetIncomeLoss --latest 10 --jsonl
```

输出字段：`concept`、`taxonomy`、`label`、`description`、`value`、`unit`、`fy`、`fp`、`form`、`filed`、`start`、`end`、`frame`、`accession`、`source_url`、`fact_id`。

### search

搜索 filing 原文，返回可追溯 snippet。

```bash
sec search --ticker TSLA --form 10-K --query "risk factors" --latest 1 --pretty
sec search --ticker NVDA --form 10-K --query "export controls" --jsonl
```

输出字段：`accession`、`cik`、`company`、`form`、`filing_date`、`query`、`document`、`section`、`offset`、`snippet`、`source_url`。

### section

抽取 10-K/10-Q 常用 section。

```bash
sec section --ticker AAPL --form 10-K --item risk-factors --limit-bytes 8000 --pretty
sec section --ticker MSFT --form 10-K --item mda --latest 1 --pretty
```

支持：`business`/`1`、`risk-factors`/`1A`、`cybersecurity`/`1C`、`properties`/`2`、`legal-proceedings`/`3`、`mda`/`7`、`market-risk`/`7A`、`financial-statements`/`8`。

输出字段：`accession`、`item`、`title`、`start_offset`、`end_offset`、`byte_length`、`returned_bytes`、`truncated`、`document_url`、`source_url`、`content`。

### report

生成 Markdown 专业汇报。

```bash
sec report --ticker AAPL --kind insider --latest 5 --limit 10
sec report --investor 段永平 --kind portfolio --limit 10
sec report --cik 1067983 --kind portfolio --limit 10
sec report --ticker AAPL --kind risk --limit-bytes 4000
```

`--kind`：

- `insider`：Form 4 owner、role、净股数、交易金额、SEC 来源
- `portfolio`：13F 摘要、Top holdings、可视化条、最大仓位变化
- `risk`：10-K Risk Factors 和 MD&A 摘要

### investor

把投资人别名解析到 SEC 13F filing manager 和 CIK。SEC 文件通常由法律实体提交，不一定直接用公众熟悉的投资人姓名。

```bash
sec investor --query 段永平 --pretty
sec investor --query "Warren Buffett" --pretty
```

输出字段：`investor`、`manager`、`cik`、`relationship`、`aliases`、`confidence`、`note`。

### docs / doc

`docs` 列出 submission 内所有 documents；`doc` 读取具体 document。

```bash
sec docs --ticker AAPL --form 10-K --latest 1 --limit 20 --pretty
sec doc --ticker AAPL --form 10-K --primary --limit-bytes 4000 --pretty
sec doc --ticker AAPL --form 10-K --sequence 1 --text --limit-bytes 12000
sec doc --cik 320193 --accession 0000320193-25-000079 --filename aapl-20250927.htm --raw
```

`docs` 输出字段：`accession`、`document_type`、`sequence`、`filename`、`description`、`content_type`、`byte_length`、`is_primary`、`document_url`、`source_url`。

`doc` 额外输出：`returned_bytes`、`truncated`、`content`。

### form4 / form4-summary

解析高管/董事 Form 4。

```bash
sec form4 --ticker AAPL --latest 3 --limit 10 --pretty
sec form4-summary --ticker AAPL --latest 3 --limit 10 --pretty
```

`form4` 输出交易字段：`issuer`、`issuer_ticker`、`reporting_owner`、`owner_cik`、`officer_title`、`transaction_date`、`transaction_code`、`transaction_type`、`security_title`、`shares`、`price`、`value`、`shares_owned_after`、`direct_or_indirect`、`derivative`、`source_url` 等。

`form4-summary` 输出报告字段：`period_of_report`、`owners`、`signatures`、`footnotes`、`transaction_count`、`acquisition_count`、`disposition_count`、`total_shares_acquired`、`total_shares_disposed`、`net_shares`、`total_value`、`source_url`。

### 13F

解析和分析机构 13F。

```bash
sec 13f --cik 1067983 --latest 1 --limit 20 --pretty
sec 13f-aggregate --cik 1067983 --latest 1 --limit 20 --pretty
sec 13f-diff --cik 1067983 --limit 20 --pretty
sec 13f-diff --investor 段永平 --pretty
sec 13f-summary --cik 1067983 --latest 1 --pretty
```

`13f` 输出字段：`issuer`、`class`、`cusip`、`value_reported`、`value_scale`、`value_usd`、`shares`、`share_type`、`put_call`、`investment_discretion`、`voting_sole`、`voting_shared`、`voting_none`、`source_url`。

`13f-aggregate` 输出聚合字段：`issuer`、`class`、`cusip`、`put_call`、`value_usd`、`shares`、`voting_sole`、`voting_shared`、`voting_none`、`rows`。

`13f-diff` 输出变化字段：`current_accession`、`previous_accession`、`current_report_date`、`previous_report_date`、`issuer`、`cusip`、`change_type`、`current_value_usd`、`previous_value_usd`、`change_value_usd`、`current_shares`、`previous_shares`、`change_shares`、`current_source_url`、`previous_source_url`。

`13f-summary` 输出报告字段：`report_date`、`report_type`、`total_holdings_reported`、`total_value_reported`、`value_scale`、`total_value_usd`、`filing_manager_name`、`signature_name`、`other_managers`、`source_url`。

### parse / forms

统一 parser pipeline 和 parser 列表。

```bash
sec parse --ticker AAPL --form 4 --latest 1 --limit 5 --pretty
sec parse --cik 1067983 --form 13F-HR --latest 1 --limit 20 --jsonl
sec forms --pretty
```

## 参数参考

全局参数：

| 参数 | 含义 |
| --- | --- |
| `--identity <TEXT>` | SEC 请求身份 / user agent |
| `--cache-dir <PATH>` | 指定本地缓存目录 |

命令参数：

| 命令 | 必要选择器 | 重要参数 |
| --- | --- | --- |
| `filings` | `--ticker` 或 `--cik` | `--form`、`--latest`、`--from`、`--to`、`--include-amends`、`--jsonl`、`--pretty` |
| `facts` | `--ticker` 或 `--cik`，`--concept` | `--form`、`--unit`、`--latest`、`--jsonl`、`--pretty` |
| `search` | `--ticker` 或 `--cik`，`--query` | `--form`、`--latest`、`--context`、`--include-amends`、`--jsonl`、`--pretty` |
| `section` | `--ticker` 或 `--cik`，`--item` | `--form`、`--latest`、`--accession`、`--limit-bytes`、`--include-amends`、`--jsonl`、`--pretty` |
| `report` | `--ticker`、`--cik` 或 `--investor`，`--kind` | `--latest`、`--limit`、`--limit-bytes`、`--include-amends` |
| `investor` | `--query` | `--jsonl`、`--pretty` |
| `docs` | `--ticker` 或 `--cik` | `--form`、`--latest`、`--limit`、`--include-amends`、`--jsonl`、`--pretty` |
| `doc` | `--ticker` 或 `--cik` | `--form`、`--latest`、`--accession`、`--filename`、`--sequence`、`--primary`、`--limit-bytes`、`--raw`、`--text`、`--jsonl`、`--pretty` |
| `form4` / `form4-summary` | `--ticker` 或 `--cik` | `--latest`、`--limit`、`--include-amends`、`--jsonl`、`--pretty` |
| `13f` / `13f-aggregate` / `13f-diff` / `13f-summary` | `--ticker`、`--cik` 或 `--investor` | `--latest`、`--limit`、`--include-amends`、`--jsonl`、`--pretty` |
| `parse` | `--ticker` 或 `--cik`，`--form` | `--latest`、`--limit`、`--include-amends`、`--jsonl`、`--pretty` |
| `forms` | 无 | `--jsonl`、`--pretty` |

## 输出模式

- 默认：紧凑 JSON
- `--pretty`：格式化 JSON
- `--jsonl`：一行一个 JSON record
- `sec report`：Markdown 汇报
- `sec doc --raw`：原始 document 内容
- `sec doc --text`：简化纯文本

## Agent 工作流

推荐：

```bash
sec form4-summary --ticker AAPL --latest 5 --pretty
sec 13f-diff --cik 1067983 --limit 20 --jsonl
sec 13f-diff --investor 段永平 --pretty
sec section --ticker AAPL --form 10-K --item risk-factors --limit-bytes 12000 --pretty
sec report --ticker AAPL --kind risk > aapl-risk.md
```

如果下一步是计算、过滤、引用，优先 JSON/JSONL；如果下一步是给人读或让 LLM 写分析，优先 `sec report`。

## 缓存

默认缓存目录：

```text
~/Library/Caches/sec-cli   # macOS
~/.cache/sec-cli           # Linux
```

覆盖缓存目录：

```bash
sec --cache-dir ./cache filings --ticker AAPL
```

## 与 edgartools 的关系

edgartools 已经有 Python 对象、Rich display、DataFrame、AI context、多 filing 类型解析。`sec-cli` 的目标不是复制 Python API，而是做独立 Rust CLI：

- 原生命令行
- 稳定 JSON/JSONL
- 每条结果带 source URL / accession / document
- 可被 agent 可靠调用
- 快速按需下载、缓存、解析
- Markdown 报告适合直接进入研究笔记

目标是逐步覆盖 edgartools 里有意义的结构化输出，并在自动化、速度、低内存、可追溯性上更强。
