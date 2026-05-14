# TrustRAG RAG 管线设计

> 参考：Docify 11 步管线、RAG-Knowledge-Base-Platform 位置映射

---

## 总览

```
用户提问
    │
    ▼
┌──────────────────────────────────────────────────────────────┐
│                    RAG Pipeline (10 步)                       │
│                                                              │
│  1. Query Analysis ──► 2. Query Expansion                    │
│         │                      │                             │
│         ▼                      ▼                             │
│  3. Hybrid Retrieval (Vector + BM25/pg_bigm)                 │
│         │                                                    │
│         ▼                                                    │
│  4. Re-Ranking (Cross-encoder 或 LLM re-rank)               │
│         │                                                    │
│         ▼                                                    │
│  5. Context Assembly (Token 预算管理 + 去重)                  │
│         │                                                    │
│         ▼                                                    │
│  6. Prompt Engineering (反幻觉 + 引用指令)                    │
│         │                                                    │
│         ▼                                                    │
│  7. LLM Generation (流式输出)                                │
│         │                                                    │
│         ▼                                                    │
│  8. Citation Extraction (从回答中解析引用标记)                │
│         │                                                    │
│         ▼                                                    │
│  9. Citation Verification (验证引用准确性)                    │
│         │                                                    │
│         ▼                                                    │
│  10. Response Assembly (最终消息 + 引用列表)                  │
└──────────────────────────────────────────────────────────────┘
    │
    ▼
前端展示（流式 Markdown + 引用卡片 + 跳转）
```

---

## 阶段详解

### 阶段 1: Query Analysis（查询分析）

**职责**：理解用户意图，判断是否需要检索

| 输入 | 输出 |
|------|------|
| 用户消息 + 对话历史 | 分析结果 |

```rust
struct QueryAnalysis {
    intent: QueryIntent,        // Factual / Exploratory / Comparison / Summary
    needs_retrieval: bool,      // 闲聊不需要检索
    language: String,           // 检测语言
    key_entities: Vec<String>,  // 提取关键实体
    document_scope: Vec<Uuid>,  // 用户指定的文档范围
}

enum QueryIntent {
    Factual,       // 事实性问题 → 精确检索
    Exploratory,   // 探索性问题 → 广泛检索
    Comparison,    // 对比类问题 → 多文档检索
    Summary,       // 总结类问题 → 全文检索
    Chitchat,      // 闲聊 → 跳过检索
}
```

**实现方式**：
- MVP：基于规则 + 关键词匹配
- 后续：用 LLM 做意图分类

### 阶段 2: Query Expansion（查询扩展）

**职责**：扩展原始查询以提高召回率

| 策略 | 说明 | MVP |
|------|------|-----|
| 同义词扩展 | "认证" → "认证, 身份验证, authentication" | ✓ |
| 多查询生成 | 用 LLM 生成 2-3 个相关查询变体 | Phase 2 |
| 对话历史融合 | 将上下文注入当前查询 | ✓ |

```rust
struct ExpandedQuery {
    original: String,
    rewritten: String,         // 融合了上下文的完整查询
    variants: Vec<String>,     // 查询变体（用于多路检索）
}
```

### 阶段 3: Hybrid Retrieval（混合检索）

**职责**：从文档库中检索相关分块

```
                 查询
                  │
        ┌─────────┼─────────┐
        ▼                    ▼
   Vector Search        Full-Text Search
   (pgvector)           (pg_bigm)
        │                    │
        ▼                    ▼
   Vec<(chunk, score)>  Vec<(chunk, score)>
        │                    │
        └─────────┬──────────┘
                  ▼
          Score Fusion (RRF)
                  │
                  ▼
         Top-K Candidates
```

**Vector Search**：
```sql
SELECT id, content, heading_path, page_start, page_end,
       1 - (embedding <=> $1) AS score
FROM document_chunks
WHERE document_id = ANY($2)
ORDER BY embedding <=> $1
LIMIT $3;
```

**Full-Text Search (pg_bigm)**：
```sql
SELECT id, content, heading_path, page_start, page_end,
       similarity(content, $1) AS score
FROM document_chunks
WHERE content LIKE '%' || $1 || '%'
  AND document_id = ANY($2)
ORDER BY score DESC
LIMIT $3;
```

**Score Fusion - Reciprocal Rank Fusion (RRF)**：
```
RRF_score(d) = Σ 1/(k + rank_i(d))
```
其中 k=60（常数），rank_i 是文档 d 在第 i 个检索系统中的排名。

