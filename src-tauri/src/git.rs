use crate::CommitInfo;
use git2::Repository;
use std::path::Path;

pub struct GitState {
    path: Option<String>,
}

impl GitState {
    pub fn new() -> Self {
        GitState { path: None }
    }

    pub fn from_path(path: &str) -> Result<Self, String> {
        Repository::open(path).map_err(|e| format!("打开仓库失败: {}", e))?;
        Ok(GitState { path: Some(path.to_string()) })
    }

    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    pub fn open_repo(&mut self, path: &str) -> Result<(), String> {
        Repository::open(path).map_err(|e| format!("打开仓库失败: {}", e))?;
        self.path = Some(path.to_string());
        Ok(())
    }

    fn open_handle(&self) -> Result<Repository, String> {
        let path = self.path.as_deref().ok_or("未打开仓库")?;
        Repository::open(path).map_err(|e| format!("打开仓库失败: {}", e))
    }

    pub fn get_current_branch(&self) -> Result<String, String> {
        let repo = self.open_handle()?;
        let head = repo.head().map_err(|e| format!("获取 HEAD 失败: {}", e))?;
        if head.is_branch() {
            Ok(head.shorthand().unwrap_or("unknown").to_string())
        } else {
            let target = head.target().ok_or("HEAD 无目标")?;
            Ok(target.to_string()[..7].to_string())
        }
    }

    pub fn get_branches(&self) -> Result<Vec<String>, String> {
        let repo = self.open_handle()?;
        let mut names: Vec<String> = repo
            .branches(Some(git2::BranchType::Local))
            .map_err(|e| format!("获取分支失败: {}", e))?
            .filter_map(|branch_result| {
                if let Err(ref e) = branch_result {
                    eprintln!("警告: 读取分支时跳过（错误: {}）", e);
                }
                branch_result.ok()
            })
            .filter_map(|(branch, _)| branch.name().ok()?.map(|n| n.to_string()))
            .collect();
        names.sort();
        Ok(names)
    }

    pub fn get_commits(&self, branch: Option<&str>, limit: i32) -> Result<Vec<CommitInfo>, String> {
        let repo = self.open_handle()?;
        let revspec = branch.map_or("HEAD".to_string(), |b| format!("refs/heads/{}", b));
        let rev = repo.revparse_single(&revspec).map_err(|e| format!("解析引用失败: {}", e))?;

        let mut revwalk = repo.revwalk().map_err(|e| format!("创建 revwalk 失败: {}", e))?;
        revwalk.push(rev.id()).map_err(|e| format!("push rev 失败: {}", e))?;

        let commits: Vec<CommitInfo> = revwalk
            .take(limit as usize)
            .map(|oid| {
                let oid = oid.map_err(|e| format!("遍历提交失败: {}", e))?;
                let commit = repo.find_commit(oid).map_err(|e| format!("查找提交失败: {}", e))?;
                Ok(commit_to_info(oid, &commit))
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(commits)
    }

    pub fn get_diff(&self, from: &str, to: &str) -> Result<Vec<crate::DiffInfo>, String> {
        let repo = self.open_handle()?;
        let from_commit = repo.find_commit(git2::Oid::from_str(from).map_err(|e| format!("无效的 from hash: {}", e))?)
            .map_err(|e| format!("查找 from 提交失败: {}", e))?;
        let to_commit = repo.find_commit(git2::Oid::from_str(to).map_err(|e| format!("无效的 to hash: {}", e))?)
            .map_err(|e| format!("查找 to 提交失败: {}", e))?;

        let from_tree = from_commit.tree().map_err(|e| format!("获取 from tree 失败: {}", e))?;
        let to_tree = to_commit.tree().map_err(|e| format!("获取 to tree 失败: {}", e))?;

        let diff = repo.diff_tree_to_tree(Some(&from_tree), Some(&to_tree), None)
            .map_err(|e| format!("计算差异失败: {}", e))?;

        Self::process_diff(&diff)
    }

    pub fn get_commit_diff(&self, hash: &str) -> Result<Vec<crate::DiffInfo>, String> {
        let repo = self.open_handle()?;
        let commit = repo.find_commit(git2::Oid::from_str(hash).map_err(|e| format!("无效的提交哈希: {}", e))?)
            .map_err(|e| format!("查找提交失败: {}", e))?;

        let commit_tree = commit.tree().map_err(|e| format!("获取 tree 失败: {}", e))?;
        let parent_tree = commit.parent(0).ok()
            .map(|p| p.tree().map_err(|e| format!("获取父 tree 失败: {}", e)))
            .transpose()?;

        let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&commit_tree), None)
            .map_err(|e| format!("计算差异失败: {}", e))?;

        Self::process_diff(&diff)
    }

