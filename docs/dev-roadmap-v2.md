# TrustRAG 项目发展深度分析 + 分阶段 TODO List

> 更新日期：2026-05-14
> 基于：`.notes/` 全部 8 份文档 + `docs/dev-todolist.md` + 当前代码库状态

---

## 一、项目现状评估

### 已完成（Phase 1 MVP — 100%）

| 模块 | 状态 | 关键能力 |
|------|------|---------|
| 后端基础 | ✅ 完成 | Axum + JWT + RBAC + 工作区 CRUD |
| 文档处理 | ✅ 完成 | PDF/DOCX/TXT → Markdown → 分块 → Embedding |
| 混合检索 | ✅ 完成 | pgvector 向量 + pg_trgm 全文 + RRF 融合 |
| RAG 管线 | ✅ 完成 | Query Analysis → 检索 → Context Assembly → LLM 生成 |
| 引用系统 | ✅ 完成 | Citation Extraction + Verification + SSE 推送 |
| 前端 Web | ✅ 完成 | 登录/注册 + 工作台 + 资料库 + 聊天 + 设置 |
| Citation UI | ✅ 完成 | SSE 解析 + 引用卡片 + 详情弹窗 |
| 文档查看器 | ✅ 完成 | Markdown 内容 + 分块索引 + 搜索 |
| 部署 | ✅ 完成 | Docker Compose 6 容器 + Caddy |

### 核心代码量

- 后端 Rust: ~35 个源文件，覆盖 API/服务/中间件/任务
- 前端 Flutter: ~17 个源文件，features 结构清晰
- 文档处理 Python: PDF/DOCX/TXT 解析 + 位置映射
- 测试: 40+ 单元测试

---

## 二、战略方向分析

### 2.1 核心差异化（vs Open WebUI）

TrustRAG 的差异化不在"聊天"，而在**"可信"**。三个核心差异化必须落地：

1. **引用精确度** — chunk_id 级精确引用 + Citation Verification（已有骨架）
2. **原文跳转** — PDF 页码高亮、段落定位（Phase 2 核心）
3. **复核流程** — 7 种复核状态 + 审计追踪（Phase 3 核心）

### 2.2 发展策略建议

**短期（Phase 2，2-3 周）**: 把"可信"体验做扎实
- 原文跳转是让用户**感受到**引用价值的关键动作
- Re-ranking 显著提升回答质量，投入产出比高
- PDF 查看器是高频使用场景

**中期（Phase 3，3-4 周）**: 复核闭环 + 质量提升
- 审核系统是企业用户付费的核心功能
- Query Expansion 让搜索更智能
- 多语言支持扩大用户群

**长期（Phase 4-5，持续迭代）**: 设计系统 + 本地模型
- Perplexity 风格 UI 重构
- 本地模型支持（Ollama/GGUF）降低使用门槛
- 桌面端 + 移动端扩展

### 2.3 技术债务（需在 Phase 2 初期处理）

| 技术债 | 影响 | 优先级 |
|--------|------|--------|
| 无 Dark 模式 | UI 设计规范已定义但未实现 | P1 |
| 流式 Markdown 非真正流式 | 使用 flutter_markdown 逐段重建，性能差 | P2 |
| 无对话侧边栏历史 | 只能在工作台首页看历史 | P1 |
| 无分块预览（原文定位） | Citation 点击无法跳到原文 | P0 |
| API Base URL normalize 全局化 | 已修复但应抽成配置层 | P3 |

---

## 三、分阶段 TODO List

---

### Phase 2: 可信体验增强（2-3 周）

> 目标：让用户在使用中真正感受到"引用可验证、可追溯"

#### Sprint 5: 原文跳转 + PDF 查看器（第 1 周）

