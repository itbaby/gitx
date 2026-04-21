use crate::{ChatContext, InputMessage};
use serde_json::{json, Value};
use tauri::Emitter;

// ============================================================
// AI Config (OpenAI-compatible)
// ============================================================

const ANALYSIS_SYSTEM_PROMPT: &str = "你是一个专业的 Git 代码差异分析助手。你的任务是根据提供的 Git diff 内容，回答用户的问题。\n\n分析 diff 时请注意：\n1. 哪些文件发生了变更\n2. 具体做了什么改动\n3. 改动的目的和影响范围\n4. 潜在的问题或改进建议\n\n请用中文回答，使用 Markdown 格式，让回答清晰、简洁、有结构。";

#[derive(Clone)]
pub struct AiConfig {
    pub model: String,
    pub api_key: String,
    pub base_url: String,
    client: reqwest::Client,
}

impl AiConfig {
    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var("OPENAI_API_KEY").ok()?;
        let model = std::env::var("AI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());
        let base_url = std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| {
            "https://api.openai.com/v1".to_string()
        });

        Some(AiConfig {
            model,
            api_key,
            base_url,
            client: reqwest::Client::builder()
                .connect_timeout(std::time::Duration::from_secs(10))
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        })
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
            format!("Bearer {}", self.api_key)
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
        let user_prompt = format!(
            "## Git Diff:\n```\n{}\n```\n\n## 问题:\n{}",
            diff, prompt
        );

        let body = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": ANALYSIS_SYSTEM_PROMPT },
                { "role": "user", "content": user_prompt }
            ],
            "max_completion_tokens": 4000,
            "temperature": 0.3
        });

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
    ) {
        let user_prompt = format!(
            "## Git Diff:\n```\n{}\n```\n\n## 问题:\n{}",
            diff, prompt
        );

        let body = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": ANALYSIS_SYSTEM_PROMPT },
                { "role": "user", "content": user_prompt }
            ],
            "max_completion_tokens": 4000,
            "temperature": 0.3,
            "stream": true
        });

        if let Err(e) = self.stream_sse(app, body, "analyze").await {
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

        use futures::StreamExt;
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
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
                            if let Some(content) =
                                parsed["choices"][0]["delta"]["content"].as_str()
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
        let body = json!({
            "model": config.model,
            "messages": &messages,
            "tools": &tool_defs,
            "tool_choice": "auto"
        });

        match send_non_streaming(&config, &body).await {
            Ok(result) => {
                let choice = &result["choices"][0];
                let msg = &choice["message"];

                if let Some(tool_calls) = msg["tool_calls"].as_array() {
                    if tool_calls.is_empty() {
                        messages.push(msg.clone());
                        let stream_body = json!({
                            "model": config.model,
                            "messages": &messages,
                            "stream": true
                        });
                        if let Err(e) = config.stream_sse(&app, stream_body, "chat").await {
                            emit_log(&app, "ai-error", &e);
                        }
                        return;
                    }

                    messages.push(msg.clone());

                    for tc in tool_calls {
                        let tool_name = tc["function"]["name"].as_str().unwrap_or("unknown");
                        let tool_args = tc["function"]["arguments"].as_str().unwrap_or("{}");
                        let tool_call_id = tc["id"].as_str().unwrap_or("").to_string();

                        let display = get_tool_display_name(tool_name);
                        emit_log(
                            &app,
                            "ai-tool",
                            &json!({
                                "name": tool_name,
                                "display": display
                            }),
                        );

                        let tool_result =
                            super::tools::call_tool(tool_name, tool_args, &repo_path);

                        messages.push(json!({
                            "role": "tool",
                            "content": tool_result,
                            "tool_call_id": tool_call_id
                        }));
                    }
                    continue;
                } else {
                    messages.push(msg.clone());
                    let stream_body = json!({
                        "model": config.model,
                        "messages": &messages,
                        "stream": true
                    });
                    if let Err(e) = config.stream_sse(&app, stream_body, "chat").await {
                        emit_log(&app, "ai-error", &e);
                    }
                    return;
                }
            }
            Err(e) => {
                emit_log(&app, "ai-error", &format!("AI 调用失败: {}", e));
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

async fn send_non_streaming(config: &AiConfig, body: &Value) -> Result<Value, String> {
    let response = config
        .client()
        .post(config.chat_url())
        .headers(config.headers()?)
        .json(body)
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

/// Emit a Tauri event, logging any errors instead of silently discarding them.
fn emit_log(app: &tauri::AppHandle, event: &str, payload: &(impl serde::Serialize + ?Sized)) {
    if let Err(e) = app.emit(event, payload) {
        eprintln!("[emit error] {}: {}", event, e);
    }
}