**配置参数**：
```rust
struct RetrievalConfig {
    mode: RetrievalMode,       // Vector | FullText | Hybrid
    top_k: usize,              // 每路检索的候选数量 (默认 20)
    final_k: usize,            // 融合后保留数量 (默认 10)
    min_score: f32,            // 最低分数阈值 (默认 0.3)
    use_mmr: bool,             // 是否启用 MMR 多样性
    mmr_lambda: f32,           // MMR 多样性参数 (默认 0.7)
    rrf_k: usize,              // RRF 常数 (默认 60)
}
```

### 阶段 4: Re-Ranking（重排序）

**职责**：对候选分块进行精细排序

| 策略 | 准确度 | 速度 | MVP |
|------|--------|------|-----|
| 无重排序 | 低 | 快 | ✓（默认） |
| LLM re-rank | 高 | 慢 | Phase 2 |
| Cross-encoder | 高 | 中 | Phase 2 |

**MVP 实现**：直接用 RRF 融合分数排序
**Phase 2**：用 fastembed 的 reranker 或 LLM 做精细重排

```rust
struct RankedChunk {
    chunk: DocumentChunk,
    vector_score: Option<f32>,
    bm25_score: Option<f32>,
    rrf_score: f32,
    rerank_score: Option<f32>,  // Phase 2
    final_score: f32,
}
```

### 阶段 5: Context Assembly（上下文组装）

**职责**：将检索结果组装为 LLM 上下文，管理 Token 预算

```rust
struct ContextAssembly {
    max_context_tokens: usize,  // Token 预算 (默认 4000)
    strategy: AssemblyStrategy,
}

enum AssemblyStrategy {
    TopK,                       // 按分数取前 K
    WindowExpand,               // 扩展窗口（加入前后分块）
    SectionAware,               // 按章节完整性组装
}
```

**组装规则**：
1. 按 final_score 降序排列
2. 逐一加入，检查 Token 预算
3. 去重：content_hash 相同的分块只保留分数最高的
4. 窗口扩展：如果分块太短，加入相邻分块提供上下文
5. 为每个分块标注来源元数据（文档名、页码、章节）

**输出格式**：
```
[Source 1: 安全规范.pdf | 第12页 | 第3章 > 3.2 认证]
原文内容片段...

[Source 2: 设计文档.docx | 第5页]
原文内容片段...

[Source 3: ...]
...
```

### 阶段 6: Prompt Engineering（提示工程）

**职责**：构造反幻觉 Prompt，强制引用格式

**系统 Prompt**：
```
你是 TrustRAG 知识助手。你的回答必须严格基于提供的参考资料。

规则：
1. 只使用参考资料中的信息回答问题
2. 每个事实性陈述必须附上引用标记 [1], [2] 等
3. 引用编号对应参考资料的 Source 编号
4. 如果参考资料中没有相关信息，明确说"根据提供的资料，我无法回答这个问题"
5. 不要编造、推测或使用参考资料以外的知识
6. 如果不确定某个信息是否准确，用"根据资料 [X] 的描述"等限定语
7. 每个句子最多引用 3 个来源

参考资料：
{assembled_context}

用户问题：{user_query}
```

### 阶段 7: LLM Generation（大模型生成）

**职责**：调用 LLM 生成回答，支持流式输出

```rust
struct GenerationConfig {
    model_config: ModelConfig,
    temperature: f32,          // 默认 0.1（事实性问题低温）
    max_tokens: usize,         // 默认 4096
    stream: bool,              // 是否流式
    stop_sequences: Vec<String>,
}
```

**流式输出**：
- 使用 Axum SSE
- 每个 token 即时推送到前端
- 同时在内存中累积完整回答（用于后续引用解析）

### 阶段 8: Citation Extraction（引用提取）

**职责**：从 LLM 回答中解析引用标记

```rust
fn extract_citations(response: &str) -> Vec<CitationRef> {
    // 正则匹配 [1], [2] 等标记
    // 返回：引用编号 + 在回答中的位置
}

struct CitationRef {
    index: usize,           // 引用编号
    position: usize,        // 在回答文本中的字符位置
    source_chunk: &RankedChunk,  // 对应的检索分块
}
```

### 阶段 9: Citation Verification（引用验证）

**职责**：验证 LLM 声称的引用是否与原文匹配

**验证策略**（参考 Docify）：

