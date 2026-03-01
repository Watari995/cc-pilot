use anyhow::Result;
use log::{error, info, warn};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

use crate::parser::parse_session_file;
use crate::session::Session;

/// セッション状態を保持する共有ステート
pub type SessionStore = Arc<Mutex<HashMap<String, Session>>>;

/// 初回起動時に ~/.claude/projects/ を一括スキャンしてセッション一覧を構築
pub fn initial_scan(store: &SessionStore) -> Result<()> {
    let projects_dir = get_projects_dir()?;
    if !projects_dir.exists() {
        warn!(
            "Projects directory does not exist: {}",
            projects_dir.display()
        );
        return Ok(());
    }

    info!("Scanning sessions in: {}", projects_dir.display());
    let mut count = 0;

    scan_directory(&projects_dir, store, &mut count)?;

    info!("Initial scan complete: {} sessions found", count);
    Ok(())
}

/// ディレクトリを再帰的にスキャンして .jsonl ファイルをパース
fn scan_directory(dir: &Path, store: &SessionStore, count: &mut usize) -> Result<()> {
    let entries = std::fs::read_dir(dir)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // subagents ディレクトリはスキップ
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .map_or(false, |n| n == "subagents" || n == "memory")
            {
                continue;
            }
            scan_directory(&path, store, count)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            match parse_session_file(&path) {
                Ok(session) => {
                    let mut sessions = store.lock().unwrap();
                    sessions.insert(session.id.clone(), session);
                    *count += 1;
                }
                Err(e) => {
                    warn!("Failed to parse {}: {}", path.display(), e);
                }
            }
        }
    }
    Ok(())
}

/// ファイル監視を開始する（別スレッドで実行）
pub fn start_watching(app_handle: AppHandle, store: SessionStore) -> Result<()> {
    let projects_dir = get_projects_dir()?;
    if !projects_dir.exists() {
        warn!(
            "Projects directory does not exist, watching will not start: {}",
            projects_dir.display()
        );
        return Ok(());
    }

    std::thread::spawn(move || {
        if let Err(e) = watch_loop(&app_handle, &store, &projects_dir) {
            error!("File watcher error: {}", e);
        }
    });

    Ok(())
}

/// 監視ループ本体
fn watch_loop(app_handle: &AppHandle, store: &SessionStore, projects_dir: &Path) -> Result<()> {
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(projects_dir, RecursiveMode::Recursive)?;
    info!("File watcher started on: {}", projects_dir.display());

    // デバウンス用: パスごとに最後のイベント時刻を記録
    let mut last_events: HashMap<PathBuf, Instant> = HashMap::new();
    let debounce_duration = Duration::from_millis(100);

    for event in rx {
        match event {
            Ok(event) => {
                for path in &event.paths {
                    // .jsonl ファイルのみ対象
                    if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                        continue;
                    }

                    // subagents, memory ディレクトリ内はスキップ
                    let path_str = path.to_string_lossy();
                    if path_str.contains("/subagents/") || path_str.contains("/memory/") {
                        continue;
                    }

                    // デバウンス: 100ms 以内の同一ファイルイベントはスキップ
                    let now = Instant::now();
                    if let Some(last) = last_events.get(path) {
                        if now.duration_since(*last) < debounce_duration {
                            continue;
                        }
                    }
                    last_events.insert(path.clone(), now);

                    // ファイルが削除された場合
                    if !path.exists() {
                        let session_id = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_string();
                        if !session_id.is_empty() {
                            let mut sessions = store.lock().unwrap();
                            sessions.remove(&session_id);
                            let _ = app_handle
                                .emit("session-removed", serde_json::json!({ "id": session_id }));
                        }
                        continue;
                    }

                    // パースして更新
                    match parse_session_file(path) {
                        Ok(session) => {
                            let mut sessions = store.lock().unwrap();
                            sessions.insert(session.id.clone(), session.clone());
                            let _ = app_handle.emit("session-update", &session);
                        }
                        Err(e) => {
                            warn!("Failed to parse updated file {}: {}", path.display(), e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Watch error: {}", e);
            }
        }
    }

    Ok(())
}

/// ~/.claude/projects/ のパスを取得
fn get_projects_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
    Ok(home.join(".claude").join("projects"))
}
