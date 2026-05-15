# API 概览

TrustRAG 后端提供 RESTful API，桌面模式和服务器模式对外暴露相同的接口。

## 基本信息

| 项目 | 值 |
|------|-----|
| 协议 | HTTP |
| 格式 | JSON |
| 默认端口 | 3000（后端）/ 8080（Web UI） |
| 认证 | Bearer Token |

## API 模块

| 模块 | 路径前缀 | 说明 |
|------|---------|------|
| [认证](/zh/api/authentication) | `/api/auth` | 登录、Token 管理 |
| [工作区](/zh/api/workspaces) | `/api/workspaces` | 工作区 CRUD |
| [文档](/zh/api/documents) | `/api/workspaces/:id/documents` | 文档上传与管理 |
| [对话](/zh/api/chat) | `/api/workspaces/:id/chats` | 对话与消息 |
| [引用](/zh/api/citations) | `/api/citations` | 引用查询与审核 |

## 通用响应格式

### 成功

```json
{
  "data": { ... },
  "message": "success"
}
```

### 错误

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Workspace not found"
  }
}
```

## HTTP 状态码

| 状态码 | 含义 |
|--------|------|
| 200 | 成功 |
| 201 | 创建成功 |
| 400 | 请求参数错误 |
| 401 | 未认证 |
| 404 | 资源不存在 |
| 500 | 服务器内部错误 |