| # | 任务 | 优先级 | 预估 | 依赖 |
|---|------|--------|------|------|
| 5.1 | **Syncfusion PDF Viewer 集成** | P0 | 1.5天 | 无 |
| | - pubspec.yaml 添加 syncfusion_flutter_pdfviewer | | | |
| | - 创建 PDFViewerPage，支持 URL 加载和本地加载 | | | |
| | - 页码导航、缩放、搜索功能 | | | |
| | - 涉及文件: `apps/client/lib/features/reader/pages/pdf_viewer_page.dart` | | | |
| | - 验收: 打开 PDF，可翻页、搜索 | | | |
| 5.2 | **引用 → 原文跳转** | P0 | 1.5天 | 5.1 |
| | - Citation 卡片点击 → 打开 PDF viewer 并跳转到指定页码 | | | |
| | - 文本高亮（基于 char_start/char_end 或 page_start） | | | |
| | - 后端 `/documents/:id/download` 返回原始 PDF 文件 | | | |
| | - 涉及文件: `chat_page.dart`, `pdf_viewer_page.dart`, `document_viewer_page.dart` | | | |
| | - 验收: 点击引用 [1] → 跳转到 PDF 第 N 页并高亮相关段落 | | | |
| 5.3 | **对话历史侧边栏** | P1 | 1天 | 无 |
| | - 左侧栏增加对话历史列表（按时间分组：今天/昨天/更早） | | | |
| | - 点击切换对话，加载消息历史 | | | |
| | - 新建对话按钮 | | | |
| | - 涉及文件: `chat_page.dart`, `chat_provider.dart` | | | |
| | - 验收: 侧边栏显示对话列表，点击切换 | | | |
| 5.4 | **Dark 模式** | P1 | 0.5天 | 无 |
| | - 根据 `.notes/08-ui-design-style.md` 定义的 Dark 色彩系统实现 | | | |
| | - ThemeData.dark() 配置 | | | |
| | - 主题切换开关（设置页面） | | | |
| | - 涉及文件: `app_theme.dart`, `dashboard_page.dart` | | | |
| | - 验收: 切换 Dark 模式，所有页面显示正常 | | | |

#### Sprint 6: 检索质量提升（第 2 周）

| # | 任务 | 优先级 | 预估 | 依赖 |
|---|------|--------|------|------|
| 6.1 | **Re-Ranking 模块** | P0 | 2天 | 无 |
| | - ReRanker trait 定义（score + rerank 方法） | | | |
| | - LLM Re-Rank 实现（使用 LLM 对 top-N 结果打分） | | | |
| | - 可选: Cross-Encoder 本地重排（fastembed 或外部 API） | | | |
| | - 集成到 RAG 管线（检索后、Context Assembly 前） | | | |
| | - 涉及文件: `backend/src/services/reranker.rs`, `backend/src/services/rag.rs` | | | |
| | - 参考: `docs/rag-pipeline.md` → 阶段 4 | | | |
| | - 验收: 对比有/无 Re-Rank 的搜索结果质量 | | | |
| 6.2 | **Query Expansion（查询扩展）** | P1 | 1.5天 | 无 |
| | - LLM 生成 2-3 个变体查询 | | | |
| | - 多查询并行检索 + 结果合并去重 | | | |
| | - 涉及文件: `backend/src/services/rag.rs`, `backend/src/services/search.rs` | | | |
| | - 参考: `docs/rag-pipeline.md` → 阶段 2 | | | |
| | - 验收: 模糊查询的召回率提升 | | | |
| 6.3 | **流式 Markdown 优化** | P1 | 1天 | 无 |
| | - 评估 streaming_markdown 包替换 flutter_markdown | | | |
| | - 逐字渲染 + 引用标记实时出现 | | | |
| | - 代码块语法高亮 | | | |
| | - 涉及文件: `chat_page.dart` | | | |
| | - 验收: 流式输出更流畅，无闪烁重建 | | | |
| 6.4 | **Embedding 模型本地化（可选）** | P2 | 1天 | 无 |
| | - 集成 fastembed-rs（Rust 本地 ONNX 推理） | | | |
| | - 支持 Qwen3-Embedding / Nomic 等模型 | | | |
| | - 无需外部 Embedding API | | | |
| | - 涉及文件: `backend/src/services/embedding.rs` | | | |
| | - 验收: 纯本地 Embedding 生成，无外部 API 依赖 | | | |

