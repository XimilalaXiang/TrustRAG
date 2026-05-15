# 工作区

工作区是 TrustRAG 中的顶层组织单位，每个工作区包含独立的文档集和对话。

## 列出工作区

```http
GET /api/workspaces
```

**响应**：

```json
{
  "data": [
    {
      "id": 1,
      "name": "技术文档",
      "description": "产品技术文档知识库",
      "document_count": 15,
      "created_at": "2026-05-01T10:00:00Z"
    }
  ]
}
```

## 创建工作区

```http
POST /api/workspaces
Content-Type: application/json

{
  "name": "技术文档",
  "description": "产品技术文档知识库"
}
```

## 获取单个工作区

```http
GET /api/workspaces/:id
```

## 更新工作区

```http
PUT /api/workspaces/:id
Content-Type: application/json

{
  "name": "更新后的名称",
  "description": "更新后的描述"
}
```

## 删除工作区

```http
DELETE /api/workspaces/:id
```

::: warning
删除工作区将同时删除该工作区下的所有文档、对话和引用数据。此操作不可撤销。
:::
