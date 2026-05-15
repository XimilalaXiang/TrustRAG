# TrustRAG

**TrustRAG — Trustworthy Retrieval-Augmented Generation Knowledge Workbench**

TrustRAG is a self-hostable RAG knowledge workbench focused on **verifiable answers, precise citations, and document-grounded review workflows**. It combines a Rust/Axum backend, a Python/FastAPI document-processing sidecar, a Flutter client, PostgreSQL + pgvector search, Redis, MinIO object storage, and Docker Compose deployment.

> Status: active MVP. Core backend, document parsing, hybrid retrieval, RAG chat, citation persistence, review APIs, knowledge graph APIs, and the Flutter web client are present in this repository.

---

## Languages / 語言 / 言語

- [English](#english)
- [简体中文](#简体中文)
- [繁體中文](#繁體中文)
- [日本語](#日本語)

---

## English

### What TrustRAG does

TrustRAG helps teams build a private knowledge base, upload documents, ask questions, and inspect the evidence behind each generated answer. Its design goal is not only to chat with documents, but to make each answer **traceable back to original content**.

### Key features

- **Workspace-based knowledge management**: users, JWT authentication, workspaces, workspace members, and role-aware access.
- **Document ingestion**: upload PDF, DOCX, Markdown, TXT, and HTML files; store originals in MinIO/S3-compatible storage.
- **Document processing sidecar**: Python service parses PDF/DOCX/TXT into Markdown plus structured page/block metadata.
- **Semantic chunking**: Markdown-aware chunking with heading paths, character offsets, hashes, and page metadata.
- **Hybrid retrieval**: pgvector vector search + PostgreSQL trigram/full-text similarity + Reciprocal Rank Fusion (RRF).
- **RAG chat**: conversation APIs, streaming SSE responses, OpenAI-compatible LLM providers, and prompt grounding.
- **Citation system**: generated answers can persist verified citations linked to message, document, chunk, page, and heading metadata.
- **Review workflow foundation**: review records and APIs for approving, rejecting, flagging, or correcting citations.
- **Model configuration UI/API**: configure OpenAI-compatible, Ollama, Anthropic, or custom model endpoints.
- **Knowledge graph foundation**: entity and relationship APIs/tables for cross-document exploration.
- **Flutter client**: web-first client with login/register, dashboard, documents, search, chat, settings, citation UI, and document/PDF viewer pages.
- **Containerized deployment**: Docker Compose stack with backend, doc processor, PostgreSQL, Redis, MinIO, and Caddy.

### Architecture

```text
Flutter Client
  └── HTTP/SSE
      └── Rust Backend (Axum)
          ├── Auth, workspace, document, search, chat, citation, review APIs
          ├── RAG orchestration and provider registry
          ├── PostgreSQL + pgvector / pg_trgm
          ├── Redis
          ├── MinIO / S3-compatible object storage
          └── Python Document Processor (FastAPI)
              ├── PDF parsing with PyMuPDF
              ├── DOCX parsing with python-docx
              ├── OCR hooks with Tesseract
              └── conversion hooks with Pandoc
```

### Repository layout

```text
.
├── apps/client/          # Flutter web/desktop/mobile client
├── backend/              # Rust Axum backend service
├── doc-processor/        # Python FastAPI document-processing service
├── infra/                # Docker Compose, Caddy, PostgreSQL init scripts
├── docs/                 # Product, API, RAG pipeline, roadmap, and tech-stack docs
├── .env.example          # Environment variable template
└── LICENSE               # Apache-2.0 license
```

### Tech stack

| Layer | Main choices |
| --- | --- |
| Client | Flutter, Dart, Riverpod, go_router, dio, Syncfusion PDF Viewer |
| Backend | Rust, Axum, tokio, SQLx, JWT, argon2, tracing, utoipa, async-openai-compatible calls |
| Document processor | Python, FastAPI, PyMuPDF, pdfplumber, python-docx, pytesseract, Pillow, Pandoc |
| Data | PostgreSQL 16, pgvector, pg_trgm, Redis 7, MinIO |
| Deployment | Docker Compose, Caddy |

### Prerequisites

For the full stack:

- Docker and Docker Compose
- Flutter 3.24+ for building/running the client locally
- Rust toolchain for backend development
- Python 3.12+ for document-processor development

### Quick start with Docker Compose

1. Copy the environment template if you want local overrides:

   ```bash
   cp .env.example .env
   ```

2. Build the Flutter web client so Caddy can serve it:

   ```bash
   cd apps/client
   flutter pub get
   flutter build web --dart-define=API_BASE_URL=/api
   cd ../..
   ```

3. Start the full stack:

   ```bash
   cd infra
   docker compose up --build
   ```

4. Open the services:

   - Web app: <http://localhost>
   - Backend health: <http://localhost:8080/health>
   - Document processor health: <http://localhost:8081/health>
   - MinIO console: <http://localhost:9001>

### Development setup

#### Infrastructure only

Start PostgreSQL, Redis, and MinIO for local service development:

```bash
cd infra
docker compose -f docker-compose.dev.yml up -d
```

#### Backend

```bash
cd backend
cargo test
cargo run
```

The backend listens on `0.0.0.0:8080` by default and reads `TRUSTRAG__...` environment variables. It runs SQLx migrations from `backend/migrations` on startup.

#### Document processor

```bash
cd doc-processor
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
uvicorn app.main:app --host 0.0.0.0 --port 8081 --reload
pytest
```

#### Flutter client

```bash
cd apps/client
flutter pub get
flutter run -d chrome --dart-define=API_BASE_URL=http://localhost:8080
```

When served behind Caddy, build with `API_BASE_URL=/api` so requests are reverse-proxied to the backend.

### Important environment variables

| Variable | Purpose | Default/example |
| --- | --- | --- |
| `TRUSTRAG__LISTEN_ADDR` | Backend listen address | `0.0.0.0:8080` |
| `TRUSTRAG__DATABASE_URL` | PostgreSQL connection string | `postgres://trustrag:trustrag@localhost:5432/trustrag` |
| `TRUSTRAG__REDIS_URL` | Redis connection string | `redis://localhost:6379` |
| `TRUSTRAG__MINIO_ENDPOINT` | S3-compatible endpoint | `http://localhost:9000` |
| `TRUSTRAG__MINIO_ACCESS_KEY` | MinIO/S3 access key | `minioadmin` |
| `TRUSTRAG__MINIO_SECRET_KEY` | MinIO/S3 secret key | `minioadmin` |
| `TRUSTRAG__MINIO_BUCKET` | Object-storage bucket | `trustrag` |
| `TRUSTRAG__JWT_SECRET` | JWT signing secret | Change in production |
| `TRUSTRAG__DOC_PROCESSOR_URL` | Document processor base URL | `http://localhost:8081` |
| `TRUSTRAG__MAX_UPLOAD_SIZE_MB` | Upload limit | `100` |

### Core API surface

The backend currently exposes these main route groups:

- `GET /health`, `GET /metrics`
- Auth: `/auth/register`, `/auth/login`, `/auth/me`, `/auth/me/password`
- Workspaces: `/workspaces`, `/workspaces/{id}`
- Workspace members: `/workspaces/{ws_id}/members`
- Documents: `/workspaces/{ws_id}/documents`, download, Markdown, chunks, reprocess
- Search: `/workspaces/{ws_id}/search`
- Conversations/messages: `/workspaces/{ws_id}/conversations`, `/messages`
- Citations: `/messages/{message_id}/citations`
- Reviews: citation review endpoints
- Model configs: `/model-configs`, `/model-configs/{id}/test`, Ollama/Hugging Face discovery helpers
- Embedding configs: `/embedding-configs`, `/embedding-configs/{id}/test`
- Knowledge graph: `/workspaces/{ws_id}/knowledge-graph`

The document processor exposes:

- `GET /health`
- `POST /api/parse/pdf`
- `POST /api/parse/docx`
- `POST /api/parse/txt`
- `POST /api/convert/to-markdown`
- `POST /api/ocr/extract`

### Typical workflow

1. Register or log in.
2. Create a workspace.
3. Configure an embedding provider and an LLM provider.
4. Upload documents to the workspace.
5. Wait until processing status becomes `ready`.
6. Search documents or chat with the workspace.
7. Inspect citations, source pages, chunks, and review records.

### Testing

Useful commands:

```bash
cd backend && cargo test
cd doc-processor && pytest
cd apps/client && flutter test
cd infra && docker compose config
```

### License

TrustRAG is licensed under the Apache License 2.0. See [LICENSE](LICENSE).

---

## 简体中文

### TrustRAG 是什么

TrustRAG 是一个可自托管的 RAG 知识工作台，重点关注**可信回答、精确引用与可复核流程**。它不仅用于“和文档聊天”，更强调把 AI 回答追溯到原始文档、页码、章节和分块。

### 核心能力

- **工作区知识管理**：用户、JWT 认证、工作区、成员与角色权限。
- **文档上传与存储**：支持 PDF、DOCX、Markdown、TXT、HTML，原始文件存入 MinIO/S3 兼容对象存储。
- **独立文档处理服务**：Python/FastAPI 将 PDF/DOCX/TXT 解析为 Markdown，并保留页面、文本块、标题等结构化元数据。
- **语义分块**：基于 Markdown 的分块，保留 heading path、字符位置、内容哈希、页码等信息。
- **混合检索**：pgvector 向量检索 + PostgreSQL trigram/全文相似度 + RRF 融合排序。
- **RAG 对话**：对话 API、SSE 流式输出、OpenAI-compatible 模型调用与防幻觉提示。
- **引用系统**：回答中的引用可持久化，并关联到 message、document、chunk、page、heading 等元数据。
- **复核流程基础**：提供 citation review 记录与 API，可支持通过、拒绝、标记、纠正等状态。
- **模型配置**：支持配置 OpenAI-compatible、Ollama、Anthropic、自定义模型服务。
- **知识图谱基础**：提供实体与关系表/API，用于跨文档关系探索。
- **Flutter 客户端**：Web 优先，包含登录/注册、工作台、资料库、搜索、聊天、设置、引用 UI、文档/PDF 查看器。
- **容器化部署**：Docker Compose 一键启动 backend、doc-processor、PostgreSQL、Redis、MinIO、Caddy。

### 架构概览

```text
Flutter Client
  └── HTTP/SSE
      └── Rust Backend (Axum)
          ├── 认证、工作区、文档、搜索、聊天、引用、复核 API
          ├── RAG 编排与模型 Provider 管理
          ├── PostgreSQL + pgvector / pg_trgm
          ├── Redis
          ├── MinIO / S3 兼容对象存储
          └── Python Document Processor (FastAPI)
              ├── PyMuPDF 解析 PDF
              ├── python-docx 解析 DOCX
              ├── Tesseract OCR 能力
              └── Pandoc 格式转换能力
```

### 目录结构

```text
.
├── apps/client/          # Flutter Web/Desktop/Mobile 客户端
├── backend/              # Rust Axum 后端主服务
├── doc-processor/        # Python FastAPI 文档处理服务
├── infra/                # Docker Compose、Caddy、PostgreSQL 初始化脚本
├── docs/                 # 产品、API、RAG 管线、路线图、技术栈文档
├── .env.example          # 环境变量模板
└── LICENSE               # Apache-2.0 许可证
```

### 技术栈

| 层级 | 主要技术 |
| --- | --- |
| 客户端 | Flutter、Dart、Riverpod、go_router、dio、Syncfusion PDF Viewer |
| 后端 | Rust、Axum、tokio、SQLx、JWT、argon2、tracing、utoipa、OpenAI-compatible 调用 |
| 文档处理 | Python、FastAPI、PyMuPDF、pdfplumber、python-docx、pytesseract、Pillow、Pandoc |
| 数据层 | PostgreSQL 16、pgvector、pg_trgm、Redis 7、MinIO |
| 部署 | Docker Compose、Caddy |

### 环境要求

完整运行建议准备：

- Docker 与 Docker Compose
- Flutter 3.24+（本地构建/运行客户端）
- Rust toolchain（后端开发）
- Python 3.12+（文档处理服务开发）

### Docker Compose 快速启动

1. 如需本地覆盖配置，复制环境变量模板：

   ```bash
   cp .env.example .env
   ```

2. 构建 Flutter Web，供 Caddy 静态托管：

   ```bash
   cd apps/client
   flutter pub get
   flutter build web --dart-define=API_BASE_URL=/api
   cd ../..
   ```

3. 启动完整服务：

   ```bash
   cd infra
   docker compose up --build
   ```

4. 访问服务：

   - Web 应用：<http://localhost>
   - 后端健康检查：<http://localhost:8080/health>
   - 文档处理服务健康检查：<http://localhost:8081/health>
   - MinIO 控制台：<http://localhost:9001>

### 本地开发

#### 只启动基础设施

```bash
cd infra
docker compose -f docker-compose.dev.yml up -d
```

#### 后端

```bash
cd backend
cargo test
cargo run
```

后端默认监听 `0.0.0.0:8080`，读取 `TRUSTRAG__...` 环境变量，并在启动时执行 `backend/migrations` 中的 SQLx 迁移。

#### 文档处理服务

```bash
cd doc-processor
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
uvicorn app.main:app --host 0.0.0.0 --port 8081 --reload
pytest
```

#### Flutter 客户端

```bash
cd apps/client
flutter pub get
flutter run -d chrome --dart-define=API_BASE_URL=http://localhost:8080
```

通过 Caddy 部署时，请使用 `API_BASE_URL=/api`，让请求反向代理到后端。

### 关键环境变量

| 变量 | 用途 | 默认/示例 |
| --- | --- | --- |
| `TRUSTRAG__LISTEN_ADDR` | 后端监听地址 | `0.0.0.0:8080` |
| `TRUSTRAG__DATABASE_URL` | PostgreSQL 连接串 | `postgres://trustrag:trustrag@localhost:5432/trustrag` |
| `TRUSTRAG__REDIS_URL` | Redis 连接串 | `redis://localhost:6379` |
| `TRUSTRAG__MINIO_ENDPOINT` | S3 兼容服务地址 | `http://localhost:9000` |
| `TRUSTRAG__MINIO_ACCESS_KEY` | MinIO/S3 Access Key | `minioadmin` |
| `TRUSTRAG__MINIO_SECRET_KEY` | MinIO/S3 Secret Key | `minioadmin` |
| `TRUSTRAG__MINIO_BUCKET` | 对象存储 bucket | `trustrag` |
| `TRUSTRAG__JWT_SECRET` | JWT 签名密钥 | 生产环境必须更换 |
| `TRUSTRAG__DOC_PROCESSOR_URL` | 文档处理服务地址 | `http://localhost:8081` |
| `TRUSTRAG__MAX_UPLOAD_SIZE_MB` | 上传大小限制 | `100` |

### 主要 API

后端主要路由组：

- `GET /health`、`GET /metrics`
- 认证：`/auth/register`、`/auth/login`、`/auth/me`、`/auth/me/password`
- 工作区：`/workspaces`、`/workspaces/{id}`
- 成员：`/workspaces/{ws_id}/members`
- 文档：`/workspaces/{ws_id}/documents`、下载、Markdown、chunks、reprocess
- 搜索：`/workspaces/{ws_id}/search`
- 对话/消息：`/workspaces/{ws_id}/conversations`、`/messages`
- 引用：`/messages/{message_id}/citations`
- 复核：citation review 相关接口
- 模型配置：`/model-configs`、`/model-configs/{id}/test`、Ollama/Hugging Face 发现辅助接口
- Embedding 配置：`/embedding-configs`、`/embedding-configs/{id}/test`
- 知识图谱：`/workspaces/{ws_id}/knowledge-graph`

文档处理服务主要接口：

- `GET /health`
- `POST /api/parse/pdf`
- `POST /api/parse/docx`
- `POST /api/parse/txt`
- `POST /api/convert/to-markdown`
- `POST /api/ocr/extract`

### 典型使用流程

1. 注册或登录。
2. 创建工作区。
3. 配置 embedding provider 与 LLM provider。
4. 上传文档。
5. 等待处理状态变为 `ready`。
6. 在工作区中搜索或对话。
7. 查看引用、原文页码、分块与复核记录。

### 测试

常用命令：

```bash
cd backend && cargo test
cd doc-processor && pytest
cd apps/client && flutter test
cd infra && docker compose config
```

### 许可证

TrustRAG 使用 Apache License 2.0。详见 [LICENSE](LICENSE)。

---

## 繁體中文

### TrustRAG 是什麼

TrustRAG 是一個可自託管的 RAG 知識工作台，重點在於**可信回答、精確引用與可複核流程**。它不只是用來「和文件聊天」，更強調將 AI 回答追溯到原始文件、頁碼、章節與分塊。

### 核心能力

- **工作區知識管理**：使用者、JWT 認證、工作區、成員與角色權限。
- **文件上傳與儲存**：支援 PDF、DOCX、Markdown、TXT、HTML，原始檔案存入 MinIO/S3 相容物件儲存。
- **獨立文件處理服務**：Python/FastAPI 將 PDF/DOCX/TXT 解析為 Markdown，並保留頁面、文字區塊、標題等結構化中繼資料。
- **語意分塊**：基於 Markdown 的分塊，保留 heading path、字元位置、內容雜湊、頁碼等資訊。
- **混合檢索**：pgvector 向量檢索 + PostgreSQL trigram/全文相似度 + RRF 融合排序。
- **RAG 對話**：對話 API、SSE 串流輸出、OpenAI-compatible 模型呼叫與防幻覺提示。
- **引用系統**：回答中的引用可持久化，並關聯到 message、document、chunk、page、heading 等中繼資料。
- **複核流程基礎**：提供 citation review 記錄與 API，可支援通過、拒絕、標記、修正等狀態。
- **模型設定**：支援設定 OpenAI-compatible、Ollama、Anthropic、自訂模型服務。
- **知識圖譜基礎**：提供實體與關係表/API，用於跨文件關係探索。
- **Flutter 用戶端**：Web 優先，包含登入/註冊、工作台、資料庫、搜尋、聊天、設定、引用 UI、文件/PDF 檢視器。
- **容器化部署**：Docker Compose 一鍵啟動 backend、doc-processor、PostgreSQL、Redis、MinIO、Caddy。

### 架構概覽

```text
Flutter Client
  └── HTTP/SSE
      └── Rust Backend (Axum)
          ├── 認證、工作區、文件、搜尋、聊天、引用、複核 API
          ├── RAG 編排與模型 Provider 管理
          ├── PostgreSQL + pgvector / pg_trgm
          ├── Redis
          ├── MinIO / S3 相容物件儲存
          └── Python Document Processor (FastAPI)
              ├── PyMuPDF 解析 PDF
              ├── python-docx 解析 DOCX
              ├── Tesseract OCR 能力
              └── Pandoc 格式轉換能力
```

### 目錄結構

```text
.
├── apps/client/          # Flutter Web/Desktop/Mobile 用戶端
├── backend/              # Rust Axum 後端主服務
├── doc-processor/        # Python FastAPI 文件處理服務
├── infra/                # Docker Compose、Caddy、PostgreSQL 初始化腳本
├── docs/                 # 產品、API、RAG 管線、路線圖、技術棧文件
├── .env.example          # 環境變數範本
└── LICENSE               # Apache-2.0 授權
```

### 技術棧

| 層級 | 主要技術 |
| --- | --- |
| 用戶端 | Flutter、Dart、Riverpod、go_router、dio、Syncfusion PDF Viewer |
| 後端 | Rust、Axum、tokio、SQLx、JWT、argon2、tracing、utoipa、OpenAI-compatible 呼叫 |
| 文件處理 | Python、FastAPI、PyMuPDF、pdfplumber、python-docx、pytesseract、Pillow、Pandoc |
| 資料層 | PostgreSQL 16、pgvector、pg_trgm、Redis 7、MinIO |
| 部署 | Docker Compose、Caddy |

### 環境需求

完整運行建議準備：

- Docker 與 Docker Compose
- Flutter 3.24+（本機建置/執行用戶端）
- Rust toolchain（後端開發）
- Python 3.12+（文件處理服務開發）

### Docker Compose 快速啟動

1. 如需本機覆寫設定，複製環境變數範本：

   ```bash
   cp .env.example .env
   ```

2. 建置 Flutter Web，供 Caddy 靜態託管：

   ```bash
   cd apps/client
   flutter pub get
   flutter build web --dart-define=API_BASE_URL=/api
   cd ../..
   ```

3. 啟動完整服務：

   ```bash
   cd infra
   docker compose up --build
   ```

4. 存取服務：

   - Web 應用：<http://localhost>
   - 後端健康檢查：<http://localhost:8080/health>
   - 文件處理服務健康檢查：<http://localhost:8081/health>
   - MinIO 控制台：<http://localhost:9001>

### 本機開發

#### 只啟動基礎設施

```bash
cd infra
docker compose -f docker-compose.dev.yml up -d
```

#### 後端

```bash
cd backend
cargo test
cargo run
```

後端預設監聽 `0.0.0.0:8080`，讀取 `TRUSTRAG__...` 環境變數，並在啟動時執行 `backend/migrations` 中的 SQLx 遷移。

#### 文件處理服務

```bash
cd doc-processor
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
uvicorn app.main:app --host 0.0.0.0 --port 8081 --reload
pytest
```

#### Flutter 用戶端

```bash
cd apps/client
flutter pub get
flutter run -d chrome --dart-define=API_BASE_URL=http://localhost:8080
```

透過 Caddy 部署時，請使用 `API_BASE_URL=/api`，讓請求反向代理到後端。

### 關鍵環境變數

| 變數 | 用途 | 預設/範例 |
| --- | --- | --- |
| `TRUSTRAG__LISTEN_ADDR` | 後端監聽位址 | `0.0.0.0:8080` |
| `TRUSTRAG__DATABASE_URL` | PostgreSQL 連線字串 | `postgres://trustrag:trustrag@localhost:5432/trustrag` |
| `TRUSTRAG__REDIS_URL` | Redis 連線字串 | `redis://localhost:6379` |
| `TRUSTRAG__MINIO_ENDPOINT` | S3 相容服務位址 | `http://localhost:9000` |
| `TRUSTRAG__MINIO_ACCESS_KEY` | MinIO/S3 Access Key | `minioadmin` |
| `TRUSTRAG__MINIO_SECRET_KEY` | MinIO/S3 Secret Key | `minioadmin` |
| `TRUSTRAG__MINIO_BUCKET` | 物件儲存 bucket | `trustrag` |
| `TRUSTRAG__JWT_SECRET` | JWT 簽章密鑰 | 生產環境必須更換 |
| `TRUSTRAG__DOC_PROCESSOR_URL` | 文件處理服務位址 | `http://localhost:8081` |
| `TRUSTRAG__MAX_UPLOAD_SIZE_MB` | 上傳大小限制 | `100` |

### 主要 API

後端主要路由組：

- `GET /health`、`GET /metrics`
- 認證：`/auth/register`、`/auth/login`、`/auth/me`、`/auth/me/password`
- 工作區：`/workspaces`、`/workspaces/{id}`
- 成員：`/workspaces/{ws_id}/members`
- 文件：`/workspaces/{ws_id}/documents`、下載、Markdown、chunks、reprocess
- 搜尋：`/workspaces/{ws_id}/search`
- 對話/訊息：`/workspaces/{ws_id}/conversations`、`/messages`
- 引用：`/messages/{message_id}/citations`
- 複核：citation review 相關介面
- 模型設定：`/model-configs`、`/model-configs/{id}/test`、Ollama/Hugging Face 探索輔助介面
- Embedding 設定：`/embedding-configs`、`/embedding-configs/{id}/test`
- 知識圖譜：`/workspaces/{ws_id}/knowledge-graph`

文件處理服務主要介面：

- `GET /health`
- `POST /api/parse/pdf`
- `POST /api/parse/docx`
- `POST /api/parse/txt`
- `POST /api/convert/to-markdown`
- `POST /api/ocr/extract`

### 典型使用流程

1. 註冊或登入。
2. 建立工作區。
3. 設定 embedding provider 與 LLM provider。
4. 上傳文件。
5. 等待處理狀態變為 `ready`。
6. 在工作區中搜尋或對話。
7. 查看引用、原文頁碼、分塊與複核記錄。

### 測試

常用命令：

```bash
cd backend && cargo test
cd doc-processor && pytest
cd apps/client && flutter test
cd infra && docker compose config
```

### 授權

TrustRAG 使用 Apache License 2.0。詳見 [LICENSE](LICENSE)。

---

## 日本語

### TrustRAG とは

TrustRAG はセルフホスト可能な RAG ナレッジワークベンチです。主な目的は、**信頼できる回答、正確な引用、レビュー可能な根拠管理**を提供することです。単に「ドキュメントと会話する」だけではなく、AI の回答を元ドキュメント、ページ、見出し、チャンクへ追跡できるように設計されています。

### 主な機能

- **ワークスペース型ナレッジ管理**：ユーザー、JWT 認証、ワークスペース、メンバー、ロールベースのアクセス管理。
- **ドキュメント取り込み**：PDF、DOCX、Markdown、TXT、HTML をアップロードし、原本を MinIO/S3 互換ストレージに保存。
- **ドキュメント処理サイドカー**：Python/FastAPI サービスが PDF/DOCX/TXT を Markdown とページ・ブロック・見出しメタデータへ変換。
- **セマンティックチャンク化**：Markdown を意識した分割、heading path、文字オフセット、ハッシュ、ページ情報を保持。
- **ハイブリッド検索**：pgvector ベクトル検索 + PostgreSQL trigram/全文類似度 + RRF 融合ランキング。
- **RAG チャット**：会話 API、SSE ストリーミング、OpenAI-compatible LLM provider、根拠付きプロンプト。
- **引用システム**：回答中の引用を message、document、chunk、page、heading メタデータへ永続的に関連付け。
- **レビュー基盤**：引用の承認、拒否、フラグ、修正に使える review record と API。
- **モデル設定**：OpenAI-compatible、Ollama、Anthropic、カスタムモデルサービスを設定可能。
- **ナレッジグラフ基盤**：ドキュメント横断の探索に向けた entity/relation テーブルと API。
- **Flutter クライアント**：Web 優先。ログイン/登録、ダッシュボード、ドキュメント、検索、チャット、設定、引用 UI、ドキュメント/PDF ビューアを搭載。
- **コンテナ化デプロイ**：Docker Compose で backend、doc-processor、PostgreSQL、Redis、MinIO、Caddy を起動。

### アーキテクチャ

```text
Flutter Client
  └── HTTP/SSE
      └── Rust Backend (Axum)
          ├── 認証、ワークスペース、ドキュメント、検索、チャット、引用、レビュー API
          ├── RAG オーケストレーションと Provider 管理
          ├── PostgreSQL + pgvector / pg_trgm
          ├── Redis
          ├── MinIO / S3 互換オブジェクトストレージ
          └── Python Document Processor (FastAPI)
              ├── PyMuPDF による PDF 解析
              ├── python-docx による DOCX 解析
              ├── Tesseract OCR フック
              └── Pandoc 変換フック
```

### リポジトリ構成

```text
.
├── apps/client/          # Flutter Web/Desktop/Mobile クライアント
├── backend/              # Rust Axum バックエンドサービス
├── doc-processor/        # Python FastAPI ドキュメント処理サービス
├── infra/                # Docker Compose、Caddy、PostgreSQL 初期化スクリプト
├── docs/                 # プロダクト、API、RAG パイプライン、ロードマップ、技術スタック文書
├── .env.example          # 環境変数テンプレート
└── LICENSE               # Apache-2.0 ライセンス
```

### 技術スタック

| レイヤー | 主な技術 |
| --- | --- |
| クライアント | Flutter、Dart、Riverpod、go_router、dio、Syncfusion PDF Viewer |
| バックエンド | Rust、Axum、tokio、SQLx、JWT、argon2、tracing、utoipa、OpenAI-compatible 呼び出し |
| ドキュメント処理 | Python、FastAPI、PyMuPDF、pdfplumber、python-docx、pytesseract、Pillow、Pandoc |
| データ | PostgreSQL 16、pgvector、pg_trgm、Redis 7、MinIO |
| デプロイ | Docker Compose、Caddy |

### 前提条件

フルスタックで動かす場合は、以下を用意してください。

- Docker と Docker Compose
- Flutter 3.24+（クライアントのローカルビルド/実行用）
- Rust toolchain（バックエンド開発用）
- Python 3.12+（ドキュメント処理サービス開発用）

### Docker Compose クイックスタート

1. ローカル設定を上書きしたい場合は、環境変数テンプレートをコピーします。

   ```bash
   cp .env.example .env
   ```

2. Caddy で配信する Flutter Web をビルドします。

   ```bash
   cd apps/client
   flutter pub get
   flutter build web --dart-define=API_BASE_URL=/api
   cd ../..
   ```

3. フルスタックを起動します。

   ```bash
   cd infra
   docker compose up --build
   ```

4. サービスへアクセスします。

   - Web アプリ：<http://localhost>
   - バックエンド health：<http://localhost:8080/health>
   - ドキュメント処理 service health：<http://localhost:8081/health>
   - MinIO コンソール：<http://localhost:9001>

### 開発セットアップ

#### インフラのみ起動

```bash
cd infra
docker compose -f docker-compose.dev.yml up -d
```

#### バックエンド

```bash
cd backend
cargo test
cargo run
```

バックエンドはデフォルトで `0.0.0.0:8080` を listen し、`TRUSTRAG__...` 環境変数を読み込みます。起動時に `backend/migrations` の SQLx マイグレーションを実行します。

#### ドキュメント処理サービス

```bash
cd doc-processor
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
uvicorn app.main:app --host 0.0.0.0 --port 8081 --reload
pytest
```

#### Flutter クライアント

```bash
cd apps/client
flutter pub get
flutter run -d chrome --dart-define=API_BASE_URL=http://localhost:8080
```

Caddy 配下で配信する場合は、`API_BASE_URL=/api` を指定してバックエンドへリバースプロキシしてください。

### 重要な環境変数

| 変数 | 用途 | デフォルト/例 |
| --- | --- | --- |
| `TRUSTRAG__LISTEN_ADDR` | バックエンド listen アドレス | `0.0.0.0:8080` |
| `TRUSTRAG__DATABASE_URL` | PostgreSQL 接続文字列 | `postgres://trustrag:trustrag@localhost:5432/trustrag` |
| `TRUSTRAG__REDIS_URL` | Redis 接続文字列 | `redis://localhost:6379` |
| `TRUSTRAG__MINIO_ENDPOINT` | S3 互換エンドポイント | `http://localhost:9000` |
| `TRUSTRAG__MINIO_ACCESS_KEY` | MinIO/S3 Access Key | `minioadmin` |
| `TRUSTRAG__MINIO_SECRET_KEY` | MinIO/S3 Secret Key | `minioadmin` |
| `TRUSTRAG__MINIO_BUCKET` | オブジェクトストレージ bucket | `trustrag` |
| `TRUSTRAG__JWT_SECRET` | JWT 署名シークレット | 本番環境では必ず変更 |
| `TRUSTRAG__DOC_PROCESSOR_URL` | ドキュメント処理サービス URL | `http://localhost:8081` |
| `TRUSTRAG__MAX_UPLOAD_SIZE_MB` | アップロード上限 | `100` |

### 主要 API

バックエンドの主なルートグループ：

- `GET /health`、`GET /metrics`
- 認証：`/auth/register`、`/auth/login`、`/auth/me`、`/auth/me/password`
- ワークスペース：`/workspaces`、`/workspaces/{id}`
- メンバー：`/workspaces/{ws_id}/members`
- ドキュメント：`/workspaces/{ws_id}/documents`、download、Markdown、chunks、reprocess
- 検索：`/workspaces/{ws_id}/search`
- 会話/メッセージ：`/workspaces/{ws_id}/conversations`、`/messages`
- 引用：`/messages/{message_id}/citations`
- レビュー：citation review 関連エンドポイント
- モデル設定：`/model-configs`、`/model-configs/{id}/test`、Ollama/Hugging Face discovery helper
- Embedding 設定：`/embedding-configs`、`/embedding-configs/{id}/test`
- ナレッジグラフ：`/workspaces/{ws_id}/knowledge-graph`

ドキュメント処理サービスの主なエンドポイント：

- `GET /health`
- `POST /api/parse/pdf`
- `POST /api/parse/docx`
- `POST /api/parse/txt`
- `POST /api/convert/to-markdown`
- `POST /api/ocr/extract`

### 典型的な利用フロー

1. 登録またはログインします。
2. ワークスペースを作成します。
3. Embedding provider と LLM provider を設定します。
4. ドキュメントをアップロードします。
5. 処理状態が `ready` になるまで待ちます。
6. ワークスペース内で検索またはチャットします。
7. 引用、元ページ、チャンク、レビュー記録を確認します。

### テスト

よく使うコマンド：

```bash
cd backend && cargo test
cd doc-processor && pytest
cd apps/client && flutter test
cd infra && docker compose config
```

### ライセンス

TrustRAG は Apache License 2.0 で提供されています。詳しくは [LICENSE](LICENSE) を参照してください。
