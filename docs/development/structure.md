# Project Structure

```text
TrustRAG/
├── apps/client/                # Flutter multi-platform client
│   ├── lib/
│   │   ├── features/           # Feature modules (chat, documents, search, etc.)
│   │   ├── core/               # Shared utilities, theme, routing
│   │   └── main.dart           # App entry point
│   ├── windows/                # Windows-specific (Inno Setup installer)
│   └── pubspec.yaml
│
├── backend/                    # Rust Axum backend
│   ├── src/
│   │   ├── api/                # HTTP handlers (chat, documents, search, etc.)
│   │   ├── services/           # Business logic (citation, review, RAG)
│   │   ├── db/                 # Database compatibility layer
│   │   └── main.rs
│   ├── migrations/             # PostgreSQL migrations
│   ├── migrations_sqlite/      # SQLite schema
│   └── Cargo.toml
│
├── doc-processor/              # Python document processing service
│   ├── app/
│   │   ├── parsers/            # PDF, DOCX, TXT parsers
│   │   └── main.py
│   └── requirements.txt
│
├── infra/                      # Deployment infrastructure
│   ├── docker-compose.yml      # Full stack
│   ├── docker-compose.dev.yml  # Dev infrastructure only
│   └── Caddyfile
│
├── docs/                       # VitePress documentation site
├── scripts/                    # Release notes generator
├── CHANGELOG.md
└── .github/workflows/          # CI/CD workflows
    ├── ci.yml                  # Quality checks (push/PR)
    ├── test-build.yml          # Test branch builds
    └── release.yml             # Tag-triggered release
```
