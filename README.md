# GitX - AI Git Diff Analyzer

一个基于 AI 的 Git 差异分析工具，支持仓库浏览、分支比较、文件历史追踪和智能代码变更解读。

## 功能特性

- 📂 **仓库管理**：打开本地 Git 仓库，浏览分支结构
- 🔀 **分支比较**：Side-by-Side 可视化对比任意两个分支的差异
- 📝 **文件历史**：追踪指定文件在时间范围内的变更记录
- 🤖 **AI 分析**：基于大语言模型的智能代码差异解读
- 💬 **自然语言交互**：通过自然语言命令执行 Git 操作

## 技术栈

| 层级 | 技术 |
|------|------|
| 后端 | Go + Gin + go-git + OpenAI API |
| 前端 | Vue 3 + TypeScript + Vite + diff2html |
| 样式 | CSS 自定义属性（深色主题） |

## 项目结构

```
gitx/
├── backend/
│   ├── cmd/
│   │   └── main.go           # 服务入口
│   ├── internal/
│   │   ├── ai/ai.go          # AI 客户端
│   │   ├── git/git.go        # Git 操作
│   │   └── intent/intent.go  # 意图解析
│   ├── .env.example          # 环境变量模板
│   └── go.mod
├── frontend/
│   ├── src/
│   │   ├── App.vue           # 根组件
│   │   ├── main.ts           # 入口
│   │   ├── style.css         # 全局样式
│   │   ├── types/            # 类型定义
│   │   ├── api/              # API 封装
│   │   ├── composables/      # 组合式函数
│   │   └── components/       # UI 组件
│   ├── index.html
│   ├── vite.config.ts
│   └── package.json
└── .gitignore
```

## 快速开始

### 前置条件

- Go 1.21+
- Node.js 18+
- OpenAI API Key（或兼容的 API 端点）

### 后端

```bash
cd backend
cp .env.example .env
# 编辑 .env 填入你的 API Key
go mod tidy
go run cmd/main.go
```

### 前端

```bash
cd frontend
npm install
npm run dev
```

开发模式下前端默认运行在 `http://localhost:5173`，API 请求会代理到后端 `http://localhost:8080`。

## 配置

在 `backend/.env` 中配置：

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `PORT` | 服务端口 | `8080` |
| `AI_MODEL` | AI 模型名称 | `gpt-4o` |
| `OPENAI_API_KEY` | API 密钥 | - |
| `OPENAI_BASE_URL` | 自定义 API 端点（可选） | - |

## License

MIT
