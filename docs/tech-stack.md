# TrustRAG 技术选型终稿

> 确认日期：2026-05-14
> 状态：已锁定，后续不变更

## 项目基本信息

- **项目名**：TrustRAG
- **英文全称**：Trustworthy Retrieval-Augmented Generation Knowledge Workbench
- **中文名**：可信 AI 知识工作台
- **许可证**：Apache 2.0
- **仓库名**：trustrag

## 客户端层

| 项目 | 选择 | 说明 |
|------|------|------|
| 框架 | Flutter 3.24+ | 一套代码覆盖 Web/Desktop/Mobile |
| 语言 | Dart 3.5+ | Flutter 绑定 |
| 状态管理 | Riverpod 2.x | 类型安全，可测试性好 |
| 路由 | go_router | Flutter 官方推荐 |
| HTTP 客户端 | dio | 拦截器、取消请求支持好 |
| PDF 渲染 | syncfusion_flutter_pdfviewer | 页码跳转+高亮 |
| Markdown 渲染 | flutter_markdown | 标准选择 |
| 本地缓存 | drift (SQLite) | 客户端离线缓存 |

### 平台优先级

1. **Web 版**（MVP 阶段首发）
2. **Windows + Linux 桌面端**
3. **Android 移动端**

## 后端主服务（Rust）

| 项目 | 选择 | 说明 |
|------|------|------|
| Web 框架 | Axum 0.8+ | tokio 官方，tower 中间件 |
| 异步运行时 | tokio | Rust 异步标准 |
| ORM/数据库 | SQLx | 编译期 SQL 校验 |
| 认证 | jsonwebtoken + argon2 | JWT + 密码哈希 |
| 序列化 | serde + serde_json | Rust 标准 |
| 日志 | tracing + tracing-subscriber | 结构化日志 |
| 配置 | config-rs | 多源配置合并 |
| 对象存储 | opendal | 支持 S3/MinIO/本地 FS 多后端 |
| Redis 客户端 | redis-rs (deadpool 连接池) | 异步 Redis |
| 任务队列 | apalis | Rust 原生异步任务框架 |
| LLM 调用 | async-openai | OpenAI-compatible API |
| 流式输出 | Axum SSE | 内置支持 |
| 数据库迁移 | sqlx-cli | 标准工具 |
| API 文档 | utoipa (OpenAPI) | 自动生成 Swagger |
| 错误处理 | thiserror + anyhow | 库错误 + 应用错误 |

## 文档处理服务（Python）

| 项目 | 选择 | 说明 |
|------|------|------|
| Web 框架 | FastAPI | 异步，自动 OpenAPI 文档 |
| PDF 解析 | PyMuPDF (fitz) | 速度快，功能全 |
| PDF 表格 | pdfplumber | 坐标级精确提取 |
| DOCX 解析 | python-docx | 标准选择 |
| OCR | pytesseract + Pillow | 开源 OCR |
| 格式转换 | Pandoc (CLI) | 万能格式转换 |
| Markdown 处理 | markdown-it-py | 解析+渲染 |
| 位置映射 | 自研 (基于 PyMuPDF 坐标) | 核心差异化功能 |
| 运行方式 | 独立 Docker 容器 | 与 Rust 后端解耦 |
| 通信协议 | HTTP REST (JSON) | Rust 后端调用 |

## 数据层

| 项目 | 选择 | 说明 |
|------|------|------|
| 关系数据库 | PostgreSQL 16+ | 主数据库 |
| 向量扩展 | pgvector | 集成在 PostgreSQL 中 |
| 中文全文检索 | pg_bigm | 2-gram 中文分词 |
| 对象存储 | MinIO | S3-compatible，自托管 |
| 缓存 | Redis 7+ | 标准选择 |

## 基础设施

| 项目 | 选择 | 说明 |
|------|------|------|
| 容器编排 | Docker Compose | 一键部署 |
| 反向代理 | Caddy | 自动 HTTPS |
| CI/CD | GitHub Actions | 标准选择 |

### Docker Compose 容器清单

1. `trustrag-backend` — Rust 后端主服务
2. `trustrag-doc-processor` — Python 文档处理服务
3. `postgres` — PostgreSQL 16 + pgvector + pg_bigm
4. `redis` — Redis 7+
5. `minio` — MinIO 对象存储
6. `caddy` — 反向代理 + 自动 HTTPS

## 项目结构

```
trustrag/
  apps/
    client/                  -- Flutter 客户端（Web/Desktop/Mobile）
      lib/
        core/
        features/
        shared/
      android/
      linux/
      windows/
      web/
      pubspec.yaml

  backend/                   -- Rust 后端主服务
    src/
      main.rs
      config.rs
      error.rs
      auth/
      api/
      db/
      services/
      workers/
      traits/
    migrations/
    Cargo.toml

  doc-processor/             -- Python 文档处理服务
    app/
      main.py
      routers/
      processors/
      models/
      utils/
    requirements.txt
    Dockerfile

  infra/
    docker-compose.yml
    docker-compose.dev.yml
    postgres/
    redis/
    minio/
    caddy/

  docs/
  design/
  scripts/
  .github/
    workflows/
      ci.yml
      release.yml

  LICENSE                    -- Apache 2.0
  README.md
```

## 核心架构决策

1. **后端语言**：Rust（性能 + 类型安全）
2. **文档解析**：Python 微服务（生态优势）
3. **前端统一**：Flutter 一套代码，Web 先行
4. **模型调用**：统一走 OpenAI-compatible API，不内置模型运行
5. **数据三合一**：PostgreSQL 同时承担关系数据、向量检索、全文检索
6. **部署方式**：Docker Compose 一键部署
