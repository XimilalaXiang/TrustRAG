# TrustRAG API 设计文档

> 版本：v1.0
> 基础路径：`/api/v1`
> 协议：REST + SSE（流式响应）
> 认证：Bearer JWT Token

---

## 认证规则

| 标记 | 含义 |
|------|------|
| 🔓 | 公开接口，无需 Token |
| 🔒 | 需要有效 JWT |
| 👑 | 需要 admin 角色 |

---

## 1. 认证 `/auth`

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| POST | `/auth/register` | 用户注册 | 🔓 |
| POST | `/auth/login` | 登录，返回 JWT | 🔓 |
| POST | `/auth/refresh` | 刷新 Token | 🔒 |
| GET | `/auth/me` | 获取当前用户信息 | 🔒 |
| PUT | `/auth/me` | 更新个人信息 | 🔒 |
| PUT | `/auth/me/password` | 修改密码 | 🔒 |

### POST /auth/register
```json
// Request
{ "email": "user@example.com", "password": "...", "display_name": "张三" }
// Response 201
{ "id": "uuid", "email": "...", "display_name": "...", "role": "user" }
```

### POST /auth/login
```json
// Request
{ "email": "user@example.com", "password": "..." }
// Response 200
{ "access_token": "jwt...", "refresh_token": "jwt...", "expires_in": 3600, "user": {...} }
```

---

## 2. 工作区 `/workspaces`

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/workspaces` | 列出我的工作区 | 🔒 |
| POST | `/workspaces` | 创建工作区 | 🔒 |
| GET | `/workspaces/:id` | 获取工作区详情 | 🔒 |
| PUT | `/workspaces/:id` | 更新工作区 | 🔒 |
| DELETE | `/workspaces/:id` | 删除工作区 | 🔒 |
| GET | `/workspaces/:id/members` | 列出成员 | 🔒 |
| POST | `/workspaces/:id/members` | 添加成员 | 🔒 |
| DELETE | `/workspaces/:id/members/:user_id` | 移除成员 | 🔒 |

---

## 3. 文档管理 `/workspaces/:ws_id/documents`

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/workspaces/:ws_id/documents` | 列出文档（分页+筛选） | 🔒 |
| POST | `/workspaces/:ws_id/documents` | 上传文档（multipart/form-data） | 🔒 |
| GET | `/workspaces/:ws_id/documents/:id` | 获取文档详情 | 🔒 |
| DELETE | `/workspaces/:ws_id/documents/:id` | 删除文档 | 🔒 |
| POST | `/workspaces/:ws_id/documents/:id/reprocess` | 重新处理文档 | 🔒 |
| GET | `/workspaces/:ws_id/documents/:id/chunks` | 获取文档分块列表 | 🔒 |
| GET | `/workspaces/:ws_id/documents/:id/download` | 下载原始文件 | 🔒 |
| GET | `/workspaces/:ws_id/documents/:id/markdown` | 获取 Markdown 版本 | 🔒 |

### POST /workspaces/:ws_id/documents
```
Content-Type: multipart/form-data
file: <binary>
title: "技术规范 v2.0"  (可选，默认用文件名)
tags: ["技术", "规范"]  (可选)
```

### Response（文档对象）
```json
{
  "id": "uuid",
  "title": "技术规范 v2.0",
  "original_filename": "spec-v2.pdf",
  "file_type": "pdf",
  "file_size_bytes": 1048576,
  "page_count": 42,
  "processing_status": "pending",
  "tags": ["技术", "规范"],
  "uploaded_by": "uuid",
  "created_at": "2026-05-14T10:00:00Z"
}
```

---

## 4. 检索 `/workspaces/:ws_id/search`

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| POST | `/workspaces/:ws_id/search` | 混合检索（不生成回答） | 🔒 |

### POST /workspaces/:ws_id/search
```json
// Request
{
  "query": "用户认证的安全要求是什么？",
  "top_k": 10,
  "mode": "hybrid",          // "vector" | "fulltext" | "hybrid"
  "document_ids": ["uuid"],   // 可选，限定文档范围
  "min_score": 0.5,           // 可选，最低相关度
  "use_mmr": true,            // 可选，多样性检索
  "mmr_diversity": 0.3        // 可选
}
// Response 200
{
  "results": [
    {
      "chunk_id": "uuid",
      "document_id": "uuid",
      "document_title": "安全规范.pdf",
      "content": "原文片段...",
      "heading_path": "第3章 > 3.2 认证",
      "page_number": 12,
      "relevance_score": 0.92,
      "highlight": "...用户<em>认证</em>的<em>安全要求</em>..."
    }
  ],
  "total": 10,
  "search_time_ms": 45
}
```

---