---

### Phase 3: 复核闭环 + 企业特性（3-4 周）✅ 已完成

> 目标：让 TrustRAG 从"工具"变成"工作流"，支持团队协作和质量管理

#### Sprint 7: 审核系统（第 1-2 周）

| # | 任务 | 优先级 | 预估 | 依赖 |
|---|------|--------|------|------|
| 7.1 | ✅ **审核 API 完善** | P0 | 1.5天 | 无 |
| | - ✅ 后端 `reviews.rs` service + API routes 完成 | | | |
| | - ✅ 审核状态: approved / rejected / flagged / pending | | | |
| | - ✅ 审核历史追踪（reviewer_id + timestamp + comment + corrected_text） | | | |
| | - ✅ 对话级审核统计 API（review-stats） | | | |
| | - 批量审核接口 | | | |
| | - 涉及文件: `backend/src/api/reviews.rs`, `backend/src/services/review.rs` | | | |
| | - 验收: 对引用进行逐条审核 + 查看审核历史 | | | |
| 7.2 | **审核面板 UI** | P0 | 2天 | 7.1 |
| | - 审核面板：展示引用列表 + 原文对比 + 审核操作 | | | |
| | - 引用旁审核徽章（✓ 已验证 / ⚠ 待审核 / ✗ 有异议） | | | |
| | - 审核统计仪表盘 | | | |
| | - 涉及文件: `apps/client/lib/features/review/` | | | |
| | - 验收: 完整的审核工作流 | | | |
| 7.3 | **审核报告导出** | P2 | 1天 | 7.2 |
| | - 生成审核报告（Markdown/PDF） | | | |
| | - 包含: 引用统计、幻觉率、审核覆盖率 | | | |
| | - 验收: 可导出可分享的审核报告 | | | |

#### Sprint 8: 质量与体验（第 3-4 周）

| # | 任务 | 优先级 | 预估 | 依赖 |
|---|------|--------|------|------|
| 8.1 | **多语言 UI** | P1 | 1.5天 | 无 |
| | - i18n 框架集成（flutter_localizations + intl） | | | |
| | - 中文 + 英文双语 | | | |
| | - 语言切换（设置页面） | | | |
| | - 验收: 界面可切换中英文 | | | |
| 8.2 | **跟进问题推荐** | P1 | 1天 | 无 |
| | - LLM 生成 2-3 个相关后续问题 | | | |
| | - 在回答下方显示为 pill 按钮 | | | |
| | - 点击直接发送 | | | |
| | - 涉及文件: `rag.rs`, `chat.rs`, `chat_page.dart` | | | |
| | - 参考: `.notes/08-ui-design-style.md` → 跟进问题 | | | |
| | - 验收: 回答后出现推荐问题 | | | |
| 8.3 | **文档批量操作** | P2 | 1天 | 无 |
| | - 批量上传（多文件选择） | | | |
| | - 批量删除 | | | |
| | - 批量重新处理 | | | |
| | - 验收: 可一次操作多个文档 | | | |
| 8.4 | **Workspace 级搜索** | P1 | 0.5天 | 无 |
| | - 跨文档全局搜索（前端搜索框） | | | |
| | - 搜索结果展示：匹配文档 + 分块 + 高亮 | | | |
| | - 验收: 输入关键词 → 返回跨文档搜索结果 | | | |

---

### Phase 4: 设计系统 + 性能优化（2-3 周）✅ 已完成

> 目标：从功能可用到体验优秀，对标 Perplexity AI

#### Sprint 9: UI 重构（第 1-2 周）

