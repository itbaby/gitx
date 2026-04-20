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
  <img src="https://img.shields.io/badge/Go-1.21+-00ADD8?style=flat-square&logo=go" alt="Go" />
  <img src="https://img.shields.io/badge/Vue-3.5-4FC08D?style=flat-square&logo=vue.js" alt="Vue" />
  <img src="https://img.shields.io/badge/TypeScript-6.0-3178C6?style=flat-square&logo=typescript" alt="TypeScript" />
  <img src="https://img.shields.io/badge/License-MIT-yellow?style=flat-square" alt="MIT" />
  <img src="https://img.shields.io/badge/PRs-Welcome-brightgreen?style=flat-square" alt="PRs Welcome" />
</p>

<p align="center">
  <a href="#features">Features</a> &middot;
  <a href="#architecture">Architecture</a> &middot;
  <a href="#quick-start">Quick Start</a> &middot;
  <a href="#configuration">Configuration</a> &middot;
  <a href="#api-reference">API</a>
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
| **SSE Streaming** | Real-time streaming responses with tool status, chunk buffering, and Markdown output |
| **Self-Hosted** | Runs locally — your code never leaves your machine. Works with any OpenAI-compatible API |

---

## Architecture

<p align="center">
  <img src="docs/gitx-arch.svg" alt="GitX Architecture" width="760" />
</p>

```
Browser  ──SSE/REST──►  Go Backend
  │                       ├── Git Engine (go-git)
  │                       ├── AI Agent (OpenAI API + Function Calling)
  │                       ├── SSE Streaming (Chunk Buffered)
  │                       └── Intent Parser (NL → Git actions)
  │
  └── Vite Dev Server proxies /api → :8080
```

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| **Backend** | Go 1.21 · Gin · go-git · OpenAI-compatible API |
| **Frontend** | Vue 3 · TypeScript · Vite · diff2html · highlight.js |
| **AI** | Function Calling · Tool Use · SSE Streaming |
| **Style** | CSS Custom Properties (Dark Theme) |

---

## Quick Start

### Prerequisites

- Go 1.21+
- Node.js 18+
- An OpenAI-compatible API key

### Backend

```bash
cd backend
cp .env.example .env      # Edit .env and add your API key
go mod tidy
go run cmd/main.go         # Starts on http://localhost:8080
```

### Frontend

```bash
cd frontend
npm install
npm run dev                # Starts on http://localhost:5173
```

The frontend dev server proxies all `/api` requests to the backend automatically.

---

## Configuration

Environment variables in `backend/.env`:

| Variable | Description | Default |
|----------|-------------|---------|
| `PORT` | Server port | `8080` |
| `AI_MODEL` | AI model name | `gpt-4o` |
| `OPENAI_API_KEY` | API key (required) | — |
| `OPENAI_BASE_URL` | Custom API endpoint (optional) | — |

---

## API Reference

### Git Operations

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/git/open` | Open a local Git repository |
| `GET` | `/api/git/branches` | List all local branches |
| `GET` | `/api/git/branches/current` | Get current branch name |
| `GET` | `/api/git/commits?branch=&limit=` | Get commit history |
| `POST` | `/api/git/diff` | Get diff between two commits |
| `POST` | `/api/git/branch-diff` | Get diff between two branches |
| `GET` | `/api/git/file-history?file=&timeRange=` | Get file change history |

### AI Operations

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/ai/chat` | Agent chat (SSE streaming with tool calling) |
| `POST` | `/api/ai/analyze` | Analyze diff (non-streaming) |
| `POST` | `/api/ai/analyze-stream` | Analyze diff (SSE streaming) |
| `POST` | `/api/ai/parse-intent` | Parse natural language to Git intent |

---

## Project Structure

```
gitx/
├── backend/
│   ├── cmd/main.go              # Server entry, singleton AI client
│   └── internal/
│       ├── ai/
│       │   ├── agent.go         # Agent chat with chunk buffering
│       │   ├── ai.go            # AI client (OpenAI-compatible)
│       │   └── tools.go         # 6 Git function tools
│       ├── git/git.go           # Repository operations via go-git
│       └── intent/intent.go     # Natural language intent parser
├── frontend/
│   └── src/
│       ├── App.vue              # Root component
│       ├── api/index.ts         # SSE client with multi-line parser
│       ├── components/
│       │   ├── AIPanel.vue      # AI chat with streaming Markdown
│       │   ├── DiffViewer.vue   # Syntax-highlighted diff view
│       │   ├── Sidebar.vue      # Branch selector & repo browser
│       │   ├── CommitList.vue   # Commit history panel
│       │   └── FileHistoryList.vue
│       └── types/index.ts       # TypeScript type definitions
├── site/                        # Landing page (Vercel)
└── .github/workflows/build.yml  # CI: 5-platform build + release
```

---

## License

[MIT](LICENSE)
