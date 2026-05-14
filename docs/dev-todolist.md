# TrustRAG 开发 TODO List

> 本文档是开发的唯一路线图
> 规则：完成一个阶段后，自动查阅相关设计文档，进入下一阶段
> 每个任务标注：参考文档、涉及文件、验收标准

---

## Phase 1: MVP（4-6 周）

### Sprint 1: 后端基础 + 数据库 + 认证（第 1-2 周）

#### ✅ 1.0 项目初始化
- [x] Git 初始化 + 首次提交
- [x] .env.example 模板
- [x] 项目骨架代码

#### ✅ 1.1 数据库 Migration
- [x] `backend/migrations/0001_init_schema.sql`
- 参考：`docs/rag-pipeline.md` (数据模型)
- 验收：11 张表 + pgvector/pg_bigm 索引 + 触发器

#### ✅ 1.2 用户注册/登录 API
- [x] `backend/src/api/users.rs`
- [x] `backend/src/auth/jwt.rs`
- 参考：`docs/api-design.md` → §1 认证
- 涉及文件：`api/users.rs`, `auth/jwt.rs`
- 验收：POST /auth/register + /auth/login + /auth/me

#### ✅ 1.3 RBAC 中间件
- [x] `backend/src/auth/middleware.rs`
- 参考：`docs/api-design.md` → 认证规则
- 涉及文件：`auth/middleware.rs`
- 验收：AuthUser 提取器，admin/reviewer/user 三级角色

#### ✅ 1.4 工作区 CRUD API
- [x] `backend/src/api/workspaces.rs`
- 参考：`docs/api-design.md` → §2 工作区
- 验收：GET/POST/PUT/DELETE + 权限控制

#### ✅ 1.5 MinIO + 文件上传 API
- [x] 初始化 opendal S3 客户端
- [x] 文件上传接口 (multipart/form-data)
- [x] 文件下载接口
- 参考：`docs/api-design.md` → §3 文档管理 (upload/download)
- 参考：`docs/tech-stack.md` → opendal
- 涉及文件：新建 `backend/src/services/storage.rs`
- 验收：上传文件到 MinIO，返回文件路径；下载文件

#### ✅ 1.6 文档元数据 CRUD API
- [x] 文档列表（分页 + 筛选）
- [x] 文档详情
- [x] 删除文档（级联删除分块）
- [x] 重新处理文档接口
- [x] 分块列表接口
- [x] Markdown 版本获取接口
- 参考：`docs/api-design.md` → §3 文档管理
- 涉及文件：`backend/src/api/documents.rs`
- 验收：GET/POST/DELETE /workspaces/:ws_id/documents + /chunks + /markdown

#### 🔲 1.7 Docker Compose 开发环境联调
- [ ] 验证 postgres + pgvector + pg_bigm 启动
- [ ] 验证 redis 连接
- [ ] 验证 minio 启动 + bucket 创建
- [ ] 验证 caddy 反向代理
- [ ] Rust 后端连接所有服务
- [ ] 编写 docker-compose.dev.yml（热重载）
- 参考：`docs/tech-stack.md` → Docker Compose 容器清单
- 涉及文件：`infra/docker-compose.yml`, `infra/docker-compose.dev.yml`
- 验收：`docker compose up` 一键启动所有服务

---

### Sprint 2: 文档处理 + 检索（第 2-3 周）

#### 🔲 2.1 Python PDF 解析 + Markdown 转换
- [ ] PyMuPDF 文本+坐标提取
- [ ] 标题结构提取（heading_path, section_level）
- [ ] 页码映射（物理页码 + 逻辑页码）
- [ ] Markdown 输出
- 参考：`docs/rag-pipeline.md` → 文档摄入管线 §3-4
- 参考：`.notes/07-component-sdk-map.md` → 文档处理层
- 涉及文件：`doc-processor/app/processors/pdf.py`
- 验收：上传 PDF → 返回 JSON（markdown + pages + headings + metadata）

