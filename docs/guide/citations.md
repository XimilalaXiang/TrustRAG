# Citations & Review

Citations are the core differentiator of TrustRAG. Every AI-generated answer can be traced back to its original source.

## Citation Structure

Each citation contains:

| Field | Description |
|-------|-------------|
| `document_id` | The source document |
| `chunk_id` | The specific text chunk |
| `citation_index` | Position in the answer |
| `quoted_text` | The exact quoted passage |
| `page_number` | Page in the original document |
| `heading_path` | Section hierarchy |
| `relevance_score` | How relevant the chunk is |

## Review Workflow

Citations can be reviewed for accuracy:

- **Approve** — Citation is accurate and well-sourced
- **Reject** — Citation is inaccurate or misleading
- **Flag** — Citation needs further investigation

Review records are stored with timestamps and reviewer information, providing a full audit trail.

## Viewing Citations

When you re-enter a conversation, all citations are loaded alongside the messages. Click any citation to see:

- The quoted text
- The source document and page
- The heading path
- Review status and history
