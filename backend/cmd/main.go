package main

import (
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"strings"

	"github.com/gin-contrib/cors"
	"github.com/gin-gonic/gin"
	"github.com/joho/godotenv"

	"gitx/internal/ai"
	"gitx/internal/git"
	"gitx/internal/intent"
)

// aiClient is the singleton AI client, initialized once at startup.
var aiClient *ai.Client

func main() {
	// 加载环境变量
	if err := godotenv.Overload(); err != nil {
		log.Println("No .env file found, using environment variables")
	}

	// 初始化 AI 客户端（单例）
	var err error
	aiClient, err = ai.NewClient(ai.AIConfig{
		Model:   os.Getenv("AI_MODEL"),
		APIKey:  os.Getenv("OPENAI_API_KEY"),
		BaseURL: os.Getenv("OPENAI_BASE_URL"),
	})
	if err != nil {
		log.Fatalf("AI client init failed: %v", err)
	}

	// 初始化 AI 工具
	ai.InitGitTools()

	// 设置Gin模式
	gin.SetMode(gin.ReleaseMode)

	r := gin.Default()

	// 配置CORS
	r.Use(cors.New(cors.Config{
		AllowOrigins:     []string{"*"},
		AllowMethods:     []string{"GET", "POST", "PUT", "DELETE", "OPTIONS"},
		AllowHeaders:     []string{"Origin", "Content-Type", "Accept", "Authorization"},
		ExposeHeaders:    []string{"Content-Length"},
		AllowCredentials: true,
	}))

	// 健康检查
	r.GET("/health", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"status": "ok"})
	})

	// Git 路由
	gitGroup := r.Group("/api/git")
	{
		gitGroup.POST("/open", openGitRepo)
		gitGroup.GET("/branches", getBranches)
		gitGroup.GET("/branches/current", getCurrentBranch)
		gitGroup.GET("/commits", getCommits)
		gitGroup.POST("/diff", getDiff)
		gitGroup.POST("/branch-diff", getBranchDiff)
		gitGroup.GET("/file-history", getFileHistory)
	}

	// AI 路由
	aiGroup := r.Group("/api/ai")
	{
		aiGroup.POST("/chat", agentChat)
		aiGroup.POST("/analyze", analyzeDiff)
		aiGroup.POST("/analyze-stream", analyzeDiffStream)
		aiGroup.POST("/parse-intent", parseIntent)
	}

	// 静态文件服务
	r.Static("/assets", "./frontend/dist/assets")
	r.Static("/public", "./frontend/dist/public")
	r.NoRoute(func(c *gin.Context) {
		// SPA fallback: 所有非 API 路由返回 index.html
		if !strings.HasPrefix(c.Request.URL.Path, "/api/") {
			c.File("./frontend/dist/index.html")
			return
		}
		c.JSON(http.StatusNotFound, gin.H{"error": "not found"})
	})

	// 启动服务器
	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	fmt.Printf("🚀 GitX server running on http://localhost:%s\n", port)
	if err := r.Run(":" + port); err != nil {
		log.Fatalf("Failed to start server: %v", err)
	}
}

// openGitRepo 打开Git仓库
func openGitRepo(c *gin.Context) {
	var req struct {
		Path string `json:"path" binding:"required"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "请提供仓库路径"})
		return
	}

	if err := git.OpenRepo(req.Path); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": fmt.Sprintf("打开仓库失败: %v", err)})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "仓库打开成功", "path": req.Path})
}

// getBranches 获取分支列表
func getBranches(c *gin.Context) {
	branches, err := git.GetBranches()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": fmt.Sprintf("获取分支失败: %v", err)})
		return
	}

	if branches == nil {
		branches = []string{}
	}

	c.JSON(http.StatusOK, gin.H{"branches": branches})
}

// getCurrentBranch 获取当前分支
func getCurrentBranch(c *gin.Context) {
	branch, err := git.GetCurrentBranch()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": fmt.Sprintf("获取当前分支失败: %v", err)})
		return
	}
	c.JSON(http.StatusOK, gin.H{"current_branch": branch})
}

// getCommits 获取提交历史
func getCommits(c *gin.Context) {
	branch := c.Query("branch")
	var limit int
	if _, err := fmt.Sscanf(c.DefaultQuery("limit", "20"), "%d", &limit); err != nil || limit <= 0 {
		limit = 20
	}

	commits, err := git.GetCommits(branch, limit)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": fmt.Sprintf("获取提交历史失败: %v", err)})
		return
	}

	if commits == nil {
		commits = []git.CommitInfo{}
	}

	c.JSON(http.StatusOK, gin.H{"commits": commits})
}

// getDiff 获取差异
func getDiff(c *gin.Context) {
	var req struct {
		From string `json:"from" binding:"required"`
		To   string `json:"to" binding:"required"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "请提供 from 和 to 提交哈希"})
		return
	}

	diff, err := git.GetDiff(req.From, req.To)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": fmt.Sprintf("获取差异失败: %v", err)})
		return
	}

	if diff == nil {
		diff = []git.DiffInfo{}
	}

	c.JSON(http.StatusOK, gin.H{"diff": diff})
}

