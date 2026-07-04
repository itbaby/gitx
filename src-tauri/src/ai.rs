use crate::{ChatContext, InputMessage};
use futures::StreamExt;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::Emitter;

// ============================================================
// AI Config (OpenAI-compatible)
// ============================================================

const ANALYSIS_SYSTEM_PROMPT: &str = "你是一个专业的 Git 代码差异分析助手。你的任务是根据提供的 Git diff 内容，回答用户的问题。\n\n分析 diff 时请注意：\n1. 哪些文件发生了变更\n2. 具体做了什么改动\n3. 改动的目的和影响范围\n4. 潜在的问题或改进建议\n\n请用中文回答，使用 Markdown 格式，让回答清晰、简洁、有结构。";

/// Wrapper that masks the API key in Debug output.
#[derive(Clone)]
pub struct Secret(String);

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret(**REDACTED**)")
    }
}

impl Secret {
    pub fn new(val: String) -> Self {
        Self(val)
    }
    pub fn expose(&self) -> &str {
        &self.0
    }
}

#[derive(Clone)]
pub struct AiConfig {
    pub model: String,
    pub api_key: Secret,
    pub base_url: String,
    /// Sampling temperature. Hoisted out of the four call sites so it's tunable
    /// per deployment via `AI_TEMPERATURE`.
    pub temperature: f64,
    /// Max output tokens for each completion.
    pub max_tokens: u32,
    /// JSON field name used to carry `max_tokens`. OpenAI's newer API wants
    /// `max_completion_tokens`, but most OpenAI-compatible gateways only
    /// accept the legacy `max_tokens`. Configurable via `AI_MAX_TOKENS_FIELD`.
    pub max_tokens_field: String,
    client: reqwest::Client,
}

impl AiConfig {
    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var("OPENAI_API_KEY").ok()?;
        let model = std::env::var("AI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());
        let base_url = std::env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let temperature = std::env::var("AI_TEMPERATURE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.3);
        let max_tokens = std::env::var("AI_MAX_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4000);
        let max_tokens_field = std::env::var("AI_MAX_TOKENS_FIELD")
            .unwrap_or_else(|_| "max_completion_tokens".to_string());

        Some(AiConfig {
            model,
            api_key: Secret::new(api_key),
            base_url,
            temperature,
            max_tokens,
            max_tokens_field,
            client: reqwest::Client::builder()
                .connect_timeout(std::time::Duration::from_secs(10))
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        })
    }

    /// Build the shared base of every chat request body (model, temperature,
    /// max-tokens field). Call sites merge in their own `messages`/`tools`/
    /// `stream` keys, so the shared params are defined in exactly one place.
    fn base_body(&self) -> serde_json::Map<String, Value> {
        let mut map = serde_json::Map::new();
        map.insert("model".to_string(), Value::String(self.model.clone()));
        map.insert("temperature".to_string(), json!(self.temperature));
        map.insert(self.max_tokens_field.clone(), json!(self.max_tokens));
        map
    }

    fn chat_url(&self) -> String {
        format!("{}/chat/completions", self.base_url.trim_end_matches('/'))
    }

    fn client(&self) -> &reqwest::Client {
        &self.client
    }

    fn headers(&self) -> Result<reqwest::header::HeaderMap, String> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", self.api_key.expose())
                .parse()
                .map_err(|e| format!("无效的 API key 格式: {}", e))?,
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json"
                .parse()
                .map_err(|_| "内部错误: 无法解析 Content-Type".to_string())?,
        );
        Ok(headers)
    }

    // ========================================================
    // Non-streaming analysis
    // ========================================================

