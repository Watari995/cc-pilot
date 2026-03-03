use anyhow::{Context, Result};
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::process_detector::ProcessDetector;
use crate::session::{ApprovalDetail, Environment, Session, SessionStatus};

/// JSONL ファイルの末尾から情報を抽出してSessionを構築する
pub fn parse_session_file(
    path: &Path,
    default_ide: &Environment,
    process_detector: Option<&ProcessDetector>,
) -> Result<Session> {
    let file = File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
    let metadata = file.metadata()?;
    let file_size = metadata.len();
    let file_modified = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let seconds_since_update = now.saturating_sub(file_modified);

    // セッションIDをファイル名から取得
    let session_id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    // プロジェクトパスをディレクトリ名からデコード
    let (project_path, project_name) = extract_project_info(path);

    // 末尾から最大200行を読み取り
    let tail_lines = read_tail_lines(&file, file_size, 200)?;
    if tail_lines.is_empty() {
        return Ok(Session {
            id: session_id,
            project_path,
            project_name,
            ..Default::default()
        });
    }

    // 先頭から最初のuserメッセージを取得（タイトル用）+ cwd — 先頭20行のみ読む
    let head_lines = read_head_lines(path, 20)?;
    let title = extract_title(&head_lines);
    let cwd_from_head = extract_cwd(&head_lines);
    let env_from_head = detect_environment(&head_lines, default_ide);

    // 末尾のエントリから情報を抽出
    let mut model: Option<String> = None;
    let mut input_tokens: u64 = 0;
    let mut output_tokens: u64 = 0;
    let mut branch_name: Option<String> = None;
    let mut started_at: Option<String> = None;
    let mut last_activity_at: Option<String> = None;
    let mut active_tools: Vec<String> = Vec::new();
    let mut last_entry_type: Option<String> = None;
    let mut has_pending_tool_use = false;
    let mut pending_tool_name: Option<String> = None;
    let mut pending_tool_desc: Option<String> = None;
    let mut cwd_from_file: Option<String> = None;
    let mut detected_env = Environment::Terminal;

    // 全 tail 行を走査
    for line in &tail_lines {
        if let Ok(entry) = serde_json::from_str::<Value>(line) {
            let entry_type = entry.get("type").and_then(|v| v.as_str()).unwrap_or("");

            // タイムスタンプ
            if let Some(ts) = entry.get("timestamp").and_then(|v| v.as_str()) {
                if started_at.is_none() {
                    started_at = Some(ts.to_string());
                }
                last_activity_at = Some(ts.to_string());
            }

            // cwd（プロジェクトパス）
            if cwd_from_file.is_none() {
                if let Some(cwd) = entry.get("cwd").and_then(|v| v.as_str()) {
                    if !cwd.is_empty() {
                        cwd_from_file = Some(cwd.to_string());
                    }
                }
            }

            // ブランチ名
            if branch_name.is_none() {
                if let Some(branch) = entry.get("gitBranch").and_then(|v| v.as_str()) {
                    if !branch.is_empty() {
                        branch_name = Some(branch.to_string());
                    }
                }
            }

            if entry_type == "assistant" {
                if let Some(message) = entry.get("message") {
                    // モデル名
                    if let Some(m) = message.get("model").and_then(|v| v.as_str()) {
                        model = Some(m.to_string());
                    }

                    // トークン使用量（累積加算）
                    if let Some(usage) = message.get("usage") {
                        input_tokens += usage
                            .get("input_tokens")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        output_tokens += usage
                            .get("output_tokens")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                    }

                    // tool_use チェック
                    if let Some(content) = message.get("content").and_then(|v| v.as_array()) {
                        active_tools.clear();
                        has_pending_tool_use = false;
                        for block in content {
                            if block.get("type").and_then(|v| v.as_str()) == Some("tool_use") {
                                let tool_name = block
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown");
                                active_tools.push(tool_name.to_string());
                                has_pending_tool_use = true;
                                pending_tool_name = Some(tool_name.to_string());
                                pending_tool_desc = block
                                    .get("input")
                                    .map(|v| {
                                        let s = v.to_string();
                                        let chars: Vec<char> = s.chars().collect();
                                        if chars.len() > 100 {
                                            format!("{}...", chars[..100].iter().collect::<String>())
                                        } else {
                                            s
                                        }
                                    });
                            }
                        }
                    }
                }
                last_entry_type = Some("assistant".to_string());
            } else if entry_type == "user" {
                // user エントリに tool_result があれば承認済み
                if let Some(message) = entry.get("message") {
                    if let Some(content) = message.get("content").and_then(|v| v.as_array()) {
                        for block in content {
                            if block.get("type").and_then(|v| v.as_str()) == Some("tool_result") {
                                has_pending_tool_use = false;
                            }
                            // IDE 環境検出: ide_selection / ide_opened_file タグがあれば IDE セッション
                            if detected_env == Environment::Terminal {
                                if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                                    if text.contains("<ide_selection>")
                                        || text.contains("<ide_opened_file>")
                                    {
                                        detected_env = default_ide.clone();
                                    }
                                }
                            }
                        }
                    }
                }
                last_entry_type = Some("user".to_string());
            } else {
                last_entry_type = Some(entry_type.to_string());
            }
        }
    }

    // ステータス判定
    let status = determine_status(
        seconds_since_update,
        last_entry_type.as_deref(),
        has_pending_tool_use,
    );

    // 承認待ち詳細
    let approval_detail = if status == SessionStatus::NeedsApproval {
        Some(ApprovalDetail {
            tool: pending_tool_name.unwrap_or_default(),
            description: pending_tool_desc.unwrap_or_default(),
        })
    } else {
        None
    };

    active_tools.sort();
    active_tools.dedup();

    // cwd からプロジェクトパスを優先的に取得
    let resolved_cwd = cwd_from_file.or(cwd_from_head);
    let (final_project_path, final_project_name) = if let Some(cwd) = resolved_cwd {
        let name = cwd
            .rsplit('/')
            .find(|s| !s.is_empty())
            .unwrap_or("unknown")
            .to_string();
        (cwd, name)
    } else {
        (project_path, project_name)
    };

    // 環境判定: IDE タグ → ProcessDetector → Terminal フォールバック
    let mut final_env = if env_from_head != Environment::Terminal {
        env_from_head
    } else {
        detected_env
    };

    if final_env == Environment::Terminal {
        if let Some(detector) = process_detector {
            if let Some(detected) = detector.detect_environment(&session_id, &final_project_path) {
                final_env = detected;
            }
        }
    }

    Ok(Session {
        id: session_id,
        project_path: final_project_path,
        project_name: final_project_name,
        branch_name,
        title,
        alias: None,
        environment: final_env,
        status,
        model,
        input_tokens,
        output_tokens,
        active_tools,
        started_at: started_at.unwrap_or_default(),
        last_activity_at: last_activity_at.unwrap_or_default(),
        approval_detail,
        error_message: None,
    })
}

