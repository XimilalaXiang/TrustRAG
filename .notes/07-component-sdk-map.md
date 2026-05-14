# TrustRAG 全组件 → 参考项目/SDK 映射表（2026-05-14）

> 原则：每个组件优先找 Rust 原生方案，次选成熟的第三方参考

---

## 一、后端主服务（Rust + Axum）

### 1.1 Web 框架 + HTTP

| 组件 | 我们的选择 | 参考项目/SDK | 说明 |
|------|-----------|-------------|------|
| Web 框架 | Axum 0.8 | [tokio-rs/axum](https://github.com/tokio-rs/axum) | Rust 最流行的异步 Web 框架 |
| 异步运行时 | tokio | [tokio-rs/tokio](https://github.com/tokio-rs/tokio) | Rust 异步生态核心 |
| 中间件 | tower + tower-http | [tower-rs/tower](https://github.com/tower-rs/tower) | CORS、Trace、压缩等 |
| API 文档 | utoipa | [juhaku/utoipa](https://github.com/juhaku/utoipa) | OpenAPI 自动生成 Swagger |

### 1.2 认证 + 授权

| 组件 | 我们的选择 | 参考项目/SDK | 说明 |
|------|-----------|-------------|------|
| JWT 验证 | jsonwebtoken | [Keats/jsonwebtoken](https://github.com/Keats/jsonwebtoken) | 12M+ 下载，标准选择 |
| JWT 中间件 | axum-jwt-auth | [cmackenzie1/axum-jwt-auth](https://github.com/cmackenzie1/axum-jwt-auth) | 0.6.3, 专为 Axum 设计的 JWT 提取器 |
| 密码哈希 | argon2 | [RustCrypto/password-hashes](https://github.com/RustCrypto/password-hashes) | OWASP 推荐算法 |

### 1.3 LLM 调用 + AI 流式响应

| 组件 | 我们的选择 | 参考项目/SDK | 说明 |
|------|-----------|-------------|------|
| OpenAI API 客户端 | async-openai | [64bit/async-openai](https://github.com/64bit/async-openai) | Rust 最成熟的 OpenAI 客户端 |
| **多 Provider 统一接口** | **rig-core** | [0xPlaygrounds/rig](https://github.com/0xPlaygrounds/rig) | 统一 OpenAI/Anthropic/Ollama/Gemini，支持 Agent/Tool/Streaming |
| SSE 流式输出 | Axum SSE 内置 | [ellix.ai 的 SSE 实现](https://ellix.ai/blog/streaming-ai-responses) | 完整的生产级 SSE handler 参考 |
| **Vercel AI SDK 协议** | **rig-ai-sdk** | [rig-ai-sdk crate](https://crates.io/crates/rig-ai-sdk) | 适配 Vercel AI SDK Data Stream Protocol，前端可用 useChat 等 hook |
| 替代方案: 统一 LLM | cognate-llm | [vornyx-rs/cognate](https://github.com/vornyx-rs/cognate) | 多 provider + 类型安全 Tool + Axum 集成 + RAG |
| 替代方案: 统一 LLM | saorsa-ai | [saorsa-ai crate](https://crates.io/crates/saorsa-ai) | 多 provider 统一 stream event，支持 Axum SSE |
| 替代方案: Axum AI SDK | aisdk (aisdk.rs) | [aisdk.rs](https://aisdk.rs/) | Rust 版 Vercel AI SDK，原生 Axum 集成 |

### 1.4 RAG 检索管线

| 组件 | 我们的选择 | 参考项目/SDK | 说明 |
|------|-----------|-------------|------|
| **RAG 核心库** | rag crate | [crates.io/crates/rag](https://crates.io/crates/rag) | 纯 Rust RAG，向量+BM25+图检索 |
| RAG 工具链 | rag-toolchain | [rag-toolchain crate](https://crates.io/crates/rag-toolchain) | PostgreSQL+pgvector+OpenAI，BasicRAGChain |
| pgvector Rust 绑定 | pgvector crate | [pgvector/pgvector-rust](https://github.com/pgvector/pgvector-rust) | 12M+ 下载，支持 SQLx |
| rig + pgvector 集成 | rig-postgres | [rig-postgres crate](https://crates.io/crates/rig-postgres) | rig 框架的 PostgreSQL 向量存储 |
| RAG 管线架构参考 | - | [Docify](https://github.com/keshavashiya/docify) | 11 步管线设计，含 Citation Verification |
| 引用溯源参考 | - | [RAG-Knowledge-Base-Platform](https://github.com/loglux/RAG-Knowledge-Base-Platform) | Section-aware + PDF 页码追踪 |
| API 设计参考 | - | [R2R by SciPhi-AI](https://github.com/sciphi-ai/r2r) (7.7k stars) | 最成熟的 RESTful RAG API |

### 1.5 文本分块（Chunking）

| 组件 | 参考项目/SDK | Stars/Downloads | 说明 |
|------|-------------|-----------------|------|
| **推荐: 语义分块** | [text-splitter](https://github.com/benbrandt/text-splitter) | Rust+Python 双绑定 | Markdown/Code/Text 分块，支持 tiktoken+tokenizers |
| 替代: AI 原生分块 | [chunkedrs](https://crates.io/crates/chunkedrs) | Rust | tiktoken 精确 token 计数，递归/Markdown/语义三种策略 |
| 替代: 高速分块 | [chunk](https://crates.io/crates/chunk) | Rust, 1TB/s | SIMD 加速，chonkie 核心 |
| 替代: 语义分块 | [wg-ragsmith](https://crates.io/crates/wg-ragsmith) | Rust | 语义分块+SQLite向量存储，支持 HTML/JSON |
| 代码分块 | [code-splitter](https://crates.io/crates/code-splitter) | Rust | tree-sitter AST 感知分块 |
| 平衡分块 | [llm_utils](https://github.com/shelbyJenkins/llm_utils) | Rust | 平衡长度分块，避免孤立尾块 |

### 1.6 Embedding 生成

| 组件 | 参考项目/SDK | 说明 |
|------|-------------|------|
| **推荐: 本地 Embedding** | [fastembed-rs](https://github.com/Anush008/fastembed-rs) (1M+ 下载) | ONNX 推理，支持 Qwen3/Nomic 等，无需 API |
| API Embedding | async-openai | 通过 OpenAI-compatible API 生成 |
| ADK RAG 模块 | [adk-rag](https://docs.rs/adk-rag) | 模块化 RAG，支持 OpenAI/Gemini embedding + pgvector |

### 1.7 数据库 + 存储

| 组件 | 我们的选择 | 参考项目/SDK | 说明 |
|------|-----------|-------------|------|
| PostgreSQL 客户端 | SQLx 0.8 | [launchbadge/sqlx](https://github.com/launchbadge/sqlx) | 编译期 SQL 校验 |
| pgvector 支持 | pgvector crate | [pgvector/pgvector-rust](https://github.com/pgvector/pgvector-rust) | SQLx 集成 |
| Redis 客户端 | redis-rs | [redis-rs/redis-rs](https://github.com/redis-rs/redis-rs) | 异步 Redis |
| S3/MinIO 存储 | opendal | [apache/opendal](https://github.com/apache/opendal) | Apache 基金会项目，多后端 |
| 异步任务队列 | apalis | [geofmureithi/apalis](https://github.com/geofmureithi/apalis) | Rust 原生异步任务框架 |

### 1.8 配置 + 日志 + 错误处理

| 组件 | 我们的选择 | 参考项目/SDK | 说明 |
|------|-----------|-------------|------|
| 配置管理 | config-rs | [mehcode/config-rs](https://github.com/mehcode/config-rs) | 多源配置合并 |
| 结构化日志 | tracing | [tokio-rs/tracing](https://github.com/tokio-rs/tracing) | Rust 日志标准 |
| 错误处理 | thiserror + anyhow | [dtolnay/thiserror](https://github.com/dtolnay/thiserror) | 库错误+应用错误 |
| 序列化 | serde | [serde-rs/serde](https://github.com/serde-rs/serde) | Rust 序列化标准 |

---

## 二、文档处理服务（Python + FastAPI）

| 组件 | 我们的选择 | 参考项目/SDK | 说明 |
|------|-----------|-------------|------|
| Web 框架 | FastAPI | [tiangolo/fastapi](https://github.com/tiangolo/fastapi) (80k+ stars) | 异步 Python Web 标准 |
| PDF 解析 | PyMuPDF (fitz) | [pymupdf/PyMuPDF](https://github.com/pymupdf/PyMuPDF) | 高性能，坐标+文本+图像 |
| PDF 表格提取 | pdfplumber | [jsvine/pdfplumber](https://github.com/jsvine/pdfplumber) | 坐标级精确表格 |
| DOCX 解析 | python-docx | [python-docx](https://github.com/python-openxml/python-docx) | 标准选择 |
| OCR | pytesseract + Pillow | [tesseract-ocr](https://github.com/tesseract-ocr/tesseract) | 开源 OCR 标准 |
| 格式转换 | Pandoc | [jgm/pandoc](https://github.com/jgm/pandoc) | 万能格式转换 |
| **位置映射参考** | - | [RAG-Knowledge-Base-Platform](https://github.com/loglux/RAG-Knowledge-Base-Platform) | Section-aware + 页码追踪 |
| **文档管线参考** | - | [Docify](https://github.com/keshavashiya/docify) | 11 步 RAG 管线 |

---

## 三、Flutter 前端

### 3.1 核心框架

| 组件 | 我们的选择 | 参考项目/SDK | 说明 |
|------|-----------|-------------|------|
| UI 框架 | Flutter 3.24+ | [flutter/flutter](https://github.com/flutter/flutter) | 跨平台 UI |
| 状态管理 | Riverpod 2.x | [riverpod](https://pub.dev/packages/flutter_riverpod) | 类型安全 |
| 路由 | go_router | [pub.dev/go_router](https://pub.dev/packages/go_router) | 官方推荐 |
| HTTP 客户端 | dio | [pub.dev/dio](https://pub.dev/packages/dio) | 拦截器、取消支持 |

### 3.2 AI 聊天 UI

| 组件 | 参考项目/SDK | Stars | 说明 |
|------|-------------|-------|------|
| **推荐: 聊天 UI** | [flyer.chat (flutter_chat_ui)](https://github.com/flyerhq/flutter_chat_ui) | 2.3k | 开源 Chat SDK，支持流式消息、Markdown、文件，Apache 2.0 |
| 替代: AI 聊天 UI | [flutter_gen_ai_chat_ui](https://pub.dev/packages/flutter_gen_ai_chat_ui) | - | ChatGPT 风格，word-by-word streaming，Markdown |
| 替代: AI 聊天视图 | [ai_chatview](https://github.com/Princewil/ai_chatview) | - | 轻量 AI 聊天组件 |

### 3.3 流式 Markdown 渲染

| 组件 | 参考项目/SDK | 说明 |
|------|-------------|------|
| **推荐: 流式 Markdown** | [streaming_markdown](https://pub.dev/packages/streaming_markdown) | ChatGPT/Claude/Grok 预设动画，自定义语法（引用卡片！），Stream<String> 支持 |
| 替代: 流式 Markdown | [flutter_markdown_stream](https://github.com/NarekManukyan/flutter_markdown_stream) | 零闪烁，crash-safe，sanitizer 处理半成品 Markdown |
| 替代: 流式文本+MD | [flutter_streaming_text_markdown](https://pub.dev/packages/flutter_streaming_text_markdown) | ChatGPT/Claude 风格，LaTeX 支持 |
| 基础 Markdown | [flutter_markdown](https://pub.dev/packages/flutter_markdown) | Flutter 官方 Markdown 渲染 |

### 3.4 文档查看

| 组件 | 我们的选择 | 参考项目/SDK | 说明 |
|------|-----------|-------------|------|
| PDF 渲染 | syncfusion_flutter_pdfviewer | [Syncfusion](https://pub.dev/packages/syncfusion_flutter_pdfviewer) | 页码跳转+文本高亮+搜索 |
| 本地缓存 | drift | [pub.dev/drift](https://pub.dev/packages/drift) | SQLite 封装 |

---

## 四、Vercel AI SDK 的角色（用户提到的参考）

| 项目 | 链接 | 与 TrustRAG 的关系 |
|------|------|-------------------|
| **Vercel AI SDK** | [vercel/ai](https://github.com/vercel/ai) (43k+ stars) | TypeScript AI 应用框架，定义了 Data Stream Protocol |
| **aisdk.rs** | [aisdk.rs](https://aisdk.rs/) | Rust 版 Vercel AI SDK，Axum 原生集成 |
| **rig-ai-sdk** | [rig-ai-sdk crate](https://crates.io/crates/rig-ai-sdk) | rig 框架到 Vercel AI SDK Protocol 的适配器 |

**Vercel AI SDK 的启发**：
- **Data Stream Protocol**：标准化 SSE 流式格式（文本、工具调用、结构化数据）
- **useChat / useCompletion**：前端 hook 模式（Flutter 版需自行实现或参考 aisdk.rs）
- **Tool Calling**：类型安全的工具调用+自动循环
- **Structured Output**：基于 schema 的结构化输出
- **但**：Vercel AI SDK 是 TypeScript 生态，TrustRAG 用 Rust + Flutter，所以我们参考其 **协议和架构理念**，用 `rig-ai-sdk` 或 `aisdk.rs` 在 Rust 端实现兼容

---

## 五、基础设施

| 组件 | 我们的选择 | 参考项目/SDK | 说明 |
|------|-----------|-------------|------|
| PostgreSQL 16 | pgvector + pg_bigm | [pgvector/pgvector](https://github.com/pgvector/pgvector) (14k+ stars) | 向量检索扩展 |
| Redis 7+ | 缓存+消息 | [redis/redis](https://github.com/redis/redis) | 标准 |
| MinIO | S3 对象存储 | [minio/minio](https://github.com/minio/minio) (49k+ stars) | 自托管 S3 |
| Caddy | 反向代理 | [caddyserver/caddy](https://github.com/caddyserver/caddy) (62k+ stars) | 自动 HTTPS |
| Docker Compose | 容器编排 | - | 一键部署 |
| GitHub Actions | CI/CD | - | 标准 |

---

## 六、推荐优先级排序

### 必须集成（核心依赖）
1. **rig-core** — 统一 LLM 多 Provider 接口 + Agent + Streaming
2. **pgvector crate** — 向量检索 Rust 绑定
3. **text-splitter** — 语义文本分块
4. **fastembed** — 本地 embedding（可选，也可纯走 API）
5. **flutter_chat_ui (flyer.chat)** — 聊天 UI 框架
6. **streaming_markdown** — 流式 Markdown 渲染（支持自定义引用卡片）

### 强烈推荐（提升效率）
7. **rig-ai-sdk** 或 **aisdk.rs** — Vercel AI SDK 协议兼容
8. **axum-jwt-auth** — JWT 认证中间件
9. **rig-postgres** — rig + pgvector 开箱即用
10. **chunkedrs** — token 精确分块

### 架构参考（不直接引入代码，参考设计）
11. **Docify** — 11 步 RAG 管线 + Citation Verification
12. **RAG-Knowledge-Base-Platform** — 位置映射 + 引用溯源
13. **R2R** — API 设计
14. **Vercel AI SDK** — 流式协议 + 前端交互模式
