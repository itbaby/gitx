mod ai;
mod git;
mod intent;
mod tools;

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::State;
use tokio::sync::Semaphore;

pub struct AppState {
    pub git: Mutex<git::GitState>,
    pub ai_config: Mutex<Option<ai::AiConfig>>,
    pub ai_semaphore: Arc<Semaphore>,
}

impl AppState {
    fn repo_path(&self) -> Result<String, String> {
        self.git.lock().unwrap().path()
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

#[derive(Debug, Deserialize)]
pub struct IntentRequest {
    pub input: String,
}

#[tauri::command]
async fn open_repo(state: State<'_, AppState>, path: String) -> Result<String, String> {
    state.git.lock().unwrap().open_repo(&path)?;
    Ok(format!("仓库打开成功: {}", path))
}

#[tauri::command]
async fn get_branches(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    spawn_git(state.repo_path()?, |gs| gs.get_branches()).await
}

#[tauri::command]
async fn get_current_branch(state: State<'_, AppState>) -> Result<String, String> {
    spawn_git(state.repo_path()?, |gs| gs.get_current_branch()).await
}

#[tauri::command]
async fn get_commits(
    state: State<'_, AppState>,
    branch: Option<String>,
    limit: Option<i32>,
) -> Result<Vec<CommitInfo>, String> {
    if let Some(ref b) = branch { tools::validate_branch(b)?; }
    let limit = limit.unwrap_or(20).min(100);
    spawn_git(state.repo_path()?, move |gs| gs.get_commits(branch.as_deref(), limit)).await
}

#[tauri::command]
async fn get_diff(
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> Result<Vec<DiffInfo>, String> {
    tools::validate_hash(&from)?;
    tools::validate_hash(&to)?;
    spawn_git(state.repo_path()?, move |gs| gs.get_diff(&from, &to)).await
}

#[tauri::command]
async fn get_branch_diff(
    state: State<'_, AppState>,
    branch1: String,
    branch2: String,
) -> Result<Vec<DiffInfo>, String> {
    tools::validate_branch(&branch1)?;
    tools::validate_branch(&branch2)?;
    spawn_git(state.repo_path()?, move |gs| gs.get_branch_diff(&branch1, &branch2)).await
}

#[tauri::command]
async fn get_commit_diff(
    state: State<'_, AppState>,
    hash: String,
) -> Result<Vec<DiffInfo>, String> {
    tools::validate_hash(&hash)?;
    spawn_git(state.repo_path()?, move |gs| gs.get_commit_diff(&hash)).await
}

#[tauri::command]
async fn get_file_history(
    state: State<'_, AppState>,
    file: String,
    time_range: Option<String>,
) -> Result<Vec<CommitInfo>, String> {
    tools::validate_file_path(&file)?;
    let since = intent::parse_time_range(time_range.as_deref().unwrap_or("3d"))?;
    spawn_git(state.repo_path()?, move |gs| gs.get_file_history(&file, since)).await
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

    let config = state.ai_config.lock().unwrap().clone()
        .ok_or("AI 客户端未初始化，请检查 .env 配置")?;

    let repo_path = state.git.lock().unwrap().path().map(|s| s.to_string());

    tokio::spawn(async move {
        let _permit = permit;
        ai::run_agent_chat(app, config, request.messages, request.context, tools::get_tool_defs(), repo_path).await;
    });

    Ok(())
}

#[tauri::command]
async fn ai_analyze(state: State<'_, AppState>, request: AnalyzeRequest) -> Result<String, String> {
    let _permit = state.ai_semaphore.clone().acquire_owned().await
        .map_err(|e| format!("获取并发许可失败: {}", e))?;
    let config = state.ai_config.lock().unwrap().clone()
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
    let config = state.ai_config.lock().unwrap().clone()
        .ok_or("AI 客户端未初始化")?;

    tokio::spawn(async move {
        let _permit = permit;
        config.analyze_diff_stream(&app, &tools::format_diff(&request.diff), &request.prompt).await;
    });

    Ok(())
}

#[tauri::command]
async fn parse_intent(request: IntentRequest) -> Result<serde_json::Value, String> {
    Ok(serde_json::to_value(intent::parse_intent(&request.input)).unwrap_or_default())
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
            parse_intent,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
