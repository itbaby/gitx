mod ai;
mod git;
mod intent;
mod tools;

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::State;
use tokio::sync::{Mutex, Semaphore};

pub struct AppState {
    pub git: Mutex<git::GitState>,
    pub ai_config: Mutex<Option<ai::AiConfig>>,
    pub ai_semaphore: Arc<Semaphore>,
    /// Global cancel flag for the in-flight AI stream. Set by `ai_cancel`,
    /// cleared at the start of every new AI request so it never carries
    /// across sessions. Single-flag design matches the UX: the frontend
    /// disables the send button while streaming, so at most one stream is
    /// "live" from the user's perspective at any time.
    pub ai_cancel: Arc<AtomicBool>,
}

impl AppState {
    async fn repo_path(&self) -> Result<String, String> {
        self.git.lock().await.path()
            .map(|s| s.to_string())
            .ok_or_else(|| "未打开仓库".to_string())
    }
}

async fn spawn_git<T>(
    path: String,
    f: impl FnOnce(&git::GitState) -> Result<T, String> + Send + 'static,
) -> Result<T, String>
where
    T: Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        let gs = git::GitState::from_path(&path)?;
        f(&gs)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[tauri::command]
async fn open_repo(state: State<'_, AppState>, path: String) -> Result<String, String> {
    state.git.lock().await.open_repo(&path)?;
    Ok(format!("仓库打开成功: {}", path))
}

#[tauri::command]
async fn get_branches(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    spawn_git(state.repo_path().await?, |gs| gs.get_branches()).await
}

#[tauri::command]
async fn get_current_branch(state: State<'_, AppState>) -> Result<String, String> {
    spawn_git(state.repo_path().await?, |gs| gs.get_current_branch()).await
}

#[tauri::command]
async fn get_commits(
    state: State<'_, AppState>,
    branch: Option<String>,
    limit: Option<i32>,
) -> Result<Vec<CommitInfo>, String> {
    if let Some(ref b) = branch { tools::validate_branch(b)?; }
    // Clamp to [1, 100], mirroring the agent tool path in tools.rs. A
    // negative IPC value would otherwise wrap to a huge `usize` via
    // `as usize` and drive `revwalk.take(usize::MAX)`, walking the
    // entire repository history.
    let limit = limit.unwrap_or(20).clamp(1, 100);
    spawn_git(state.repo_path().await?, move |gs| gs.get_commits(branch.as_deref(), limit)).await
}