    pub fn get_branch_diff(&self, branch1: &str, branch2: &str) -> Result<Vec<crate::DiffInfo>, String> {
        let repo = self.open_handle()?;
        let b1 = repo.find_reference(&format!("refs/heads/{}", branch1))
            .map_err(|e| format!("查找分支 {} 失败: {}", branch1, e))?;
        let b2 = repo.find_reference(&format!("refs/heads/{}", branch2))
            .map_err(|e| format!("查找分支 {} 失败: {}", branch2, e))?;

        let b1_tree = repo.find_commit(b1.target().ok_or_else(|| format!("分支 {} 无目标", branch1))?)
            .map_err(|e| format!("查找提交失败: {}", e))?.tree().map_err(|e| format!("获取 tree 失败: {}", e))?;
        let b2_tree = repo.find_commit(b2.target().ok_or_else(|| format!("分支 {} 无目标", branch2))?)
            .map_err(|e| format!("查找提交失败: {}", e))?.tree().map_err(|e| format!("获取 tree 失败: {}", e))?;

        let diff = repo.diff_tree_to_tree(Some(&b1_tree), Some(&b2_tree), None)
            .map_err(|e| format!("计算分支差异失败: {}", e))?;

        Self::process_diff(&diff)
    }

    pub fn get_file_history(&self, file_path: &str, since_timestamp: i64) -> Result<Vec<CommitInfo>, String> {
        let repo = self.open_handle()?;
        let mut revwalk = repo.revwalk().map_err(|e| format!("创建 revwalk 失败: {}", e))?;
        revwalk.push_glob("refs/heads/*").map_err(|e| format!("push glob 失败: {}", e))?;
        revwalk.push_head().map_err(|e| format!("push head 失败: {}", e))?;
        revwalk.set_sorting(git2::Sort::TIME).map_err(|e| format!("设置排序失败: {}", e))?;

        let mut commits = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for oid in revwalk.take(2000) {
            let oid = oid.map_err(|e| format!("遍历提交失败: {}", e))?;
            if !seen.insert(oid) {
                continue;
            }
            let commit = repo.find_commit(oid).map_err(|e| format!("查找提交失败: {}", e))?;
            if commit.author().when().seconds() < since_timestamp {
                break;
            }
            if let Ok(tree) = commit.tree() {
                if tree.get_path(Path::new(file_path)).is_ok() {
                    commits.push(commit_to_info(oid, &commit));
                }
            }
        }
        Ok(commits)
    }

    fn process_diff(diff: &git2::Diff) -> Result<Vec<crate::DiffInfo>, String> {
        let mut result = Vec::new();

        for delta_idx in 0..diff.deltas().len() {
            let delta = diff.get_delta(delta_idx).ok_or_else(|| format!("无效的 delta 索引: {}", delta_idx))?;
            let file_path = delta.new_file().path()
                .or_else(|| delta.old_file().path())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let patch = match git2::Patch::from_diff(diff, delta_idx) {
                Ok(Some(mut p)) => {
                    let mut buf = String::new();
                    p.print(&mut |_delta, _hunk, line| {
                        let origin = line.origin();
                        let content = std::str::from_utf8(line.content()).unwrap_or("");
                        match origin {
                            // Content lines: emit the origin prefix followed
                            // by the (newline-normalized) line text.
                            '+' | '-' | ' ' => {
                                buf.push(origin);
                                buf.push_str(content.trim_end());
                                buf.push('\n');
                            }
                            // File/hunk headers and binary markers —
                            // libgit2 is inconsistent about trailing
                            // newlines on these, so normalize to exactly
                            // one. Without this, header lines jam together
                            // (e.g. `+++ b/x@@ -0,0 +1 @@+added`) which
                            // breaks both the patch rendering and the
                            // +/- line counts for added/removed files.
                            _ => {
                                let trimmed = content.trim_end();
                                if !trimmed.is_empty() {
                                    buf.push_str(trimmed);
                                    buf.push('\n');
                                }
                            }
                        }
                        true
                    }).map_err(|e| format!("打印 patch 失败: {}", e))?;
                    buf
                }
                _ => String::new(),
            };

            result.push(crate::DiffInfo {
                file: file_path,
                added: count_adds(&patch),
                deleted: count_dels(&patch),
                patch,
            });
        }
        Ok(result)
    }
}

fn commit_to_info(oid: git2::Oid, commit: &git2::Commit) -> CommitInfo {
    let hash_str = oid.to_string();
    let sig = commit.author();
    CommitInfo {
        hash: hash_str.clone(),
        short_hash: hash_str[..7].to_string(),
        message: commit.message().unwrap_or("").chars().take(100).collect(),
        author: sig.name().unwrap_or("unknown").to_string(),
        email: sig.email().unwrap_or("").to_string(),
        timestamp: sig.when().seconds().to_string(),
    }
}

fn count_adds(patch: &str) -> i32 {
    patch.lines().filter(|l| l.starts_with('+') && !l.starts_with("+++")).count() as i32
}

