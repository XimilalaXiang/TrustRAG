# Citation System

The citation system is TrustRAG's core differentiator.

## Citation Lifecycle

```
LLM Response → Citation Extraction → Database Storage → Client Display → User Review
```

## Database Schema

```sql
CREATE TABLE citations (
    id TEXT PRIMARY KEY,
    message_id TEXT REFERENCES messages(id),
    document_id TEXT REFERENCES documents(id),
    chunk_id TEXT REFERENCES document_chunks(id),
    citation_index INTEGER NOT NULL,
    quoted_text TEXT,
    page_number INTEGER,
    heading_path TEXT,
    relevance_score REAL,
    verified INTEGER DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now'))
);
```

## Review Records

```sql
CREATE TABLE review_records (
    id TEXT PRIMARY KEY,
    citation_id TEXT REFERENCES citations(id),
    reviewer_id TEXT REFERENCES users(id),
    status TEXT NOT NULL,  -- 'approved', 'rejected', 'flagged'
    comment TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);
```

## API Flow

1. **During chat**: Citations are extracted and stored; IDs sent via SSE
2. **On page load**: `GET /messages` returns messages with embedded citations
3. **On review**: `POST /reviews` creates a review record and updates citation status
