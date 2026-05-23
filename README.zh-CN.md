# sec-cli

面向 AI Agent 和金融分析自动化的 SEC EDGAR 高速解析 CLI，Rust 实现。

[![Rust](https://img.shields.io/badge/Rust-2024-orange)](https://www.rust-lang.org/)
[![CI](https://github.com/okloorcl/sec-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/okloorcl/sec-cli/actions/workflows/ci.yml)
[![Release](https://github.com/okloorcl/sec-cli/actions/workflows/release.yml/badge.svg)](https://github.com/okloorcl/sec-cli/actions/workflows/release.yml)
[![SEC EDGAR](https://img.shields.io/badge/Data-SEC%20EDGAR-blue)](https://www.sec.gov/edgar)
[![Output](https://img.shields.io/badge/Output-JSON%20%7C%20CSV%20%7C%20Parquet-green)](#输出模式)
[![Agent Ready](https://img.shields.io/badge/Agent-ready-111827)](#agent-工作流)
[![LLM Resolver](https://img.shields.io/badge/LLM-OpenAI%20%7C%20Anthropic-7c3aed)](#llm-resolver)
[![English](https://img.shields.io/badge/README-English-blue)](README.md)

| 核心能力 | 能拿到什么 |
| --- | --- |
| 高管/董事交易 | Form 4 owner、职位、交易代码、股数、价格、金额、脚注、签名 |
| 机构持仓 | 13F 持仓、组合摘要、Top holdings、季度变化 |
| 公司披露 | 8-K 事件、10-K/10-Q 风险因素、MD&A、20-F/6-K/40-F 外国发行人披露、全文搜索 |
| 基金披露 | N-PORT 持仓、N-CSR 股东报告、N-CEN 年度运营、N-PX 投票、497K 摘要、24F 通知 |
| 资本市场 | S-1/F-1/424B 招股书条款、IPO 信号、募资用途、风险、承销商 |
| 财务分析 | SEC 数据推导的利润率、增长率、自由现金流、ROA/ROE、流动性、杠杆 |
| 市场监控 | 按日期、表格、公司扫描 SEC daily master index 全市场新增 filing |
| 全市场搜索 | SEC EDGAR Full-Text Search / EFTS，按关键词、公司、form、日期搜全文 |
| Agent 接口 | 稳定 JSON/JSONL、source URL、accession、document 元数据 |

```bash
sec filings --ticker AAPL --form 10-K
sec daily --date 2026-05-15 --form 8-K --limit 50 --pretty
sec efts --query "supply chain risk" --form 10-K --from 2024-01-01 --to 2024-12-31 --limit 10 --pretty
sec facts --ticker AAPL --concept revenue
sec statements --ticker AAPL --statement income --period annual --latest 4
sec stitch --ticker AAPL --statement income --latest 8 --pretty
sec metrics --ticker AAPL --period annual --latest 4 --pretty
sec scores --ticker AAPL --period annual --latest 1 --pretty
sec export --kind metrics --ticker AAPL --period annual --latest 4 --format parquet --out aapl_metrics.parquet
sec archive --ticker AAPL --form 10-K --latest 2 --primary-only --out-dir ./archives/aapl
sec agent-pack --ticker AAPL --sections risk-factors,mda --metrics-latest 4 --pretty
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

`sec-cli` 把 SEC filing 转成可追溯、结构稳定的 JSON/JSONL/Markdown。它不是 Python 包的外壳，而是独立 Rust CLI：适合 shell、数据流水线、本地服务、MCP/Agent 调用。

## 当前状态

当前 MVP 已支持：

- 查询 company filings
- 扫描 SEC daily master index：按日期、form、公司名和修正版过滤全市场新增 filing
- 调用 SEC EDGAR Full-Text Search / EFTS：按关键词、ticker/CIK、form、日期做全市场全文搜索
- 查询 SEC CompanyFacts
- 从 CompanyFacts 组装更宽的标准化 10-K/10-Q 三大表：利润表、资产负债表、现金流量表
- 基于 SEC CompanyFacts 计算二次分析指标：增长率、利润率、自由现金流、ROA/ROE、流动比率、杠杆
- 直接从 filing HTML 流式解析 Inline XBRL facts
- 解析 XBRL presentation、calculation、definition、label 和 schema linkbase 附件
- 从 filing 主 HTML 抽取表格
- 深度解析 10-K/10-Q 专题表：segment revenue、geography、debt maturity、contract obligations、lease、tax、share repurchases
- 解析 DEF 14A 股东大会委托书：会议、投票事项、董事候选人、审计师、高管薪酬表
- 解析 S-1/F-1/424B 招股书和发行说明书：证券类型、ticker/交易所、价格区间、募资用途、风险、承销商、关键表格
- 解析 20-F/6-K/40-F 外国发行人披露：年度报告、当前报告、交易所/代码、审计师、事件信号、关键章节摘要
- 解析 N-PORT/N-CSR/N-CEN/N-PX/497K/24F-2NT 基金披露：组合持仓、基金元数据、代理投票、摘要招募书、年度证券销售通知、股东报告摘要、财务报表和内控章节
- 搜索 filing 原文并返回 snippet
- 抽取 10-K/10-Q 常用 section：Business、Risk Factors、MD&A 等
- 生成 Markdown 专业汇报：insider、portfolio、risk
- 用 LLM 解析投资人/基金/公众人物名称，再用 SEC 13F filing 验证候选 CIK
- 列出和读取 complete submission 内的 documents
- 解析 Form 4 交易明细
- 汇总 Form 4 报告、owner、签名、脚注、净买卖
- 解析 Form 8-K 当前报告事件 item
- 发现并分类 8-K 附件：earnings release、press release、material contract、agreement、XBRL、accountant letter 等
- 解析 Schedule 13D/13G 大股东 5% 受益持仓报告
- 解析 13F-HR information table
- 聚合 13F 持仓
- 比较最近两期 13F 组合变化
- 解析 13F cover、summary、signature、other managers
- 输出 JSON、JSONL、CSV、终端表格和 Markdown 汇报
- 本地缓存 SEC 响应
- 通过本地 JSON HTTP API 对外提供同一套核心查询
- 通过 stdio MCP adapter 给 Agent 暴露 SEC 查询、解析和报告工具

长期目标：基于当前 CLI/HTTP/MCP/export/offline archive 继续补强更深 Agent 查询工作流。

## 能准确回答什么

| 问题 | 命令 |
| --- | --- |
| 最近高管/董事买卖了什么？ | `sec form4 --ticker AAPL --latest 5 --pretty` |
| 哪些 owner 提交了 Form 4，净买卖是多少？ | `sec form4-summary --ticker AAPL --latest 5 --pretty` |
| 公司最近提交了哪些 8-K 事件？ | `sec 8k --ticker AAPL --latest 5 --pretty` |
| 公司有没有 earnings 相关 8-K？ | `sec 8k --ticker AAPL --item 2.02 --latest 5 --pretty` |
| 8-K 附件里有哪些 earnings release 或重要合同？ | `sec 8k-exhibits --ticker AAPL --category earnings_release --latest 5 --pretty` |
| 某一天全市场提交了哪些 filing？ | `sec daily --date 2026-05-15 --form 8-K --limit 50 --pretty` |
| 哪些公司在 SEC 文件里提到了某个关键词？ | `sec efts --query "supply chain risk" --form 10-K --from 2024-01-01 --to 2024-12-31 --pretty` |
| 最新标准化财报三大表是什么？ | `sec statements --ticker AAPL --statement all --period annual --latest 1 --pretty` |
| 最新 SEC 原始数据推导的财务指标是什么？ | `sec metrics --ticker AAPL --period annual --latest 4 --pretty` |
| 能不能直接生成一份财务趋势 Markdown？ | `sec report --ticker AAPL --kind financial --latest 4` |
| filing HTML 里嵌入了哪些 Inline XBRL facts？ | `sec ixbrl --ticker AAPL --form 10-K --concept RevenueFromContractWithCustomerExcludingAssessedTax --pretty` |
| filing 里有哪些 HTML 表格？ | `sec tables --ticker AAPL --form 10-K --limit-tables 5 --limit-rows 10 --pretty` |
| 10-K/10-Q 里哪些专题表涉及分部、地域、债务、义务、租赁、税、回购？ | `sec company-report --ticker AAPL --form 10-K --topic segment --pretty` |
| 最新 proxy statement 里有哪些投票事项和高管薪酬？ | `sec proxy --ticker AAPL --latest 1 --pretty` |
| IPO 招股书关键条款是什么？ | `sec prospectus --ticker RDDT --form S-1 --include-amends --latest 1 --pretty` |
| 外国发行人最新年报/当前报告披露了什么？ | `sec foreign --ticker TSM --form 20-F --latest 1 --pretty` |
| 基金在 N-PORT 里披露了哪些持仓？ | `sec fund --cik 0000036405 --form NPORT-P --latest 1 --limit-holdings 10 --pretty` |
| 基金在 N-PX 里怎么投票？ | `sec fund --cik 0000036405 --form N-PX --latest 1 --limit-holdings 20 --pretty` |
| 哪些 5% 大股东提交了 13D/13G？ | `sec 13d --ticker TSLA --form 13g --include-amends --pretty` |
| Berkshire 最新 13F 持仓是什么？ | `sec 13f-aggregate --cik 1067983 --limit 20 --pretty` |
| 最近两期 13F 哪些仓位变化最大？ | `sec 13f-diff --cik 1067983 --limit 20 --pretty` |
| 我只知道投资人名字，不知道 CIK？ | `sec resolve --query 段永平 --pretty`，然后 `sec 13f-diff --investor 段永平 --pretty` |
| 公司最新 10-K 风险因素是什么？ | `sec section --ticker AAPL --form 10-K --item risk-factors --pretty` |
| 生成能直接给人看的分析摘要？ | `sec report --ticker AAPL --kind risk` |
| 本地 app 或 agent 怎么通过 HTTP 调用？ | `sec serve --port 8716`，然后 `curl "http://127.0.0.1:8716/v1/filings?ticker=AAPL&form=10-K&latest=1"` |
| 支持 MCP 的 Agent 怎么直接调用？ | 先运行 `sec config set-identity ...`，再在 Agent 配置里启动 `sec mcp` |
| 答案来源在哪里？ | 结构化结果包含 `source_url`，document 结果还包含 `document_url` |

## 参数怎么选

`sec-cli` 的查询入口分成两类：公司披露和 13F 投资经理披露。

公司披露类命令用 `--ticker` 或 `--cik`：

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

13F 投资经理类命令可以用四种选择器：

- `--cik`：最准确，已知 SEC CIK 时优先使用。
- `--manager`：已知 SEC filing manager 法律实体名时使用，不走 LLM。
- `--investor`：只知道公众人物/基金简称时使用，会优先查本地 verified cache，必要时走 LLM，再用 SEC 验证。
- `--ticker`：当投资经理本身也是上市公司，并且 ticker 能映射到提交 13F 的 CIK 时使用，例如 `BRK-B`。

同一个对象可以这样查，结果最终都会落到同一个 SEC CIK：

```bash
# 段永平 / H&H：自然语言输入，适合 agent 或人直接问
sec resolve --query 段永平 --pretty
sec 13f-diff --investor 段永平 --limit 10 --pretty
sec report --investor 段永平 --kind portfolio --limit 10

# 段永平 / H&H：法律实体名，确定性查询，不走 LLM
sec resolve --manager "H&H International Investment LLC" --pretty
sec 13f-summary --manager "H&H International Investment LLC" --latest 2 --pretty
sec 13f-diff --manager "H&H International Investment LLC" --limit 10 --pretty

# 段永平 / H&H：CIK，最稳定，适合脚本和生产任务
sec resolve --cik 1759760 --pretty
sec 13f-aggregate --cik 1759760 --latest 1 --limit 20 --pretty
sec 13f-diff --cik 1759760 --limit 10 --jsonl

# 巴菲特 / Berkshire：公众名字、法律实体、CIK、ticker 都可以
sec resolve --query 巴菲特 --pretty
sec 13f-summary --manager "BERKSHIRE HATHAWAY INC" --latest 1 --pretty
sec 13f-diff --cik 1067983 --limit 20 --pretty
sec 13f-diff --ticker BRK-B --limit 20 --pretty
```

使用建议：

- 人和 Agent：优先 `--investor` / `--query`，因为输入自然。
- 数据脚本：优先 `--cik`，因为最稳定。
- 已知 SEC 法律实体：优先 `--manager`，因为确定性强且不依赖 LLM。
- 公司财报和高管交易：优先 `--ticker`，必要时换 `--cik`。

## 数据源和输出表

`sec-cli` 直接使用 SEC 公开数据，不依赖付费行情 API。每条重要结果都会尽量保留来源字段，方便人、脚本和 Agent 复核。

| 数据源 | 对应命令 | 里面有什么 | 输出表 / record |
| --- | --- | --- | --- |
| SEC submissions JSON | `filings`、`archive`、`export --kind filings` | 公司提交过哪些 filing、日期、accession、主文档名、filing 元数据导出，以及离线 filing 文档归档 | filing records、archive manifests、Arrow/Parquet filing tables |
| SEC daily master index | `daily`、`monitor` | 全市场某日新增 filing feed：CIK、公司、form、提交日期、archive 文件名、accession、来源 URL | daily filing records |
| SEC EDGAR Full-Text Search / EFTS | `efts`、`full-text`、`global-search` | 全市场全文搜索命中：分数、公司、CIK、form、日期、accession、document URL | EFTS search records |
| SEC CompanyFacts JSON | `facts`、`statements`、`stitch`、`metrics`、`scores`、`export`、`report --kind financial` | XBRL 财务事实：营收、净利润、资产、单位、期间、财年/季度、标准化报表行、去重拼接的 10-K/10-Q 时间序列、二次推导指标、Piotroski/Altman/Beneish 健康评分，以及 Arrow/Parquet 导出 | fact records、financial statement rows、stitched statement rows、financial metric records、health score records、Arrow/Parquet tables、Markdown financial report |
| Inline XBRL filing HTML | `ixbrl` | filing HTML 内嵌的 `ix:nonFraction` / `ix:nonNumeric`、context、unit、scale、decimals、原始值 | Inline XBRL fact records |
| XBRL linkbase 附件 | `xbrl-links`、`linkbase`、`xbrl-tree`、`xbrl-calc`、`xbrl-statement` | EX-101.PRE/CAL/DEF/LAB/SCH 关系：presentation arcs、calculation weights、definition arcs、标签、schema elements，以及挂载同一 accession CompanyFacts 数值后的报表行 | XBRL linkbase relationship records、presentation tree rows、calculation checks、rendered XBRL statement rows |
| Filing HTML tables | `tables` | 主 HTML 文档里的表格行，例如薪酬表、分部表、注册证券表、债务表、合同表 | HTML table records |
| 10-K/10-Q company report 主文档 | `company-report`、`parse --form "10-K"` | 已分类专题表：分部收入、地域收入、收入拆分、债务到期、合同义务、租赁、税、股票回购 | company report records |
| DEF 14A proxy statement 主文档 | `proxy`、`parse --form "DEF 14A"` | 股东大会日期/地点、投票事项、董事会建议、董事候选人、审计师、NEO、高管薪酬表 | proxy statement records |
| S-1/F-1/424B prospectus 主文档 | `prospectus`、`parse --form "S-1"` | 发行证券、IPO/招股书类型、ticker/交易所、价格区间、发行股数、募资用途、承销商、审计师、风险/业务/摊薄摘要 | prospectus records |
| 20-F/6-K/40-F foreign issuer 主文档 | `foreign`、`parse --form "20-F"` | 外国私营发行人年报/当前报告、交易所、股票代码、审计师、事件信号、风险/业务/经营回顾/内控/财报摘要 | foreign issuer records |
| N-PORT/N-CSR/N-CEN/N-PX/497K/24F-2NT fund documents | `fund`、`parse --form "NPORT-P"` | 基金 registrant/series/class 元数据、N-PORT 持仓、N-PX 代理投票、497K 摘要招募书、24F 年度证券销售通知、资产/负债/净资产、N-CSR 股东报告、内控和财务报表章节 | fund disclosure records |
| SEC complete submission text / archive documents | `search`、`section`、`docs`、`doc` | 原始 filing 文本、HTML/XML 附件、exhibit、可引用片段 | snippet、section、document records |
| Form 3/4/5 XML ownership report | `form4`、`form4-summary`、`report --kind insider` | 内部人、职位、交易代码、股数、价格、金额、脚注、签名 | transaction records、ownership report records |
| Form 8-K primary document | `8k` | 当前报告事件 item，例如 2.02 业绩、5.02 高管变化、8.01 其他事件、9.01 附件 | 8-K event records |
| Form 8-K exhibits | `8k-exhibits` | 附件 EX 文档分类：earnings release、press release、material contract、transaction agreement、charter/bylaws、security instrument、XBRL、accountant letter | 8-K exhibit records |
| Schedule 13D/13G primary document | `13d`、`13g`、`schedule13` | 5% 大股东受益持仓、申报人、持股比例、投票权/处分权、主动/被动意图信号 | Schedule 13 records |
| Form 13F-HR information table | `13f`、`13f-aggregate`、`13f-diff`、`report --kind portfolio` | 机构多头持仓：issuer、class、CUSIP、市值、股数、投票权 | holding、aggregate holding、diff records |
| Form 13F-HR primary document | `13f-summary` | filing manager、报告期、总持仓数、总市值、签名、included managers | 13F report summary records |
| 10-K / 10-Q primary document | `section`、`report --kind risk` | Business、Risk Factors、Cybersecurity、MD&A、Financial Statements 等章节 | section records、Markdown report |
| LLM resolver + SEC 验证 | `resolve`、带 `--investor` 的 13F 命令 | 公众名字到 SEC 法律实体 / CIK 的候选，并验证是否真的有 13F | resolve candidate records |

常见来源字段：

- `source_url`：SEC filing index URL，最重要的引用入口。
- `document_url`：具体文档 URL，例如 10-K HTML、exhibit、XML。
- `accession`：SEC filing accession number，定位一份 filing。
- `document` / `filename`：submission 里的具体文件名。
- `section` / `item`：搜索或 section 抽取命中的章节。
- `fact_id`：财务事实的稳定 ID，方便去重、引用和 Agent 追踪。

输出表字段速查：

| 输出表 | 来自命令 | 先看哪些字段 | 来源字段 |
| --- | --- | --- | --- |
| Filing | `filings` | `company`、`form`、`filing_date`、`report_date`、`primary_document` | `accession`、`source_url`、`text_url` |
| Daily filing | `daily`、`monitor` | `company`、`form`、`filing_date`、`filename` | `accession`、`source_url`、`text_url` |
| EFTS search hit | `efts`、`full-text`、`global-search` | `company`、`form`、`file_date`、`score`、`document` | `accession`、`source_url`、`document_url` |
| Fact | `facts` | `concept`、`label`、`value`、`unit`、`fy`、`fp`、`filed` | `accession`、`source_url`、`fact_id` |
| Financial statement row | `statements` | `statement`、`line_order`、`line_item`、`value`、`unit`、`fiscal_year`、`fiscal_period` | `accession`、`source_url`、`fact_id` |
| Stitched statement row | `stitch`、`statement-stitch` | `statement`、`line_item`、`period_kind`、`form`、`value`、`duplicate_forms`、`source_count` | `accession`、`source_url`、`fact_id` |
| Financial metric | `metrics` | `metric`、`category`、`value`、`display_value`、`period_end`、`calculation`、`components` | `source_urls`、component `accession`、component `fact_id` |
| Financial health score | `scores` | `score_name`、`score`、`max_score`、`rating`、`period_end`、`signals` | `source_urls`、signal `calculation` |
| Inline XBRL fact | `ixbrl` | `name`、`context_ref`、`unit_ref`、`scale`、`raw_value`、`numeric_value` | `accession`、`document_url`、`source_url` |
| XBRL linkbase relationship | `xbrl-links` | `linkbase`、`relationship`、`role`、`parent_concept`、`child_concept`、`concept`、`label`、`order`、`weight` | `accession`、`document_url`、`source_url` |
| XBRL presentation tree row | `xbrl-tree` | `role`、`depth`、`line_order`、`concept`、`label`、`parent_concept`、`path` | `accession`、`document_url`、`source_url` |
| XBRL calculation check | `xbrl-calc` | `parent_concept`、`parent_value`、`calculated_value`、`difference`、`status`、`matched_children` | `accession`、`document_url`、`source_url` |
| Rendered XBRL statement row | `xbrl-statement`、`statement-render` | `role`、`depth`、`line_order`、`concept`、`label`、`value`、`numeric_value`、`calculation_status`、`path` | `accession`、`fact_id`、`document_url`、`source_url` |
| Company report topic table | `company-report` | `topics[].topic`、`confidence`、`headers`、`rows`、`matched_table_count`、`scanned_table_count` | `accession`、`document_url`、`source_url` |
| HTML table | `tables` | `title_hint`、`row_count`、`column_count`、`headers`、`rows`、`truncated` | `accession`、`document_url`、`source_url` |
| Proxy statement | `proxy`、`parse --form "DEF 14A"` | `meeting_date`、`proposals`、`director_nominees`、`auditor`、`named_executive_officers`、`summary_compensation_table` | `accession`、`document_url`、`source_url` |
| Prospectus | `prospectus`、`parse --form "S-1"` | `securities_offered`、`proposed_ticker`、`exchange`、`price_range`、`shares_offered`、`underwriters`、`risk_factors` | `accession`、`document_url`、`source_url` |
| Foreign issuer | `foreign`、`parse --form "20-F"` | `report_type`、`exchange`、`ticker_or_symbol`、`auditor`、`event_signals`、`risk_factors`、`operating_review` | `accession`、`document_url`、`source_url` |
| Fund disclosure | `fund`、`parse --form "NPORT-P"` | `disclosure_type`、`registrant_name`、`series_name`、`period_end`、`holdings`、`proxy_votes`、`summary_prospectus`、`registration_fee_notice`、`net_assets` | `accession`、`document_url`、`source_url` |
| Search snippet | `search` | `query`、`snippet`、`offset`、`form`、`filing_date` | `accession`、`source_url`、`document`、`section` |
| Section | `section` | `item`、`title`、`content`、`truncated` | `accession`、`document_url`、`source_url` |
| Document | `docs`、`doc` | `filename`、`document_type`、`description`、`content_type`、`content` | `accession`、`document_url`、`source_url` |
| Form 4 transaction | `form4` | `reporting_owner`、`officer_title`、`transaction_code`、`shares`、`price`、`value` | `accession`、`source_url` |
| Form 4 report summary | `form4-summary` | `owners`、`transaction_count`、`net_shares`、`total_value`、`footnotes` | `accession`、`source_url` |
| 8-K event | `8k` | `item`、`item_title`、`category`、`is_furnished_item`、`content` | `accession`、`document_url`、`source_url` |
| 8-K exhibit | `8k-exhibits` | `document_type`、`category`、`is_earnings_release`、`description`、`content` | `accession`、`document_url`、`source_url` |
| Schedule 13D/13G | `13d`、`13g`、`schedule13` | `reporting_persons`、`beneficially_owned_shares`、`percent_of_class`、`activist_intent` | `accession`、`document_url`、`source_url` |
| 13F holding | `13f` | `manager`、`issuer`、`class`、`cusip`、`value_usd`、`shares` | `accession`、`source_url` |
| 13F aggregate holding | `13f-aggregate` | `issuer`、`cusip`、`value_usd`、`shares`、`rows` | `source_url` |
| 13F diff row | `13f-diff` | `issuer`、`change_type`、`change_value_usd`、`change_shares` | `current_source_url`、`previous_source_url` |
| 13F report summary | `13f-summary` | `manager`、`report_date`、`total_holdings_reported`、`total_value_usd`、`signature_name` | `accession`、`source_url` |
| Resolve candidate | `resolve` | `investor`、`manager`、`cik`、`confidence`、`validation.status` | `validation.source_url`、`validation.latest_accession` |

## 架构

| 模块 | 职责 |
| --- | --- |
| `cli` | 命令参数和 orchestration |
| `client` | SEC domain facade、ticker 到 CIK |
| `http` | SEC HTTP 请求 |
| `storage` | 本地缓存 |
| `edgar` | SEC submissions、facts、archive URL |
| `company` | 10-K/10-Q 专题表深度解析 |
| `metrics` | 基于 SEC facts 的财务指标和二次分析 |
| `documents` | complete submission 拆分、document 选择、读取 |
| `llm` | OpenAI-compatible / Anthropic-compatible 模型客户端 |
| `resolve` | LLM 候选解析 + SEC 13F 验证 |
| `parsers` | XML helper、form-specific parsers |
| `sections` | 10-K/10-Q section 抽取 |
| `reports` | Markdown 汇报生成 |
| `models` | query DTO 和稳定输出 record |
| `registry` | parser discovery |
| `pipeline` | 统一 form parser 分发 |
| `search` | filing text search |
| `output` | JSON / JSONL 输出 |

## 安装

普通使用直接下载 GitHub 最新 Release 里编译好的二进制，不需要安装 Rust，也不需要用 Cargo。

Release 页面：<https://github.com/okloorcl/sec-cli/releases/latest>

| 平台 | 架构 | Release 资产 |
| --- | --- | --- |
| macOS | Apple Silicon / arm64 | `sec-cli-aarch64-apple-darwin.tar.gz` |
| Windows | amd64 / x86_64 | `sec-cli-x86_64-pc-windows-msvc.zip` |
| Linux | amd64 / x86_64 | `sec-cli-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | amd32 / i686 | `sec-cli-i686-unknown-linux-gnu.tar.gz` |
| Linux | arm64 / AArch64 | `sec-cli-aarch64-unknown-linux-gnu.tar.gz` |
| Linux | arm32 / ARMv7 hard-float | `sec-cli-armv7-unknown-linux-gnueabihf.tar.gz` |

macOS Apple Silicon：

```bash
curl -L -o sec-cli.tar.gz \
  https://github.com/okloorcl/sec-cli/releases/latest/download/sec-cli-aarch64-apple-darwin.tar.gz
tar -xzf sec-cli.tar.gz
sudo mv sec-cli-aarch64-apple-darwin/sec /usr/local/bin/sec
sec --help
```

Linux amd64：

```bash
curl -L -o sec-cli.tar.gz \
  https://github.com/okloorcl/sec-cli/releases/latest/download/sec-cli-x86_64-unknown-linux-gnu.tar.gz
tar -xzf sec-cli.tar.gz
sudo mv sec-cli-x86_64-unknown-linux-gnu/sec /usr/local/bin/sec
sec --help
```

Windows PowerShell：

```powershell
Invoke-WebRequest `
  -Uri https://github.com/okloorcl/sec-cli/releases/latest/download/sec-cli-x86_64-pc-windows-msvc.zip `
  -OutFile sec-cli.zip
Expand-Archive sec-cli.zip -DestinationPath .
.\sec-cli-x86_64-pc-windows-msvc\sec.exe --help
```

SEC identity 是必需的，建议设置为真实姓名和邮箱。最省心的方式是写入本地配置：

```bash
sec config set-identity "Your Name your.email@example.com"
sec config show
```

也可以使用环境变量：

```bash
export SEC_IDENTITY="Your Name your.email@example.com"
```

或单次命令传入：

```bash
sec --identity "Your Name your.email@example.com" filings --ticker AAPL
```

Shell 补全脚本可以本地生成：

```bash
sec completions zsh > ~/.zfunc/_sec
sec completions bash > sec.bash
sec completions fish > ~/.config/fish/completions/sec.fish
```

## 开发

只有开发、调试、跑测试时才需要 Cargo：

```bash
cargo build
cargo test

SEC_IDENTITY="Your Name your.email@example.com" \
  cargo run --bin sec -- filings --ticker AAPL --form 10-K --latest 2 --pretty
```

## CI 和 Release 支持平台

GitHub Actions 会在每次 push 和 pull request 时检查项目。推送 `v*` tag 时，会为同一批 target 构建并上传 Release 二进制：

| 平台 | 架构 | Rust target | CI 做什么 |
| --- | --- | --- | --- |
| Ubuntu Linux | amd64 / x86_64 | `x86_64-unknown-linux-gnu` | check、test、release build |
| Ubuntu Linux | amd32 / i686 | `i686-unknown-linux-gnu` | cross check、cross release build |
| Ubuntu Linux | arm64 / AArch64 | `aarch64-unknown-linux-gnu` | cross check、cross release build |
| Ubuntu Linux | arm32 / ARMv7 hard-float | `armv7-unknown-linux-gnueabihf` | cross check、cross release build |
| Windows | amd64 / x86_64 | `x86_64-pc-windows-msvc` | check、test、release build |
| macOS | arm64 / Apple Silicon | `aarch64-apple-darwin` | check、test、release build |

GitHub 有原生 runner 的平台会直接跑测试；Linux 32 位和 ARM 目标用
`cross` 做交叉编译验证，所以会确认能编译出对应架构的 release binary，
但不会在 x86_64 runner 上强行运行这些二进制。

## LLM Resolver

`sec resolve` 不再依赖硬编码别名表。解析是分层的：标准输入先走确定性的 SEC 规则查询，只有非标准公众名字才交给 LLM 理解。

- `--cik`：直接验证这个 CIK 是否有 SEC `13F-HR` filing。
- `--manager`：用 SEC company search 查法律实体 filing manager。
- `--query`：先查本地已验证 cache，再让 LLM 给出可能的法律实体，最后仍然回到 SEC filing 验证和纠错。

LLM 负责理解名字，SEC 文件才是事实来源。

验证成功的解析结果会写入本地 sec-cli cache。之后 `sec 13f-diff --investor <NAME>` 这类命令会优先复用上一次 SEC 验证过的 CIK，不会每次都被 LLM 的随机输出影响。

OpenAI 兼容配置，比如 GLM / BigModel：

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

Anthropic 兼容配置：

```json
{
  "provider": "anthropic",
  "base_url": "https://open.bigmodel.cn/api/anthropic",
  "model": "GLM-5.1",
  "api_key_env": "BIGMODEL_API_KEY"
}
```

环境变量覆盖：

| 变量 | 含义 |
| --- | --- |
| `SEC_CLI_LLM_CONFIG` | 指定配置文件路径 |
| `SEC_CLI_LLM_PROVIDER` | `openai` 或 `anthropic` |
| `SEC_CLI_LLM_BASE_URL` | provider base URL |
| `SEC_CLI_LLM_MODEL` | 模型名 |
| `SEC_CLI_LLM_API_KEY_ENV` | 保存 API key 的环境变量名 |
| `SEC_CLI_LLM_API_KEY` | 直接 API key fallback；更推荐 `api_key_env` |

也可以单次命令覆盖：

```bash
sec resolve --query "Warren Buffett" \
  --llm-provider openai \
  --llm-base-url https://open.bigmodel.cn/api/coding/paas/v4 \
  --llm-model GLM-5.1 \
  --llm-api-key-env BIGMODEL_API_KEY \
  --pretty
```

不要把 API key 提交进仓库。建议把 key 放在环境变量里，配置文件只写 `api_key_env`。

## 本地完整测试

下面这段可以直接复制到终端运行，用来构建项目、配置 GLM、跑单元测试，并验证真实 SEC 查询和真实 LLM 解析。运行前把 `SEC_IDENTITY` 和 `BIGMODEL_API_KEY` 换成你自己的。

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

预期结果：

- `cargo test` 应该全部通过。
- `resolve --query 段永平` 应该得到 `validation.status = verified_13f`，CIK 为 `1759760`。
- `resolve --manager "H&H International Investment LLC"` 和 `resolve --cik 1759760` 应该不走 LLM，直接得到同一个已验证 CIK。
- `13f-diff --investor 段永平` 应该显示 H&H International Investment, LLC，并列出 Apple、Tesla、Nvidia、Berkshire、PDD 等变化。
- `resolve --query 巴菲特` 应该解析到 Berkshire Hathaway Inc，CIK 为 `1067983`。

## 命令

### filings

查询某公司近期 filing。

```bash
sec filings --ticker AAPL --form 10-K --latest 3 --pretty
sec filings --cik 320193 --form 10-Q --from 2023-01-01 --to 2025-12-31
sec filings --ticker TSLA --form 8-K --latest 5 --jsonl
sec filings --ticker NVDA --form 10-K --include-amends --latest 2 --pretty
```

输出字段：`accession`、`cik`、`company`、`form`、`filing_date`、`report_date`、`primary_document`、`source_url`、`text_url`。

### daily

扫描 SEC 全市场 daily master index。这是高频监控入口：它不是从单个 ticker 出发，而是从某个 SEC filing 日期出发，过滤当天全市场新增 filing。

```bash
sec daily --date 2026-05-15 --form 8-K --limit 50 --pretty
sec daily --date 2026-05-15 --form 13F-HR --include-amends --jsonl
sec daily --date 2026-05-15 --company apple --pretty
sec monitor --form 4 --limit 100 --jsonl
```

如果不传 `--date`，sec-cli 会使用 UTC 下最近的 SEC 工作日；周末默认回退到周五。输出字段：`cik`、`company`、`form`、`filing_date`、`accession`、`filename`、`text_url`、`source_url`。

### efts

调用官方 SEC EDGAR Full-Text Search 全市场全文索引。适合“不知道是哪家公司提到这个词”的场景，也适合按主题扫描大量公司。

```bash
sec efts --query "supply chain risk" --form 10-K --from 2024-01-01 --to 2024-12-31 --limit 10 --pretty
sec efts --ticker AAPL --query "artificial intelligence" --form 10-K --limit 5 --pretty
sec efts --cik 320193 --query "services revenue" --form 10-K,10-Q --from 2023-01-01 --pretty
sec full-text --query "GLP-1" --form 10-K --limit 20 --jsonl
```

`--ticker` 和 `--cik` 可选；不传就是全市场搜索。`--form` 支持单个 form，也支持逗号分隔多个 form。输出字段：`score`、`cik`、`company`、`form`、`file_date`、`period_ending`、`accession`、`document`、`source_url`、`document_url`。

### facts

查询 SEC CompanyFacts。常用别名例如 `revenue`、`cogs`、`grossprofit`、`rd`、`sga`、`cash`、`receivables`、`currentdebt`、`ocf`、`capex` 会映射到覆盖 100+ 常见 US-GAAP concept 的候选字典。

```bash
sec facts --ticker AAPL --concept revenue --form 10-K --latest 5 --pretty
sec facts --ticker MSFT --concept us-gaap:NetIncomeLoss --latest 10 --jsonl
sec facts --cik 320193 --concept us-gaap:RevenueFromContractWithCustomerExcludingAssessedTax --unit USD --latest 8 --pretty
```

输出字段：`concept`、`taxonomy`、`label`、`description`、`value`、`unit`、`fy`、`fp`、`form`、`filed`、`start`、`end`、`frame`、`accession`、`source_url`、`fact_id`。

### statements

从 SEC CompanyFacts 组装标准化 10-K/10-Q 财报三大表。输出是长表结构，不是排版后的 Excel：每一行代表一个报表项目、一个期间、一个 XBRL concept、一个单位和一个来源 filing。

```bash
sec statements --ticker AAPL --statement income --period annual --latest 4 --pretty
sec statements --ticker AAPL --statement balance --period annual --latest 2 --pretty
sec statements --ticker AAPL --statement cashflow --period quarterly --latest 4 --jsonl
sec statements --cik 320193 --statement all --period annual --latest 1 --pretty
```

`--statement`：

- `income`：营收、收入成本、毛利、研发、SG&A、营业利润、利息、税、净利润、EPS、股数
- `balance`：现金、证券、应收、存货、流动资产、PP&E、商誉、无形资产、租赁、债务、负债、股东权益
- `cashflow`：净利润、折旧摊销、股权激励、营运资本变化、经营现金流、资本开支、收购、分红、回购、债务发行/偿还、现金变化
- `all`：一次输出利润表、资产负债表、现金流量表

`--period`：

- `annual`：只看 10-K
- `quarterly`：只看 10-Q
- `all`：不过滤 filing form

输出字段：`cik`、`company`、`statement`、`line_order`、`line_item`、`concept`、`taxonomy`、`label`、`value`、`numeric_value`、`unit`、`fiscal_year`、`fiscal_period`、`form`、`filed`、`start`、`end`、`frame`、`accession`、`source_url`、`fact_id`。

### stitch

从标准化 CompanyFacts 财报行里构建去重后的 10-K / 10-Q 时间序列。它会按报表行和 period end 分组，年度 FY 优先选择 10-K，季度保留 10-Q，并输出 `duplicate_forms` 和 `source_count`，让 Agent 能看见同一个期间是否存在多个来源事实。

```bash
sec stitch --ticker AAPL --statement income --latest 8 --pretty
sec stitch --ticker AAPL --statement all --latest 6 --jsonl
sec statement-stitch --cik 320193 --statement cashflow --latest 10 --output table
```

输出字段：`statement`、`line_item`、`period_kind`、`form`、`fiscal_period`、`value`、`numeric_value`、`duplicate_forms`、`source_count`、`accession`、`source_url`、`fact_id`。

### metrics

基于标准化 CompanyFacts 财报行计算二次分析指标。这个命令不是黑箱估算：每个指标都会在 `components` 里列出使用了哪些 SEC fact，包括 accession、fact id 和 source URL，方便人或 Agent 继续追溯原始文件。

```bash
sec metrics --ticker AAPL --period annual --latest 4 --pretty
sec metrics --ticker AAPL --period quarterly --latest 8 --jsonl
sec metrics --cik 320193 --period annual --latest 1 --pretty
```

`--period`：

- `annual`：只用 10-K facts 推导年度指标
- `quarterly`：只用 10-Q facts 推导季度指标
- `all`：不过滤 filing form

当前在底层 facts 足够时可输出 50+ 个 SEC 派生指标：

- 盈利能力：`gross_margin`、`operating_margin`、`net_margin`、`cost_of_revenue_margin`、`pretax_margin`
- 盈利/税率：`effective_tax_rate`
- 增长：`revenue_growth`、`net_income_growth`
- 费用强度：`rd_to_revenue`、`sga_to_revenue`、`operating_expense_ratio`
- 现金流：`free_cash_flow`、`free_cash_flow_margin`、`operating_cash_flow_margin`、`cash_conversion`、`free_cash_flow_to_net_income`
- 回报率：`return_on_assets`、`return_on_equity`、`roic`、`cash_flow_return_on_assets`
- 流动性：`working_capital`、`current_ratio`、`quick_ratio`、`cash_ratio`、`cash_to_assets`、`cash_and_securities_to_assets`、`cash_and_securities_coverage`
- 杠杆/偿债：`total_debt`、`net_debt`、`liabilities_to_assets`、`debt_to_equity`、`debt_to_assets`、`net_debt_to_equity`、`debt_to_capital`、`cash_flow_to_debt`、`fcf_to_debt`、`interest_coverage`
- 效率：`asset_turnover`、`inventory_turnover`、`receivables_turnover`、`inventory_to_current_assets`、`receivables_to_revenue`
- 资产质量：`goodwill_to_assets`、`intangibles_to_assets`、`marketable_securities_to_assets`
- 资本强度/资本回报：`capex_to_revenue`、`capex_to_operating_cash_flow`、`dividend_payout_ratio`、`share_repurchases_to_revenue`、`share_repurchases_to_free_cash_flow`

输出字段：`metric`、`category`、`value`、`display_value`、`unit`、`period_end`、`fiscal_year`、`fiscal_period`、`form`、`calculation`、`components`、`source_urls`。

### scores

基于同一套标准化 CompanyFacts 财报行和 `metrics` 指标计算财务健康评分。输出不是一个孤立数字，而是每个评分一条 record，并附带 `signals` 明细，方便 Agent 解释每一分从哪里来。

```bash
sec scores --ticker AAPL --period annual --latest 1 --pretty
sec scores --ticker MSFT --period annual --latest 3 --jsonl
sec scores --cik 320193 --period annual --latest 1 --output table
```

已实现评分：

- `piotroski_f_score`：9 个二元质量信号，覆盖盈利能力、应计质量、杠杆/流动性、稀释、毛利率和资产周转。
- `altman_z_score_private`：只使用 SEC 数据的 Altman Z'' 近似版，使用账面权益而非市值。
- `beneish_m_score`：Beneish M-Score 近似版，`watch` 表示分数高于 `-1.78` 的观察阈值。

输出字段：`score_name`、`score`、`max_score`、`rating`、`period_end`、`calculation`、`signals`、`source_urls`。缺少底层 fact 时会输出 `insufficient_data` 或 null signal value，不会编造输入。

### export

把结构化查询结果写成 Arrow IPC 或 Parquet 文件，方便 DuckDB、Polars、Spark、pandas 和数据工程管道直接读取。`export` 会先执行和普通 CLI 完全相同的 SEC 查询，再把每条 record 扁平化成稳定列。像 `components`、`signals` 这种嵌套来源明细会保留成 JSON 字符串，不会丢掉溯源信息。

```bash
sec export --kind filings --ticker AAPL --form 10-K --latest 5 --format parquet --out data/aapl_10k.parquet
sec export --kind facts --ticker AAPL --concept revenue --latest 20 --format arrow --out data/aapl_revenue.arrow
sec export --kind statements --ticker AAPL --statement income --period annual --latest 4 --format parquet --out data/aapl_income.parquet
sec export --kind stitch --ticker AAPL --statement all --latest 8 --format parquet --out data/aapl_stitch.parquet
sec export --kind metrics --ticker AAPL --period annual --latest 4 --format parquet --out data/aapl_metrics.parquet
sec export --kind scores --ticker AAPL --period annual --latest 1 --format arrow --out data/aapl_scores.arrow
```

支持的 `--kind`：`filings`、`facts`、`statements`、`stitch`、`metrics`、`scores`。其中 `--kind facts` 必须传 `--concept`。`--format` 支持 `arrow` 和 `parquet`。命令会自动创建输出目录，并把写入 record 数打印到 stderr。

### archive

把 SEC complete submission 文档批量下载到离线目录。这个命令适合长时间 Agent 任务、CI、离线分析：归档根目录会写 `manifest.json`，每个 accession 子目录会写 `filing.json` 和对应 document 文件。

```bash
sec archive --ticker AAPL --form 10-K --latest 2 --primary-only --out-dir ./archives/aapl --pretty
sec archive --cik 320193 --form 8-K --latest 5 --include-amends --out-dir ./archives/aapl_8k --jsonl
sec archive --ticker MSFT --form 10-Q --latest 4 --limit-bytes 2000000 --out-dir ./archives/msft_10q
```

关键参数：

- `--primary-only`：每个 filing 只保存 sequence 1 主文档。
- `--limit-bytes`：按 UTF-8 边界限制每个 document 保存字节数。
- `--include-amends`：包含 `10-K/A` 这类修正版。

输出 manifest 包含 `filing_count`、`document_count`、`manifest_path`、每个 filing 的 SEC 来源 URL，以及每个 document 的本地路径。

### agent-pack

生成一个给 LLM Agent 或本地自动化直接消费的 source-backed research packet。它会组合最近 filing、指定 10-K/10-Q section、SEC 派生 metrics、财务健康评分、去重后的来源 URL，以及下一步建议命令。

```bash
sec agent-pack --ticker AAPL --sections risk-factors,mda --metrics-latest 4 --pretty
sec pack --cik 320193 --form 10-K --latest 1 --sections business,risk-factors,mda --section-limit-bytes 15000 --output json
```

默认 section 是 `risk-factors,mda`。当 Agent 需要一个足够完整、但又严格基于 SEC 来源的首轮分析输入时，用这个命令最省事。

### ixbrl

直接从 filing 主 HTML 流式解析 Inline XBRL facts。这个命令适合你想看“某一份 10-K/10-Q HTML 里原样嵌入的事实”，而不是 SEC CompanyFacts 已经整理后的公司级事实库。

```bash
sec ixbrl --ticker AAPL --form 10-K --concept RevenueFromContractWithCustomerExcludingAssessedTax --latest 1 --limit 3 --pretty
sec ixbrl --ticker AAPL --form 10-K --concept us-gaap:NetIncomeLoss --limit 5 --jsonl
sec ixbrl --cik 320193 --form 10-Q --latest 1 --limit 100 --pretty
```

`--concept` 可以写完整概念，例如 `us-gaap:NetIncomeLoss`，也可以只写本地名，例如 `NetIncomeLoss`。

输出字段：`accession`、`fact_type`、`name`、`namespace`、`local_name`、`context_ref`、`unit_ref`、`decimals`、`scale`、`format`、`sign`、`id`、`raw_value`、`value`、`numeric_value`、`document_url`、`source_url`。

### xbrl-links

解析完整 SEC submission 里的 XBRL linkbase 附件。这是未来做“真正 filing 专属财报树渲染”的底座：CompanyFacts 只给事实值，linkbase 才告诉你 presentation tree、calculation 权重、definition 关系、人类可读 label 和 schema element。

```bash
sec xbrl-links --ticker AAPL --form 10-K --linkbase presentation --concept Revenue --limit 20 --pretty
sec xbrl-links --ticker AAPL --form 10-K --linkbase calculation --concept NetIncomeLoss --pretty
sec linkbase --cik 320193 --form 10-Q --linkbase label --concept Revenue --jsonl
```

`--linkbase` 支持：`presentation`、`calculation`、`definition`、`label`、`schema`。`--concept` 会匹配 parent、child 或 label concept，可以写 `us-gaap:Revenues`，也可以只写 `Revenues`。

输出字段：`linkbase`、`relationship`、`role`、`arcrole`、`parent_concept`、`child_concept`、`concept`、`label`、`label_role`、`order`、`weight`、`preferred_label`、`document_url`、`source_url`。

### xbrl-tree

把 filing 专属的 XBRL presentation arcs 渲染成先序树形行。它是 `EX-101.PRE` 的更友好视图：每行都有 `depth`、`line_order`、`path`、parent concept、role URI 和来源 document URL。如果你想同时看到同一 filing 的 fact 数值，用 `xbrl-statement`。

```bash
sec xbrl-tree --ticker AAPL --form 10-K --role OPERATIONS --limit 30 --pretty
sec xbrl-tree --ticker AAPL --form 10-K --concept NetIncomeLoss --pretty
sec presentation-tree --cik 320193 --form 10-Q --limit 50 --jsonl
```

`--role` 是 role URI 的大小写不敏感子串过滤，所以通常写 `OPERATIONS`、`BALANCE`、`CASH`、`Revenue` 这种短词就够。

输出字段：`role`、`depth`、`line_order`、`concept`、`label`、`parent_concept`、`order`、`preferred_label`、`path`、`document_url`、`source_url`。

### xbrl-calc

用 XBRL calculation linkbase 校验同一 accession 的 CompanyFacts 数值。它会按 role 和 parent concept 聚合 `EX-101.CAL` arcs，应用每个 child 的 weight，然后判断 SEC fact 的 parent value 是否等于 child 加权合计。

```bash
sec xbrl-calc --ticker AAPL --form 10-K --role OPERATIONS --limit 20 --pretty
sec xbrl-calc --ticker AAPL --form 10-K --concept GrossProfit --tolerance 1 --pretty
sec calculation-checks --cik 320193 --form 10-Q --unit USD --limit 50 --jsonl
```

输出字段：`parent_concept`、`parent_value`、`calculated_value`、`difference`、`relative_difference`、`status`、`children_count`、`matched_children`、`missing_children`、`document_url`、`source_url`。

### xbrl-statement

把 filing 专属 presentation tree、同一 accession 的 CompanyFacts 数值、calculation 校验状态合成一张真正接近 SEC 原生财报的行级表：`EX-101.PRE` 决定层级和顺序，CompanyFacts 提供数值，CompanyFacts label 补齐 extension label 缺失，`EX-101.CAL` 给总计行补 `calculation_status`。

```bash
sec xbrl-statement --ticker AAPL --form 10-K --role OPERATIONS --values-only --limit 30 --pretty
sec xbrl-statement --ticker AAPL --form 10-K --role BALANCE --unit USD --limit 50 --pretty
sec statement-render --cik 320193 --form 10-Q --concept NetIncomeLoss --jsonl
```

常用参数：

- `--role`：按 role URI 子串过滤，例如 `OPERATIONS`、`BALANCE`、`CASH`。
- `--values-only`：只返回匹配到 fact value 的行，隐藏 abstract/heading 行。
- `--unit`：选择 CompanyFacts 单位，常见是 `USD` 或 `shares`。
- `--tolerance`：控制 calculation 校验的容差。

输出字段：`role`、`depth`、`line_order`、`concept`、`label`、`value`、`numeric_value`、`unit`、`fact_id`、`calculation_status`、`calculation_difference`、`calculation_relative_difference`、`path`、`document_url`、`source_url`。

### tables

从 filing 主 HTML 抽取表格。这个命令是通用表格入口：先把表格行、表头和来源稳定拿出来，后续 DEF 14A 薪酬表、分部表、债务表、合同表都可以基于它继续做专用结构化。

```bash
sec tables --ticker AAPL --form 10-K --latest 1 --limit-tables 5 --limit-rows 10 --pretty
sec tables --ticker TSLA --form "DEF 14A" --include-amends --limit-tables 20 --limit-rows 8 --jsonl
sec tables --cik 320193 --form 10-Q --latest 1 --limit-tables 10 --pretty
```

输出字段：`table_index`、`title_hint`、`row_count`、`column_count`、`returned_rows`、`truncated`、`headers`、`rows`、`document_url`、`source_url`。

### company-report

深度解析 10-K/10-Q 主文档里的高价值专题表。它比 `tables` 更有判断力：会把可能的分部收入、地域收入、收入拆分、债务到期、合同义务、租赁到期、税、股票回购表分类出来。

```bash
sec company-report --ticker AAPL --form 10-K --latest 1 --pretty
sec company-report --ticker AAPL --form 10-K --topic segment --limit-tables 5 --limit-rows 12 --pretty
sec company-report --cik 320193 --form 10-Q --topic debt --jsonl
sec parse --ticker AAPL --form 10-K --limit 5 --pretty
```

输出字段：`matched_table_count`、`scanned_table_count`、`topics[].topic`、`topics[].confidence`、`title_hint`、`headers`、`rows`、`document_url`、`source_url`。

### proxy

解析 DEF 14A proxy statement，也就是年度股东大会委托书。它回答的是：什么时候开股东大会、要投哪些议案、董事会建议怎么投、董事候选人是谁、审计师是谁、NEO 是谁、高管薪酬表是什么。

```bash
sec proxy --ticker AAPL --latest 1 --pretty
sec proxy --cik 320193 --latest 2 --include-amends --limit-rows 20 --pretty
sec parse --ticker AAPL --form "DEF 14A" --latest 1 --pretty
```

输出字段：`meeting_date`、`meeting_time`、`meeting_site`、`record_date`、`materials_available_date`、`proposals`、`director_nominees`、`auditor`、`named_executive_officers`、`summary_compensation_table`、`document_url`、`source_url`。

### prospectus

解析 S-1、F-1 和 424B 招股书/发行说明书。这个命令回答的是：发行什么证券、是不是 IPO、拟用什么 ticker、在哪个交易所、价格区间和发行股数是多少、承销商是谁、审计师是谁、募资用途/风险因素/业务/摊薄章节写了什么。

```bash
sec prospectus --ticker RDDT --form S-1 --include-amends --latest 1 --pretty
sec prospectus --cik 1713445 --form all --latest 3 --limit-bytes 800 --limit-tables 5 --pretty
sec parse --ticker RDDT --form "424B4" --latest 1 --pretty
```

`--form` 支持：`all`、`S-1`、`S-1/A`、`F-1`、`F-1/A`、`424B`、`424B1` 到 `424B5`，以及 `424B7`。想看 S-1 修正版时加 `--include-amends`。

输出字段：`prospectus_type`、`is_ipo_related`、`securities_offered`、`proposed_ticker`、`exchange`、`price_range`、`shares_offered`、`offering_amount`、`underwriters`、`auditor`、`use_of_proceeds`、`risk_factors`、`business`、`dilution`、`tables`、`document_url`、`source_url`。

### foreign

解析 20-F、6-K、40-F 外国发行人披露。这个命令适合 ADR 和外国私营发行人，例如 TSM、BABA、ASML、SHOP、SONY。它会抽取报告类型、交易所/代码线索、审计师、当前报告事件信号，以及 Risk Factors、Business、Operating Review、Controls、Financial Statements 等关键章节。

```bash
sec foreign --ticker TSM --form 20-F --latest 1 --pretty
sec foreign --ticker BABA --form 6-K --latest 3 --limit-bytes 800 --pretty
sec foreign --ticker SHOP --form 40-F --latest 1 --pretty
sec foreign --cik 1046179 --form all --latest 5 --include-amends --jsonl
sec parse --ticker TSM --form "20-F" --latest 1 --pretty
sec parse --ticker BABA --form "6-K" --latest 1 --pretty
```

`--form` 支持：`all`、`20-F`、`20-F/A`、`6-K`、`6-K/A`、`40-F`、`40-F/A`。如果要看修正版，加 `--include-amends`。

输出字段：`report_type`、`is_amendment`、`exchange`、`ticker_or_symbol`、`auditor`、`event_signals`、`risk_factors`、`business`、`operating_review`、`controls`、`financial_statements`、`document_url`、`source_url`。

### fund

解析 N-PORT、N-CSR/N-CSRS、N-CEN、N-PX、497K 和 24F-2NT 基金披露。`NPORT-P` 最结构化，包含组合持仓、证券标识、美元市值、组合占比、资产分类、发行人分类、国家和受限证券标记。`N-PX` 是基金代理投票记录，`497K` 是摘要招募书，`24F-2NT` 是年度证券销售通知，`N-CSR` / `N-CSRS` 是股东报告，`N-CEN` 是年度基金 census / 运营信息。

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

`--form` 支持：`all`、`NPORT-P`、`NPORT-P/A`、`N-PORT`、`N-PORT/A`、`N-CSR`、`N-CSR/A`、`N-CSRS`、`N-CSRS/A`、`N-CEN`、`N-CEN/A`、`N-PX`、`N-PX/A`、`497K`、`497K/A`、`24F-2NT`、`24F-2NT/A`。用 `--limit-holdings` 控制返回的 N-PORT 持仓或 N-PX 投票记录数量。

输出字段：`disclosure_type`、`registrant_name`、`series_name`、`class_name`、`period_end`、`fiscal_year_end`、`total_assets`、`total_liabilities`、`net_assets`、`holdings_count`、`holdings`、`proxy_votes_count`、`proxy_votes`、`shareholder_report`、`portfolio_summary`、`proxy_voting_record`、`summary_prospectus`、`registration_fee_notice`、`financial_statements`、`controls`、`document_url`、`source_url`。

### search

搜索 filing 原文，返回可追溯 snippet。

```bash
sec search --ticker TSLA --form 10-K --query "risk factors" --latest 1 --pretty
sec search --ticker NVDA --form 10-K --query "export controls" --jsonl
sec search --cik 320193 --form 10-K --query "supply chain" --context 300 --latest 2 --pretty
```

输出字段：`accession`、`cik`、`company`、`form`、`filing_date`、`query`、`document`、`section`、`offset`、`snippet`、`source_url`。

### section

抽取 10-K/10-Q 常用 section。

```bash
sec section --ticker AAPL --form 10-K --item risk-factors --limit-bytes 8000 --pretty
sec section --ticker MSFT --form 10-K --item mda --latest 1 --pretty
sec section --cik 320193 --form 10-K --item 1A --latest 1 --jsonl
sec section --ticker TSLA --form 10-Q --item market-risk --limit-bytes 6000 --pretty
```

支持：`business`/`1`、`risk-factors`/`1A`、`cybersecurity`/`1C`、`properties`/`2`、`legal-proceedings`/`3`、`mda`/`7`、`market-risk`/`7A`、`financial-statements`/`8`。

输出字段：`accession`、`item`、`title`、`start_offset`、`end_offset`、`byte_length`、`returned_bytes`、`truncated`、`document_url`、`source_url`、`content`。

### report

生成 Markdown 专业汇报。

```bash
sec report --ticker AAPL --kind insider --latest 5 --limit 10
sec report --investor 段永平 --kind portfolio --limit 10
sec report --manager "H&H International Investment LLC" --kind portfolio --limit 10
sec report --cik 1067983 --kind portfolio --limit 10
sec report --ticker AAPL --kind financial --latest 4 --limit 20
sec report --ticker AAPL --kind risk --limit-bytes 4000
sec report --ticker AAPL --kind risk --latest 1 --limit-bytes 12000 > aapl-risk.md
```

`--kind`：

- `financial`：SEC 原始财务指标表 + 多期趋势快照 + 规则型信号
- `insider`：Form 4 owner、role、净股数、交易金额、SEC 来源
- `portfolio`：13F 摘要、Top holdings、可视化条、最大仓位变化
- `risk`：10-K Risk Factors 和 MD&A 摘要

### resolve

把投资人、基金、公众人物、已知 filing manager 或 CIK 解析到 SEC 13F filing manager 候选。标准选择器走确定性规则；自然语言 `--query` 可以用 LLM，但开启验证时最终必须经过 SEC filing 检查。

```bash
sec resolve --query 段永平 --pretty
sec resolve --manager "H&H International Investment LLC" --pretty
sec resolve --cik 1759760 --pretty
sec resolve --query "Warren Buffett" --pretty
sec resolve --query Bridgewater --pretty
sec resolve --query "Seth Klarman" --no-verify --pretty
sec resolve --query 段永平 --llm-provider openai --llm-model GLM-5.1 --pretty
```

输出字段：`query`、`candidate_type`、`investor`、`manager`、`cik`、`confidence`、`relationship`、`evidence_queries`、`notes`、`validation`、`next_commands`。

只有 `validation.status` 为 `verified_13f` 时，才代表候选 CIK 有 SEC `13F-HR` filing。`sec 13f-diff --investor <NAME>` 这类命令会要求验证通过。

### docs / doc

`docs` 列出 submission 内所有 documents；`doc` 读取具体 document。

```bash
sec docs --ticker AAPL --form 10-K --latest 1 --limit 20 --pretty
sec doc --ticker AAPL --form 10-K --primary --limit-bytes 4000 --pretty
sec doc --ticker AAPL --form 10-K --sequence 1 --text --limit-bytes 12000
sec doc --cik 320193 --accession 0000320193-25-000079 --filename aapl-20250927.htm --raw
sec docs --cik 320193 --form 8-K --latest 2 --limit 50 --jsonl
sec doc --ticker AAPL --form 10-K --filename a10-kexhibit21109272025.htm --limit-bytes 4000 --pretty
```

`docs` 输出字段：`accession`、`document_type`、`sequence`、`filename`、`description`、`content_type`、`byte_length`、`is_primary`、`document_url`、`source_url`。

`doc` 额外输出：`returned_bytes`、`truncated`、`content`。

### form4 / form4-summary

解析高管/董事 Form 4。

```bash
sec form4 --ticker AAPL --latest 3 --limit 10 --pretty
sec form4-summary --ticker AAPL --latest 3 --limit 10 --pretty
sec form4 --cik 320193 --latest 10 --jsonl
sec form4-summary --ticker TSLA --include-amends --latest 5 --pretty
```

`form4` 输出交易字段：`issuer`、`issuer_ticker`、`reporting_owner`、`owner_cik`、`officer_title`、`transaction_date`、`transaction_code`、`transaction_type`、`security_title`、`shares`、`price`、`value`、`shares_owned_after`、`direct_or_indirect`、`derivative`、`source_url` 等。

`form4-summary` 输出报告字段：`period_of_report`、`owners`、`signatures`、`footnotes`、`transaction_count`、`acquisition_count`、`disposition_count`、`total_shares_acquired`、`total_shares_disposed`、`net_shares`、`total_value`、`source_url`。

### 8k

解析 Form 8-K 当前报告事件 item，把原本自由文本的 8-K HTML 转成带官方 item 标题、事件分类和来源链接的结构化记录。

```bash
sec 8k --ticker AAPL --latest 5 --pretty
sec 8k --ticker AAPL --item 2.02 --latest 5 --limit-bytes 600 --pretty
sec 8k --ticker TSLA --item 5.02 --latest 10 --jsonl
sec 8k --cik 320193 --item 9.01 --include-amends --pretty
sec 8k-exhibits --ticker AAPL --category earnings_release --latest 5 --limit-bytes 1200 --pretty
sec 8k-exhibits --ticker MSFT --category material_contract --latest 10 --jsonl
```

常见 item：

- `1.01`：重大协议
- `2.02`：经营业绩和财务状况，常见于 earnings release
- `4.02`：以前发布的财报不可依赖
- `5.02`：董事/高管离任、任命或薪酬安排
- `7.01`：Regulation FD Disclosure
- `8.01`：其他事件
- `9.01`：财务报表和附件

`8k-exhibits` 分类包括：`earnings_release`、`press_release`、`material_contract`、`transaction_agreement`、`charter_or_bylaws`、`security_instrument`、`accountant_letter`、`xbrl`、`other_exhibit`。

输出字段：`accession`、`item`、`item_title`、`category`、`is_furnished_item`、`start_offset`、`end_offset`、`byte_length`、`returned_bytes`、`truncated`、`document`、`document_url`、`source_url`、`content`。

### 13d / 13g / schedule13

解析 Schedule 13D / 13G 大股东受益持仓报告。它们回答的是“谁持有这家公司 5% 以上股份、持有多少、有没有主动影响控制权的意图”。

```bash
sec 13d --ticker TSLA --form 13g --latest 2 --include-amends --pretty
sec 13g --ticker TSLA --latest 5 --include-amends --jsonl
sec schedule13 --cik 1318605 --form all --latest 5 --include-amends --pretty
sec parse --ticker TSLA --form "SC 13G" --latest 1 --include-amends --pretty
```

`--form` 支持：`13d`、`13g`、`SC 13D`、`SC 13G`、`SC 13D/A`、`SC 13G/A`、`all`。如果你想看最新持股状态，通常要加 `--include-amends`，因为 13D/13G 的修正文件非常关键。

输出字段：`accession`、`form`、`filing_type`、`is_amendment`、`activist_intent`、`issuer_name`、`security_title`、`cusip`、`event_date`、`reporting_persons`、`filing_rule`、`beneficially_owned_shares`、`percent_of_class`、`sole_voting_power`、`shared_voting_power`、`sole_dispositive_power`、`shared_dispositive_power`、`purpose_of_transaction`、`ownership_summary`、`signatures`、`document_url`、`source_url`。

### 13F

解析和分析机构 13F。

```bash
sec 13f --cik 1067983 --latest 1 --limit 20 --pretty
sec 13f-aggregate --cik 1067983 --latest 1 --limit 20 --pretty
sec 13f-diff --cik 1067983 --limit 20 --pretty
sec 13f-diff --manager "H&H International Investment LLC" --limit 20 --pretty
sec 13f-diff --investor 段永平 --pretty
sec 13f-summary --cik 1067983 --latest 1 --pretty
sec 13f --manager "H&H International Investment LLC" --latest 1 --limit 20 --pretty
sec 13f-aggregate --investor 巴菲特 --latest 1 --limit 20 --pretty
sec 13f-summary --ticker BRK-B --latest 2 --pretty
sec 13f-diff --investor Bridgewater --limit 20 --jsonl
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

### serve

启动本地 JSON HTTP API，给本地 app、dashboard 或 agent 调用。HTTP server 使用同一个 `SecClient`、缓存、parser 和可追溯输出 schema。

```bash
SEC_IDENTITY="Your Name your.email@example.com" sec serve --host 127.0.0.1 --port 8716

curl "http://127.0.0.1:8716/health"
curl "http://127.0.0.1:8716/v1/forms"
curl "http://127.0.0.1:8716/v1/filings?ticker=AAPL&form=10-K&latest=1"
curl "http://127.0.0.1:8716/v1/daily?date=2026-05-15&form=8-K&limit=50"
curl "http://127.0.0.1:8716/v1/efts?query=supply%20chain%20risk&form=10-K&from=2024-01-01&to=2024-12-31&limit=10"
curl "http://127.0.0.1:8716/v1/facts?ticker=AAPL&concept=revenue&latest=3"
curl "http://127.0.0.1:8716/v1/statements?ticker=AAPL&statement=income&period=annual&latest=2"
curl "http://127.0.0.1:8716/v1/stitch?ticker=AAPL&statement=income&latest=8"
curl "http://127.0.0.1:8716/v1/metrics?ticker=AAPL&period=annual&latest=4"
curl "http://127.0.0.1:8716/v1/scores?ticker=AAPL&period=annual&latest=1"
curl "http://127.0.0.1:8716/v1/agent-pack?ticker=AAPL&sections=risk-factors,mda&metrics_latest=4"
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

端点对应关系：

| Endpoint | 对应 CLI |
| --- | --- |
| `/health` | 健康检查 |
| `/v1/forms` | `sec forms` |
| `/v1/filings` | `sec filings` |
| `/v1/daily` | `sec daily` |
| `/v1/efts` | `sec efts` |
| `/v1/facts` | `sec facts` |
| `/v1/statements` | `sec statements` |
| `/v1/stitch` | `sec stitch` |
| `/v1/metrics` | `sec metrics` |
| `/v1/scores` | `sec scores` |
| `/v1/agent-pack` | `sec agent-pack` |
| `/v1/company-report` | `sec company-report` |
| `/v1/ixbrl` | `sec ixbrl` |
| `/v1/sections` | `sec section` |
| `/v1/docs` | `sec docs` |
| `/v1/form4`、`/v1/form4-summary` | `sec form4`、`sec form4-summary` |
| `/v1/8k` | `sec 8k` |
| `/v1/8k-exhibits` | `sec 8k-exhibits` |
| `/v1/schedule13` | `sec 13d` / `sec 13g` |
| `/v1/13f`、`/v1/13f-summary`、`/v1/13f-diff` | `sec 13f`、`sec 13f-summary`、`sec 13f-diff` |
| `/v1/proxy` | `sec proxy` |
| `/v1/prospectus` | `sec prospectus` |
| `/v1/foreign` | `sec foreign` |
| `/v1/fund` | `sec fund` |
| `/v1/parse` | `sec parse` |

### mcp

启动 stdio Model Context Protocol adapter，给支持 MCP 的 Agent 使用。它通过 stdin/stdout 读写 JSON-RPC，不需要启动 HTTP server。

```bash
sec config set-identity "Your Name your.email@example.com"
sec mcp
```

当前 MCP tools：

| Tool | 对应能力 |
| --- | --- |
| `sec_forms` | parser registry |
| `sec_filings` | 等价于 `sec filings` |
| `sec_daily` | 等价于 `sec daily` 全市场 index 扫描 |
| `sec_efts` | 等价于 `sec efts` SEC 全文搜索 |
| `sec_facts` | 等价于 `sec facts` |
| `sec_statements` | 等价于 `sec statements` |
| `sec_stitch` | 等价于 `sec stitch` |
| `sec_metrics` | 等价于 `sec metrics` |
| `sec_scores` | 等价于 `sec scores` |
| `sec_agent_pack` | 等价于 `sec agent-pack` |
| `sec_ixbrl` | 等价于 `sec ixbrl` |
| `sec_tables` | 等价于 `sec tables` |
| `sec_company_report` | 等价于 `sec company-report` |
| `sec_proxy` | 等价于 `sec proxy` |
| `sec_prospectus` | 等价于 `sec prospectus` |
| `sec_foreign` | 等价于 `sec foreign` |
| `sec_fund` | 等价于 `sec fund` |
| `sec_search` | 等价于 `sec search` |
| `sec_section` | 等价于 `sec section` |
| `sec_docs` | 等价于 `sec docs` |
| `sec_doc` | 等价于 `sec doc` |
| `sec_form4` | 等价于 `sec form4` |
| `sec_form4_summary` | 等价于 `sec form4-summary` |
| `sec_8k` | 等价于 `sec 8k` |
| `sec_8k_exhibits` | 等价于 `sec 8k-exhibits` |
| `sec_schedule13` | 等价于 `sec 13d` / `sec 13g` |
| `sec_13f` | 等价于 `sec 13f` |
| `sec_13f_aggregate` | 等价于 `sec 13f-aggregate` |
| `sec_13f_diff` | 等价于 CIK/ticker 选择器下的 `sec 13f-diff` |
| `sec_13f_summary` | 等价于 `sec 13f-summary` |
| `sec_report` | 生成 `insider`、`portfolio`、`risk` Markdown 报告 |
| `sec_parse` | 统一 parser pipeline |

MCP tool 参数示例：

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

## 参数参考

全局参数：

| 参数 | 含义 |
| --- | --- |
| `--identity <TEXT>` | SEC 请求身份 / user agent；未设置本地配置、`SEC_IDENTITY` 或 `EDGAR_IDENTITY` 时必填 |
| `--cache-dir <PATH>` | 指定本地缓存目录 |
| `--output <MODE>` | 全局覆盖结构化输出：`json`、`pretty`、`jsonl`、`csv`、`table` |

命令参数：

| 命令 | 必要选择器 | 重要参数 |
| --- | --- | --- |
| `filings` | `--ticker` 或 `--cik` | `--form`、`--latest`、`--from`、`--to`、`--include-amends`、`--jsonl`、`--pretty` |
| `daily` / `monitor` | 无 | `--date`、`--form`、`--company`、`--limit`、`--include-amends`、`--jsonl`、`--pretty` |
| `efts` / `full-text` / `global-search` | `--query` | `--ticker`、`--cik`、`--form`、`--from`、`--to`、`--limit`、`--jsonl`、`--pretty` |
| `facts` | `--ticker` 或 `--cik`，`--concept` | `--form`、`--unit`、`--latest`、`--jsonl`、`--pretty` |
| `statements` | `--ticker` 或 `--cik` | `--statement`、`--period`、`--unit`、`--latest`、`--jsonl`、`--pretty` |
| `stitch` / `statement-stitch` | `--ticker` 或 `--cik` | `--statement`、`--unit`、`--latest`、`--jsonl`、`--pretty` |
| `metrics` | `--ticker` 或 `--cik` | `--period`、`--unit`、`--latest`、`--jsonl`、`--pretty` |
| `scores` | `--ticker` 或 `--cik` | `--period`、`--unit`、`--latest`、`--jsonl`、`--pretty` |
| `export` | `--ticker` 或 `--cik`，`--kind`，`--format`，`--out` | `--concept`、`--form`、`--statement`、`--period`、`--unit`、`--latest`、`--include-amends` |
| `archive` | `--ticker` 或 `--cik`，`--out-dir` | `--form`、`--latest`、`--include-amends`、`--primary-only`、`--limit-bytes`、`--jsonl`、`--pretty` |
| `agent-pack` / `pack` | `--ticker` 或 `--cik` | `--form`、`--latest`、`--sections`、`--section-limit-bytes`、`--metrics-latest`、`--jsonl`、`--pretty` |
| `company-report` | `--ticker` 或 `--cik` | `--form`、`--topic`、`--latest`、`--limit-tables`、`--limit-rows`、`--include-amends`、`--jsonl`、`--pretty` |
| `ixbrl` | `--ticker` 或 `--cik` | `--form`、`--concept`、`--latest`、`--limit`、`--include-amends`、`--jsonl`、`--pretty` |
| `xbrl-links` / `linkbase` | `--ticker` 或 `--cik` | `--form`、`--linkbase`、`--role`、`--concept`、`--latest`、`--limit`、`--include-amends`、`--jsonl`、`--pretty` |
| `xbrl-tree` / `presentation-tree` | `--ticker` 或 `--cik` | `--form`、`--role`、`--concept`、`--latest`、`--limit`、`--include-amends`、`--jsonl`、`--pretty` |
| `xbrl-calc` / `calculation-checks` | `--ticker` 或 `--cik` | `--form`、`--role`、`--concept`、`--unit`、`--tolerance`、`--latest`、`--limit`、`--include-amends`、`--jsonl`、`--pretty` |
| `xbrl-statement` / `statement-render` | `--ticker` 或 `--cik` | `--form`、`--role`、`--concept`、`--unit`、`--tolerance`、`--values-only`、`--latest`、`--limit`、`--include-amends`、`--jsonl`、`--pretty` |
| `tables` | `--ticker` 或 `--cik` | `--form`、`--latest`、`--limit-tables`、`--limit-rows`、`--include-amends`、`--jsonl`、`--pretty` |
| `proxy` | `--ticker` 或 `--cik` | `--latest`、`--limit-rows`、`--include-amends`、`--jsonl`、`--pretty` |
| `prospectus` | `--ticker` 或 `--cik` | `--form`、`--latest`、`--limit-bytes`、`--limit-tables`、`--limit-rows`、`--include-amends`、`--jsonl`、`--pretty` |
| `foreign` | `--ticker` 或 `--cik` | `--form`、`--latest`、`--limit-bytes`、`--include-amends`、`--jsonl`、`--pretty` |
| `fund` | `--ticker` 或 `--cik` | `--form`、`--latest`、`--limit-holdings`、`--limit-bytes`、`--include-amends`、`--jsonl`、`--pretty` |
| `search` | `--ticker` 或 `--cik`，`--query` | `--form`、`--latest`、`--context`、`--include-amends`、`--jsonl`、`--pretty` |
| `section` | `--ticker` 或 `--cik`，`--item` | `--form`、`--latest`、`--accession`、`--limit-bytes`、`--include-amends`、`--jsonl`、`--pretty` |
| `report` | `--ticker`、`--cik`、`--manager` 或 `--investor`，`--kind` | `--latest`、`--limit`、`--limit-bytes`、`--include-amends` |
| `resolve` | `--query`、`--manager` 或 `--cik` | `--no-verify`、`--llm-provider`、`--llm-base-url`、`--llm-model`、`--llm-api-key-env`、`--jsonl`、`--pretty` |
| `docs` | `--ticker` 或 `--cik` | `--form`、`--latest`、`--limit`、`--include-amends`、`--jsonl`、`--pretty` |
| `doc` | `--ticker` 或 `--cik` | `--form`、`--latest`、`--accession`、`--filename`、`--sequence`、`--primary`、`--limit-bytes`、`--raw`、`--text`、`--jsonl`、`--pretty` |
| `form4` / `form4-summary` | `--ticker` 或 `--cik` | `--latest`、`--limit`、`--include-amends`、`--jsonl`、`--pretty` |
| `8k` | `--ticker` 或 `--cik` | `--item`、`--latest`、`--limit`、`--limit-bytes`、`--include-amends`、`--jsonl`、`--pretty` |
| `8k-exhibits` | `--ticker` 或 `--cik` | `--category`、`--latest`、`--limit`、`--limit-bytes`、`--include-amends`、`--jsonl`、`--pretty` |
| `13d` / `13g` / `schedule13` | `--ticker` 或 `--cik` | `--form`、`--latest`、`--include-amends`、`--limit-bytes`、`--jsonl`、`--pretty` |
| `13f` / `13f-aggregate` / `13f-diff` / `13f-summary` | `--ticker`、`--cik`、`--manager` 或 `--investor` | `--latest`、`--limit`、`--include-amends`、`--jsonl`、`--pretty` |
| `parse` | `--ticker` 或 `--cik`，`--form` | `--latest`、`--limit`、`--include-amends`、`--jsonl`、`--pretty` |
| `forms` | 无 | `--jsonl`、`--pretty` |
| `config` | 无 | `set-identity <TEXT>`、`show`、`path` |
| `completions` | shell 名称 | `bash`、`zsh`、`fish`、`power-shell`、`elvish` |
| `serve` | 无 | `--host`、`--port` |
| `mcp` | 无 | stdio JSON-RPC server；在 Agent 环境里设置本地 identity 或 `SEC_IDENTITY` |

## 输出模式

- 默认：紧凑 JSON
- `--pretty`：格式化 JSON
- `--jsonl`：一行一个 JSON record
- `--output csv`：CSV，可直接导入 Excel / Sheets
- `--output table`：终端表格，适合人工快速扫读
- `sec report`：Markdown 汇报
- `sec doc --raw`：原始 document 内容
- `sec doc --text`：简化纯文本

示例：

```bash
sec --output csv filings --ticker AAPL --form 10-K --latest 3
sec --output table filings --ticker AAPL --form 10-K --latest 3
sec export --kind metrics --ticker AAPL --period annual --latest 4 --format parquet --out data/aapl_metrics.parquet
sec export --kind scores --ticker AAPL --period annual --latest 1 --format arrow --out data/aapl_scores.arrow
```

## Agent 工作流

推荐：

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