fn count_dels(patch: &str) -> i32 {
    patch.lines().filter(|l| l.starts_with('-') && !l.starts_with("---")).count() as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // --- pure helper unit tests ---

    #[test]
    fn count_adds_ignores_diff_headers() {
        let patch = "+++ b/x.rs\n+ fn a()\n+ 1\n- old\n context";
        assert_eq!(count_adds(patch), 2);
    }

    #[test]
    fn count_dels_ignores_diff_headers() {
        let patch = "--- a/x.rs\n- old\n- older\n+ new";
        assert_eq!(count_dels(patch), 2);
    }

    #[test]
    fn count_empty_patch() {
        assert_eq!(count_adds(""), 0);
        assert_eq!(count_dels(""), 0);
    }

    // --- temp-repo integration tests ---

    struct RepoFixture {
        _dir: tempfile::TempDir,
        state: GitState,
        head_branch: String,
        c0: String,
        c1: String,
    }

    /// Build a repo with two commits on the default branch, plus a `feature`
    /// branch pointing at the first commit. Returns the bound `GitState`.
    fn build_fixture() -> RepoFixture {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = Repository::init(dir.path()).expect("init");
        {
            let mut cfg = repo.config().expect("config");
            cfg.set_str("user.name", "tester").unwrap();
            cfg.set_str("user.email", "tester@example.com").unwrap();
        }

        // Commit a file -> c0, then modify/add -> c1, both on the default branch.
        let commit_file = |name: &str, content: &str, parent: Option<&git2::Commit>| -> git2::Commit {
            fs::write(dir.path().join(name), content).unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new(name)).unwrap();
            index.write().unwrap();
            let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
            let sig = repo.signature().unwrap();
            let parents: Vec<&git2::Commit> = parent.into_iter().collect();
            let oid = repo
                .commit(Some("HEAD"), &sig, &sig, &format!("add {}", name), &tree, &parents)
                .unwrap();
            repo.find_commit(oid).unwrap()
        };

        let c0 = commit_file("a.txt", "hello\n", None);
        let c1 = commit_file("b.txt", "world\n", Some(&c0));

        // feature branch off c0 (doesn't touch working tree / HEAD).
        repo.branch("feature", &c0, false).unwrap();

        let head_branch = repo.head().unwrap().shorthand().unwrap().to_string();

        let state = GitState::from_path(dir.path().to_str().unwrap()).expect("GitState");
        RepoFixture {
            _dir: dir,
            state,
            head_branch,
            c0: c0.id().to_string(),
            c1: c1.id().to_string(),
        }
    }

    #[test]
    fn get_branches_lists_feature_and_head() {
        let f = build_fixture();
        let branches = f.state.get_branches().unwrap();
        assert!(branches.contains(&"feature".to_string()));
        assert!(branches.contains(&f.head_branch));
        assert!(branches.len() >= 2);
    }

    #[test]
    fn get_current_branch_matches_head() {
        let f = build_fixture();
        assert_eq!(f.state.get_current_branch().unwrap(), f.head_branch);
    }

    #[test]
    fn get_commits_returns_history_descending() {
        let f = build_fixture();
        let commits = f.state.get_commits(Some(&f.head_branch), 10).unwrap();
        assert!(commits.len() >= 2);
        // Most recent first.
        assert_eq!(commits[0].hash, f.c1);
        assert_eq!(commits[1].hash, f.c0);
    }

    #[test]
    fn get_commit_diff_detects_added_file() {
        let f = build_fixture();
        let diff = f.state.get_commit_diff(&f.c1).unwrap();
        let files: Vec<&str> = diff.iter().map(|d| d.file.as_str()).collect();
        assert!(files.contains(&"b.txt"));
        // b.txt is pure addition.
        let b = diff.iter().find(|d| d.file == "b.txt").unwrap();
        assert!(b.added >= 1);
        assert_eq!(b.deleted, 0);
    }

    #[test]
    fn get_diff_between_commits() {
        let f = build_fixture();
        let diff = f.state.get_diff(&f.c0, &f.c1).unwrap();
        let files: Vec<&str> = diff.iter().map(|d| d.file.as_str()).collect();
        assert!(files.contains(&"b.txt"));
    }

    #[test]
    fn get_branch_diff_feature_vs_head() {
        let f = build_fixture();
        let diff = f.state.get_branch_diff("feature", &f.head_branch).unwrap();
        // b.txt was added on head after the feature branch forked at c0.
        let files: Vec<String> = diff.iter().map(|d| d.file.clone()).collect();
        assert!(files.iter().any(|name| name == "b.txt"));
    }

    #[test]
    fn get_file_history_finds_touching_commits() {
        let f = build_fixture();
        // since=0 (epoch) so the time filter doesn't prune anything.
        let history = f.state.get_file_history("a.txt", 0).unwrap();
        assert!(!history.is_empty());
        let hashes: Vec<&str> = history.iter().map(|c| c.hash.as_str()).collect();
        assert!(hashes.contains(&f.c0.as_str()));
    }
}
