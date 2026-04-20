package ai

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/openai/openai-go"
)

const maxToolRounds = 5
const aiCallTimeout = 120 * time.Second

const systemPromptBase = `你是一个专业的 Git 代码分析助手，名叫 GitX AI。你可以帮助用户浏览 Git 仓库、比较分支差异、查看提交历史和文件变更记录。

工作方式：
1. 根据用户的问题，使用提供的工具获取 Git 数据
2. 基于获取到的数据，给出专业、清晰的回答
3. 如果需要多个工具配合使用，请依次调用

输出格式要求（严格遵守）：
- 使用 Markdown 格式，结构清晰
- 提交列表：用有序列表，每项包含 **提交哈希**（短哈希）、提交信息、作者、时间，用 - 子项排列
- 差异分析：先总结变更概览，再按文件逐个说明
- 代码内容：用代码块包裹，标注语言类型
- 关键信息：用 **加粗** 标注
- 简洁为主，避免冗余描述

请用中文回答。`

type InputMessage struct {
	Role    string `json:"role"`
	Content string `json:"content"`
}

type ChatContext struct {
	BaseBranch    string `json:"base_branch"`
	CompareBranch string `json:"compare_branch"`
	HasDiff       bool   `json:"has_diff"`
}

type StreamEvent struct {
	Type    string `json:"type"`
	Content string `json:"content,omitempty"`
	Name    string `json:"name,omitempty"`
	Display string `json:"display,omitempty"`
}

var toolDisplayNames = map[string]string{
	"get_branches":       "正在获取分支列表...",
	"get_current_branch": "正在获取当前分支...",
	"get_branch_diff":    "正在比较分支差异...",
	"get_commits":        "正在获取提交历史...",
	"get_file_history":   "正在获取文件变更记录...",
	"get_diff":           "正在获取代码差异...",
}

func buildSystemPrompt(chatCtx *ChatContext) string {
	p := systemPromptBase
	if chatCtx != nil && chatCtx.HasDiff && chatCtx.BaseBranch != "" && chatCtx.CompareBranch != "" {
		p += fmt.Sprintf(
			"\n\n当前用户正在查看分支 %s 和 %s 之间的代码差异。当用户提到「分析当前差异」或「这些变更」时，请使用 get_branch_diff 工具获取这两个分支的差异进行分析。",
			chatCtx.BaseBranch, chatCtx.CompareBranch,
		)
	}
	return p
}

func (c *Client) AgentChat(ctx context.Context, history []InputMessage, chatCtx *ChatContext) <-chan StreamEvent {
	events := make(chan StreamEvent, 200)

	go func() {
		defer close(events)

		callCtx, cancel := context.WithTimeout(ctx, aiCallTimeout)
		defer cancel()

		msgs := []openai.ChatCompletionMessageParamUnion{
			openai.SystemMessage(buildSystemPrompt(chatCtx)),
		}
		for _, m := range history {
			switch m.Role {
			case "user":
				msgs = append(msgs, openai.UserMessage(m.Content))
			case "assistant":
				msgs = append(msgs, openai.AssistantMessage(m.Content))
			}
		}

		for round := 0; round < maxToolRounds; round++ {
			resp, err := c.client.Chat.Completions.New(callCtx, openai.ChatCompletionNewParams{
				Model:    c.config.Model,
				Messages: msgs,
				Tools:    GetToolDefs(),
			})
			if err != nil {
				events <- StreamEvent{Type: "error", Content: fmt.Sprintf("AI 调用失败: %v", err)}
				return
			}
			if len(resp.Choices) == 0 {
				events <- StreamEvent{Type: "error", Content: "AI 未返回响应"}
				return
			}

			msg := resp.Choices[0].Message

			if len(msg.ToolCalls) > 0 {
				msgs = append(msgs, msg.ToParam())

				for _, tc := range msg.ToolCalls {
					display, ok := toolDisplayNames[tc.Function.Name]
					if !ok {
						display = fmt.Sprintf("正在执行 %s...", tc.Function.Name)
					}
					events <- StreamEvent{Type: "tool", Name: tc.Function.Name, Display: display}

					result, err := CallTool(tc.Function.Name, tc.Function.Arguments)
					if err != nil {
						result = fmt.Sprintf("工具执行失败: %v", err)
					}
					msgs = append(msgs, openai.ToolMessage(result, tc.ID))
				}
				continue
			}

			stream := c.client.Chat.Completions.NewStreaming(callCtx, openai.ChatCompletionNewParams{
				Model:    c.config.Model,
				Messages: msgs,
			})
			var buf strings.Builder
			flushBuf := func() {
				if buf.Len() > 0 {
					events <- StreamEvent{Type: "content", Content: buf.String()}
					buf.Reset()
				}
			}
			for stream.Next() {
				chunk := stream.Current()
				if len(chunk.Choices) > 0 {
					if content := chunk.Choices[0].Delta.Content; content != "" {
						buf.WriteString(content)
						if buf.Len() >= 16 {
							flushBuf()
						}
					}
				}
			}
			flushBuf()
			if err := stream.Err(); err != nil {
				events <- StreamEvent{Type: "error", Content: fmt.Sprintf("流式传输错误: %v", err)}
			}
			return
		}

		events <- StreamEvent{Type: "content", Content: "抱歉，操作步骤过多，请尝试更具体的描述。"}
	}()

	return events
}
