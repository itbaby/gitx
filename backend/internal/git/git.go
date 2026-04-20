package git

import (
	"fmt"
	"sort"
	"time"

	"github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing"
	"github.com/go-git/go-git/v5/plumbing/object"
	"github.com/go-git/go-git/v5/plumbing/storer"
)

var (
	CurrentRepo *git.Repository
	RepoPath    string
)

// OpenRepo 打开Git仓库
func OpenRepo(path string) error {
	var err error
	CurrentRepo, err = git.PlainOpen(path)
	if err != nil {
		return fmt.Errorf("failed to open git repo: %w", err)
	}
	RepoPath = path
	return nil
}

// GetCurrentBranch 获取当前分支名称
func GetCurrentBranch() (string, error) {
	if CurrentRepo == nil {
		return "", fmt.Errorf("no repo open")
	}

	headRef, err := CurrentRepo.Head()
	if err != nil {
		return "", fmt.Errorf("failed to get HEAD: %w", err)
	}

	// 如果 HEAD 指向一个分支，返回分支名
	if headRef.Name().IsBranch() {
		return headRef.Name().Short(), nil
	}

	// HEAD 处于 detached 状态
	return headRef.Hash().String()[:7], nil
}

// GetBranches 获取仓库的所有分支（本地），按名称排序
func GetBranches() ([]string, error) {
	if CurrentRepo == nil {
		return nil, fmt.Errorf("no repo open")
	}

	branches, err := CurrentRepo.Branches()
	if err != nil {
		return nil, fmt.Errorf("failed to get branches: %w", err)
	}

	var branchNames []string
	err = branches.ForEach(func(branch *plumbing.Reference) error {
		branchNames = append(branchNames, branch.Name().Short())
		return nil
	})
	if err != nil {
		return nil, fmt.Errorf("failed to iterate branches: %w", err)
	}

	sort.Strings(branchNames)
	return branchNames, nil
}

// CommitInfo 提交信息
type CommitInfo struct {
	Hash      string    `json:"hash"`
	ShortHash string    `json:"shortHash"`
	Message   string    `json:"message"`
	Author    string    `json:"author"`
	Email     string    `json:"email"`
	Timestamp time.Time `json:"timestamp"`
}

