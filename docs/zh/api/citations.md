# 引用与审核

引用 API 用于查询和管理引用数据，以及执行审核操作。

## 获取消息的引用

```http
GET /api/citations?message_id=:message_id
```

**响应**：

```json
{
  "data": [
    {
      "id": 1,
      "message_id": 2,
      "chunk_id": 42,
      "position": 0,
      "document": {
        "id": 1,
        "filename": "architecture.pdf"
      },
      "chunk": {
        "page": 5,
        "paragraph": 3,
        "text": "引用系统采用多层架构设计..."
      },
      "review_status": "pending",
      "reviewer_note": null
    }
  ]
}
```

## 更新审核状态

```http
PUT /api/citations/:id/review
Content-Type: application/json

{
  "status": "approved",
  "note": "引用内容与原文一致"
}
```

### 可用状态

| 状态 | 说明 |
|------|------|
| `pending` | 待审核（默认） |
| `approved` | 已确认准确 |
| `rejected` | 标记为不准确 |
| `needs_review` | 需要进一步核查 |

## 批量审核

```http
POST /api/citations/batch-review
Content-Type: application/json

{
  "citation_ids": [1, 2, 3],
  "status": "approved"
}
```

## 引用统计

```http
GET /api/citations/stats?workspace_id=:workspace_id
```

**响应**：

```json
{
  "data": {
    "total": 150,
    "approved": 120,
    "rejected": 5,
    "pending": 20,
    "needs_review": 5,
    "accuracy_rate": 0.96
  }
}
```
