# 双数据库设计

TrustRAG 的核心架构创新是通过 Rust 编译时特性（Cargo features）实现两套数据库后端的无缝切换。

## 设计动机

| 需求 | 桌面模式 | 服务器模式 |
|------|---------|-----------|
| 部署难度 | 零配置 | Docker 部署 |
| 数据存储 | 本地文件 | 远程数据库 |
| 搜索能力 | 全文搜索 | 向量 + 全文混合搜索 |
| 并发支持 | 单用户 | 多用户 |

## 实现方式

### Cargo Features

```toml
[features]
default = ["postgres"]
desktop = ["dep:rusqlite", "dep:sqlite-fts5"]
postgres = ["dep:sqlx", "dep:pgvector"]
```

### 数据库抽象层

```rust
// 统一的数据库 trait
pub trait Database: Send + Sync {
    async fn search_documents(&self, query: &str, workspace_id: i64)
        -> Result<Vec<SearchResult>>;
    async fn insert_document(&self, doc: &Document) -> Result<i64>;
    // ... 其他操作
}
```

编译时根据 feature 选择具体实现：

- `--features desktop` → `SqliteDatabase`
- `--features postgres` → `PostgresDatabase`

## SQLite + FTS5（桌面模式）

- 内嵌数据库，无需安装
- FTS5 全文搜索扩展，支持 BM25 排序
- 单文件存储，便于备份和迁移

## PostgreSQL + pgvector（服务器模式）

- 企业级关系数据库
- pgvector 扩展支持向量相似度搜索
- 支持并发访问和事务隔离
