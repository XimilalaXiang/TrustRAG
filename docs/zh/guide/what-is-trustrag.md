# 什么是 TrustRAG？

TrustRAG 是一个**可信的检索增强生成（RAG）知识工作台**，致力于让 AI 生成的回答**可验证、可溯源、可审核**。

## 核心理念

传统 RAG 系统生成答案后，用户往往无法判断答案是否准确。TrustRAG 从设计之初就围绕**信任**构建：

- **精确引用** — 每段回答都标注来源文档和具体段落
- **文档审核** — 内置审核工作流，支持逐段校对
- **可验证** — 用户可以直接跳转到原文查看上下文

## 双模式架构

TrustRAG 同时支持两种运行模式：

### 桌面模式（推荐入门）
- 单个安装包，开箱即用
- 内嵌 SQLite + FTS5，数据完全存储在本地
- 无需服务器，无需 Docker
- 支持 Windows、macOS、Linux

### 服务器模式
- PostgreSQL + pgvector 向量检索
- 适合团队协作和大规模知识库
- Docker Compose 一键部署

## 技术栈

| 组件 | 技术 |
|------|------|
| 后端 | Rust (Axum) |
| 前端 | Flutter (跨平台) |
| 桌面数据库 | SQLite + FTS5 |
| 服务器数据库 | PostgreSQL + pgvector |
| 文档处理 | Python (PDF/DOCX/TXT) |
