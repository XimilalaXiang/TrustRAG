# Getting Started

## Desktop App (Recommended)

The fastest way to use TrustRAG — download and run.

### 1. Download

Go to [GitHub Releases](https://github.com/XimilalaXiang/TrustRAG/releases/latest) and download the installer for your platform:

| Platform | File | Notes |
|----------|------|-------|
| Windows | `TrustRAG-Setup-Windows-x64.exe` | One-click installer |
| Windows | `trustrag-windows-x64-portable.zip` | No install needed |
| macOS | `trustrag-macos.tar.gz` | Unsigned — see note below |
| Linux | `trustrag-linux-x64.tar.gz` | Extract and run |
| Android | `app-release.apk` | Enable "Install from unknown sources" |

### 2. Install and Launch

- **Windows**: Run the `.exe` installer, or extract the portable zip
- **macOS**: Extract and move to Applications. Right-click → Open (unsigned app)
- **Linux**: Extract and run the `trustrag` binary

### 3. Start Using

1. The app creates a local user automatically on first launch
2. Create a workspace
3. Configure an LLM provider (OpenAI-compatible, Ollama, etc.)
4. Upload documents (PDF, DOCX, Markdown, TXT)
5. Start chatting with your documents

::: warning macOS Users
TrustRAG is not signed by Apple. On first launch, right-click the app and select "Open", or go to System Settings → Privacy & Security → Allow.
:::

::: warning Windows Users
Windows SmartScreen may show a warning on first launch. Click **More info** → **Run anyway**.
:::

## Server Mode (Docker Compose)

For teams needing PostgreSQL vector search, Redis, and MinIO:

```bash
# Clone the repository
git clone https://github.com/XimilalaXiang/TrustRAG.git
cd TrustRAG

# Copy environment template
cp .env.example .env

# Build Flutter web client
cd apps/client
flutter pub get
flutter build web --dart-define=API_BASE_URL=/api
cd ../..

# Start all services
cd infra
docker compose up --build
```

Open [http://localhost](http://localhost) to access the web app.
