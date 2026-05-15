# Workspaces

Workspaces are the top-level container for documents, conversations, and knowledge graphs.

## Create Workspace

```http
POST /workspaces
Authorization: Bearer <token>
Content-Type: application/json

{"name": "My Project", "description": "Research documents"}
```

## List Workspaces

```http
GET /workspaces
Authorization: Bearer <token>
```

## Workspace Members

```http
GET /workspaces/{ws_id}/members
POST /workspaces/{ws_id}/members
```

Members have roles: `owner`, `admin`, `member`, `viewer`.