#[tauri::command]
async fn get_diff(
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> Result<Vec<DiffInfo>, String> {
    tools::validate_hash(&from)?;
    tools::validate_hash(&to)?;
    spawn_git(state.repo_path().await?, move |gs| gs.get_diff(&from, &to)).await
}

#[tauri::command]
async fn get_branch_diff(
    state: State<'_, AppState>,
    branch1: String,
    branch2: String,
) -> Result<Vec<DiffInfo>, String> {
    tools::validate_branch(&branch1)?;
    tools::validate_branch(&branch2)?;
    spawn_git(state.repo_path().await?, move |gs| gs.get_branch_diff(&branch1, &branch2)).await
}

#[tauri::command]
async fn get_commit_diff(
    state: State<'_, AppState>,
    hash: String,
) -> Result<Vec<DiffInfo>, String> {
    tools::validate_hash(&hash)?;
    spawn_git(state.repo_path().await?, move |gs| gs.get_commit_diff(&hash)).await
}

#[tauri::command]
async fn get_file_history(
    state: State<'_, AppState>,
    file: String,
    time_range: Option<String>,
) -> Result<Vec<CommitInfo>, String> {
    tools::validate_file_path(&file)?;
    let since = intent::parse_time_range(time_range.as_deref().unwrap_or("3d"))?;
    spawn_git(state.repo_path().await?, move |gs| gs.get_file_history(&file, since)).await
}

// ============================================================
// AI Commands
// ============================================================

#[tauri::command]
async fn ai_chat(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: ChatRequest,
) -> Result<(), String> {
    if request.messages.len() > 50 {
        return Err("消息历史过长，最多支持 50 条".into());
    }

    let permit = state.ai_semaphore.clone().acquire_owned().await
        .map_err(|e| format!("获取并发许可失败: {}", e))?;

    let config = state.ai_config.lock().await.clone()
        .ok_or("AI 客户端未初始化，请检查 .env 配置")?;

    let repo_path = state.git.lock().await.path().map(|s| s.to_string());

    // Clear any stale cancel from a prior session before starting.
    state.ai_cancel.store(false, Ordering::SeqCst);
    let cancel = state.ai_cancel.clone();

    tokio::spawn(async move {
        let _permit = permit;
        ai::run_agent_chat(app, config, request.messages, request.context, tools::get_tool_defs(), repo_path, cancel).await;
    });

    Ok(())
}

#[tauri::command]
async fn ai_analyze(state: State<'_, AppState>, request: AnalyzeRequest) -> Result<String, String> {
    let _permit = state.ai_semaphore.clone().acquire_owned().await
        .map_err(|e| format!("获取并发许可失败: {}", e))?;
    let config = state.ai_config.lock().await.clone()
        .ok_or("AI 客户端未初始化")?;
    config.analyze_diff(&tools::format_diff(&request.diff), &request.prompt).await
}

#[tauri::command]
async fn ai_analyze_stream(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: AnalyzeRequest,
) -> Result<(), String> {
    let permit = state.ai_semaphore.clone().acquire_owned().await
        .map_err(|e| format!("获取并发许可失败: {}", e))?;
    let config = state.ai_config.lock().await.clone()
        .ok_or("AI 客户端未初始化")?;

    state.ai_cancel.store(false, Ordering::SeqCst);
    let cancel = state.ai_cancel.clone();

    tokio::spawn(async move {
        let _permit = permit;
        config.analyze_diff_stream(&app, &tools::format_diff(&request.diff), &request.prompt, &cancel).await;
    });

    Ok(())
}

/// Cancel the in-flight AI stream (chat or analyze). The backend stream
/// loop checks the flag between chunks and emits the terminal `done` event
/// so the frontend closes out the message normally.
#[tauri::command]
async fn ai_cancel(state: State<'_, AppState>) -> Result<(), String> {
    state.ai_cancel.store(true, Ordering::SeqCst);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(e) = dotenvy::dotenv() {
        if !matches!(&e, dotenvy::Error::Io(e) if e.kind() == std::io::ErrorKind::NotFound) {
            eprintln!("Warning: failed to load .env file: {}", e);
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            git: Mutex::new(git::GitState::new()),
            ai_config: Mutex::new(ai::AiConfig::from_env()),
            ai_semaphore: Arc::new(Semaphore::new(3)),
            ai_cancel: Arc::new(AtomicBool::new(false)),
        })
        .invoke_handler(tauri::generate_handler![
            open_repo,
            get_branches,
            get_current_branch,
            get_commits,
            get_diff,
            get_branch_diff,
            get_commit_diff,
            get_file_history,
            ai_chat,
            ai_analyze,
            ai_analyze_stream,
            ai_cancel,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::CommitInfo;

    /// Locks the JSON contract between backend and frontend. The Vue templates
    /// read `commit.shortHash`, so the serialized key MUST be camelCase — a
    /// regression here renders commit hashes blank in the UI.
    #[test]
    fn commit_info_serializes_camel_case() {
        let info = CommitInfo {
            hash: "0123456789abcdef".to_string(),
            short_hash: "0123456".to_string(),
            message: "msg".to_string(),
            author: "tester".to_string(),
            email: "t@x.com".to_string(),
            timestamp: "0".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(
            json.contains("\"shortHash\""),
            "expected camelCase shortHash in JSON, got: {}",
            json
        );
        assert!(!json.contains("short_hash"), "snake_case leaked into JSON: {}", json);
    }
}
