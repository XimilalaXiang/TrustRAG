# 桌面模式

桌面模式是 TrustRAG 的核心亮点：**单个安装包、开箱即用、数据完全本地化**。

## 工作原理

启动桌面应用时，后端 Rust 服务以嵌入方式运行（作为 sidecar 进程），使用 SQLite 作为数据库，FTS5 扩展提供全文搜索能力。

```text
TrustRAG Desktop
├── Flutter UI（用户界面）
├── Rust Backend（嵌入式后端）
│   ├── SQLite + FTS5（数据存储 + 全文搜索）
│   └── Axum HTTP Server（本地 API）
└── 所有数据保存在 ~/TrustRAG/
```

## 数据存储

- 默认路径：`~/TrustRAG/` （可在设置中修改）
- 数据库文件：`trustrag.db`
- 上传的文档：`documents/`

## 支持平台

| 平台 | 安装方式 | 状态 |
|------|---------|------|
| Windows | `.exe` 安装程序 | 已发布 |
| macOS | `.dmg` | 已发布 |
| Linux | `.tar.gz` | 已发布 |

## 与服务器模式的区别

| 特性 | 桌面模式 | 服务器模式 |
|------|---------|-----------|
| 数据库 | SQLite + FTS5 | PostgreSQL + pgvector |
| 搜索 | 全文搜索 (FTS5) | 向量搜索 + 全文搜索 |
| 部署 | 零配置 | 需要 Docker |
| 多用户 | 单用户 | 多用户 |
| 适用场景 | 个人知识管理 | 团队协作 |