    pub async fn analyze_diff(&self, diff: &str, prompt: &str) -> Result<String, String> {
        let user_prompt = format!("## Git Diff:\n```\n{}\n```\n\n## 问题:\n{}", diff, prompt);

        let mut body = self.base_body();
        body.insert(
            "messages".to_string(),
            json!([
                { "role": "system", "content": ANALYSIS_SYSTEM_PROMPT },
                { "role": "user", "content": user_prompt }
            ]),
        );
        let body = Value::Object(body);

        let response = self
            .client()
            .post(self.chat_url())
            .headers(self.headers()?)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("AI API 请求失败: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("AI API 错误 ({}): {}", status, body));
        }

        let result: Value = response
            .json()
            .await
            .map_err(|e| format!("AI 响应解析失败: {}", e))?;

        result["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "AI 未返回有效响应".to_string())
    }

    // ========================================================
    // Streaming analysis
    // ========================================================

    pub async fn analyze_diff_stream(
        &self,
        app: &tauri::AppHandle,
        diff: &str,
        prompt: &str,
        cancel: &Arc<AtomicBool>,
    ) {
        let user_prompt = format!("## Git Diff:\n```\n{}\n```\n\n## 问题:\n{}", diff, prompt);

        let mut body = self.base_body();
        body.insert(
            "messages".to_string(),
            json!([
                { "role": "system", "content": ANALYSIS_SYSTEM_PROMPT },
                { "role": "user", "content": user_prompt }
            ]),
        );
        body.insert("stream".to_string(), json!(true));
        let body = Value::Object(body);

        if let Err(e) = self.stream_sse(app, body, "analyze", cancel).await {
            emit_log(app, "ai-error", &e);
        }
    }

    // ========================================================
    // Internal: SSE streaming
    // ========================================================

    pub(crate) async fn stream_sse(
        &self,
        app: &tauri::AppHandle,
        body: Value,
        event_prefix: &str,
        cancel: &Arc<AtomicBool>,
    ) -> Result<(), String> {
        let response = self
            .client()
            .post(self.chat_url())
            .headers(self.headers()?)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("AI 流式请求失败: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("AI API 错误 ({}): {}", status, body));
        }

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            // Cooperative cancel: user hit Stop in the frontend. Emit the
            // terminal event so the UI closes out the streaming message
            // instead of hanging on a half-finished reply.
            if cancel.load(Ordering::Relaxed) {
                let ev_name = format!("ai-{}-done", event_prefix);
                emit_log(app, &ev_name, &());
                return Ok(());
            }
            let chunk = chunk.map_err(|e| format!("流式读取错误: {}", e))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(pos) = buffer.find("\n\n") {
                let event_block = buffer[..pos].to_string();
                buffer = buffer[pos + 2..].to_string();

                for line in event_block.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if data.trim() == "[DONE]" {
                            let ev_name = format!("ai-{}-done", event_prefix);
                            emit_log(app, &ev_name, &());
                            return Ok(());
                        }

                        if let Ok(parsed) = serde_json::from_str::<Value>(data) {
                            if let Some(content) = parsed["choices"][0]["delta"]["content"].as_str()
                            {
                                if !content.is_empty() {
                                    let ev_name = format!("ai-{}-chunk", event_prefix);
                                    emit_log(app, &ev_name, content);
                                }
                            }
                        }
                    }
                }
            }
        }

        let ev_name = format!("ai-{}-done", event_prefix);
        emit_log(app, &ev_name, &());
        Ok(())
    }
}

// ============================================================
// Agent Chat (free function - called from lib.rs)
// ============================================================

