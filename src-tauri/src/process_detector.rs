use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use log::warn;

use crate::session::Environment;

const PROCESS_CACHE_TTL: Duration = Duration::from_secs(5);
const DESKTOP_CACHE_TTL: Duration = Duration::from_secs(30);

struct ProcessCache {
    /// CWD → Environment（アクティブな Cursor/VS Code プロセス）
    entries: HashMap<String, Environment>,
    last_refreshed: Instant,
}

struct DesktopSessionCache {
    /// Desktop メタデータから取得した cliSessionId のセット
    session_ids: HashSet<String>,
    last_refreshed: Instant,
}

/// Desktop メタデータ + プロセス情報による環境検出
pub struct ProcessDetector {
    process_cache: Mutex<ProcessCache>,
    desktop_session_cache: Mutex<DesktopSessionCache>,
}

impl ProcessDetector {
    pub fn new() -> Self {
        let now = Instant::now() - Duration::from_secs(9999);
        Self {
            process_cache: Mutex::new(ProcessCache {
                entries: HashMap::new(),
                last_refreshed: now,
            }),
            desktop_session_cache: Mutex::new(DesktopSessionCache {
                session_ids: HashSet::new(),
                last_refreshed: now,
            }),
        }
    }

    /// セッション ID と CWD から環境を検出する。
    /// 優先順位: Desktop メタデータ → プロセス検出 → None（既存ロジックにフォールバック）
    pub fn detect_environment(&self, session_id: &str, cwd: &str) -> Option<Environment> {
        // 1. Desktop メタデータチェック
        {
            let mut cache = self.desktop_session_cache.lock().unwrap();
            if cache.last_refreshed.elapsed() > DESKTOP_CACHE_TTL {
                let ids = scan_desktop_sessions();
                cache.session_ids = ids;
                cache.last_refreshed = Instant::now();
            }
            if cache.session_ids.contains(session_id) {
                return Some(Environment::Desktop);
            }
        }

        // 2. プロセスマップチェック
        {
            let mut cache = self.process_cache.lock().unwrap();
            if cache.last_refreshed.elapsed() > PROCESS_CACHE_TTL {
                let entries = scan_processes();
                cache.entries = entries;
                cache.last_refreshed = Instant::now();
            }
            if let Some(env) = cache.entries.get(cwd) {
                return Some(env.clone());
            }
        }

        None
    }

    /// 両方のキャッシュを即座にリフレッシュする（初回起動時用）
    pub fn refresh(&self) {
        {
            let mut cache = self.desktop_session_cache.lock().unwrap();
            cache.session_ids = scan_desktop_sessions();
            cache.last_refreshed = Instant::now();
        }
        {
            let mut cache = self.process_cache.lock().unwrap();
            cache.entries = scan_processes();
            cache.last_refreshed = Instant::now();
        }
    }
}

/// ~/Library/Application Support/Claude/claude-code-sessions/ を走査して
/// Desktop セッションの cliSessionId を収集する
fn scan_desktop_sessions() -> HashSet<String> {
    let mut ids = HashSet::new();

    let base = match dirs::home_dir() {
        Some(home) => home
            .join("Library")
            .join("Application Support")
            .join("Claude")
            .join("claude-code-sessions"),
        None => return ids,
    };

    if !base.exists() {
        return ids;
    }

    // 構造: base/{orgId}/{conversationId}/local_*.json
    let org_entries = match std::fs::read_dir(&base) {
        Ok(entries) => entries,
        Err(_) => return ids,
    };

    for org_entry in org_entries.flatten() {
        if !org_entry.path().is_dir() {
            continue;
        }
        let conv_entries = match std::fs::read_dir(org_entry.path()) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for conv_entry in conv_entries.flatten() {
            if !conv_entry.path().is_dir() {
                continue;
            }
            let session_files = match std::fs::read_dir(conv_entry.path()) {
                Ok(entries) => entries,
                Err(_) => continue,
            };
            for file in session_files.flatten() {
                let name = file.file_name();
                let name_str = name.to_string_lossy();
                if !name_str.starts_with("local_") || !name_str.ends_with(".json") {
                    continue;
                }
                match std::fs::read_to_string(file.path()) {
                    Ok(content) => {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(cli_id) = v.get("cliSessionId").and_then(|v| v.as_str()) {
                                ids.insert(cli_id.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to read desktop session metadata {:?}: {}", file.path(), e);
                    }
                }
            }
        }
    }

    ids
}

/// `ps` と `lsof` を使ってアクティブな claude プロセスの CWD → 環境マップを構築する
fn scan_processes() -> HashMap<String, Environment> {
    let mut map = HashMap::new();

    // ps -eo pid,tty,command
    let output = match Command::new("ps")
        .args(["-eo", "pid,tty,command"])
        .output()
    {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return map,
    };

    struct ProcInfo {
        pid: String,
        env: Environment,
    }

    let mut ide_procs = Vec::new();

    for line in output.lines().skip(1) {
        let line = line.trim();
        let parts: Vec<&str> = line.splitn(3, |c: char| c.is_whitespace())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        if parts.len() < 3 {
            continue;
        }

        let command = parts[2];
        let command_lower = command.to_lowercase();

        // claude バイナリのみ対象（Claude.app 本体は除外）
        if !command_lower.contains("/claude") && !command_lower.starts_with("claude ") {
            continue;
        }
        if command_lower.contains("claude.app/contents/") || command_lower.contains("claude helper") {
            continue;
        }

        // IDE プロセスを判定
        if command.contains(".cursor/extensions/") {
            ide_procs.push(ProcInfo {
                pid: parts[0].to_string(),
                env: Environment::Cursor,
            });
        } else if command.contains(".vscode/extensions/") {
            ide_procs.push(ProcInfo {
                pid: parts[0].to_string(),
                env: Environment::Vscode,
            });
        }
        // Terminal (TTY あり) や Desktop (TTY なし, IDE パスなし) は
        // JSONL タグ検出 / Desktop メタデータ検出に任せるのでここでは処理しない
    }

    // IDE プロセスの CWD を lsof で取得
    for proc in &ide_procs {
        if let Some(cwd) = get_process_cwd(&proc.pid) {
            map.insert(cwd, proc.env.clone());
        }
    }

    map
}

/// lsof で指定 PID の CWD を取得する
fn get_process_cwd(pid: &str) -> Option<String> {
    let output = Command::new("lsof")
        .args(["-d", "cwd", "-Fn", "-p", pid])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix('n') {
            if path.starts_with('/') {
                return Some(path.to_string());
            }
        }
    }
    None
}
