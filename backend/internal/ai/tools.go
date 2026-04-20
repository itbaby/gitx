package ai

import (
	"encoding/json"
	"fmt"
	"strings"

	"gitx/internal/git"
	"gitx/internal/intent"

	"github.com/openai/openai-go"
	"github.com/openai/openai-go/shared"
)

const maxPatchChars = 2000
const maxResultChars = 25000

type Tool struct {
	Name        string
	Description string
	Parameters  shared.FunctionParameters
	Execute     func(args map[string]any) (string, error)
}

var registry []Tool

func RegisterTools(tools ...Tool) {
	registry = append(registry, tools...)
}

func GetToolDefs() []openai.ChatCompletionToolParam {
	defs := make([]openai.ChatCompletionToolParam, 0, len(registry))
	for _, t := range registry {
		defs = append(defs, openai.ChatCompletionToolParam{
			Function: shared.FunctionDefinitionParam{
				Name:        t.Name,
				Description: openai.String(t.Description),
				Parameters:  t.Parameters,
			},
		})
	}
	return defs
}

func CallTool(name, arguments string) (string, error) {
	var tool *Tool
	for i := range registry {
		if registry[i].Name == name {
			tool = &registry[i]
			break
		}
	}
	if tool == nil {
		return "", fmt.Errorf("unknown tool: %s", name)
	}
	var args map[string]any
	if arguments != "" {
		if err := json.Unmarshal([]byte(arguments), &args); err != nil {
			return "", fmt.Errorf("invalid arguments: %w", err)
		}
	}
	result, err := tool.Execute(args)
	if err != nil {
		return "", err
	}
	if len(result) > maxResultChars {
		result = result[:maxResultChars] + "\n\n...(内容过长已截断)"
	}
	return result, nil
}

func toJSON(v any) string {
	data, err := json.Marshal(v)
	if err != nil {
		return fmt.Sprintf(`{"error":"%v"}`, err)
	}
	return string(data)
}

func formatDiff(diff []git.DiffInfo) string {
	var buf strings.Builder
	for i, d := range diff {
		if i > 0 {
			buf.WriteString("\n")
		}
		buf.WriteString(fmt.Sprintf("File: %s (+%d -%d)\n", d.File, d.Added, d.Deleted))
		patch := d.Patch
		if len(patch) > maxPatchChars {
			patch = patch[:maxPatchChars] + "\n...(truncated)"
		}
		buf.WriteString(patch)
	}
	result := buf.String()
	if len(result) > maxResultChars {
		result = result[:maxResultChars] + "\n...(truncated)"
	}
	return result
}

func InitGitTools() {
	RegisterTools(
		Tool{
			Name:        "get_branches",
			Description: "获取当前 Git 仓库的所有本地分支名称列表",
			Parameters: shared.FunctionParameters{
				"type":       "object",
				"properties": map[string]any{},
			},
			Execute: func(args map[string]any) (string, error) {
				branches, err := git.GetBranches()
				if err != nil {
					return "", err
				}
				return toJSON(map[string]any{"branches": branches, "count": len(branches)}), nil
			},
		},
		Tool{
			Name:        "get_current_branch",
			Description: "获取当前工作目录所在的 Git 分支名称",
			Parameters: shared.FunctionParameters{
				"type":       "object",
				"properties": map[string]any{},
			},
			Execute: func(args map[string]any) (string, error) {
				branch, err := git.GetCurrentBranch()
				if err != nil {
					return "", err
				}
				return fmt.Sprintf(`{"current_branch":"%s"}`, branch), nil
			},
		},
		Tool{
			Name:        "get_branch_diff",
			Description: "比较两个 Git 分支之间的代码差异，返回变更文件列表和 diff patch",
			Parameters: shared.FunctionParameters{
				"type": "object",
				"properties": map[string]any{
					"branch1": map[string]any{
						"type":        "string",
						"description": "第一个分支名称",
					},
					"branch2": map[string]any{
						"type":        "string",
						"description": "第二个分支名称",
					},
				},
				"required": []string{"branch1", "branch2"},
			},
			Execute: func(args map[string]any) (string, error) {
				b1, _ := args["branch1"].(string)
				b2, _ := args["branch2"].(string)
				diff, err := git.GetBranchDiff(b1, b2)
				if err != nil {
					return "", err
				}
				if len(diff) == 0 {
					return `{"message":"两个分支没有差异"}`, nil
				}
				return formatDiff(diff), nil
			},
		},
		Tool{
			Name:        "get_commits",
			Description: "获取指定分支的最近提交历史记录，包含哈希、作者、时间和提交信息",
			Parameters: shared.FunctionParameters{
				"type": "object",
				"properties": map[string]any{
					"branch": map[string]any{
						"type":        "string",
						"description": "分支名称，不传则使用当前分支",
					},
					"limit": map[string]any{
						"type":        "integer",
						"description": "返回数量，默认20",
					},
				},
			},
			Execute: func(args map[string]any) (string, error) {
				branch, _ := args["branch"].(string)
				limit := 20
				if v, ok := args["limit"].(float64); ok {
					limit = int(v)
				}
				commits, err := git.GetCommits(branch, limit)
				if err != nil {
					return "", err
				}
				return toJSON(commits), nil
			},
		},
		Tool{
			Name:        "get_file_history",
			Description: "获取指定文件在一段时间内的修改历史记录",
			Parameters: shared.FunctionParameters{
				"type": "object",
				"properties": map[string]any{
					"file": map[string]any{
						"type":        "string",
						"description": "文件路径",
					},
					"time_range": map[string]any{
						"type":        "string",
						"description": "时间范围如 3d(3天) 7d(7天) 24h(24小时)，默认3d",
					},
				},
				"required": []string{"file"},
			},
			Execute: func(args map[string]any) (string, error) {
				file, _ := args["file"].(string)
				timeRange, _ := args["time_range"].(string)
				if timeRange == "" {
					timeRange = "3d"
				}
				startTime, err := intent.GetTimeRangeStart(timeRange)
				if err != nil {
					return "", err
				}
				commits, err := git.GetFileHistory(file, startTime)
				if err != nil {
					return "", err
				}
				return toJSON(commits), nil
			},
		},
		Tool{
			Name:        "get_diff",
			Description: "获取两个提交哈希之间的代码差异",
			Parameters: shared.FunctionParameters{
				"type": "object",
				"properties": map[string]any{
					"from": map[string]any{
						"type":        "string",
						"description": "起始提交哈希值",
					},
					"to": map[string]any{
						"type":        "string",
						"description": "结束提交哈希值",
					},
				},
				"required": []string{"from", "to"},
			},
			Execute: func(args map[string]any) (string, error) {
				from, _ := args["from"].(string)
				to, _ := args["to"].(string)
				diff, err := git.GetDiff(from, to)
				if err != nil {
					return "", err
				}
				if len(diff) == 0 {
					return `{"message":"两个提交没有差异"}`, nil
				}
				return formatDiff(diff), nil
			},
		},
	)
}