pub async fn run_agent_chat(
    app: tauri::AppHandle,
    config: AiConfig,
    history: Vec<InputMessage>,
    chat_ctx: Option<ChatContext>,
    tool_defs: Vec<Value>,
    repo_path: Option<String>,
    cancel: Arc<AtomicBool>,
) {
    let system_prompt = build_system_prompt(&chat_ctx);
    let max_rounds = 5;

    let mut messages: Vec<Value> = vec![json!({
        "role": "system",
        "content": system_prompt
    })];

    for msg in &history {
        messages.push(json!({
            "role": msg.role,
            "content": msg.content
        }));
    }

    for _round in 0..max_rounds {
        // Honor a cancel that landed between rounds — emit the terminal
        // event so the frontend closes out the streaming message.
        if cancel.load(Ordering::Relaxed) {
            emit_log(&app, "ai-chat-done", &());
            return;
        }

        match agent_round(&config, &messages, &tool_defs, &cancel).await {
            AgentStep::Cancelled => {
                emit_log(&app, "ai-chat-done", &());
                return;
            }
            AgentStep::Failed(err) => {
                emit_log(&app, "ai-error", &format!("AI 调用失败: {}", err));
                return;
            }
            AgentStep::Reply(result) => {
                let msg = &result["choices"][0]["message"];
                if handle_tool_calls(&app, msg, &mut messages, &repo_path).await {
                    continue;
                }
                messages.push(msg.clone());
                stream_final_reply(&config, &app, &messages, &cancel).await;
                return;
            }
        }
    }

    emit_log(
        &app,
        "ai-chat-chunk",
        "抱歉，操作步骤过多，请尝试更具体的描述。",
    );
    emit_log(&app, "ai-chat-done", &());
}

// ============================================================
// Agent-loop helpers (extracted from `run_agent_chat`)
// ============================================================

/// Outcome of one agent round — a tool-decision call to the model.
enum AgentStep {
    /// Model returned a response (may contain tool calls or final text).
    Reply(Value),
    /// Both the initial call and the retry failed, or a non-transient error.
    Failed(String),
    /// User hit Stop while the request was in flight.
    Cancelled,
}

/// Send the tool-decision request, retrying once on transient errors and
/// aborting if the user cancels. Encapsulates the retry + cancel race that
/// used to be inlined in `run_agent_chat`.
async fn agent_round(
    config: &AiConfig,
    messages: &[Value],
    tool_defs: &[Value],
    cancel: &Arc<AtomicBool>,
) -> AgentStep {
    let mut body = config.base_body();
    body.insert("messages".to_string(), json!(messages));
    body.insert("tools".to_string(), json!(tool_defs));
    body.insert("tool_choice".to_string(), json!("auto"));
    let body = Value::Object(body);

    let mut last_err: Option<String> = None;
    loop {
        // Race the network call against the cancel flag. If the user hits
        // Stop mid-request, the `send_non_streaming` future is dropped
        // (aborting the underlying connection).
        let res = tokio::select! {
            r = send_non_streaming(config, &body) => r,
            _ = cancel_signal(cancel) => return AgentStep::Cancelled,
        };
        match res {
            Ok(r) => return AgentStep::Reply(r),
            Err(e) => {
                if last_err.is_some() || !is_transient_error(&e) {
                    return AgentStep::Failed(e);
                }
                last_err = Some(e);
                // Brief backoff before retry — also cancellable.
                tokio::select! {
                    _ = tokio::time::sleep(std::time::Duration::from_millis(800)) => {},
                    _ = cancel_signal(cancel) => return AgentStep::Cancelled,
                }
            }
        }
    }
}

/// Execute every tool call in `msg`, emit progress/result events, and append
/// the tool responses to `messages`. Returns `true` if any tool call ran
/// (meaning the agent loop should continue to the next round).
async fn handle_tool_calls(
    app: &tauri::AppHandle,
    msg: &Value,
    messages: &mut Vec<Value>,
    repo_path: &Option<String>,
) -> bool {
    let Some(tool_calls) = msg["tool_calls"].as_array() else {
        return false;
    };
    if tool_calls.is_empty() {
        return false;
    }

    messages.push(msg.clone());
    for tc in tool_calls {
        let tool_name = tc["function"]["name"].as_str().unwrap_or("unknown");
        let tool_args = tc["function"]["arguments"].as_str().unwrap_or("{}");
        let tool_call_id = tc["id"].as_str().unwrap_or("").to_string();

        emit_log(
            app,
            "ai-tool",
            &json!({
                "name": tool_name,
                "display": get_tool_display_name(tool_name)
            }),
        );

        let tn = tool_name.to_string();
        let ta = tool_args.to_string();
        let rp = repo_path.clone();
        let tool_result = tokio::task::spawn_blocking(move || super::tools::call_tool(&tn, &ta, &rp))
            .await
            .unwrap_or_else(|e| format!("工具执行失败: {}", e));

        emit_log(
            app,
            "ai-tool-result",
            &json!({
                "name": tool_name,
                "result": truncate_for_display(&tool_result),
            }),
        );

        messages.push(json!({
            "role": "tool",
            "content": tool_result,
            "tool_call_id": tool_call_id
        }));
    }
    true
}