| # | 任务 | 优先级 | 预估 | 依赖 |
|---|------|--------|------|------|
| 9.1 | **Perplexity 风格 UI 重构** | P1 | 3天 | 无 |
| | - 严格按照 `.notes/08-ui-design-style.md` 的设计规范 | | | |
| | - 单栏居中布局（720px 最大宽度） | | | |
| | - 引用系统上标 + Hover 预览卡 | | | |
| | - 无阴影原则（色彩层级 + 细边框） | | | |
| | - 思源黑体 + Inter 字体组合 | | | |
| | - 响应式断点（桌面宽屏/标准/平板/手机） | | | |
| | - 验收: 视觉风格接近 Perplexity AI | | | |
| 9.2 | **动效系统** | P2 | 1天 | 9.1 |
| | - 按钮状态切换 150ms | | | |
| | - 引用预览弹出 200ms | | | |
| | - 流式文字渲染 15-30ms/字符 | | | |
| | - 页面路由切换 300ms | | | |
| | - 验收: 交互流畅自然 | | | |
| 9.3 | **响应式布局完善** | P1 | 1天 | 9.1 |
| | - ≥1200px: 侧边栏 + 内容 + 来源面板 | | | |
| | - 900-1199px: 侧边栏 + 内容 | | | |
| | - 600-899px: 折叠侧边栏 + 内容 | | | |
| | - <600px: 底部导航 + 全屏内容 | | | |
| | - 验收: 各断点布局合理 | | | |

#### Sprint 10: 性能优化（第 2-3 周）

| # | 任务 | 优先级 | 预估 | 依赖 |
|---|------|--------|------|------|
| 10.1 | **后端性能优化** | P1 | 1.5天 | 无 |
| | - 连接池调优（PostgreSQL, Redis） | | | |
| | - 查询优化（EXPLAIN ANALYZE 关键查询） | | | |
| | - 缓存层（Redis 缓存热点查询） | | | |
| | - Embedding 批量处理优化 | | | |
| | - 验收: P95 延迟降低 30%+ | | | |
| 10.2 | **前端性能优化** | P1 | 1天 | 无 |
| | - 消息列表虚拟滚动（长对话） | | | |
| | - 图片懒加载 | | | |
| | - 缓存管理（dio cache interceptor） | | | |
| | - 验收: 长对话无卡顿 | | | |
| 10.3 | **可观测性** | P2 | 1天 | 无 |
| | - 结构化日志完善（tracing spans） | | | |
| | - API 响应时间监控 | | | |
| | - 错误率统计 | | | |
| | - 可选: Prometheus metrics 端点 | | | |
| | - 验收: 可监控系统健康状况 | | | |

---

### Phase 5: 本地模型 + 扩展（持续迭代）

> 目标：降低使用门槛，支持完全私有化部署

#### Sprint 11: 本地模型支持

| # | 任务 | 优先级 | 预估 | 依赖 |
|---|------|--------|------|------|
| 11.1 | **Ollama 集成优化** | P0 | 1天 | 无 |
| | - Ollama 自动发现（本地端口检测） | | | |
| | - 模型列表获取（/api/tags） | | | |
| | - 一键切换 Ollama 模型 | | | |
| | - 验收: 检测到本地 Ollama → 自动配置 | | | |
| 11.2 | **HuggingFace 模型导入** | P1 | 2天 | 无 |
| | - HuggingFace Hub API 搜索模型 | | | |
| | - GGUF 模型下载 + 本地管理 | | | |
| | - Ollama modelfile 自动生成 | | | |
| | - 验收: 搜索 HuggingFace → 下载 → 可用 | | | |
| 11.3 | **fastembed 本地 Embedding** | P1 | 1天 | 无 |
| | - 已在 Phase 2 Sprint 6 评估 | | | |
| | - 正式集成为默认 Embedding 后端 | | | |
| | - 验收: 无需外部 API 即可完成文档索引 | | | |

