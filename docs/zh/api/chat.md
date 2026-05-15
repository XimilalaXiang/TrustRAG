# 对话与消息

对话 API 用于管理 RAG 对话和消息。

## 列出对话

```http
GET /api/workspaces/:workspace_id/chats
```

## 创建对话

```http
POST /api/workspaces/:workspace_id/chats
Content-Type: application/json

{
  "title": "关于产品架构的问题"
}
```

## 发送消息

```http
POST /api/workspaces/:workspace_id/chats/:chat_id/messages
Content-Type: application/json

{
  "content": "TrustRAG 的引用系统是如何工作的？"
}
```

**响应**（流式）：

```
data: {"type": "text", "content": "TrustRAG 的引用系统"}
data: {"type": "text", "content": "通过以下流程工作"}
data: {"type": "citation", "index": 1, "chunk_id": 42}
data: {"type": "done"}
```

## 获取消息历史

```http
GET /api/workspaces/:workspace_id/chats/:chat_id/messages
```

**响应**：

```json
{
  "data": [
    {
      "id": 1,
      "role": "user",
      "content": "TrustRAG 的引用系统是如何工作的？",
      "created_at": "2026-05-01T11:00:00Z"
    },
    {
      "id": 2,
      "role": "assistant",
      "content": "TrustRAG 的引用系统通过以下流程工作...[1]",
      "citations": [
        {
          "index": 1,
          "chunk_id": 42,
          "document_name": "architecture.pdf",
          "page": 5,
          "text_preview": "引用系统采用..."
        }
      ],
      "created_at": "2026-05-01T11:00:01Z"
    }
  ]
}
```

## 删除对话

```http
DELETE /api/workspaces/:workspace_id/chats/:chat_id
```