```rust
enum VerificationResult {
    Accurate,          // 引用准确
    Paraphrased,       // 改述但含义正确
    Unsupported,       // 引用来源中找不到支持
    Fabricated,        // 引用编号不存在
}

struct CitationVerification {
    citation_index: usize,
    result: VerificationResult,
    confidence: f32,
    original_text: String,    // 原文片段
    claimed_text: String,     // LLM 声称的内容
}
```

**MVP 实现**：
1. 检查引用编号是否在有效范围内
2. 检查引用的文本是否与对应分块有语义重叠（简单字符串匹配）

**Phase 2**：
1. 用 LLM 判断引用是否准确
2. 计算引用文本与原文的语义相似度
3. 标记可疑引用供人工审核

### 阶段 10: Response Assembly（响应组装）

**职责**：将验证后的回答 + 引用打包为最终响应

```rust
struct FinalResponse {
    message: Message,
    citations: Vec<Citation>,
    verification_summary: VerificationSummary,
    metadata: ResponseMetadata,
}

struct ResponseMetadata {
    search_time_ms: u64,
    generation_time_ms: u64,
    total_time_ms: u64,
    chunks_retrieved: usize,
    chunks_used: usize,
    model_name: String,
    prompt_tokens: usize,
    completion_tokens: usize,
}
```

---

## 文档摄入管线（Ingestion Pipeline）

```
用户上传文件
    │
    ▼
1. 文件验证（类型、大小、安全检查）
    │
    ▼
2. 存储原始文件 → MinIO
    │
    ▼
3. 调用 Python doc-processor 服务
    ├── PDF → PyMuPDF 解析 → 提取文本+坐标+页码
    ├── DOCX → python-docx 解析
    ├── MD/TXT → 直接读取
    └── OCR（如需） → pytesseract
    │
    ▼
4. 结构化输出
    ├── Markdown 全文
    ├── 位置映射表（每段文字 → 原文页码+坐标）
    └── 章节结构（heading_path, section_level）
    │
    ▼
5. 存储 Markdown → MinIO
    │
    ▼
6. 文本分块（text-splitter / chunkedrs）
    ├── Markdown-aware 分块
    ├── 保留 heading_path 上下文
    ├── 目标：500-1000 tokens/chunk
    └── 重叠：50-100 tokens
    │
    ▼
7. Embedding 生成
    ├── 方案 A：API（async-openai → text-embedding-3-small）
    └── 方案 B：本地（fastembed → nomic-embed-text-v2）
    │
    ▼
8. 写入 PostgreSQL
    ├── document_chunks 表（content + metadata）
    ├── embedding 列（vector(1536)）
    └── 更新 document.processing_status = 'ready'
```

**doc-processor 返回格式**：
```json
{
  "markdown": "# 标题\n\n正文...",
  "pages": [
    {
      "page_number": 1,
      "text": "页面全文",
      "blocks": [
        {
          "type": "text",
          "content": "段落内容",
          "bbox": [x0, y0, x1, y1],
          "heading_level": null
        },
        {
          "type": "heading",
          "content": "第一章 概述",
          "bbox": [x0, y0, x1, y1],
          "heading_level": 1
        }
      ]
    }
  ],
  "headings": [
    {"text": "第一章 概述", "level": 1, "page": 1},
    {"text": "1.1 背景", "level": 2, "page": 1}
  ],
  "metadata": {
    "title": "技术规范",
    "author": "...",
    "page_count": 42,
    "language": "zh"
  }
}
```

---

## 配置参数汇总

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `retrieval.mode` | hybrid | vector / fulltext / hybrid |
| `retrieval.top_k` | 20 | 每路检索候选数 |
| `retrieval.final_k` | 10 | 融合后保留数 |
| `retrieval.min_score` | 0.3 | 最低分数阈值 |
| `retrieval.use_mmr` | true | 多样性检索 |
| `retrieval.mmr_lambda` | 0.7 | MMR 参数 |
| `chunking.target_tokens` | 500 | 目标分块大小 |
| `chunking.max_tokens` | 1000 | 最大分块大小 |
| `chunking.overlap_tokens` | 50 | 分块重叠 |
| `context.max_tokens` | 4000 | 上下文 Token 预算 |
| `generation.temperature` | 0.1 | LLM 温度 |
| `generation.max_tokens` | 4096 | 最大生成长度 |
| `embedding.dimensions` | 1536 | 向量维度 |
| `verification.enabled` | true | 是否启用引用验证 |
