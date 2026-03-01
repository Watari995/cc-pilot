use serde::{Deserialize, Serialize};

/// セッションの起動元環境
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Terminal,
    Vscode,
    Cursor,
    Desktop,
    Web,
}

/// セッションのステータス
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Working,
    NeedsApproval,
    Idle,
    Done,
    Error,
}

/// 承認待ちの詳細情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalDetail {
    pub tool: String,
    pub description: String,
}

/// cc-pilot のセッション情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub project_path: String,
    pub project_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_name: Option<String>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    pub environment: Environment,
    pub status: SessionStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub active_tools: Vec<String>,
    pub started_at: String,
    pub last_activity_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_detail: Option<ApprovalDetail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            id: String::new(),
            project_path: String::new(),
            project_name: String::new(),
            branch_name: None,
            title: String::new(),
            alias: None,
            environment: Environment::Terminal,
            status: SessionStatus::Idle,
            model: None,
            input_tokens: 0,
            output_tokens: 0,
            active_tools: Vec::new(),
            started_at: String::new(),
            last_activity_at: String::new(),
            approval_detail: None,
            error_message: None,
        }
    }
}
