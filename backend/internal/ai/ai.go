package ai

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"strings"

	"github.com/openai/openai-go"
	"github.com/openai/openai-go/option"
)

// AIConfig AI配置结构
type AIConfig struct {
	Model   string
	APIKey  string
	BaseURL string
}

// Client AI客户端
type Client struct {
	config AIConfig
	client *openai.Client
}

// NewClient 创建新的AI客户端
func NewClient(config AIConfig) (*Client, error) {
	if config.APIKey == "" {
		config.APIKey = os.Getenv("OPENAI_API_KEY")
		if config.APIKey == "" {
			return nil, fmt.Errorf("未提供 API Key，请在 .env 中设置 OPENAI_API_KEY")
		}
	}

	if config.Model == "" {
		config.Model = os.Getenv("AI_MODEL")
		if config.Model == "" {
			config.Model = "gpt-4o"
		}
	}

	if config.BaseURL == "" {
		config.BaseURL = os.Getenv("OPENAI_BASE_URL")
	}

	options := []option.RequestOption{
		option.WithAPIKey(config.APIKey),
	}

	if config.BaseURL != "" {
		options = append(options, option.WithBaseURL(config.BaseURL))
	}

	client := openai.NewClient(options...)

	return &Client{
		config: config,
		client: &client,
	}, nil
}

const analyzeSystemPrompt = `你是一个专业的 Git 代码差异分析助手。你的任务是根据提供的 Git diff 内容，回答用户的问题。

分析 diff 时请注意：
1. 哪些文件发生了变更
2. 具体做了什么改动
3. 改动的目的和影响范围
4. 潜在的问题或改进建议

请用中文回答，使用 Markdown 格式，让回答清晰、简洁、有结构。`

// AnalyzeDiff 分析Git差异（非流式）
func (c *Client) AnalyzeDiff(diff string, prompt string) (string, error) {
	userPrompt := fmt.Sprintf("## Git Diff:\n```\n%s\n```\n\n## 问题:\n%s", diff, prompt)

	response, err := c.client.Chat.Completions.New(context.Background(), openai.ChatCompletionNewParams{
		Model: c.config.Model,
		Messages: []openai.ChatCompletionMessageParamUnion{
			openai.SystemMessage(analyzeSystemPrompt),
			openai.UserMessage(userPrompt),
		},
		MaxCompletionTokens: openai.Int(4000),
		Temperature:         openai.Float(0.3),
	})

	if err != nil {
		return "", fmt.Errorf("AI API 调用失败: %w", err)
	}

	if len(response.Choices) == 0 {
		return "", fmt.Errorf("AI 未返回任何响应")
	}

	return response.Choices[0].Message.Content, nil
}

// AnalyzeDiffStream 流式分析Git差异
func (c *Client) AnalyzeDiffStream(diff string, prompt string) (<-chan string, <-chan error) {
	streamChan := make(chan string, 100)
	errChan := make(chan error, 1)

	userPrompt := fmt.Sprintf("## Git Diff:\n```\n%s\n```\n\n## 问题:\n%s", diff, prompt)

	go func() {
		defer close(streamChan)
		defer close(errChan)

		stream := c.client.Chat.Completions.NewStreaming(context.Background(), openai.ChatCompletionNewParams{
			Model: c.config.Model,
			Messages: []openai.ChatCompletionMessageParamUnion{
				openai.SystemMessage(analyzeSystemPrompt),
				openai.UserMessage(userPrompt),
			},
			MaxCompletionTokens: openai.Int(4000),
			Temperature:         openai.Float(0.3),
		})

		for stream.Next() {
			chunk := stream.Current()
			if len(chunk.Choices) > 0 {
				content := chunk.Choices[0].Delta.Content
				if content != "" {
					streamChan <- content
				}
			}
		}

		if err := stream.Err(); err != nil {
			errChan <- fmt.Errorf("AI 流式传输错误: %w", err)
		}
	}()

	return streamChan, errChan
}

// ParseIntent 使用 AI 解析用户意图
func (c *Client) ParseIntent(input string) (string, error) {
	systemPrompt := `你是一个 Git 意图解析助手。解析用户的自然语言输入，判断用户想执行什么 Git 操作。

可能的操作类型：
1. 比较分支: "比较当前分支和 main 分支"
2. 查看文件变更: "过去3天 customer-data 文件有什么改动"
3. 比较提交: "比较 abc123 和 def456 两个提交"

对每个输入，返回一个 JSON 对象:
- type: 操作类型 (branch_diff, file_diff, commit_diff, unknown)
- details: 操作的详细信息

示例输出:
{"type": "branch_diff", "details": {"branch1": "current", "branch2": "main"}}`

	response, err := c.client.Chat.Completions.New(context.Background(), openai.ChatCompletionNewParams{
		Model: c.config.Model,
		Messages: []openai.ChatCompletionMessageParamUnion{
			openai.SystemMessage(systemPrompt),
			openai.UserMessage(input),
		},
		MaxCompletionTokens: openai.Int(500),
		Temperature:         openai.Float(0.1),
	})

	if err != nil {
		return "", fmt.Errorf("AI API 调用失败: %w", err)
	}

	if len(response.Choices) == 0 {
		return "", fmt.Errorf("AI 未返回任何响应")
	}

	result := response.Choices[0].Message.Content
	// 清理可能的 markdown 代码块标记
	result = strings.TrimSpace(result)
	result = strings.TrimPrefix(result, "```json")
	result = strings.TrimPrefix(result, "```")
	result = strings.TrimSuffix(result, "```")
	result = strings.TrimSpace(result)

	// 验证是否为有效 JSON
	var parsed interface{}
	if err := json.Unmarshal([]byte(result), &parsed); err != nil {
		return result, nil // 即使不是 JSON 也返回原始内容
	}

	return result, nil
}
