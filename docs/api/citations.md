# Citations & Reviews

## Get Citations for a Message

```http
GET /messages/{message_id}/citations
Authorization: Bearer <token>
```

## Submit Review

```http
POST /citations/{citation_id}/reviews
Authorization: Bearer <token>
Content-Type: application/json

{"status": "approved", "comment": "Verified against source document"}
```

Status values: `approved`, `rejected`, `flagged`.

## Review History

```http
GET /citations/{citation_id}/reviews
Authorization: Bearer <token>
```
