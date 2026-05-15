# Chat & Messages

## Create Conversation

```http
POST /workspaces/{ws_id}/conversations
Authorization: Bearer <token>
Content-Type: application/json

{"title": "Q&A about quarterly report"}
```

## Send Message (Streaming)

```http
POST /conversations/{id}/messages
Authorization: Bearer <token>
Content-Type: application/json

{"content": "What were the key findings?"}
```

Returns an SSE stream with events:

- `message` — Streaming text chunks
- `citations_stored` — Citation IDs after processing
- `done` — Stream complete

## List Messages

```http
GET /conversations/{id}/messages
Authorization: Bearer <token>
```

Returns messages with embedded citations (if any).
