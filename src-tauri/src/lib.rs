mod ai;
mod git;
mod intent;
mod tools;

use serde::{Deserialize, Serialize};
use std::sync::{Mutex, MutexGuard};
use tauri::State;

/// Lock a Mutex with better error context and recovery from poisoning.
fn lock_state<T>(lock: &Mutex<T>) -> Result<MutexGuard<'_, T>, String> {
    lock.lock().or_else(|e: std::sync::PoisonError<MutexGuard<T>>| {
        eprintln!("Mutex poisoned, recovering: {}", e);
        Ok(e.into_inner())
    }).map_err(|e: std::sync::PoisonError<MutexGuard<T>>| format!("状态锁不可用: {}", e))
}

// ============================================================
// App State
// ============================================================

pub struct AppState {
    pub git: Mutex<git::GitState>,
    pub ai_config: Mutex<Option<ai::AiConfig>>,
}

// ============================================================
// Common Types
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub email: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffInfo {
    pub file: String,
    pub patch: String,
    pub added: i32,
    pub deleted: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStats {
    pub total_files: i32,
    pub total_added: i32,
    pub total_deleted: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatContext {
    pub base_branch: String,
    pub compare_branch: String,
    pub has_diff: bool,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<InputMessage>,
    pub context: Option<ChatContext>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    pub diff: Vec<DiffInfo>,
    pub prompt: String,
}

#[derive(Debug, Deserialize)]
pub struct IntentRequest {
    pub input: String,
}

// ============================================================
// Tauri Commands - Git Operations
// ============================================================

#[tauri::command]
async fn open_repo(state: State<'_, AppState>, path: String) -> Result<String, String> {
    let mut git_state = lock_state(&state.git)?;
    git_state.open_repo(&path)?;
    Ok(format!("仓库打开成功: {}", path))
}

#[tauri::command]
async fn get_branches(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let git_state = lock_state(&state.git)?;
    git_state.get_branches()
}

#[tauri::command]
async fn get_current_branch(state: State<'_, AppState>) -> Result<String, String> {
    let git_state = lock_state(&state.git)?;
    git_state.get_current_branch()
}

#[tauri::command]
async fn get_commits(
    state: State<'_, AppState>,
    branch: Option<String>,
    limit: Option<i32>,
) -> Result<Vec<CommitInfo>, String> {
    let git_state = lock_state(&state.git)?;
    git_state.get_commits(branch.as_deref(), limit.unwrap_or(20))
}

#[tauri::command]
async fn get_diff(
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> Result<Vec<DiffInfo>, String> {
    let git_state = lock_state(&state.git)?;
    git_state.get_diff(&from, &to)
}

#[tauri::command]
async fn get_branch_diff(
    state: State<'_, AppState>,
    branch1: String,
    branch2: String,
) -> Result<Vec<DiffInfo>, String> {
    let git_state = lock_state(&state.git)?;
    git_state.get_branch_diff(&branch1, &branch2)
}

#[tauri::command]
async fn get_file_history(
    state: State<'_, AppState>,
    file: String,
    time_range: Option<String>,
) -> Result<Vec<CommitInfo>, String> {
    let git_state = lock_state(&state.git)?;
    let since = intent::parse_time_range(time_range.as_deref().unwrap_or("3d"))?;
    git_state.get_file_history(&file, since)
}

// ============================================================
// Tauri Commands - AI Operations
// ============================================================

#[tauri::command]
async fn ai_chat(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: ChatRequest,
) -> Result<(), String> {
    let config = {
        let cfg_lock = lock_state(&state.ai_config)?;
        cfg_lock.clone().ok_or("AI 客户端未初始化，请检查 .env 配置".to_string())?
    };

    let tool_defs = tools::get_tool_defs();

    // Clone repo path for the async task
    let repo_path = {
        let git_state = lock_state(&state.git)?;
        git_state.path().map(|s| s.to_string())
    };

    tokio::spawn(async move {
        ai::run_agent_chat(app, config, request.messages, request.context, tool_defs, repo_path).await;
    });

    Ok(())
}

#[tauri::command]
async fn ai_analyze(
    state: State<'_, AppState>,
    request: AnalyzeRequest,
) -> Result<String, String> {
    let config = {
        let cfg_lock = lock_state(&state.ai_config)?;
        cfg_lock.clone().ok_or("AI 客户端未初始化".to_string())?
    };
    let diff_text = format_diff_for_ai(&request.diff);
    config.analyze_diff(&diff_text, &request.prompt).await
}

#[tauri::command]
async fn ai_analyze_stream(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: AnalyzeRequest,
) -> Result<(), String> {
    let config = {
        let cfg_lock = lock_state(&state.ai_config)?;
        cfg_lock.clone().ok_or("AI 客户端未初始化".to_string())?
    };
    let diff_text = format_diff_for_ai(&request.diff);

    tokio::spawn(async move {
        config.analyze_diff_stream(&app, &diff_text, &request.prompt).await;
    });

    Ok(())
}

#[tauri::command]
async fn parse_intent(request: IntentRequest) -> Result<serde_json::Value, String> {
    let parsed = intent::parse_intent(&request.input);
    Ok(serde_json::to_value(parsed).unwrap_or_default())
}

// ============================================================
// Helper: Format diff for AI consumption
// ============================================================

fn format_diff_for_ai(diff: &[DiffInfo]) -> String {
    let mut buf = String::new();
    for d in diff {
        buf.push_str(&format!("File: {}\n", d.file));
        buf.push_str(&format!("Added: {}, Deleted: {}\n", d.added, d.deleted));
        buf.push_str("Patch:\n");
        buf.push_str(&d.patch);
        buf.push_str("\n\n");
    }
    buf
}

// ============================================================
// Tauri Plugin Registration
// ============================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load .env file (optional — only warn on non-NotFound errors)
    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(dotenvy::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => eprintln!("Warning: failed to load .env file: {}", e),
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            git: Mutex::new(git::GitState::new()),
            ai_config: Mutex::new(ai::AiConfig::from_env()),
        })
        .invoke_handler(tauri::generate_handler![
            open_repo,
            get_branches,
            get_current_branch,
            get_commits,
            get_diff,
            get_branch_diff,
            get_file_history,
            ai_chat,
            ai_analyze,
            ai_analyze_stream,
            parse_intent,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