#### 🔲 2.2 Python DOCX 解析
- [ ] python-docx 解析
- [ ] 标题/段落/表格提取
- [ ] Markdown 输出
- 参考：同 2.1
- 涉及文件：`doc-processor/app/processors/docx.py`
- 验收：上传 DOCX → 返回结构化 JSON

#### 🔲 2.3 位置映射模块
- [ ] 每段文字 → 原文页码 + 坐标 + 段落索引
- [ ] heading_path 生成（如"第3章 > 3.2 认证"）
- [ ] 存储映射关系
- 参考：`docs/rag-pipeline.md` → 文档摄入管线 §4
- 参考：`.notes/06-reusable-sdks.md` → RAG-Knowledge-Base-Platform
- 涉及文件：`doc-processor/app/processors/position_mapper.py`
- 验收：给定 chunk → 返回精确的页码 + 位置

#### 🔲 2.4 文本分块模块 (Rust)
- [ ] 集成 text-splitter 或 chunkedrs crate
- [ ] Markdown-aware 分块（保留标题上下文）
- [ ] 目标 500-1000 tokens/chunk，50-100 tokens 重叠
- [ ] content_hash 去重
- 参考：`docs/rag-pipeline.md` → 文档摄入管线 §6
- 参考：`.notes/07-component-sdk-map.md` → 文本分块
- 涉及文件：新建 `backend/src/services/chunking.rs`
- 验收：输入 Markdown 全文 → 输出分块列表（含 heading_path, page_start/end）

#### 🔲 2.5 Embedding + pgvector 存储
- [ ] 集成 async-openai embedding API 或 fastembed
- [ ] 批量生成 embedding
- [ ] 写入 document_chunks.embedding 列
- [ ] HNSW 索引验证
- 参考：`docs/rag-pipeline.md` → 文档摄入管线 §7-8
- 参考：`.notes/07-component-sdk-map.md` → Embedding 生成
- 涉及文件：`backend/src/services/embedding.rs`
- 验收：分块后自动生成 embedding 并存入 PostgreSQL

#### 🔲 2.6 全文检索索引 (pg_bigm)
- [ ] 验证 pg_bigm 索引效果
- [ ] 中文查询测试
- [ ] similarity 函数调用
- 参考：`docs/rag-pipeline.md` → 阶段 3 Full-Text Search
- 涉及文件：SQL + `backend/src/services/search.rs`
- 验收：中文关键词 → 返回相关分块 + 相关度分数

#### 🔲 2.7 混合检索 API
- [ ] Vector search 实现
- [ ] Full-text search 实现
- [ ] RRF 分数融合
- [ ] MMR 多样性检索（可选）
- [ ] POST /workspaces/:ws_id/search 接口
- 参考：`docs/rag-pipeline.md` → 阶段 3 Hybrid Retrieval
- 参考：`docs/api-design.md` → §4 检索
- 涉及文件：`backend/src/services/search.rs`, `backend/src/api/search.rs`
- 验收：混合检索返回排序后的分块列表 + 元数据

#### 🔲 2.8 文档处理编排（异步任务）
- [ ] Rust 后端调用 Python doc-processor HTTP API
- [ ] 异步任务队列（apalis 或简单 tokio::spawn）
- [ ] processing_status 状态机管理
- [ ] 错误处理 + 重试
- 参考：`docs/rag-pipeline.md` → 文档摄入管线全流程
- 涉及文件：`backend/src/services/document.rs`, `backend/src/workers/`
- 验收：上传文档 → 自动触发处理 → 状态更新 → 分块+索引完成

---

### Sprint 3: RAG 问答 + 引用（第 3-4 周）

#### 🔲 3.1 模型配置 CRUD API
- [ ] 模型配置表 CRUD
- [ ] 连接测试接口
- [ ] API Key 加密存储
- 参考：`docs/api-design.md` → §6 模型配置
- 涉及文件：`backend/src/api/models.rs`
- 验收：创建/更新/删除模型配置 + 测试连接

