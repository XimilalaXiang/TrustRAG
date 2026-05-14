# PRD 深度分析（2026-05-14）

## 原始 PRD 概要

- 文档：`docs/prd-original.md`（38KB，1220行）
- 项目原名：TAIKW（Trustworthy AI Knowledge Workbench）
- 定位：可信 AI 知识工作台，解决 LLM 幻觉问题

## 核心功能链

1. 文档上传 → Markdown 统一中间格式 → 结构化分块
2. RAG 检索增强生成（全文 + 向量 + 混合检索 + 重排序）
3. 模型输出强制关联引用来源（chunk_id）
4. 一键跳转原文（PDF 页码高亮、Markdown 锚点等）
5. 二次复核（7 种状态：可信/引用不足/幻觉/误译等）
6. 本地模型导入（HuggingFace/GGUF/LoRA）

## 架构规模

- 9 层架构
- 15 个核心模块
- 10 类可替换插件
- 14 张数据库表
- 7 组 RESTful API

## 5 个开发阶段

1. Phase 1：MVP 资料库
2. Phase 2：可信问答 + 引用溯源
3. Phase 3：原文跳转 + 复核闭环
4. Phase 4：原创前端设计系统
5. Phase 5：本地模型 + HuggingFace 导入

## PRD 自身提出的风险

1. 开发范围过大
2. 幻觉控制效果不稳定
3. 文档解析准确性
4. 本地模型硬件限制
5. 前端原创性
6. 开源许可证
