use crate::git::GitState;
use crate::intent;
use serde_json::{json, Value};

const MAX_PATCH_CHARS: usize = 2000;
const MAX_RESULT_CHARS: usize = 25000;

// ============================================================
// Tool Definitions (OpenAI Function Calling format)
// ============================================================

pub fn get_tool_defs() -> Vec<Value> {
    vec![
        json!({
            "type": "function",
            "function": {
                "name": "get_branches",
                "description": "获取当前 Git 仓库的所有本地分支名称列表",
                "parameters": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "get_current_branch",
                "description": "获取当前工作目录所在的 Git 分支名称",
                "parameters": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "get_branch_diff",
                "description": "比较两个 Git 分支之间的代码差异，返回变更文件列表和 diff patch",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "branch1": {
                            "type": "string",
                            "description": "第一个分支名称"
                        },
                        "branch2": {
                            "type": "string",
                            "description": "第二个分支名称"
                        }
                    },
                    "required": ["branch1", "branch2"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "get_commits",
                "description": "获取指定分支的最近提交历史记录，包含哈希、作者、时间和提交信息",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "branch": {
                            "type": "string",
                            "description": "分支名称，不传则使用当前分支"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "返回数量，默认20"
                        }
                    },
                    "required": []
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "get_file_history",
                "description": "获取指定文件在一段时间内的修改历史记录",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "file": {
                            "type": "string",
                            "description": "文件路径"
                        },
                        "time_range": {
                            "type": "string",
                            "description": "时间范围如 3d(3天) 7d(7天) 24h(24小时)，默认3d"
                        }
                    },
                    "required": ["file"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "get_diff",
                "description": "获取两个提交哈希之间的代码差异",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "from": {
                            "type": "string",
                            "description": "起始提交哈希值"
                        },
                        "to": {
                            "type": "string",
                            "description": "结束提交哈希值"
                        }
                    },
                    "required": ["from", "to"]
                }
            }
        }),
    ]
}

// ============================================================
// Tool Dispatch (accepts repo_path, opens repo internally)
// ============================================================

pub fn call_tool(name: &str, arguments: &str, repo_path: &Option<String>) -> String {
    // Open repo from path
    let path = match repo_path {
        Some(p) if !p.is_empty() => p.clone(),
        _ => return "错误：未打开仓库，无法执行工具".to_string(),
    };

    let mut git_state = GitState::new();
    if let Err(e) = git_state.open_repo(&path) {
        return format!("打开仓库失败: {}", e);
    }
    // Note: open_repo validates the repo exists; actual Repository handles
    // are opened per-call inside GitState methods (thread-safe pattern).

    let args: Value = serde_json::from_str(arguments).unwrap_or(json!({}));

    match name {
        "get_branches" => exec_get_branches(&git_state),
        "get_current_branch" => exec_get_current_branch(&git_state),
        "get_branch_diff" => exec_get_branch_diff(&git_state, &args),
        "get_commits" => exec_get_commits(&git_state, &args),
        "get_file_history" => exec_get_file_history(&git_state, &args),
        "get_diff" => exec_get_diff(&git_state, &args),
        _ => format!("unknown tool: {}", name),
    }
}

// ============================================================
// Tool Implementations
// ============================================================

fn exec_get_branches(git_state: &GitState) -> String {
    match git_state.get_branches() {
        Ok(branches) => {
            let result = json!({
                "branches": branches,
                "count": branches.len()
            });
            truncate_result(result.to_string())
        }
        Err(e) => format!("工具执行失败: {}", e),
    }
}

fn exec_get_current_branch(git_state: &GitState) -> String {
    match git_state.get_current_branch() {
        Ok(branch) => json!({ "current_branch": branch }).to_string(),
        Err(e) => format!("工具执行失败: {}", e),
    }
}

fn exec_get_branch_diff(git_state: &GitState, args: &Value) -> String {
    let b1 = args["branch1"].as_str().unwrap_or("");
    let b2 = args["branch2"].as_str().unwrap_or("");

    if b1.is_empty() || b2.is_empty() {
        return "缺少必要参数: branch1, branch2".to_string();
    }

    match git_state.get_branch_diff(b1, b2) {
        Ok(diff) => {
            if diff.is_empty() {
                return json!({ "message": "两个分支没有差异" }).to_string();
            }
            truncate_result(format_diff(&diff))
        }
        Err(e) => format!("工具执行失败: {}", e),
    }
}

fn exec_get_commits(git_state: &GitState, args: &Value) -> String {
    let branch = args["branch"].as_str().map(|s| s.to_string());
    let limit = args["limit"].as_i64().unwrap_or(20) as i32;

    match git_state.get_commits(branch.as_deref(), limit) {
        Ok(commits) => {
            let result = serde_json::to_value(&commits).unwrap_or(json!([]));
            truncate_result(result.to_string())
        }
        Err(e) => format!("工具执行失败: {}", e),
    }
}

fn exec_get_file_history(git_state: &GitState, args: &Value) -> String {
    let file = args["file"].as_str().unwrap_or("");
    let time_range = args["time_range"].as_str().unwrap_or("3d");

    if file.is_empty() {
        return "缺少必要参数: file".to_string();
    }

    let since = match intent::parse_time_range(time_range) {
        Ok(s) => s,
        Err(e) => return format!("时间范围解析失败: {}", e),
    };

    match git_state.get_file_history(file, since) {
        Ok(commits) => {
            let result = serde_json::to_value(&commits).unwrap_or(json!([]));
            truncate_result(result.to_string())
        }
        Err(e) => format!("工具执行失败: {}", e),
    }
}

fn exec_get_diff(git_state: &GitState, args: &Value) -> String {
    let from = args["from"].as_str().unwrap_or("");
    let to = args["to"].as_str().unwrap_or("");

    if from.is_empty() || to.is_empty() {
        return "缺少必要参数: from, to".to_string();
    }

    match git_state.get_diff(from, to) {
        Ok(diff) => {
            if diff.is_empty() {
                return json!({ "message": "两个提交没有差异" }).to_string();
            }
            truncate_result(format_diff(&diff))
        }
        Err(e) => format!("工具执行失败: {}", e),
    }
}

// ============================================================
// Helpers
// ============================================================

fn format_diff(diff: &[crate::DiffInfo]) -> String {
    let mut buf = String::new();
    for (i, d) in diff.iter().enumerate() {
        if i > 0 {
            buf.push('\n');
        }
        buf.push_str(&format!("File: {} (+{} -{})\n", d.file, d.added, d.deleted));
        let patch = if d.patch.chars().count() > MAX_PATCH_CHARS {
            let truncated: String = d.patch.chars().take(MAX_PATCH_CHARS).collect();
            format!("{}\n...(truncated)", truncated)
        } else {
            d.patch.clone()
        };
        buf.push_str(&patch);
    }
    buf
}

fn truncate_result(result: String) -> String {
    if result.chars().count() > MAX_RESULT_CHARS {
        let truncated: String = result.chars().take(MAX_RESULT_CHARS).collect();
        format!("{}\n\n...(内容过长已截断)", truncated)
    } else {
        result
    }
}