// getBranchDiff 获取分支差异
func getBranchDiff(c *gin.Context) {
	var req struct {
		Branch1 string `json:"branch1" binding:"required"`
		Branch2 string `json:"branch2" binding:"required"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "请提供两个分支名称"})
		return
	}

	diff, err := git.GetBranchDiff(req.Branch1, req.Branch2)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": fmt.Sprintf("获取分支差异失败: %v", err)})
		return
	}

	if diff == nil {
		diff = []git.DiffInfo{}
	}

	c.JSON(http.StatusOK, gin.H{"diff": diff})
}

// getFileHistory 获取文件历史
func getFileHistory(c *gin.Context) {
	filePath := c.Query("file")
	timeRange := c.DefaultQuery("timeRange", "3d")

	if filePath == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "请提供文件路径"})
		return
	}

	startTime, err := intent.GetTimeRangeStart(timeRange)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": fmt.Sprintf("时间范围格式错误: %v", err)})
		return
	}

	commits, err := git.GetFileHistory(filePath, startTime)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": fmt.Sprintf("获取文件历史失败: %v", err)})
		return
	}

	if commits == nil {
		commits = []git.CommitInfo{}
	}

	c.JSON(http.StatusOK, gin.H{"commits": commits})
}

// analyzeDiff 分析差异（非流式）
func analyzeDiff(c *gin.Context) {
	var req struct {
		Diff   []git.DiffInfo `json:"diff" binding:"required"`
		Prompt string         `json:"prompt" binding:"required"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "请提供差异数据和提示词"})
		return
	}

	var diffBuilder strings.Builder
	for _, d := range req.Diff {
		diffBuilder.WriteString(fmt.Sprintf("File: %s\n", d.File))
		diffBuilder.WriteString(fmt.Sprintf("Added: %d, Deleted: %d\n", d.Added, d.Deleted))
		diffBuilder.WriteString("Patch:\n")
		diffBuilder.WriteString(d.Patch)
		diffBuilder.WriteString("\n\n")
	}

	analysis, err := aiClient.AnalyzeDiff(diffBuilder.String(), req.Prompt)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": fmt.Sprintf("AI 分析失败: %v", err)})
		return
	}

	c.JSON(http.StatusOK, gin.H{"analysis": analysis})
}

// analyzeDiffStream 流式分析差异（SSE）
func analyzeDiffStream(c *gin.Context) {
	var req struct {
		Diff   []git.DiffInfo `json:"diff" binding:"required"`
		Prompt string         `json:"prompt" binding:"required"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "请提供差异数据和提示词"})
		return
	}

	var diffBuilder strings.Builder
	for _, d := range req.Diff {
		diffBuilder.WriteString(fmt.Sprintf("File: %s\n", d.File))
		diffBuilder.WriteString(fmt.Sprintf("Added: %d, Deleted: %d\n", d.Added, d.Deleted))
		diffBuilder.WriteString("Patch:\n")
		diffBuilder.WriteString(d.Patch)
		diffBuilder.WriteString("\n\n")
	}

	// 设置 SSE 响应头
	c.Header("Content-Type", "text/event-stream")
	c.Header("Cache-Control", "no-cache")
	c.Header("Connection", "keep-alive")
	c.Header("X-Accel-Buffering", "no")

	// 创建流式 channel
	streamChan, errChan := aiClient.AnalyzeDiffStream(diffBuilder.String(), req.Prompt)

	c.Stream(func(w io.Writer) bool {
		select {
		case chunk, ok := <-streamChan:
			if !ok {
				// 流结束，发送 [DONE]
				c.SSEvent("", "[DONE]")
				return false
			}
			c.SSEvent("message", chunk)
			return true
		case err, ok := <-errChan:
			if ok && err != nil {
				c.SSEvent("error", err.Error())
			}
			return false
		}
	})
}

// agentChat Agent 对话（SSE 流式）
func agentChat(c *gin.Context) {
	var req struct {
		Messages []ai.InputMessage `json:"messages" binding:"required"`
		Context  *ai.ChatContext    `json:"context"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "请提供消息列表"})
		return
	}

	c.Header("Content-Type", "text/event-stream")
	c.Header("Cache-Control", "no-cache")
	c.Header("Connection", "keep-alive")
	c.Header("X-Accel-Buffering", "no")
	c.Writer.WriteHeaderNow()
	c.Writer.Flush()

	eventCh := aiClient.AgentChat(c.Request.Context(), req.Messages, req.Context)

		// writeSSE writes a single SSE event and flushes immediately.
	// Multi-line data is split into multiple data: fields per SSE spec.
	writeSSE := func(eventType, data string) {
		c.Writer.WriteString("event:" + eventType + "\n")
		for _, line := range strings.Split(data, "\n") {
			c.Writer.WriteString("data:" + line + "\n")
		}
		c.Writer.WriteString("\n")
		c.Writer.Flush()
	}

	for {
		event, ok := <-eventCh
		if !ok {
			writeSSE("done", "")
			return
		}
		switch event.Type {
		case "tool":
			toolJSON, _ := json.Marshal(map[string]string{"name": event.Name, "display": event.Display})
			writeSSE("tool", string(toolJSON))
		case "content":
			writeSSE("message", event.Content)
		case "error":
			writeSSE("error", event.Content)
			return
		}
	}
}

// parseIntent 解析用户输入意图
func parseIntent(c *gin.Context) {
	var req struct {
		Input string `json:"input" binding:"required"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "请提供输入内容"})
		return
	}

	parsedIntent := intent.ParseIntent(req.Input)

	var action string
	switch parsedIntent.Type {
	case intent.IntentTypeDiffFile:
		action = fmt.Sprintf("查看文件 %s 在过去 %s 的改动", parsedIntent.File, parsedIntent.TimeRange)
	case intent.IntentTypeDiffBranch:
		action = fmt.Sprintf("比较分支 %s 和 %s 的差异", parsedIntent.Branch1, parsedIntent.Branch2)
	case intent.IntentTypeDiffCommit:
		action = fmt.Sprintf("比较提交 %s 和 %s 的差异", parsedIntent.Commit1, parsedIntent.Commit2)
	default:
		action = "未识别的操作"
	}

	c.JSON(http.StatusOK, gin.H{
		"intent": parsedIntent,
		"action": action,
	})
}
