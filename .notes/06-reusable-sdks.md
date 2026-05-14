# 可复用的 SDK 和框架调研（2026-05-14）

## 策略

不从零造轮子，复用成熟组件，只自定义差异化功能。

---

## 完整方案参考（架构/设计参考）

### 1. RAG Knowledge Base Platform
- GitHub: github.com/loglux/RAG-Knowledge-Base-Platform
- 匹配度: 90%
- 技术栈: FastAPI + React + Qdrant + PostgreSQL
- 核心亮点:
  - Section-aware retrieval（标题结构感知检索）
  - PDF 页码追踪（物理页码 + 逻辑页码）
  - 原始文件保存 + 重处理能力
  - 引用溯源含页码和章节
  - MMR 多样性检索
  - Self-Check Validation（可选验证层）
  - Retrieve-only API（无副作用的纯检索）
- 参考价值: 文档处理 + 位置映射 + 引用机制的架构设计

### 2. R2R by SciPhi-AI
- GitHub: github.com/sciphi-ai/r2r (7.7k stars)
- 匹配度: 80%
- 技术栈: Python, RESTful API
- 核心亮点:
  - 生产级 RAG 框架，最成熟
  - 多格式摄入（PDF/JSON/PNG/MP3）
  - 混合检索 + 知识图谱
  - Deep Research API（多步推理）
  - 用户与权限管理
- 参考价值: API 设计、多步推理架构

### 3. Docify
- GitHub: github.com/keshavashiya/docify (172 stars)
- 匹配度: 85%
- 技术栈: FastAPI + React + PostgreSQL + pgvector + Celery
- 核心亮点:
  - 11 步 RAG 管线（最完整）:
    1. Resource Ingestion
    2. Chunking（语义边界保留）
    3. Embeddings（异步 Celery）
    4. Query Expansion
    5. Hybrid Search（语义 + BM25）
    6. Re-Ranking（5因素评分 + 冲突检测）
    7. Context Assembly（Token 预算管理）
    8. Prompt Engineering（反幻觉 prompt）
    9. LLM Service
    10. Citation Verification（引用验证！）
    11. Message Generation
  - 智能去重（内容指纹）
  - 工作区模型
  - Citation Verification（其他项目没有的）
- 参考价值: RAG 管线架构，特别是 Citation Verification

### 4. OpenRAG by Langflow
- GitHub: github.com/langflow-ai/openrag (3.9k stars)
- 技术栈: FastAPI + Next.js + OpenSearch + Langflow
- 核心亮点:
  - 可视化工作流构建器
  - Agentic RAG workflows
  - MCP 集成
  - Apache 2.0 许可
- 参考价值: 企业级 RAG 平台设计

---

## Rust 生态可直接引用的 crate

### 5. `rag` crate
- 来源: crates.io/crates/rag
- 许可: Apache 2.0
- 功能:
  - 纯 Rust RAG 库
  - 向量 RAG + 图 RAG + BM25 混合检索
  - PDF/代码/Wiki 摄入
  - 多种分块策略（固定/段落/句子）
  - OpenAI/Ollama embedding 后端
  - MCP server
  - CLI + 库 API
- 直接用于: TrustRAG 后端检索模块

### 6. `rag-toolchain` crate
- 来源: crates.io/crates/rag-toolchain
- 许可: MIT (推测)
- 功能:
  - PostgreSQL + pgvector 支持
  - OpenAI embedding + chat
  - BasicRAGChain 开箱即用
  - Token 分块
- 直接用于: RAG 管线快速搭建

### 7. Korvus by PostgresML
- GitHub: github.com/postgresml/korvus (1.4k stars)
- 许可: MIT
- 功能:
  - 整个 RAG 管线在单条 SQL 查询中完成
  - Python/JS/Rust 绑定
  - embedding 生成 + 向量检索 + 重排序 + 文本生成
  - 基于 PostgreSQL + pgml 扩展
- 直接用于: 高性能检索层（如果愿意用 pgml）

### 8. A.R.E.S (Agentic Retrieval Enhanced Server)
- 来源: crates.io/crates/ares-server
- 许可: MIT (推测)
- 功能:
  - Rust 全栈 AI server
  - 多 LLM provider（Ollama/OpenAI/Anthropic/LlamaCpp）
  - 内置 ares-vector（纯 Rust 向量存储）
  - RAG: semantic + BM25 + fuzzy + hybrid
  - MCP 集成
  - 多租户认证
- 直接用于: 如果想直接用一个 Rust RAG server（但定制性可能不够）

---

## 推荐整合策略

| 需求 | 推荐来源 |
|------|---------|
| 后端检索层 | `rag` 或 `rag-toolchain` crate |
| RAG 管线架构 | 参考 Docify 的 11 步管线 |
| 引用验证 | 参考 Docify 的 Citation Verification |
| 文档处理 + 位置映射 | 参考 RAG-Knowledge-Base-Platform |
| API 设计 | 参考 R2R |
| Flutter UI 组件 | Material 3 + Syncfusion |
