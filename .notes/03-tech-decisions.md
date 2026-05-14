# 技术选型决策记录（2026-05-14，已锁定）

详细选型见 `docs/tech-stack.md`

## 12 项决策及理由

### 1. 项目名：TrustRAG
- 简短好记，直接点明核心技术（RAG + 可信）
- 备选项：TAIKW（难记）、VeritasKB（拉丁语"真理知识库"，生僻）

### 2. 前端：Flutter 3.24+（一套代码）
- Web 先行 → Desktop → Android
- Flutter Web 2026 年已成熟，CanvasKit 渲染引擎性能好
- 工具型应用不需要 SEO，Flutter Web 够用
- 不需要维护两套前端代码

### 3. 状态管理：Riverpod 2.x
- 类型安全、可测试性好、社区活跃

### 4. 后端：Rust Axum 0.8+
- tokio 官方出品，tower 中间件生态
- 性能比 Python 快 10-50x
- 类型安全，编译期捕获错误

### 5. 数据库：PostgreSQL 16 + pgvector + pg_bigm
- 三合一方案：关系数据 + 向量检索 + 中文全文检索
- 减少组件数量，降低运维复杂度

### 6. 对象存储：MinIO
- S3 兼容，后续可无缝切换云存储

### 7. 缓存 + 任务队列：Redis 7+ + apalis
- apalis 是 Rust 原生异步任务框架

### 8. 文档处理：Python FastAPI 微服务
- Python 文档解析生态远强于 Rust
- 独立 Docker 容器，HTTP REST 通信
- 工具链：PyMuPDF + pdfplumber + python-docx + pytesseract + Pandoc

### 9. LLM 调用：async-openai
- 统一走 OpenAI-compatible API
- 不内置模型运行，用户自己跑 Ollama/vLLM
- 大幅降低复杂度

### 10. 部署：Docker Compose（6 容器）
- backend、doc-processor、postgres、redis、minio、caddy

### 11. 平台优先级：Web → Desktop → Android
- Web 版 MVP 先验证核心功能

### 12. 许可证：Apache 2.0
- 宽松 + 专利保护，鼓励企业采用

## 混合架构决策

后端用 Rust 拿性能和安全优势，文档解析独立成 Python 微服务拿生态优势。
两者通过 HTTP 解耦，可以独立部署和扩缩。
