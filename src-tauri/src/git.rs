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

    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    pub fn open_repo(&mut self, path: &str) -> Result<(), String> {
        // Validate the repo can be opened
        Repository::open(path).map_err(|e| format!("打开仓库失败: {}", e))?;
        self.path = Some(path.to_string());
        Ok(())
    }

    /// Open a fresh Repository handle from the stored path.
    /// git2::Repository::open is cheap and avoids storing a !Send handle across await points.
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
            Ok(truncate_chars(&target.to_string(), 7).to_string())
        }
    }

    pub fn get_branches(&self) -> Result<Vec<String>, String> {
        let repo = self.open_handle()?;
        let branches = repo
            .branches(Some(git2::BranchType::Local))
            .map_err(|e| format!("获取分支失败: {}", e))?;

        let mut names: Vec<String> = Vec::new();
        for branch_result in branches.flatten() {
            let (branch, _bt) = branch_result;
            let name = match branch.name() {
                Ok(Some(n)) => n.to_string(),
                _ => continue,
            };
            if let Some(short) = name.strip_prefix("refs/heads/") {
                names.push(short.to_string());
            } else if !name.is_empty() {
                names.push(name);
            }
        }
        names.sort();
        Ok(names)
    }

    pub fn get_commits(
        &self,
        branch: Option<&str>,
        limit: i32,
    ) -> Result<Vec<CommitInfo>, String> {
        let repo = self.open_handle()?;
        let revspec = if let Some(b) = branch {
            format!("refs/heads/{}", b)
        } else {
            "HEAD".to_string()
        };

        let rev = repo
            .revparse_single(&revspec)
            .map_err(|e| format!("解析引用失败: {}", e))?;

        let mut revwalk = repo
            .revwalk()
            .map_err(|e| format!("创建 revwalk 失败: {}", e))?;

        revwalk
            .push(rev.id())
            .map_err(|e| format!("push rev 失败: {}", e))?;

        let mut commits = Vec::new();
        for (i, oid) in revwalk.enumerate() {
            if i as i32 >= limit {
                break;
            }
            let oid = oid.map_err(|e| format!("遍历提交失败: {}", e))?;
            let commit = repo
                .find_commit(oid)
                .map_err(|e| format!("查找提交失败: {}", e))?;

            let msg = commit.message().unwrap_or("");
            let msg = if msg.chars().count() > 100 {
                format!("{}...", truncate_chars(msg, 100))
            } else {
                msg.to_string()
            };

            let sig = commit.author();

            commits.push(CommitInfo {
                hash: oid.to_string(),
                short_hash: truncate_chars(&oid.to_string(), 7).to_string(),
                message: msg,
                author: sig.name().unwrap_or("unknown").to_string(),
                email: sig.email().unwrap_or("").to_string(),
                timestamp: sig.when().seconds().to_string(),
            });
        }

        Ok(commits)
    }

    pub fn get_diff(&self, from: &str, to: &str) -> Result<Vec<crate::DiffInfo>, String> {
        let repo = self.open_handle()?;

        let from_commit = repo
            .find_commit(
                git2::Oid::from_str(from).map_err(|e| format!("无效的 from hash: {}", e))?,
            )
            .map_err(|e| format!("查找 from 提交失败: {}", e))?;

        let to_commit = repo
            .find_commit(
                git2::Oid::from_str(to).map_err(|e| format!("无效的 to hash: {}", e))?,
            )
            .map_err(|e| format!("查找 to 提交失败: {}", e))?;

        let from_tree = from_commit
            .tree()
            .map_err(|e| format!("获取 from tree 失败: {}", e))?;
        let to_tree = to_commit
            .tree()
            .map_err(|e| format!("获取 to tree 失败: {}", e))?;

        let diff = repo
            .diff_tree_to_tree(Some(&from_tree), Some(&to_tree), None)
            .map_err(|e| format!("计算差异失败: {}", e))?;

        Self::process_diff(&diff)
    }

    pub fn get_branch_diff(
        &self,
        branch1: &str,
        branch2: &str,
    ) -> Result<Vec<crate::DiffInfo>, String> {
        let repo = self.open_handle()?;

        let b1_ref = repo
            .find_reference(&format!("refs/heads/{}", branch1))
            .map_err(|e| format!("查找分支 {} 失败: {}", branch1, e))?;

        let b2_ref = repo
            .find_reference(&format!("refs/heads/{}", branch2))
            .map_err(|e| format!("查找分支 {} 失败: {}", branch2, e))?;

        let b1_target = b1_ref
            .target()
            .ok_or_else(|| format!("分支 {} 无目标", branch1))?;
        let b2_target = b2_ref
            .target()
            .ok_or_else(|| format!("分支 {} 无目标", branch2))?;

        let b1_tree = repo
            .find_commit(b1_target)
            .map_err(|e| format!("查找提交失败: {}", e))?
            .tree()
            .map_err(|e| format!("获取 tree 失败: {}", e))?;

        let b2_tree = repo
            .find_commit(b2_target)
            .map_err(|e| format!("查找提交失败: {}", e))?
            .tree()
            .map_err(|e| format!("获取 tree 失败: {}", e))?;

        let diff = repo
            .diff_tree_to_tree(Some(&b1_tree), Some(&b2_tree), None)
            .map_err(|e| format!("计算分支差异失败: {}", e))?;

        Self::process_diff(&diff)
    }

    pub fn get_file_history(
        &self,
        file_path: &str,
        since_timestamp: i64,
    ) -> Result<Vec<CommitInfo>, String> {
        const MAX_SCAN: usize = 2000;

        let repo = self.open_handle()?;
        let branches = self.get_branches()?;

        let mut commit_set = std::collections::HashSet::new();
        let mut commits: Vec<CommitInfo> = Vec::new();
        let mut scanned = 0usize;

        for branch in &branches {
            if scanned >= MAX_SCAN {
                break;
            }
            if let Ok(reference) = repo.find_reference(&format!("refs/heads/{}", branch)) {
                if let Some(target) = reference.target() {
                    if let Ok(mut revwalk) = repo.revwalk() {
                        revwalk
                            .push(target)
                            .map_err(|e| format!("push rev 失败: {}", e))?;
                        for oid in revwalk {
                            if scanned >= MAX_SCAN {
                                break;
                            }
                            scanned += 1;
                            let oid = oid.map_err(|e| format!("遍历提交失败: {}", e))?;
                            if commit_set.contains(&oid) {
                                continue;
                            }
                            if let Ok(commit) = repo.find_commit(oid) {
                                let sig = commit.author();
                                let ts = sig.when().seconds();
                                if ts < since_timestamp {
                                    continue;
                                }

                                if let Ok(tree) = commit.tree() {
                                    if tree.get_path(Path::new(file_path)).is_ok() {
                                        commit_set.insert(oid);
                                        let msg = commit.message().unwrap_or("");
                                        let msg = if msg.chars().count() > 100 {
                                            format!("{}...", truncate_chars(msg, 100))
                                        } else {
                                            msg.to_string()
                                        };

                                        commits.push(CommitInfo {
                                            hash: oid.to_string(),
                                            short_hash: truncate_chars(&oid.to_string(), 7)
                                                .to_string(),
                                            message: msg,
                                            author: sig.name().unwrap_or("unknown").to_string(),
                                            email: sig.email().unwrap_or("").to_string(),
                                            timestamp: ts.to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        commits.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(commits)
    }

    fn process_diff(diff: &git2::Diff) -> Result<Vec<crate::DiffInfo>, String> {
        let mut result = Vec::new();

        for delta_idx in 0..diff.deltas().len() {
            let delta = diff
                .get_delta(delta_idx)
                .ok_or_else(|| format!("无效的 delta 索引: {}", delta_idx))?;

            let file_path = delta
                .new_file()
                .path()
                .map(|p| p.to_string_lossy().to_string())
                .or_else(|| {
                    delta
                        .old_file()
                        .path()
                        .map(|p| p.to_string_lossy().to_string())
                })
                .unwrap_or_else(|| "unknown".to_string());

            let patch = match git2::Patch::from_diff(diff, delta_idx) {
                Ok(Some(mut p)) => {
                    let mut buf = String::new();
                    p.print(
                        &mut |_delta: git2::DiffDelta,
                              _hunk: Option<git2::DiffHunk>,
                              line: git2::DiffLine| {
                            let origin = line.origin();
                            let content = std::str::from_utf8(line.content()).unwrap_or("").trim_end();
                            match origin {
                                '+' => buf.push_str(&format!("+{}\n", content)),
                                '-' => buf.push_str(&format!("-{}\n", content)),
                                ' ' => buf.push_str(&format!(" {}\n", content)),
                                _ => buf.push_str(content),
                            }
                            true
                        },
                    )
                    .map_err(|e| format!("打印 patch 失败: {}", e))?;
                    buf
                }
                _ => String::new(),
            };

            let (added, deleted) = count_lines(&patch);

            result.push(crate::DiffInfo {
                file: file_path,
                patch,
                added,
                deleted,
            });
        }

        Ok(result)
    }
}

// ============================================================
// Helpers
// ============================================================

/// Truncate a string to at most `max_chars` Unicode characters (not bytes).
/// This is safe for multi-byte UTF-8 content like Chinese text.
pub(crate) fn truncate_chars(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

fn count_lines(patch: &str) -> (i32, i32) {
    let mut added = 0i32;
    let mut deleted = 0i32;
    let mut in_hunk = false;

    for line in patch.lines() {
        if line.starts_with("@@") {
            in_hunk = true;
            continue;
        }

        if !in_hunk {
            continue;
        }

        if line.starts_with("+++") || line.starts_with("---") {
            continue;
        }

        if line.starts_with('+') {
            added += 1;
        } else if line.starts_with('-') {
            deleted += 1;
        }
    }

    (added, deleted)
}