#### Sprint 12: 多平台扩展

| # | 任务 | 优先级 | 预估 | 依赖 |
|---|------|--------|------|------|
| 12.1 | **桌面端构建（GitHub Actions）** | P1 | 2天 | 无 |
| | - Windows + Linux Flutter Desktop 构建 | | | |
| | - GitHub Actions CI/CD（多平台构建必须走 CI） | | | |
| | - 本地文件系统集成（拖拽文件夹导入） | | | |
| | - 验收: CI 产出 Windows/Linux 安装包 | | | |
| 12.2 | **Android 适配** | P2 | 1.5天 | 无 |
| | - 响应式布局适配（已在 Phase 4 完成基础） | | | |
| | - 移动端特有交互（底部导航、手势） | | | |
| | - Android APK 构建 | | | |
| | - 验收: Android 设备可正常使用 | | | |

#### Sprint 13: 高级特性（可选）

| # | 任务 | 优先级 | 预估 | 依赖 |
|---|------|--------|------|------|
| 13.1 | **插件系统** | P3 | 3天 | 无 |
| | - LLM Provider 插件接口 | | | |
| | - Embedding Provider 插件接口 | | | |
| | - 文档处理器插件接口 | | | |
| 13.2 | **知识图谱可视化** | P3 | 2天 | 无 |
| | - 文档间关系图 | | | |
| | - 实体抽取 + 关系链接 | | | |
| 13.3 | **多用户协作** | P2 | 2天 | 无 |
| | - 工作区共享（邀请成员） | | | |
| | - 实时协作标注 | | | |
| | - 权限细化 | | | |

---

## 四、里程碑时间线

```
2026.05 ─── Phase 1 MVP ✅ 已完成
         │
2026.05-06 ── Phase 2 可信体验增强（2-3 周）
         │   ├── Sprint 5: 原文跳转 + PDF 查看器 + Dark 模式
         │   └── Sprint 6: Re-Ranking + Query Expansion + 流式优化
         │
2026.06 ─── Phase 3 复核闭环（3-4 周）
         │   ├── Sprint 7: 审核系统
         │   └── Sprint 8: 多语言 + 推荐问题 + 批量操作
         │
2026.07 ─── Phase 4 设计系统 + 性能（2-3 周）
         │   ├── Sprint 9: Perplexity 风格 UI 重构
         │   └── Sprint 10: 前后端性能优化
         │
2026.08+ ── Phase 5 本地模型 + 扩展（持续）
             ├── Sprint 11: Ollama + HuggingFace + fastembed
             ├── Sprint 12: 桌面端 + Android
             └── Sprint 13: 插件 + 知识图谱 + 协作
```

---

## 五、关键风险与应对

| 风险 | 影响 | 应对策略 |
|------|------|---------|
| Syncfusion PDF Viewer Web 性能 | 大文件可能卡顿 | 分页加载 + 虚拟化 |
| Re-Ranking LLM 调用成本 | 每次检索额外 1 次 LLM 调用 | 可配置开关 + 缓存 |
| Flutter Desktop 兼容性 | 平台差异 bug | GitHub Actions 多平台 CI 测试 |
| fastembed ONNX 模型体积 | Docker 镜像增大 | 独立构建层 + 可选特性 |
| 审核系统复杂度 | 7 种状态 + 历史追踪 | 先实现 3 种核心状态，渐进增加 |

---

## 六、推荐立即执行（Phase 2 Sprint 5）

基于投入产出比，建议下一步最优行动：

1. **Syncfusion PDF Viewer 集成** — 高频使用场景，差异化核心
2. **引用 → 原文跳转** — 让"可信"从概念变成体验
3. **对话历史侧边栏** — 基础 UX 提升，用户留存关键
4. **Dark 模式** — 低投入高回报，设计规范已定义

这四个任务组成一个完整的 Sprint，预计 4-5 天可完成。
