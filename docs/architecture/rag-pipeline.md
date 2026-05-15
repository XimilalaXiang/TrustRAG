# RAG Pipeline

## Document Ingestion

1. User uploads a document (PDF, DOCX, Markdown, TXT, HTML)
2. Document processor extracts text and metadata
3. Text is split into semantic chunks with heading paths and page numbers
4. Chunks are embedded using the configured embedding provider
5. Chunks and embeddings are stored in the database

## Retrieval

When a user asks a question:

1. Query is embedded using the same embedding provider
2. **Vector search** finds semantically similar chunks
3. **Full-text search** finds keyword-matching chunks
4. **Reciprocal Rank Fusion (RRF)** combines both result sets
5. Top-K chunks are selected as context

## Generation

1. Retrieved chunks are formatted into a grounding prompt
2. The LLM generates a response with inline citations
3. Response streams to the client via SSE
4. Citations are extracted and stored in the database

## Citation Extraction

After generation, the system:

1. Parses citation markers from the LLM response
2. Matches each citation to its source chunk
3. Stores citation records with document, chunk, page, and heading metadata
4. Emits a `citations_stored` event to the client with citation IDs
