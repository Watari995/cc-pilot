use std::process::Command;

use crate::session::{Environment, Session};

/// ターミナルアプリの種類（Phase 3 で設定画面から変更可能にする）
#[derive(Debug, Clone, Copy)]
pub enum TerminalApp {
    Ghostty,
    Iterm2,
    TerminalApp,
    Wezterm,
}

/// デフォルトのターミナルアプリ（Phase 3 で設定から読み取りに変更）
const DEFAULT_TERMINAL: TerminalApp = TerminalApp::Ghostty;

/// セッションの環境に応じてアプリケーションを起動する
pub fn open_session(session: &Session) -> Result<(), String> {
    log::info!(
        "Launching session {} in {:?} (project: {})",
        session.id,
        session.environment,
        session.project_path
    );

    match session.environment {
        Environment::Terminal => launch_terminal(session, DEFAULT_TERMINAL),
        Environment::Vscode => launch_editor("code", &session.project_path),
        Environment::Cursor => launch_editor("cursor", &session.project_path),
        Environment::Desktop => launch_desktop(session),
        Environment::Web => launch_web(session),
    }
}

// ─── Editor (VS Code / Cursor) ─────────────────────────────────

fn launch_editor(cmd: &str, project_path: &str) -> Result<(), String> {
    Command::new(cmd)
        .arg(project_path)
        .spawn()
        .map_err(|e| format!("Failed to launch {}: {}", cmd, e))?;
    Ok(())
}

// ─── Desktop (Claude Desktop deep link) ────────────────────────

fn launch_desktop(session: &Session) -> Result<(), String> {
    let url = format!(
        "claude://claude.ai/claude-code-desktop/{}",
        session.id
    );
    Command::new("open")
        .arg(&url)
        .spawn()
        .map_err(|e| format!("Failed to open Desktop deep link: {}", e))?;
    Ok(())
}

// ─── Web (claude.ai) ───────────────────────────────────────────

fn launch_web(session: &Session) -> Result<(), String> {
    let url = format!("https://claude.ai/chat/{}", session.id);
    Command::new("open")
        .arg(&url)
        .spawn()
        .map_err(|e| format!("Failed to open browser: {}", e))?;
    Ok(())
}

// ─── Terminal ───────────────────────────────────────────────────

fn launch_terminal(session: &Session, app: TerminalApp) -> Result<(), String> {
    match app {
        TerminalApp::Ghostty => launch_ghostty(session),
        TerminalApp::Iterm2 => launch_iterm2(session),
        TerminalApp::TerminalApp => launch_terminal_app(session),
        TerminalApp::Wezterm => launch_wezterm(session),
    }
}

/// Ghostty: Accessibility API でタブ精度ジャンプ、失敗時は activate のみ
fn launch_ghostty(session: &Session) -> Result<(), String> {
    // Python スクリプトで Accessibility API 経由のタブ切替を試みる
    let script = format!(
        r#"
import subprocess, sys
try:
    from ApplicationServices import (
        AXUIElementCreateApplication,
        AXUIElementCopyAttributeValue,
        AXUIElementPerformAction,
    )
    from CoreFoundation import CFEqual

    # Ghostty の PID を取得
    result = subprocess.run(["pgrep", "-x", "ghostty"], capture_output=True, text=True)
    if result.returncode != 0:
        print("Ghostty not running", file=sys.stderr)
        sys.exit(1)
    pid = int(result.stdout.strip().split("\n")[0])

    app = AXUIElementCreateApplication(pid)
    err, windows = AXUIElementCopyAttributeValue(app, "AXWindows", None)
    if err or not windows or len(windows) == 0:
        sys.exit(1)

    target = "{project_path}"
    found = False
    for win in windows:
        err, children = AXUIElementCopyAttributeValue(win, "AXChildren", None)
        if err or not children:
            continue
        for child in children:
            err, role = AXUIElementCopyAttributeValue(child, "AXRole", None)
            if err or role != "AXTabGroup":
                continue
            err, tabs = AXUIElementCopyAttributeValue(child, "AXChildren", None)
            if err or not tabs:
                continue
            for tab in tabs:
                err, title = AXUIElementCopyAttributeValue(tab, "AXTitle", None)
                if err or not title:
                    continue
                if target in str(title):
                    AXUIElementPerformAction(tab, "AXPress")
                    found = True
                    break
            if found:
                break
        if found:
            break
except Exception:
    pass

# activate（前面化）は必ず実行
subprocess.run(["osascript", "-e", 'tell application "Ghostty" to activate'])
"#,
        project_path = session.project_path
    );

    Command::new("python3")
        .arg("-c")
        .arg(&script)
        .spawn()
        .map_err(|e| format!("Failed to launch Ghostty: {}", e))?;
    Ok(())
}