/// ステータス判定ロジック
fn determine_status(
    seconds_since_update: u64,
    last_entry_type: Option<&str>,
    has_pending_tool_use: bool,
) -> SessionStatus {
    if seconds_since_update < 5 {
        return SessionStatus::Working;
    }

    if has_pending_tool_use && seconds_since_update < 300 {
        return SessionStatus::NeedsApproval;
    }

    if seconds_since_update > 300 {
        return SessionStatus::Done;
    }

    if seconds_since_update > 60 {
        return SessionStatus::Idle;
    }

    match last_entry_type {
        Some("assistant") => SessionStatus::Idle,
        _ => SessionStatus::Working,
    }
}

/// ディレクトリ名からプロジェクトパスとプロジェクト名を抽出
fn extract_project_info(path: &Path) -> (String, String) {
    // ~/.claude/projects/-Users-username-myproject/session.jsonl
    // → project_path: /Users/username/myproject
    // → project_name: myproject
    let parent = path.parent().unwrap_or(path);
    let dir_name = parent
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // サブディレクトリ（subagents等）の場合、親の親を見る
    let dir_name = if dir_name == "subagents" || dir_name.len() < 5 {
        parent
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or(dir_name)
    } else {
        dir_name
    };

    // ハイフン区切りをスラッシュに変換
    let project_path = if dir_name.starts_with('-') {
        dir_name.replacen('-', "/", 1).replace('-', "/")
    } else {
        dir_name.to_string()
    };

    let project_name = project_path
        .rsplit('/')
        .next()
        .unwrap_or("unknown")
        .to_string();

    (project_path, project_name)
}