#### 🔲 3.2 LLM Provider Trait + 实现
- [ ] 定义统一的 LLM 调用 trait
- [ ] OpenAI-compatible 实现（async-openai）
- [ ] 流式输出支持
- 参考：`docs/rag-pipeline.md` → 阶段 7 LLM Generation
- 参考：`.notes/07-component-sdk-map.md` → LLM 调用
- 涉及文件：`backend/src/services/llm.rs`
- 验收：调用 LLM → 获取流式/非流式回答

#### 🔲 3.3 RAG 管线核心实现
- [ ] Query Analysis（查询分析 → 是否需要检索）
- [ ] Context Assembly（上下文组装 + Token 预算）
- [ ] Prompt Engineering（反幻觉 prompt 模板）
- [ ] 引用格式指令注入
- 参考：`docs/rag-pipeline.md` → 阶段 1-2, 5-6（完整参考）
- 涉及文件：`backend/src/services/rag.rs`
- 验收：输入用户问题 → 检索 → 组装上下文 → 生成带引用的回答

#### 🔲 3.4 SSE 流式输出
- [ ] Axum SSE handler
- [ ] 文本增量推送 (text_delta)
- [ ] 引用实时推送 (citation event)
- [ ] 完成信号 (message_end + token 统计)
- 参考：`docs/api-design.md` → §5 流式响应格式
- 参考：`.notes/07-component-sdk-map.md` → SSE 流式
- 涉及文件：`backend/src/api/chat.rs`
- 验收：前端通过 SSE 接收流式回答 + 引用

#### 🔲 3.5 引用解析 + 验证
- [ ] Citation Extraction（正则匹配 [1][2] 标记）
- [ ] Citation Verification（检查引用编号有效性 + 文本匹配）
- [ ] citations 表写入
- 参考：`docs/rag-pipeline.md` → 阶段 8-9
- 涉及文件：`backend/src/services/citation.rs`（新建）
- 验收：回答中的引用可以追溯到具体分块 + 页码

#### 🔲 3.6 对话历史 API
- [ ] 对话列表（分页）
- [ ] 对话详情 + 消息列表
- [ ] 创建/删除对话
- [ ] 消息的引用列表
- 参考：`docs/api-design.md` → §5 对话
- 涉及文件：`backend/src/api/chat.rs`
- 验收：完整的对话 CRUD + 消息引用查询

---

### Sprint 4: Flutter Web 前端（第 4-6 周）

#### 🔲 4.0 Flutter 项目初始化
- [ ] flutter create（Web 优先）
- [ ] 依赖安装（riverpod, go_router, dio, etc.）
- [ ] 项目结构搭建（features/ + core/ + shared/）
- [ ] 主题配置（Perplexity 风格 Light/Dark）
- 参考：`docs/tech-stack.md` → 客户端层
- 参考：`.notes/08-ui-design-style.md`（完整参考）
- 涉及文件：`apps/client/`
- 验收：Flutter Web 项目可运行，有基础主题

#### 🔲 4.1 HTTP 客户端 + 认证层
- [ ] dio 配置（baseURL, interceptor）
- [ ] JWT token 管理（存储/刷新/过期处理）
- [ ] AuthProvider（Riverpod）
- 涉及文件：`apps/client/lib/core/api/`, `apps/client/lib/core/auth/`
- 验收：登录后自动附加 token，过期自动跳转登录

#### 🔲 4.2 登录/注册页面
- [ ] 登录表单
- [ ] 注册表单
- [ ] 表单验证
- [ ] 错误提示
- 参考：`.notes/08-ui-design-style.md` → 色彩/排版
- 涉及文件：`apps/client/lib/features/auth/`
- 验收：可注册新用户 + 登录 + 跳转工作台

#### 🔲 4.3 工作台首页
- [ ] 工作区列表
- [ ] 创建工作区
- [ ] 最近对话
- [ ] 侧边栏导航
- 参考：`.notes/08-ui-design-style.md` → 侧边栏设计
- 涉及文件：`apps/client/lib/features/dashboard/`
- 验收：显示工作区列表 + 导航

#### 🔲 4.4 资料库页面
- [ ] 文档列表（卡片/列表视图）
- [ ] 文件上传（拖拽 + 点击）
- [ ] 处理状态显示
- [ ] 文档删除
- 涉及文件：`apps/client/lib/features/documents/`
- 验收：上传 PDF → 看到处理进度 → ready 状态