// GetCommits 获取仓库的提交历史
func GetCommits(branch string, limit int) ([]CommitInfo, error) {
	if CurrentRepo == nil {
		return nil, fmt.Errorf("no repo open")
	}

	var refName plumbing.ReferenceName
	if branch != "" {
		refName = plumbing.NewBranchReferenceName(branch)
	} else {
		headRef, err := CurrentRepo.Head()
		if err != nil {
			return nil, fmt.Errorf("failed to get HEAD: %w", err)
		}
		refName = headRef.Name()
	}

	ref, err := CurrentRepo.Reference(refName, true)
	if err != nil {
		return nil, fmt.Errorf("failed to get reference: %w", err)
	}

	commitIter, err := CurrentRepo.Log(&git.LogOptions{
		From:  ref.Hash(),
		Order: git.LogOrderCommitterTime,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to get commit log: %w", err)
	}

	var commits []CommitInfo
	err = commitIter.ForEach(func(c *object.Commit) error {
		if len(commits) >= limit {
			return storer.ErrStop
		}

		// 提取第一行作为简短消息
		msg := c.Message
		if idx := len(msg); idx > 100 {
			msg = msg[:100] + "..."
		}

		commits = append(commits, CommitInfo{
			Hash:      c.Hash.String(),
			ShortHash: c.Hash.String()[:7],
			Message:   msg,
			Author:    c.Author.Name,
			Email:     c.Author.Email,
			Timestamp: c.Author.When,
		})
		return nil
	})
	if err != nil && err != storer.ErrStop {
		return nil, fmt.Errorf("failed to get commits: %w", err)
	}

	return commits, nil
}

// DiffInfo 差异信息
type DiffInfo struct {
	File    string `json:"file"`
	Patch   string `json:"patch"`
	Added   int    `json:"added"`
	Deleted int    `json:"deleted"`
}

// DiffStats 差异统计
type DiffStats struct {
	TotalFiles   int `json:"totalFiles"`
	TotalAdded   int `json:"totalAdded"`
	TotalDeleted int `json:"totalDeleted"`
}

// GetDiffStats 从差异数据计算统计信息
func GetDiffStats(diffInfos []DiffInfo) DiffStats {
	stats := DiffStats{}
	for _, d := range diffInfos {
		stats.TotalFiles++
		stats.TotalAdded += d.Added
		stats.TotalDeleted += d.Deleted
	}
	return stats
}

// GetDiff 获取两个提交之间的差异
func GetDiff(from, to string) ([]DiffInfo, error) {
	if CurrentRepo == nil {
		return nil, fmt.Errorf("no repo open")
	}

	fromHash := plumbing.NewHash(from)
	toHash := plumbing.NewHash(to)

	fromCommit, err := CurrentRepo.CommitObject(fromHash)
	if err != nil {
		return nil, fmt.Errorf("failed to get from commit: %w", err)
	}

	toCommit, err := CurrentRepo.CommitObject(toHash)
	if err != nil {
		return nil, fmt.Errorf("failed to get to commit: %w", err)
	}

	fromTree, err := fromCommit.Tree()
	if err != nil {
		return nil, fmt.Errorf("failed to get from tree: %w", err)
	}

	toTree, err := toCommit.Tree()
	if err != nil {
		return nil, fmt.Errorf("failed to get to tree: %w", err)
	}

	diff, err := toTree.Diff(fromTree)
	if err != nil {
		return nil, fmt.Errorf("failed to compute diff: %w", err)
	}

	return processDiff(diff), nil
}

// GetBranchDiff 获取两个分支之间的差异
func GetBranchDiff(branch1, branch2 string) ([]DiffInfo, error) {
	if CurrentRepo == nil {
		return nil, fmt.Errorf("no repo open")
	}

	branch1Ref, err := CurrentRepo.Reference(plumbing.NewBranchReferenceName(branch1), true)
	if err != nil {
		return nil, fmt.Errorf("failed to get branch %s: %w", branch1, err)
	}

	branch2Ref, err := CurrentRepo.Reference(plumbing.NewBranchReferenceName(branch2), true)
	if err != nil {
		return nil, fmt.Errorf("failed to get branch %s: %w", branch2, err)
	}

	branch1Commit, err := CurrentRepo.CommitObject(branch1Ref.Hash())
	if err != nil {
		return nil, fmt.Errorf("failed to get branch %s commit: %w", branch1, err)
	}

	branch2Commit, err := CurrentRepo.CommitObject(branch2Ref.Hash())
	if err != nil {
		return nil, fmt.Errorf("failed to get branch %s commit: %w", branch2, err)
	}

	branch1Tree, err := branch1Commit.Tree()
	if err != nil {
		return nil, fmt.Errorf("failed to get branch %s tree: %w", branch1, err)
	}

	branch2Tree, err := branch2Commit.Tree()
	if err != nil {
		return nil, fmt.Errorf("failed to get branch %s tree: %w", branch2, err)
	}

	diff, err := branch2Tree.Diff(branch1Tree)
	if err != nil {
		return nil, fmt.Errorf("failed to compute diff: %w", err)
	}

	return processDiff(diff), nil
}

// processDiff 处理差异对象列表
func processDiff(diff object.Changes) []DiffInfo {
	var diffInfos []DiffInfo
	for _, fileDiff := range diff {
		patch, err := fileDiff.Patch()
		if err != nil {
			continue
		}

		// 通过 Action 获取变更类型，通过 patch 统计增删行数
		toFile, fromFile := fileDiff.To, fileDiff.From
		fileName := toFile.Name
		if fileName == "" {
			fileName = fromFile.Name
		}

		// 通过 patch 的 String() 解析增删行数
		patchStr := patch.String()
		added := countLines(patchStr, '+')
		deleted := countLines(patchStr, '-')

		diffInfos = append(diffInfos, DiffInfo{
			File:    fileName,
			Patch:   patchStr,
			Added:   added,
			Deleted: deleted,
		})
	}
	return diffInfos
}

// countLines 统计 patch 中以指定前缀开头的行数（排除 +++ 和 --- 标记行）
func countLines(patch string, prefix byte) int {
	count := 0
	inHunk := false
	for i := 0; i < len(patch); i++ {
		if i > 0 && patch[i-1] == '\n' {
			inHunk = false
		}
		if i == 0 || (i > 0 && patch[i-1] == '\n') {
			if i < len(patch) && patch[i] == '@' {
				inHunk = true
			}
			if inHunk && i < len(patch) && patch[i] == prefix {
				// 跳过 +++ 和 --- 文件标记行
				if prefix == '+' && i+2 < len(patch) && patch[i+1] == '+' && patch[i+2] == '+' {
					continue
				}
				if prefix == '-' && i+2 < len(patch) && patch[i+1] == '-' && patch[i+2] == '-' {
					continue
				}
				count++
			}
		}
	}
	return count
}

// GetFileHistory 获取文件的修改历史
func GetFileHistory(filePath string, since time.Time) ([]CommitInfo, error) {
	if CurrentRepo == nil {
		return nil, fmt.Errorf("no repo open")
	}

	branches, err := CurrentRepo.Branches()
	if err != nil {
		return nil, fmt.Errorf("failed to get branches: %w", err)
	}

	commitMap := make(map[string]*object.Commit)

	err = branches.ForEach(func(branch *plumbing.Reference) error {
		commitIter, err := CurrentRepo.Log(&git.LogOptions{
			From:  branch.Hash(),
			Order: git.LogOrderCommitterTime,
		})
		if err != nil {
			return nil // 跳过错误的分支
		}

		return commitIter.ForEach(func(c *object.Commit) error {
			if c.Author.When.Before(since) {
				return storer.ErrStop
			}

			tree, err := c.Tree()
			if err != nil {
				return nil
			}

			if _, err := tree.FindEntry(filePath); err == nil {
				commitMap[c.Hash.String()] = c
			}

			return nil
		})
	})
	if err != nil {
		return nil, fmt.Errorf("failed to iterate branches: %w", err)
	}

	var commits []CommitInfo
	for _, c := range commitMap {
		msg := c.Message
		if len(msg) > 100 {
			msg = msg[:100] + "..."
		}
		commits = append(commits, CommitInfo{
			Hash:      c.Hash.String(),
			ShortHash: c.Hash.String()[:7],
			Message:   msg,
			Author:    c.Author.Name,
			Email:     c.Author.Email,
			Timestamp: c.Author.When,
		})
	}

	sort.Slice(commits, func(i, j int) bool {
		return commits[i].Timestamp.After(commits[j].Timestamp)
	})

	return commits, nil
}
