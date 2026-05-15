# Build & Release

## CI/CD Workflows

TrustRAG uses three GitHub Actions workflows:

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `ci.yml` | Push / PR | Rust `cargo check` + Flutter `analyze` |
| `test-build.yml` | Push to `test` | Full multi-platform build (artifacts kept 7 days) |
| `release.yml` | Tag `v*` | Build → generate release notes → publish GitHub Release |

## Branching Model

```text
feature/* ──► test ──► master (tagged vX.Y.Z)
```

- **`test`**: integration builds, QA validation
- **`master`**: stable releases only, triggered by version tags

## Building Locally

### Backend (Rust)

```bash
# Desktop mode (SQLite + FTS5)
cargo build --release --features desktop --no-default-features

# Server mode (PostgreSQL + pgvector)
cargo build --release --features postgres --no-default-features
```

### Flutter Client

```bash
cd apps/client

# Desktop (current platform)
flutter build windows   # or linux / macos
flutter build apk       # Android
flutter build ios        # iOS (requires macOS + Xcode)
```

### Windows Installer (Inno Setup)

The Windows release includes a one-click installer generated with Inno Setup:

```bash
# Requires Inno Setup 6+ on Windows
iscc apps/client/windows/installer.iss
```

## Release Process

1. Update `CHANGELOG.md` with a new `## [x.y.z] - YYYY-MM-DD` section
2. Commit to `master` and tag: `git tag v0.2.0 && git push --tags`
3. `release.yml` runs automatically:
   - Builds all platforms (Windows, macOS, Linux, Android, iOS)
   - Runs `scripts/generate-release-notes.mjs` to create release notes
   - Publishes GitHub Release with all artifacts

### Pre-release Detection

Tags containing `-alpha`, `-beta`, or `-rc` are automatically marked as pre-release:

- `v0.2.0` → stable release
- `v0.2.0-beta.1` → pre-release