/// Stream the final assistant reply (no tools) back to the frontend.
async fn stream_final_reply(
    config: &AiConfig,
    app: &tauri::AppHandle,
    messages: &[Value],
    cancel: &Arc<AtomicBool>,
) {
    let mut stream_body = config.base_body();
    stream_body.insert("messages".to_string(), json!(messages));
    stream_body.insert("stream".to_string(), json!(true));
    let stream_body = Value::Object(stream_body);
    if let Err(e) = config.stream_sse(app, stream_body, "chat", cancel).await {
        emit_log(app, "ai-error", &e);
    }
}

// ============================================================
// Free helper functions
// ============================================================

fn build_system_prompt(chat_ctx: &Option<ChatContext>) -> String {
    let mut p = "你是一个专业的 Git 代码分析助手，名叫 GitX AI。你可以帮助用户浏览 Git 仓库、比较分支差异、查看提交历史和文件变更记录。\n\n工作方式：\n1. 根据用户的问题，使用提供的工具获取 Git 数据\n2. 基于获取到的数据，给出专业、清晰的回答\n3. 如果需要多个工具配合使用，请依次调用\n\n输出格式要求（严格遵守）：\n- 使用 Markdown 格式，结构清晰\n- 提交列表：用有序列表，每项包含 **提交哈希**（短哈希）、提交信息、作者、时间，用 - 子项排列\n- 差异分析：先总结变更概览，再按文件逐个说明\n- 代码内容：用代码块包裹，标注语言类型\n- 关键信息：用 **加粗** 标注\n- 简洁为主，避免冗余描述\n\n请用中文回答。".to_string();

    if let Some(ctx) = chat_ctx {
        if ctx.has_diff && !ctx.base_branch.is_empty() && !ctx.compare_branch.is_empty() {
            p.push_str(&format!(
                "\n\n当前用户正在查看分支 {} 和 {} 之间的代码差异。当用户提到「分析当前差异」或「这些变更」时，请使用 get_branch_diff 工具获取这两个分支的差异进行分析。",
                ctx.base_branch, ctx.compare_branch
            ));
        }
    }

    p
}

/// Classify whether an error from `send_non_streaming` is worth retrying.
///
/// Retrying is only worthwhile for transient failures: connectivity errors
/// and HTTP 5xx server errors. Client errors (4xx, e.g. auth/bad request)
/// and response parse failures are not retried, since they will not succeed
/// on a second attempt.
fn is_transient_error(err: &str) -> bool {
    // Network / connectivity failures (produced by `.send()` and stream reads).
    if err.contains("请求失败") {
        return true;
    }
    // HTTP error responses: `AI API 错误 (503): ...` -> retry only on 5xx.
    if let Some(rest) = err.strip_prefix("AI API 错误 (") {
        if let Some(digit) = rest.chars().next() {
            return digit == '5';
        }
    }
    false
}

async fn send_non_streaming(config: &AiConfig, body: &Value) -> Result<Value, String> {
    let response = config
        .client()
        .post(config.chat_url())
        .headers(config.headers()?)
        .json(body)
        // Per-request cap so a single hung provider call can't stall the
        // whole multi-round agent loop. Lower than the client-wide timeout.
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| format!("AI API 请求失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("AI API 错误 ({}): {}", status, body));
    }

    response
        .json()
        .await
        .map_err(|e| format!("AI 响应解析失败: {}", e))
}

