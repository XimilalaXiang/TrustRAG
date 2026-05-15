# 快速开始

## 桌面版安装（推荐）

### Windows

1. 从 [Releases](https://github.com/XimilalaXiang/TrustRAG/releases/latest) 下载 `TrustRAG-Setup-x.y.z.exe`
2. 运行安装程序
3. 启动 TrustRAG — 后端自动启动，数据存储在本地

### macOS

1. 下载 `.dmg` 文件
2. 拖拽到 Applications 文件夹
3. 首次运行时需要在系统设置中允许打开

### Linux

1. 下载 `.tar.gz`
2. 解压并运行

```bash
tar -xzf TrustRAG-linux-x64.tar.gz
cd TrustRAG
./trustrag
```

## 服务器模式部署

### 前置要求

- Docker & Docker Compose
- Git

### 部署步骤

```bash
git clone https://github.com/XimilalaXiang/TrustRAG.git
cd TrustRAG/infra
docker compose up -d
```

服务启动后，访问 `http://localhost:8080` 进入 Web 界面。

## 基本使用流程

1. **创建工作区** — 为不同的知识领域创建独立工作区
2. **上传文档** — 支持 PDF、DOCX、TXT 格式
3. **开始对话** — AI 会基于文档内容生成带引用的回答
4. **验证引用** — 点击引用标记跳转到原文段落
