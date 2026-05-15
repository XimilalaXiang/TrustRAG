# RAG Chat

TrustRAG's RAG (Retrieval-Augmented Generation) chat lets you ask questions about your uploaded documents and receive grounded answers with citations.

## How It Works

1. **You ask a question** in a workspace conversation
2. **Hybrid retrieval** searches your documents using vector similarity + full-text search
3. **Relevant chunks** are retrieved and ranked using Reciprocal Rank Fusion (RRF)
4. **The LLM generates an answer** grounded in the retrieved chunks
5. **Citations are extracted** and stored, linking each claim to its source

## Supported LLM Providers

TrustRAG supports any OpenAI-compatible API:

- OpenAI (GPT-4, GPT-3.5)
- Ollama (local models)
- Anthropic
- Custom endpoints

Configure providers in **Settings → Model Configuration**.

## Streaming Responses

Chat responses stream in real-time via Server-Sent Events (SSE). Citations are processed and stored after the response completes.
