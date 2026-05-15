# 项目结构

```text
TrustRAG/
├── apps/client/                # Flutter 跨平台客户端
│   ├── lib/
│   │   ├── features/           # 功能模块（对话、文档、搜索等）
│   │   ├── core/               # 共享工具、主题、路由
│   │   └── main.dart           # 应用入口
│   ├── windows/                # Windows 专属（Inno Setup 安装程序）
│   └── pubspec.yaml
│
├── backend/                    # Rust Axum 后端
│   ├── src/
│   │   ├── api/                # HTTP 接口（对话、文档、搜索等）
│   │   ├── services/           # 业务逻辑（引用、审核、RAG）
│   │   ├── db/                 # 数据库兼容层
│   │   └── main.rs
│   ├── migrations/             # PostgreSQL 迁移脚本
│   ├── migrations_sqlite/      # SQLite schema
│   └── Cargo.toml
│
├── doc-processor/              # Python 文档处理服务
│   ├── app/
│   │   ├── parsers/            # PDF、DOCX、TXT 解析器
│   │   └── main.py
│   └── requirements.txt
│
├── infra/                      # 部署基础设施
│   ├── docker-compose.yml      # 全栈部署
│   ├── docker-compose.dev.yml  # 开发环境
│   └── Caddyfile
│
├── docs/                       # VitePress 文档站
├── scripts/                    # Release Notes 生成脚本
├── CHANGELOG.md
└── .github/workflows/          # CI/CD 工作流
    ├── ci.yml                  # 质量检查（Push / PR）
    ├── test-build.yml          # test 分支构建
    └── release.yml             # Tag 触发正式发布
```
