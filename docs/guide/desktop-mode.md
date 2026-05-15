# Desktop Mode

Desktop mode is the recommended way to use TrustRAG. Everything runs locally on your device — no external servers or databases needed.

## How It Works

When you launch TrustRAG desktop, the app:

1. Starts an embedded Rust backend (Axum)
2. Creates a local SQLite database with FTS5 search
3. Automatically creates a local user account
4. Serves the Flutter UI connected to the local backend

## Supported Platforms

| Platform | Format | Notes |
|----------|--------|-------|
| Windows | `.exe` installer, portable `.zip` | Inno Setup installer |
| macOS | `.tar.gz` | Universal binary (unsigned) |
| Linux | `.tar.gz` | x64 |
| Android | `.apk` | ARM + x86 |
| iOS | `.tar.gz` | Unsigned, requires sideloading |

## Data Storage

All data is stored locally in SQLite:

- Documents and chunks
- Conversations and messages
- Citations and review records
- Workspace and user configuration
- FTS5 full-text search index

## Limitations vs Server Mode

| Feature | Desktop | Server |
|---------|---------|--------|
| Database | SQLite + FTS5 | PostgreSQL + pgvector |
| Vector search | Basic similarity | pgvector ANN |
| Document processing | Embedded | Dedicated Python sidecar |
| Multi-user | Single user | Multi-user with roles |
| Object storage | Local filesystem | MinIO / S3 |
