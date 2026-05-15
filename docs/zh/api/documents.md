# 文档

文档 API 用于在工作区中上传、管理和查询文档。

## 列出文档

```http
GET /api/workspaces/:workspace_id/documents
```

**响应**：

```json
{
  "data": [
    {
      "id": 1,
      "filename": "product-guide.pdf",
      "mime_type": "application/pdf",
      "size_bytes": 2048576,
      "chunk_count": 42,
      "status": "indexed",
      "created_at": "2026-05-01T10:30:00Z"
    }
  ]
}
```

## 上传文档

```http
POST /api/workspaces/:workspace_id/documents
Content-Type: multipart/form-data

file: (binary)
```

支持格式：PDF、DOCX、TXT

上传后文档将自动进入处理队列：解析 → 分块 → 索引。

## 获取文档详情

```http
GET /api/workspaces/:workspace_id/documents/:id
```

## 获取文档分块

```http
GET /api/workspaces/:workspace_id/documents/:id/chunks
```

返回文档被分割后的所有文本段落，包含页码和位置信息。

## 删除文档

```http
DELETE /api/workspaces/:workspace_id/documents/:id
```

## 文档状态

| 状态 | 说明 |
|------|------|
| `uploading` | 上传中 |
| `processing` | 解析分块中 |
| `indexing` | 建立索引中 |
| `indexed` | 就绪，可用于检索 |
| `error` | 处理失败 |