#### 🔲 4.5 文档阅读器
- [ ] syncfusion_flutter_pdfviewer 集成
- [ ] 页码导航
- [ ] 文本搜索
- [ ] 引用高亮（黄色半透明覆盖）
- 参考：`.notes/08-ui-design-style.md` → 文档查看器
- 涉及文件：`apps/client/lib/features/reader/`
- 验收：打开 PDF → 支持搜索 + 引用跳转定位

#### 🔲 4.6 AI 问答页面（核心）
- [ ] 聊天 UI（flyer.chat 或自定义）
- [ ] SSE 流式接收
- [ ] streaming_markdown 渲染
- [ ] 引用上标 [1][2] 渲染（teal 色，可点击）
- [ ] 引用卡片列表（文档名+页码+相关度+摘要）
- [ ] 引用点击 → 跳转 PDF 对应位置
- 参考：`.notes/08-ui-design-style.md` → 引用系统 + 聊天区域
- 参考：`.notes/07-component-sdk-map.md` → Flutter 前端组件
- 涉及文件：`apps/client/lib/features/chat/`
- 验收：提问 → 流式回答 → 引用可点击 → 跳转原文

#### 🔲 4.7 引用跳转功能
- [ ] 引用点击 → 打开文档阅读器
- [ ] 自动定位到对应页码
- [ ] 高亮引用文本
- [ ] 引用预览卡片（hover 弹出）
- 参考：`docs/rag-pipeline.md` → 引用数据结构
- 涉及文件：`apps/client/lib/features/chat/`, `apps/client/lib/features/reader/`
- 验收：点击 [1] → 打开 PDF → 跳到第 12 页 → 高亮相关段落

#### 🔲 4.8 模型配置页面
- [ ] 模型列表
- [ ] 添加/编辑模型配置
- [ ] 连接测试
- [ ] 默认模型选择
- 涉及文件：`apps/client/lib/features/settings/`
- 验收：配置 Ollama/OpenAI → 测试通过 → 在聊天中可选

---

## Phase 2: 增强功能（后续）

#### 🔲 P2.1 审核系统
- [ ] 引用旁审核徽章（✓ ⚠ ✗）
- [ ] 审核面板：逐条审核
- [ ] 审核历史
- 参考：`docs/api-design.md` → §7 审核

#### 🔲 P2.2 Re-Ranking
- [ ] Cross-encoder 重排序
- [ ] LLM re-rank
- 参考：`docs/rag-pipeline.md` → 阶段 4

#### 🔲 P2.3 Query Expansion
- [ ] LLM 多查询生成
- [ ] 同义词扩展
- 参考：`docs/rag-pipeline.md` → 阶段 2

#### 🔲 P2.4 桌面端
- [ ] Windows + Linux 构建
- [ ] 本地文件系统集成

#### 🔲 P2.5 移动端 (Android)
- [ ] 响应式适配
- [ ] 移动端特有交互

---

## 自动化开发规则

1. **完成一个小功能后**（每个 ✅ 任务项）：
   - 编写对应的测试代码
   - 运行测试验证通过
   - 标记为 ✅
   - `git add -A && git commit` 提交代码
   - `git push origin master` 推送到 GitHub
   - 查阅下一个任务的参考文档
   - 自动进入下一个任务

2. **开始新任务前**：
   - 读取「参考」中列出的所有文档
   - 检查「涉及文件」是否已存在
   - 理解「验收标准」

3. **遇到阻塞时**：
   - 检查 `.notes/` 中是否有相关调研
   - 搜索 `07-component-sdk-map.md` 找可用 SDK
   - 通知用户需要确认

4. **Sprint 切换时**：
   - 提交当前 Sprint 的所有代码
   - 更新此 TODO 文档
   - 存储经验到 mem0

5. **GitHub 仓库**：
   - 远程仓库：https://github.com/XimilalaXiang/TrustRAG
   - 主分支：master
