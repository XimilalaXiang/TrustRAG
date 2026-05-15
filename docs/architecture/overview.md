# System Overview

TrustRAG operates in two modes with a shared codebase.

## Desktop Mode

```
Flutter Client (Windows / macOS / Linux / Android / iOS)
  └── HTTP/SSE
      └── Embedded Rust Backend (Axum)
          ├── Auth, workspace, document, search, chat, citation, review APIs
          ├── RAG orchestration and provider registry
          └── SQLite + FTS5
```

The Rust backend binary is bundled inside the Flutter app and started as a child process. Communication happens over localhost HTTP.

## Server Mode

```
Flutter Client (Web / Desktop)
  └── HTTP/SSE via Caddy reverse proxy
      └── Rust Backend (Axum)
          ├── Full API surface
          ├── PostgreSQL + pgvector + pg_trgm
          ├── Redis (caching)
          ├── MinIO (object storage)
          └── Python Document Processor (FastAPI)
```

## Key Design Decisions

### Dual-Database Architecture

The backend compiles with either `--features postgres` or `--features desktop`. A `db::compat` module abstracts differences (UUID handling, timestamp functions, boolean literals).

### Embedded Backend

Desktop apps bundle the Rust backend binary. The Flutter `BackendManager` starts it on launch and communicates via HTTP on a local port.

### Feature Flags

```toml
[features]
default = ["postgres"]
postgres = ["sqlx/postgres"]
desktop = ["sqlx/sqlite"]
```
