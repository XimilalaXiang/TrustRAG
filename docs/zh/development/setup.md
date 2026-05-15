# 开发环境

## 前置要求

| 工具 | 版本 | 用途 |
|------|------|------|
| Rust | 1.75+ | 后端编译 |
| Flutter | 3.22+ | 客户端开发 |
| Python | 3.11+ | 文档处理服务 |
| Node.js | 20+ | 文档站构建、Release 脚本 |
| Docker | 24+ | 服务器模式基础设施 |

## 克隆仓库

```bash
git clone https://github.com/XimilalaXiang/TrustRAG.git
cd TrustRAG
```

## 后端开发

### 桌面模式

```bash
cd backend
cargo run --features desktop --no-default-features
```

后端启动在 `http://localhost:3000`，使用 SQLite。

### 服务器模式

先启动基础设施：

```bash
cd infra
docker compose -f docker-compose.dev.yml up -d
```

然后运行后端：

```bash
cd backend
cargo run --features postgres --no-default-features
```

## Flutter 客户端

```bash
cd apps/client
flutter pub get
flutter run -d windows  # 或 linux / macos / chrome
```

## 文档处理服务

```bash
cd doc-processor
pip install -r requirements.txt
python -m app.main
```

## 文档站开发

```bash
cd docs
npm install
npm run docs:dev
```

访问 `http://localhost:5173/TrustRAG/` 预览。

## 推荐 IDE

- **VS Code / Cursor** — 安装 rust-analyzer、Flutter、Python 扩展
- **IntelliJ** — 安装 Rust 和 Flutter 插件