/// A future that resolves once `cancel` is set. Used to race against a
/// network request via `tokio::select!` so the user's Stop button can
/// abort an in-flight (non-streaming) tool-decision call instead of
/// waiting for it to finish. Polls at the same 100ms granularity as the
/// SSE loop's cancel check.
async fn cancel_signal(cancel: &Arc<AtomicBool>) {
    loop {
        if cancel.load(Ordering::Relaxed) {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

fn get_tool_display_name(tool_name: &str) -> String {
    match tool_name {
        "get_branches" => "正在获取分支列表...".to_string(),
        "get_current_branch" => "正在获取当前分支...".to_string(),
        "get_branch_diff" => "正在比较分支差异...".to_string(),
        "get_commits" => "正在获取提交历史...".to_string(),
        "get_file_history" => "正在获取文件变更记录...".to_string(),
        "get_diff" => "正在获取代码差异...".to_string(),
        _ => format!("正在执行 {}...", tool_name),
    }
}

/// Truncate tool result for display in the frontend chat.
/// Keeps the first 500 chars so users can see what the tool returned
/// without flooding the UI.
fn truncate_for_display(result: &str) -> String {
    const MAX_DISPLAY: usize = 500;
    if result.chars().count() > MAX_DISPLAY {
        let truncated: String = result.chars().take(MAX_DISPLAY).collect();
        format!("{}\n...(已截断)", truncated)
    } else {
        result.to_string()
    }
}

/// Emit a Tauri event, logging any errors instead of silently discarding them.
fn emit_log(app: &tauri::AppHandle, event: &str, payload: &(impl serde::Serialize + ?Sized)) {
    if let Err(e) = app.emit(event, payload) {
        eprintln!("[emit error] {}: {}", event, e);
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::{is_transient_error, AiConfig, Secret};

    fn test_config(field: &str) -> AiConfig {
        AiConfig {
            model: "test".to_string(),
            api_key: Secret::new("k".to_string()),
            base_url: "http://x".to_string(),
            temperature: 0.3,
            max_tokens: 4000,
            max_tokens_field: field.to_string(),
            client: reqwest::Client::new(),
        }
    }

    #[test]
    fn base_body_uses_configured_token_field() {
        // Default OpenAI field name.
        let cfg = test_config("max_completion_tokens");
        let body = cfg.base_body();
        assert!(body.contains_key("max_completion_tokens"));
        assert!(!body.contains_key("max_tokens"));
        assert_eq!(body.get("temperature").and_then(|v| v.as_f64()), Some(0.3));
        assert_eq!(body.get("max_completion_tokens").and_then(|v| v.as_u64()), Some(4000));

        // Legacy field name for OpenAI-compatible gateways (e.g. BigModel).
        let cfg2 = test_config("max_tokens");
        let body2 = cfg2.base_body();
        assert!(body2.contains_key("max_tokens"));
        assert!(!body2.contains_key("max_completion_tokens"));
    }

    #[test]
    fn network_failures_are_transient() {
        assert!(is_transient_error("AI API 请求失败: connection reset"));
        assert!(is_transient_error("AI 流式请求失败: dns lookup failed"));
    }

    #[test]
    fn http_5xx_is_transient() {
        assert!(is_transient_error("AI API 错误 (500): Internal Server Error"));
        assert!(is_transient_error("AI API 错误 (503): Service Unavailable"));
        assert!(is_transient_error("AI API 错误 (529): Overloaded"));
    }

    #[test]
    fn http_4xx_is_not_transient() {
        assert!(!is_transient_error("AI API 错误 (401): Unauthorized"));
        assert!(!is_transient_error("AI API 错误 (400): Bad Request"));
        assert!(!is_transient_error("AI API 错误 (404): Not Found"));
        assert!(!is_transient_error("AI API 错误 (422): Unprocessable"));
    }

    #[test]
    fn parse_failures_are_not_transient() {
        assert!(!is_transient_error("AI 响应解析失败: invalid json"));
    }

    #[test]
    fn unrecognized_errors_are_not_transient() {
        assert!(!is_transient_error("something else entirely"));
    }
}
