<p align="center">
  <img src="docs/gitx-logo.svg" width="96" height="96" alt="GitX Logo" />
</p>

<h1 align="center">GitX</h1>

<p align="center">
  <strong>AI-Powered Git Diff Analyzer</strong><br/>
  Understand your code changes instantly with intelligent diff analysis,<br/>
  commit insights, and conversational code review.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.80+-000000?style=flat-square&logo=rust" alt="Rust" />
  <img src="https://img.shields.io/badge/Tauri-2.0-24C8D8?style=flat-square" alt="Tauri" />
  <img src="https://img.shields.io/badge/Vue-3.5-4FC08D?style=flat-square&logo=vue.js" alt="Vue" />
  <img src="https://img.shields.io/badge/TypeScript-6.0-3178C6?style=flat-square&logo=typescript" alt="TypeScript" />
  <img src="https://img.shields.io/badge/License-MIT-yellow?style=flat-square" alt="MIT" />
  <img src="https://img.shields.io/badge/PRs-Welcome-brightgreen?style=flat-square" alt="PRs Welcome" />
</p>

<p align="center">
  <a href="#features">Features</a> &middot;
  <a href="#architecture">Architecture</a> &middot;
  <a href="#quick-start">Quick Start</a> &middot;
  <a href="#configuration">Configuration</a>
</p>

---

## Features

| Feature | Description |
|---------|-------------|
| **Branch Comparison** | Side-by-side visual diff of any two branches with syntax highlighting and stats |
| **AI-Powered Analysis** | Ask questions in natural language, get context-aware explanations of code changes |
| **Commit History** | Browse commit history across branches with metadata, timestamps, and author info |
| **File History** | Track changes to specific files over any time range |
| **Agent Chat** | Multi-turn conversation with AI assistant that can call Git tools autonomously |
| **Streaming Responses** | Real-time streaming with tool status indicators and Markdown rendering |
| **Privacy First** | Desktop app runs locally — your code never leaves your machine. Works with any OpenAI-compatible API |

---

## Architecture

<p align="center">
  <img src="docs/gitx-arch.svg" alt="GitX Architecture" width="760" />
</p>

```
Tauri Window
  ├── Vue 3 Frontend (via Tauri invoke/listen IPC)
  │     ├── Sidebar (repo browser, branch selector)
  │     ├── DiffViewer (syntax-highlighted diffs)
  │     ├── CommitList (timeline history)
  │     └── AIPanel (streaming chat interface)
  │
  └── Rust Backend
        ├── Git Engine (git2 / libgit2)
        ├── AI Agent (reqwest + OpenAI-compatible API + Function Calling)
        ├── Intent Parser (NL → Git actions)
        └── Tauri Events (streaming chunks to frontend)
```

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| **Desktop Shell** | Tauri 2 · Rust |
| **Backend** | Rust · git2 (libgit2) · reqwest · tokio |
| **Frontend** | Vue 3 · TypeScript · Vite · diff2html · highlight.js |
| **AI** | Function Calling · Tool Use · Streaming via Tauri Events |
| **Style** | CSS Custom Properties (Dark Theme) |

---

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) 1.80+
- [Node.js](https://nodejs.org/) 18+
- An OpenAI-compatible API key

### Build & Run

```bash
# Install frontend dependencies
cd frontend
npm install

# Configure your AI API key
cp ../src-tauri/.env.example ../src-tauri/.env
# Edit src-tauri/.env and add your API key

# Run in development mode (from project root)
cd ..
cargo tauri dev
```

### Build for Production

```bash
cargo tauri build
```

This produces platform-specific installers (`.dmg` on macOS, `.msi` on Windows, `.AppImage` on Linux).

---

## Configuration

Environment variables in `src-tauri/.env`:

| Variable | Description | Default |
|----------|-------------|---------|
| `OPENAI_API_KEY` | API key (required) | — |
| `OPENAI_BASE_URL` | Custom API endpoint (optional) | — |
| `AI_MODEL` | AI model name | `glm-4-flash` |

---

## Project Structure

```
gitx/
├── src-tauri/                     # Tauri desktop app (Rust backend)
│   ├── src/
│   │   ├── lib.rs                 # Tauri commands, state management
│   │   ├── git.rs                 # Git operations via git2
│   │   ├── ai.rs                  # AI client with streaming
│   │   ├── tools.rs               # 6 Git function tools for agent
│   │   └── intent.rs              # Natural language intent parser
│   ├── Cargo.toml
│   └── tauri.conf.json
├── frontend/
│   └── src/
│       ├── App.vue                # Root component
│       ├── api/index.ts           # Tauri IPC client
│       ├── components/
│       │   ├── AIPanel.vue        # AI chat with streaming Markdown
│       │   ├── DiffViewer.vue     # Syntax-highlighted diff view
│       │   ├── Sidebar.vue        # Branch selector & repo browser
│       │   ├── CommitList.vue     # Commit history panel
│       │   └── FileHistoryList.vue
│       └── types/index.ts         # TypeScript type definitions
├── docs/                          # Logo and architecture assets
└── .github/workflows/build.yml   # CI: multi-platform Tauri build + release
```

---

## License

[MIT](LICENSE)
