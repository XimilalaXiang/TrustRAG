# 服务器模式

服务器模式使用 PostgreSQL + pgvector 提供高级向量检索能力，适合团队协作和大规模知识库。

## 前置要求

- Docker & Docker Compose
- 至少 2GB 可用内存

## 快速部署

```bash
git clone https://github.com/XimilalaXiang/TrustRAG.git
cd TrustRAG/infra
cp .env.example .env  # 编辑配置
docker compose up -d
```

## 服务组成

| 服务 | 端口 | 说明 |
|------|------|------|
| Backend API | 3000 | Rust Axum 后端 |
| Web UI | 8080 | Flutter Web 客户端 |
| PostgreSQL | 5432 | 数据库 + pgvector |
| Doc Processor | 5000 | Python 文档解析 |

## 环境变量

在 `.env` 文件中配置：

```bash
DATABASE_URL=postgresql://trustrag:password@postgres:5432/trustrag
LLM_API_KEY=your-api-key
LLM_BASE_URL=https://api.openai.com/v1
LLM_MODEL=gpt-4o
```

## 与桌面模式的迁移

服务器模式和桌面模式的数据格式兼容。如需从桌面模式迁移到服务器模式：

1. 导出桌面模式的工作区数据
2. 部署服务器模式
3. 通过 API 导入数据

::: tip
如果只是个人使用，推荐使用桌面模式。服务器模式更适合需要多人协作或大规模知识库的场景。
:::
