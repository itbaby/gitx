package intent

import (
	"fmt"
	"regexp"
	"strings"
	"time"
)

// IntentType 意图类型
type IntentType string

const (
	IntentTypeDiffFile   IntentType = "diff_file"
	IntentTypeDiffBranch IntentType = "diff_branch"
	IntentTypeDiffCommit IntentType = "diff_commit"
	IntentTypeUnknown    IntentType = "unknown"
)

// Intent 意图结构
type Intent struct {
	Type      IntentType `json:"type"`
	File      string     `json:"file"`
	Branch1   string     `json:"branch1"`
	Branch2   string     `json:"branch2"`
	Commit1   string     `json:"commit1"`
	Commit2   string     `json:"commit2"`
	TimeRange string     `json:"timeRange"`
}

// ParseIntent 解析用户输入的意图（基于正则的本地解析）
func ParseIntent(input string) *Intent {
	input = strings.TrimSpace(strings.ToLower(input))
	intent := &Intent{Type: IntentTypeUnknown}

	// 检测分支比较
	if isBranchDiff(input) {
		intent.Type = IntentTypeDiffBranch
		intent.Branch1 = extractBranch1(input)
		intent.Branch2 = extractBranch2(input)
		return intent
	}

	// 检测提交比较
	if isCommitDiff(input) {
		intent.Type = IntentTypeDiffCommit
		intent.Commit1 = extractCommit1(input)
		intent.Commit2 = extractCommit2(input)
		return intent
	}

	// 检测文件历史
	if isFileHistory(input) {
		intent.Type = IntentTypeDiffFile
		intent.TimeRange = extractTimeRange(input)
		intent.File = extractFile(input)
		return intent
	}

	return intent
}

// 分支比较检测
func isBranchDiff(input string) bool {
	patterns := []string{
		`比较.*分支`, `比较.*branch`, `compare.*branch`,
		`分支.*对比`, `branch.*compare`, `branch.*diff`,
		`当前.*main`, `current.*main`,
	}
	for _, p := range patterns {
		if matched, _ := regexp.MatchString(p, input); matched {
			return true
		}
	}
	return false
}

func extractBranch1(input string) string {
	if strings.Contains(input, "当前") || strings.Contains(input, "current") {
		return "HEAD"
	}
	// 尝试提取第一个分支名
	re := regexp.MustCompile(`(?:分支|branch)\s*(\S+)\s*(?:和|与|vs|compare|with)`)
	matches := re.FindStringSubmatch(input)
	if len(matches) > 1 {
		return matches[1]
	}
	return "HEAD"
}

func extractBranch2(input string) string {
	if strings.Contains(input, "main") {
		return "main"
	}
	if strings.Contains(input, "master") {
		return "master"
	}
	// 尝试提取第二个分支名
	re := regexp.MustCompile(`(?:和|与|vs|compare|with)\s*(?:分支|branch)?\s*(\S+)`)
	matches := re.FindStringSubmatch(input)
	if len(matches) > 1 {
		return matches[1]
	}
	return "main"
}

// 提交比较检测
func isCommitDiff(input string) bool {
	patterns := []string{
		`比较.*提交`, `比较.*commit`,
		`commit.*compare`, `commit.*diff`,
		`[0-9a-f]{7,}`,
	}
	for _, p := range patterns {
		if matched, _ := regexp.MatchString(p, input); matched {
			return true
		}
	}
	return false
}

func extractCommit1(input string) string {
	re := regexp.MustCompile(`([0-9a-f]{7,40})`)
	matches := re.FindStringSubmatch(input)
	if len(matches) > 1 {
		return matches[1]
	}
	return ""
}

func extractCommit2(input string) string {
	re := regexp.MustCompile(`([0-9a-f]{7,40})`)
	matches := re.FindAllStringSubmatch(input, -1)
	if len(matches) >= 2 {
		return matches[1][1]
	}
	return ""
}

// 文件历史检测
func isFileHistory(input string) bool {
	patterns := []string{
		`(?:过去|最近).*天`, `(?:last|past).*days`,
		`(?:过去|最近).*小时`, `(?:last|past).*hours`,
		`文件.*历史`, `file.*history`,
		`改动`, `变更`, `changes`,
	}
	for _, p := range patterns {
		if matched, _ := regexp.MatchString(p, input); matched {
			return true
		}
	}
	return false
}

func extractTimeRange(input string) string {
	// 尝试匹配 "过去N天" / "last N days"
	if re := regexp.MustCompile(`(?:过去|最近)(\d+)天`); re != nil {
		if matches := re.FindStringSubmatch(input); len(matches) > 1 {
			return matches[1] + "d"
		}
	}
	if re := regexp.MustCompile(`(?:last|past)\s*(\d+)\s*days?`); re != nil {
		if matches := re.FindStringSubmatch(input); len(matches) > 1 {
			return matches[1] + "d"
		}
	}
	// 尝试匹配小时
	if re := regexp.MustCompile(`(?:过去|最近)(\d+)小时`); re != nil {
		if matches := re.FindStringSubmatch(input); len(matches) > 1 {
			return matches[1] + "h"
		}
	}
	return "3d" // 默认
}

func extractFile(input string) string {
	// 尝试提取引号中的文件名
	re := regexp.MustCompile(`["']([^"']+\.\w+)["']`)
	if matches := re.FindStringSubmatch(input); len(matches) > 1 {
		return matches[1]
	}

	// 尝试提取路径格式
	re = regexp.MustCompile(`([\w./\\-]+\.\w+)`)
	if matches := re.FindStringSubmatch(input); len(matches) > 1 {
		return matches[1]
	}

	return ""
}

// GetTimeRangeStart 根据时间范围获取开始时间
func GetTimeRangeStart(timeRange string) (time.Time, error) {
	if timeRange == "" {
		return time.Time{}, fmt.Errorf("empty time range")
	}

	re := regexp.MustCompile(`(\d+)([dhm])`)
	matches := re.FindStringSubmatch(timeRange)
	if len(matches) < 3 {
		return time.Time{}, fmt.Errorf("invalid time range format: %s", timeRange)
	}

	var value int
	fmt.Sscanf(matches[1], "%d", &value)
	unit := matches[2]

	now := time.Now()
	switch unit {
	case "d":
		return now.AddDate(0, 0, -value), nil
	case "h":
		return now.Add(-time.Duration(value) * time.Hour), nil
	case "m":
		return now.Add(-time.Duration(value) * time.Minute), nil
	default:
		return time.Time{}, fmt.Errorf("invalid time unit: %s", unit)
	}
}
