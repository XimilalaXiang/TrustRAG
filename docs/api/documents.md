# Documents

## Upload Document

```http
POST /workspaces/{ws_id}/documents
Authorization: Bearer <token>
Content-Type: multipart/form-data

file: <binary>
```

Supported formats: PDF, DOCX, Markdown, TXT, HTML.

## List Documents

```http
GET /workspaces/{ws_id}/documents
Authorization: Bearer <token>
```

## Document Processing

After upload, documents go through:

1. Parsing (text extraction)
2. Chunking (semantic splitting)
3. Embedding (vector generation)
4. Indexing (search index update)

Check processing status via the document's `status` field: `pending`, `processing`, `ready`, `error`.
