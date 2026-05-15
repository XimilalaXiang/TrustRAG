# 构建与发布

## CI/CD 工作流

TrustRAG 使用三个 GitHub Actions 工作流：

| 工作流 | 触发条件 | 用途 |
|--------|---------|------|
| `ci.yml` | Push / PR | Rust `cargo check` + Flutter `analyze` |
| `test-build.yml` | Push 到 `test` | 全平台构建（产物保留 7 天） |
| `release.yml` | Tag `v*` | 构建 → 生成 Release Notes → 发布 GitHub Release |

## 分支模型

```text
feature/* ──► test ──► master (tag vX.Y.Z)
```

- **`test`**：集成构建、QA 验证
- **`master`**：仅限稳定版本，通过版本 Tag 触发发布

## 本地构建

### 后端（Rust）

```bash
# 桌面模式（SQLite + FTS5）
cargo build --release --features desktop --no-default-features

# 服务器模式（PostgreSQL + pgvector）
cargo build --release --features postgres --no-default-features
```

### Flutter 客户端

```bash
cd apps/client

# 桌面（当前平台）
flutter build windows   # 或 linux / macos
flutter build apk       # Android
flutter build ios        # iOS（需要 macOS + Xcode）
```

### Windows 安装程序（Inno Setup）

Windows 版本包含一键安装程序，由 Inno Setup 生成：

```bash
# 需要 Windows 环境 + Inno Setup 6+
iscc apps/client/windows/installer.iss
```

## 发布流程

1. 在 `CHANGELOG.md` 顶部添加新的 `## [x.y.z] - YYYY-MM-DD` 章节
2. 提交到 `master` 并打 Tag：`git tag v0.2.0 && git push --tags`
3. `release.yml` 自动执行：
   - 构建所有平台（Windows、macOS、Linux、Android、iOS）
   - 运行 `scripts/generate-release-notes.mjs` 生成 Release Notes
   - 发布 GitHub Release 并上传所有产物

### 预发布版本检测

Tag 中包含 `-alpha`、`-beta` 或 `-rc` 会自动标记为预发布版本：

- `v0.2.0` → 正式版
- `v0.2.0-beta.1` → 预发布版
