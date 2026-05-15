# Dual-Database Design

TrustRAG supports two database backends via Rust feature flags.

## PostgreSQL Mode (`--features postgres`)

Used in server deployments. Provides:

- pgvector for vector similarity search
- pg_trgm for trigram-based text similarity
- Full SQL feature set
- Multi-user concurrent access

## SQLite Mode (`--features desktop`)

Used in desktop self-contained apps. Provides:

- FTS5 for full-text search
- Zero-configuration embedded database
- Single-file storage
- No external dependencies

## Compatibility Layer

The `db::compat` module handles cross-database differences:

| Concern | PostgreSQL | SQLite |
|---------|-----------|--------|
| UUIDs | Native `UUID` type | `TEXT` with string parsing |
| Booleans | `true`/`false` | `1`/`0` |
| Timestamps | `NOW()` | `datetime('now')` |
| OFFSET | Standard | Requires explicit syntax |
| RETURNING | Full support | Limited support |

## Migration Files

- `backend/migrations/` — PostgreSQL migrations (SQLx)
- `backend/migrations_sqlite/init.sql` — SQLite schema initialization