## 5. 对话 + RAG 问答 `/workspaces/:ws_id/conversations`

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/workspaces/:ws_id/conversations` | 列出对话（分页） | 🔒 |
| POST | `/workspaces/:ws_id/conversations` | 创建新对话 | 🔒 |
| GET | `/workspaces/:ws_id/conversations/:id` | 获取对话详情+消息 | 🔒 |
| DELETE | `/workspaces/:ws_id/conversations/:id` | 删除对话 | 🔒 |
| PUT | `/workspaces/:ws_id/conversations/:id` | 更新对话标题 | 🔒 |
| POST | `/workspaces/:ws_id/conversations/:id/messages` | 发送消息（触发 RAG） | 🔒 |
| GET | `/workspaces/:ws_id/conversations/:id/messages/:msg_id/citations` | 获取消息引用 | 🔒 |

### POST /workspaces/:ws_id/conversations/:id/messages
```json
// Request
{
  "content": "这个项目的安全要求有哪些？",
  "stream": true,             // 是否流式输出
  "document_scope": ["uuid"], // 可选，限定文档范围
  "model_config_id": "uuid"   // 可选，使用特定模型
}
```

### 流式响应（SSE, stream=true）
```
Content-Type: text/event-stream

event: message_start
data: {"message_id": "uuid", "model": "gpt-4o"}

event: text_delta
data: {"delta": "根据"}

event: text_delta
data: {"delta": "安全规范文档"}

event: citation
data: {"index": 1, "chunk_id": "uuid", "document_id": "uuid", "document_title": "安全规范.pdf", "page": 12, "heading": "3.2 认证", "score": 0.92, "text": "引用原文..."}

event: text_delta
data: {"delta": " [1]，项目需要满足以下安全要求："}

event: text_delta
data: {"delta": "\n\n1. 所有用户密码必须使用..."}

event: citation
data: {"index": 2, "chunk_id": "uuid", ...}

event: message_end
data: {"message_id": "uuid", "prompt_tokens": 1200, "completion_tokens": 350, "latency_ms": 2100}
```

### 非流式响应（stream=false）
```json
// Response 200
{
  "message": {
    "id": "uuid",
    "role": "assistant",
    "content": "根据安全规范文档 [1]，项目需要满足以下安全要求：...",
    "model_name": "gpt-4o",
    "prompt_tokens": 1200,
    "completion_tokens": 350,
    "created_at": "2026-05-14T10:05:00Z"
  },
  "citations": [
    {
      "index": 1,
      "chunk_id": "uuid",
      "document_id": "uuid",
      "document_title": "安全规范.pdf",
      "quoted_text": "引用原文片段...",
      "page_number": 12,
      "heading_path": "第3章 > 3.2 认证",
      "relevance_score": 0.92,
      "verified": false
    }
  ]
}
```

---

## 6. 模型配置 `/model-configs`

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/model-configs` | 列出我的模型配置 | 🔒 |
| POST | `/model-configs` | 创建模型配置 | 🔒 |
| PUT | `/model-configs/:id` | 更新模型配置 | 🔒 |
| DELETE | `/model-configs/:id` | 删除模型配置 | 🔒 |
| POST | `/model-configs/:id/test` | 测试连接 | 🔒 |

### POST /model-configs
```json
{
  "name": "本地 Ollama",
  "provider": "ollama",
  "api_base_url": "http://localhost:11434/v1",
  "model_name": "qwen2.5:72b",
  "temperature": 0.1,
  "max_tokens": 4096,
  "is_default": true
}
```

---

## 7. 审核 `/reviews`（Phase 2）

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/workspaces/:ws_id/reviews` | 列出待审核引用 | 🔒 |
| POST | `/citations/:id/review` | 提交审核意见 | 🔒 |
| GET | `/citations/:id/reviews` | 获取引用审核历史 | 🔒 |

---

## 8. 系统管理 `/admin`

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/admin/users` | 列出所有用户 | 👑 |
| PUT | `/admin/users/:id/role` | 修改用户角色 | 👑 |
| PUT | `/admin/users/:id/status` | 修改用户状态 | 👑 |
| GET | `/admin/stats` | 系统统计 | 👑 |

---

## 9. 健康检查

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/health` | 服务健康检查 | 🔓 |
| GET | `/health/ready` | 就绪检查（含 DB/Redis） | 🔓 |

---

## 通用规范

### 分页参数
```
?page=1&per_page=20&sort=created_at&order=desc
```

### 错误响应格式
```json
{
  "error": {
    "code": "DOCUMENT_NOT_FOUND",
    "message": "文档不存在",
    "details": null
  }
}
```

### HTTP 状态码
| 码 | 含义 |
|----|------|
| 200 | 成功 |
| 201 | 创建成功 |
| 204 | 删除成功 |
| 400 | 请求参数错误 |
| 401 | 未认证 |
| 403 | 权限不足 |
| 404 | 资源不存在 |
| 409 | 冲突（如邮箱已注册） |
| 422 | 数据验证失败 |
| 429 | 请求频率超限 |
| 500 | 服务器内部错误 |
