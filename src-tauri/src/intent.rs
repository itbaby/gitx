use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

// ============================================================
// Intent Types
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentType {
    DiffFile,
    DiffBranch,
    DiffCommit,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    #[serde(rename = "type")]
    pub intent_type: IntentType,
    pub file: String,
    pub branch1: String,
    pub branch2: String,
    pub commit1: String,
    pub commit2: String,
    pub time_range: String,
}

// ============================================================
// Cached Regex Patterns (compiled once)
// ============================================================

static BRANCH_DIFF_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"比较.*分支",
        r"比较.*branch",
        r"compare.*branch",
        r"分支.*对比",
        r"branch.*compare",
        r"branch.*diff",
        r"当前.*main",
        r"current.*main",
    ]
    .iter()
    .filter_map(|p| Regex::new(p).ok())
    .collect()
});

static COMMIT_DIFF_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"比较.*提交",
        r"比较.*commit",
        r"commit.*compare",
        r"commit.*diff",
        r"[0-9a-f]{7,}",
    ]
    .iter()
    .filter_map(|p| Regex::new(p).ok())
    .collect()
});

static FILE_HISTORY_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"(?:过去|最近).*天",
        r"(?:last|past).*days",
        r"(?:过去|最近).*小时",
        r"(?:last|past).*hours",
        r"文件.*历史",
        r"file.*history",
        r"改动",
        r"变更",
        r"changes",
    ]
    .iter()
    .filter_map(|p| Regex::new(p).ok())
    .collect()
});

static BRANCH1_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:分支|branch)\s*(\S+)\s*(?:和|与|vs|compare|with)").unwrap());

static BRANCH2_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:和|与|vs|compare|with)\s*(?:分支|branch)?\s*(\S+)").unwrap());

static COMMIT_HASH_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([0-9a-f]{7,40})").unwrap());

static TIME_RANGE_DAYS_CN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:过去|最近)(\d+)天").unwrap());

static TIME_RANGE_DAYS_EN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:last|past)\s*(\d+)\s*days?").unwrap());

static TIME_RANGE_HOURS_CN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:过去|最近)(\d+)小时").unwrap());

static TIME_RANGE_PARSE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+)([dhm])").unwrap());

static FILE_QUOTED_DQ: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#""([^"]+\.\w+)""#).unwrap());

static FILE_QUOTED_SQ: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"'([^']+\.\w+)'").unwrap());

static FILE_PATH_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([\w./\\-]+\.\w+)").unwrap());

// ============================================================
// Parse Intent
// ============================================================

pub fn parse_intent(input: &str) -> Intent {
    let input_lower = input.to_lowercase();

    if is_branch_diff(&input_lower) {
        return Intent {
            intent_type: IntentType::DiffBranch,
            file: String::new(),
            branch1: extract_branch1(&input_lower),
            branch2: extract_branch2(&input_lower),
            commit1: String::new(),
            commit2: String::new(),
            time_range: String::new(),
        };
    }

    if is_commit_diff(&input_lower) {
        return Intent {
            intent_type: IntentType::DiffCommit,
            file: String::new(),
            branch1: String::new(),
            branch2: String::new(),
            commit1: extract_commit1(&input_lower),
            commit2: extract_commit2(&input_lower),
            time_range: String::new(),
        };
    }

    if is_file_history(&input_lower) {
        return Intent {
            intent_type: IntentType::DiffFile,
            file: extract_file(input),
            branch1: String::new(),
            branch2: String::new(),
            commit1: String::new(),
            commit2: String::new(),
            time_range: extract_time_range(&input_lower),
        };
    }

    Intent {
        intent_type: IntentType::Unknown,
        file: String::new(),
        branch1: String::new(),
        branch2: String::new(),
        commit1: String::new(),
        commit2: String::new(),
        time_range: String::new(),
    }
}

// ============================================================
// Branch Diff Detection
// ============================================================

fn is_branch_diff(input: &str) -> bool {
    BRANCH_DIFF_PATTERNS.iter().any(|re| re.is_match(input))
}

fn extract_branch1(input: &str) -> String {
    if input.contains("当前") || input.contains("current") {
        return "HEAD".to_string();
    }
    BRANCH1_RE
        .captures(input)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_else(|| "HEAD".to_string())
}

fn extract_branch2(input: &str) -> String {
    if input.contains("main") {
        return "main".to_string();
    }
    if input.contains("master") {
        return "master".to_string();
    }
    BRANCH2_RE
        .captures(input)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_else(|| "main".to_string())
}

// ============================================================
// Commit Diff Detection
// ============================================================

fn is_commit_diff(input: &str) -> bool {
    COMMIT_DIFF_PATTERNS.iter().any(|re| re.is_match(input))
}

fn extract_commit1(input: &str) -> String {
    COMMIT_HASH_RE
        .captures(input)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_default()
}

fn extract_commit2(input: &str) -> String {
    let matches: Vec<_> = COMMIT_HASH_RE.find_iter(input).collect();
    if matches.len() >= 2 {
        matches[1].as_str().to_string()
    } else {
        String::new()
    }
}

// ============================================================
// File History Detection
// ============================================================

fn is_file_history(input: &str) -> bool {
    FILE_HISTORY_PATTERNS.iter().any(|re| re.is_match(input))
}

fn extract_time_range(input: &str) -> String {
    if let Some(c) = TIME_RANGE_DAYS_CN.captures(input) {
        if let Some(m) = c.get(1) {
            return format!("{}d", m.as_str());
        }
    }

    if let Some(c) = TIME_RANGE_DAYS_EN.captures(input) {
        if let Some(m) = c.get(1) {
            return format!("{}d", m.as_str());
        }
    }

    if let Some(c) = TIME_RANGE_HOURS_CN.captures(input) {
        if let Some(m) = c.get(1) {
            return format!("{}h", m.as_str());
        }
    }

    "3d".to_string()
}

fn extract_file(input: &str) -> String {
    if let Some(c) = FILE_QUOTED_DQ.captures(input) {
        if let Some(m) = c.get(1) {
            return m.as_str().to_string();
        }
    }

    if let Some(c) = FILE_QUOTED_SQ.captures(input) {
        if let Some(m) = c.get(1) {
            return m.as_str().to_string();
        }
    }

    FILE_PATH_RE
        .captures(input)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_default()
}

// ============================================================
// Time Range Parsing (used by file-history and tools)
// ============================================================

pub fn parse_time_range(time_range: &str) -> Result<i64, String> {
    if time_range.is_empty() {
        return Err("empty time range".to_string());
    }

    let caps = TIME_RANGE_PARSE
        .captures(time_range)
        .ok_or_else(|| format!("invalid time range format: {}", time_range))?;

    let value: i64 = caps
        .get(1)
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(3);
    let unit = caps.get(2).map(|m| m.as_str()).unwrap_or("d");

    let now = chrono::Local::now();
    let since = match unit {
        "d" => now - chrono::Duration::days(value),
        "h" => now - chrono::Duration::hours(value),
        "m" => now - chrono::Duration::minutes(value),
        _ => return Err(format!("invalid time unit: {}", unit)),
    };

    Ok(since.timestamp())
}
