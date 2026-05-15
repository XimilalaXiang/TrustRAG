# Server Mode

Server mode provides the full-featured deployment with PostgreSQL, Redis, MinIO, and a dedicated document processor.

## Prerequisites

- Docker and Docker Compose
- Flutter 3.24+ (for building the web client)

## Quick Start

```bash
cp .env.example .env
cd apps/client && flutter pub get && flutter build web --dart-define=API_BASE_URL=/api && cd ../..
cd infra && docker compose up --build
```

## Services

| Service | Port | Description |
|---------|------|-------------|
| Web App (Caddy) | 80 | Flutter web client |
| Backend (Axum) | 8080 | Rust API server |
| Doc Processor | 8081 | Python document parsing |
| PostgreSQL | 5432 | Database with pgvector |
| Redis | 6379 | Caching |
| MinIO | 9000/9001 | Object storage |

## Environment Variables

See `.env.example` for the full list. Key variables:

| Variable | Purpose |
|----------|---------|
| `TRUSTRAG__DATABASE_URL` | PostgreSQL connection |
| `TRUSTRAG__REDIS_URL` | Redis connection |
| `TRUSTRAG__MINIO_ENDPOINT` | S3-compatible endpoint |
| `TRUSTRAG__JWT_SECRET` | JWT signing secret |
| `TRUSTRAG__DOC_PROCESSOR_URL` | Document processor URL |