/// iTerm2: AppleScript で CWD マッチ → セッション select
fn launch_iterm2(session: &Session) -> Result<(), String> {
    let script = format!(
        r#"
tell application "iTerm2"
    activate
    repeat with aWindow in windows
        repeat with aTab in tabs of aWindow
            repeat with aSession in sessions of aTab
                try
                    set sessionPath to variable named "path" of aSession
                    if sessionPath starts with "{project_path}" then
                        select aSession
                        tell aTab to select
                        set index of aWindow to 1
                        return
                    end if
                end try
            end repeat
        end repeat
    end repeat
end tell
"#,
        project_path = session.project_path
    );

    Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .spawn()
        .map_err(|e| format!("Failed to launch iTerm2: {}", e))?;
    Ok(())
}

/// Terminal.app: lsof で TTY 特定 → AppleScript で tty マッチ
fn launch_terminal_app(session: &Session) -> Result<(), String> {
    // まず lsof で project_path の CWD を持つプロセスの TTY を特定
    let lsof_output = Command::new("lsof")
        .args(["+D", &session.project_path, "-t"])
        .output()
        .map_err(|e| format!("lsof failed: {}", e))?;

    let tty = if lsof_output.status.success() {
        let pids = String::from_utf8_lossy(&lsof_output.stdout);
        // 最初の PID から TTY を取得
        if let Some(pid) = pids.lines().next() {
            let ps_output = Command::new("ps")
                .args(["-o", "tty=", "-p", pid.trim()])
                .output()
                .ok();
            ps_output
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
        } else {
            None
        }
    } else {
        None
    };

    let script = if let Some(tty) = tty {
        format!(
            r#"
tell application "Terminal"
    activate
    repeat with aWindow in windows
        repeat with aTab in tabs of aWindow
            if tty of aTab is "/dev/{tty}" then
                set selected of aTab to true
                set index of aWindow to 1
                return
            end if
        end repeat
    end repeat
end tell
"#,
            tty = tty
        )
    } else {
        // フォールバック: activate のみ
        r#"tell application "Terminal" to activate"#.to_string()
    };

    Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .spawn()
        .map_err(|e| format!("Failed to launch Terminal.app: {}", e))?;
    Ok(())
}

/// WezTerm: CLI で CWD マッチ → activate-tab + activate-pane
fn launch_wezterm(session: &Session) -> Result<(), String> {
    // wezterm cli list でペイン一覧を取得
    let list_output = Command::new("wezterm")
        .args(["cli", "list", "--format", "json"])
        .output()
        .map_err(|e| format!("wezterm cli list failed: {}", e))?;

    if list_output.status.success() {
        let json_str = String::from_utf8_lossy(&list_output.stdout);
        if let Ok(panes) = serde_json::from_str::<serde_json::Value>(&json_str) {
            if let Some(arr) = panes.as_array() {
                for pane in arr {
                    let cwd = pane.get("cwd").and_then(|v| v.as_str()).unwrap_or("");
                    if cwd.contains(&session.project_path) {
                        // タブとペインをアクティベート
                        if let Some(tab_id) = pane.get("tab_id").and_then(|v| v.as_u64()) {
                            let _ = Command::new("wezterm")
                                .args(["cli", "activate-tab", "--tab-id", &tab_id.to_string()])
                                .output();
                        }
                        if let Some(pane_id) = pane.get("pane_id").and_then(|v| v.as_u64()) {
                            let _ = Command::new("wezterm")
                                .args([
                                    "cli",
                                    "activate-pane",
                                    "--pane-id",
                                    &pane_id.to_string(),
                                ])
                                .output();
                        }
                        break;
                    }
                }
            }
        }
    }

    // WezTerm を前面化
    Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "WezTerm" to activate"#)
        .spawn()
        .map_err(|e| format!("Failed to activate WezTerm: {}", e))?;
    Ok(())
}
