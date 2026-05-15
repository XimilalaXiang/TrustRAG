# Dev Setup

## Prerequisites

- Rust toolchain (stable)
- Flutter 3.24+
- Python 3.12+ (for document processor)
- Docker (for infrastructure services)

## Backend

```bash
cd backend
cargo run                              # Server mode (needs PostgreSQL)
cargo run --no-default-features --features desktop  # Desktop mode (SQLite)
```

## Flutter Client

```bash
cd apps/client
flutter pub get
flutter run -d chrome --dart-define=API_BASE_URL=http://localhost:8080
```

## Document Processor

```bash
cd doc-processor
python -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt
uvicorn app.main:app --port 8081 --reload
```

## Infrastructure (Server Mode)

```bash
cd infra
docker compose -f docker-compose.dev.yml up -d
```

Starts PostgreSQL, Redis, and MinIO for local development.
