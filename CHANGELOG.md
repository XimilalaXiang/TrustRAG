# Changelog / 更新日志

All notable changes to this project will be documented in this file.

本文件记录项目的所有重要更改。

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

**How to release / 如何发布：**
1. Add a new `## [x.y.z] - YYYY-MM-DD` section at the top (below this header)
2. List changes under: Added, Changed, Fixed, Removed, Infrastructure, Security
3. `git tag vx.y.z && git push origin vx.y.z`

---

## [0.1.1] - 2026-05-16

### Added / 新增
- 🎨 **AI provider icons / AI 提供商图标** — Model config and chat messages now show provider-specific icons (OpenAI, Claude, Gemini, etc.) instead of generic placeholders
- 🎨 **AI 提供商图标** — 模型配置和聊天消息现在显示提供商专属图标（OpenAI、Claude、Gemini 等），替代通用占位符
- 📋 **Message action bar / 消息操作栏** — Copy, retry, and edit buttons for AI responses
- 📋 **消息操作栏** — AI 回复支持复制、重试和编辑按钮

### Fixed / 修复
- 🐛 Fix double message sending on Enter key press / 修复按回车键重复发送消息
- 🐛 Fix Windows installer referencing wrong executable name / 修复 Windows 安装包引用错误的可执行文件名
- 🐛 Fix macOS bundle path from client.app to TrustRAG.app / 修复 macOS 应用包路径
- 🐛 Fix app title from "client" to "TrustRAG" across all platforms / 修复全平台应用标题
- 🐛 Remove unused import in desktop_auto_setup.dart / 移除未使用的导入

### Changed / 变更
- 🔧 Implement Issue #2 improvements + developer mode / 实现 Issue #2 改进 + 开发者模式

### Infrastructure / 基础设施
- 📚 Add VitePress documentation site with bilingual content / 添加 VitePress 双语文档站
- 📚 Overhaul README with modern layout and separate language files / 重构 README
- 📚 Add documentation site links to all README files / 添加文档站链接
- 📄 Replace abbreviated LICENSE with full Apache 2.0 text / 替换完整 Apache 2.0 许可证
- 🔧 Add test-build workflow for test branch validation / 添加 test 分支测试构建
- 🔧 Refactor release workflow with auto-generated notes from CHANGELOG / 重构发布流程

---

## [0.1.0] - 2026-05-15

### 🎉 Initial Release / 首次发布

TrustRAG is an AI-powered document Q&A system with built-in citation verification and trust scoring.

TrustRAG 是一个 AI 驱动的文档问答系统，内置引用验证和信任评分机制。

### Added / 新增
- 🖥️ **Multi-platform desktop app / 多平台桌面应用** — Windows, macOS, Linux, Android, iOS, Web all built from a single codebase
- 🖥️ **多平台桌面应用** — Windows、macOS、Linux、Android、iOS、Web 全平台一套代码构建
- 📦 **Self-contained desktop mode / 桌面端自包含模式** — Embedded SQLite backend with automatic local user setup, no external database required
- 📦 **桌面端自包含模式** — 内嵌 SQLite 后端，自动创建本地用户，无需外部数据库
- 🤖 **RAG pipeline / RAG 管线** — Retrieval-Augmented Generation with document-grounded answers and configurable LLM/Embedding providers
- 🤖 **RAG 管线** — 基于文档的检索增强生成，可配置 LLM/Embedding 提供商
- 📎 **Citation tracking / 引用追踪** — Every AI response includes traceable source citations with document, chunk, and page references
- 📎 **引用追踪** — 每条 AI 回复都包含可追溯的来源引用（文档、分块、页码）
- ✅ **Citation review / 引用审核** — Approve, reject, or flag citations for accuracy with review history
- ✅ **引用审核** — 通过、拒绝或标记引用的准确性，支持审核历史记录
- 🔍 **Full-text search / 全文搜索** — FTS5-powered search across all uploaded documents
- 🔍 **全文搜索** — 基于 FTS5 的全文档搜索
- 🪟 **Windows installer / Windows 安装包** — One-click Inno Setup `.exe` installer plus portable zip
- 🪟 **Windows 安装包** — Inno Setup 一键安装包 + 便携压缩版
- 🧠 **Knowledge graph / 知识图谱** — Entity and relation extraction with graph API
- 🧠 **知识图谱** — 实体与关系提取，提供图谱 API
- 🔌 **Plugin system / 插件系统** — Dynamic provider registry for LLM and Embedding management
- 🔌 **插件系统** — 动态提供商注册，管理 LLM 和 Embedding
- 👥 **Workspace collaboration / 工作区协作** — Multi-user workspace with member management
- 👥 **工作区协作** — 多用户工作区与成员管理
- 🌐 **Multilingual / 多语言支持** — Chinese and English interface with multilingual README
- 🌐 **多语言支持** — 中英文界面与多语言 README

### Infrastructure / 基础设施
- 🔧 GitHub Actions CI/CD with tag-triggered multi-platform builds and automated GitHub Release
- 🔧 GitHub Actions CI/CD，tag 触发多平台构建与自动 GitHub Release 发布
- 🔧 Dual-database architecture: PostgreSQL (server mode) and SQLite (desktop mode) via feature flags
- 🔧 双数据库架构：PostgreSQL（服务器模式）和 SQLite（桌面模式）通过 feature flag 切换
- 🔧 Embedded Rust backend bundled with Flutter desktop app
- 🔧 Rust 后端内嵌于 Flutter 桌面应用中
- 🔧 Automated release notes generation from CHANGELOG.md
- 🔧 从 CHANGELOG.md 自动生成 Release Notes