/// エントリから環境を検出（IDE タグの有無で判定）
fn detect_environment(lines: &[String], default_ide: &Environment) -> Environment {
    for line in lines {
        if line.contains("<ide_selection>") || line.contains("<ide_opened_file>") {
            return default_ide.clone();
        }
    }
    Environment::Terminal
}

/// エントリから cwd を抽出
fn extract_cwd(lines: &[String]) -> Option<String> {
    for line in lines {
        if let Ok(entry) = serde_json::from_str::<Value>(line) {
            if let Some(cwd) = entry.get("cwd").and_then(|v| v.as_str()) {
                if !cwd.is_empty() {
                    return Some(cwd.to_string());
                }
            }
        }
    }
    None
}

/// テキストがシステムタグ（IDE挿入のメタデータ等）かどうか判定
fn is_system_tag(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.starts_with("<ide_opened_file>")
        || trimmed.starts_with("<ide_selection>")
        || trimmed.starts_with("<system-reminder>")
        || trimmed.starts_with("<command-name>")
        || trimmed.starts_with("<fast_mode_info>")
}

/// テキストを80文字以内に切り詰め、改行をスペースに置換
fn truncate_title(text: &str) -> String {
    let title = text.trim().replace('\n', " ");
    let chars: Vec<char> = title.chars().collect();
    if chars.len() > 80 {
        format!("{}...", chars[..80].iter().collect::<String>())
    } else {
        title
    }
}

/// 最初の user メッセージからタイトルを抽出（システムタグはスキップ）
fn extract_title(lines: &[String]) -> String {
    for line in lines {
        if let Ok(entry) = serde_json::from_str::<Value>(line) {
            if entry.get("type").and_then(|v| v.as_str()) == Some("user") {
                if let Some(message) = entry.get("message") {
                    if let Some(content) = message.get("content") {
                        // content が配列の場合
                        if let Some(arr) = content.as_array() {
                            for block in arr {
                                if block.get("type").and_then(|v| v.as_str()) == Some("text") {
                                    if let Some(text) = block.get("text").and_then(|v| v.as_str())
                                    {
                                        if !is_system_tag(text) && !text.trim().is_empty() {
                                            return truncate_title(text);
                                        }
                                    }
                                }
                            }
                        }
                        // content が文字列の場合
                        if let Some(text) = content.as_str() {
                            if !is_system_tag(text) && !text.trim().is_empty() {
                                return truncate_title(text);
                            }
                        }
                    }
                }
            }
        }
    }
    "(no title)".to_string()
}

/// ファイル末尾から指定行数を読み取り
fn read_tail_lines(file: &File, file_size: u64, max_lines: usize) -> Result<Vec<String>> {
    // 末尾 64KB をバッファとして読み取り
    let buf_size = std::cmp::min(file_size, 64 * 1024) as usize;
    let mut reader = BufReader::new(file);
    let start_pos = file_size.saturating_sub(buf_size as u64);
    reader.seek(SeekFrom::Start(start_pos))?;

    let mut lines: Vec<String> = Vec::new();
    let mut buf = String::new();

    // 途中から読み始めた場合、最初の行は不完全なので捨てる
    if start_pos > 0 {
        reader.read_line(&mut buf)?;
        buf.clear();
    }

    while reader.read_line(&mut buf)? > 0 {
        let trimmed = buf.trim().to_string();
        if !trimmed.is_empty() {
            lines.push(trimmed);
        }
        buf.clear();
    }

    // 末尾 max_lines 行のみ返す
    if lines.len() > max_lines {
        Ok(lines.split_off(lines.len() - max_lines))
    } else {
        Ok(lines)
    }
}

/// ファイル先頭から指定行数を読み取り
fn read_head_lines(path: &Path, max_lines: usize) -> Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();

    for line in reader.lines().take(max_lines) {
        let line = line?;
        let trimmed = line.trim().to_string();
        if !trimmed.is_empty() {
            lines.push(trimmed);
        }
    }

    Ok(lines)
}
