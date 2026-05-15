# 认证

## 概述

TrustRAG 使用 Bearer Token 进行 API 认证。桌面模式下认证为可选，服务器模式下必须认证。

## 登录

```http
POST /api/auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "your-password"
}
```

**响应**：

```json
{
  "data": {
    "token": "eyJhbGciOi...",
    "expires_at": "2026-06-15T00:00:00Z"
  }
}
```

## 使用 Token

在后续请求的 Header 中携带 Token：

```http
GET /api/workspaces
Authorization: Bearer eyJhbGciOi...
```

## 桌面模式

桌面模式下，后端运行在 `localhost`，默认允许无认证访问。可在设置中开启密码保护。
