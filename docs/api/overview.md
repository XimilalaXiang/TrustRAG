# API Overview

TrustRAG backend exposes a REST API over HTTP with SSE streaming for chat responses.

## Base URL

- Desktop mode: `http://localhost:{port}` (auto-assigned)
- Server mode: `http://localhost:8080` (default)

## Authentication

All endpoints (except `/health` and `/auth/*`) require a JWT Bearer token:

```
Authorization: Bearer <token>
```

## Route Groups

| Group | Base Path | Description |
|-------|-----------|-------------|
| Health | `/health` | Health check |
| Auth | `/auth/*` | Register, login, profile |
| Workspaces | `/workspaces/*` | Workspace CRUD and members |
| Documents | `/workspaces/{ws_id}/documents/*` | Upload, list, download |
| Search | `/workspaces/{ws_id}/search` | Hybrid search |
| Conversations | `/workspaces/{ws_id}/conversations/*` | Chat conversations |
| Messages | `/conversations/{id}/messages` | Send and list messages |
| Citations | `/messages/{id}/citations` | Citation data |
| Reviews | `/citations/{id}/reviews` | Review records |
| Models | `/model-configs/*` | LLM provider config |
| Embeddings | `/embedding-configs/*` | Embedding provider config |
| Knowledge Graph | `/workspaces/{ws_id}/knowledge-graph/*` | Entities and relations |
