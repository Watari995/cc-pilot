# CLAUDE.md — cc-pilot

## プロジェクト概要

cc-pilot は Claude Code の全セッション（CLI / VS Code / Cursor / Desktop / Web）を1つのUIでリアルタイム監視する macOS デスクトップアプリ。Tauri v2 + React + TypeScript + Rust で構築。

## クイックリファレンス

```bash
# 開発サーバー起動
npm run tauri dev

# ビルド
npm run tauri build

# フロントのみ起動（Tauri無しでUI確認）
npm run dev

# Rustのみビルドチェック
cd src-tauri && cargo check

# lint
npm run lint
cd src-tauri && cargo clippy
```

## 技術スタック

| レイヤー | 技術 |
|---|---|
| フレームワーク | Tauri v2 |
| フロントエンド | React 19 + TypeScript 5 + Vite |
| 状態管理 | Zustand |
| バックエンド | Rust |
| 主要 crates | `notify` (ファイル監視), `reqwest` (HTTP), `serde`/`serde_json`, `tauri-plugin-notification`, `tauri-plugin-shell`, `tauri-plugin-store` |

## アーキテクチャ

### データフロー

**ローカルセッション（CLI / Cursor / VS Code / Desktop）:**
```
~/.claude/projects/**/*.jsonl
  → Rust FileWatcher (notify crate)
  → JSONパース (serde)
  → プロセス情報で環境判別
  → Tauri IPC (emit event)
  → React Zustand store
  → UI更新
```

**Webセッション（claude.ai）:**
```
claude.ai API (非公式)
  → Rust WebClient (reqwest crate, 30秒ポーリング)
  → Session構造体に変換 (environment: "web")
  → Tauri IPC (emit event)
  → React Zustand store に統合
  → UI更新
```

### ディレクトリ構造
```
src/                     # React フロントエンド
├── components/
│   ├── session-list/    # サイドバー: セッション一覧 + フィルター
│   ├── session-detail/  # メインパネル: 詳細情報 + ジャンプボタン
│   ├── settings/        # 設定画面
│   ├── status-bar/      # 下部ステータスバー
│   └── common/          # badge, spinner, icon等
├── hooks/               # use-session-store, use-settings, use-tauri-events
├── lib/                 # types, constants, formatters
└── styles/              # CSS変数, グローバルスタイル

src-tauri/src/           # Rust バックエンド
├── main.rs              # エントリーポイント
├── lib.rs               # プラグイン登録
├── watcher.rs           # ~/.claude/projects/ 監視
├── parser.rs            # JSONL パーサー
├── session.rs           # Session構造体定義
├── process_detector.rs  # プロセス情報から環境判別
├── web_client.rs        # claude.ai API クライアント
├── launcher.rs          # 外部アプリ起動 (osascript, code, cursor, open URL)
├── notifier.rs          # macOS通知
└── tray.rs              # メニューバーアイコン
```

## コア機能と実装方針

### 1. ファイル監視 (`watcher.rs`)
- `notify` crate v6+ で `~/.claude/projects/` を RecursiveMode::Recursive で監視
- ファイル変更イベント発生時に対象の `.jsonl` を再パースして差分をフロントに配信
- デバウンス: 同一ファイルの変更は 100ms でバッチ処理

### 2. セッションパース (`parser.rs`)

**実データ確認済みのJSONL構造:**

各行は以下のいずれかの `type` を持つ:
- `user` — ユーザーメッセージ
- `assistant` — アシスタント応答（`message.model`, `message.usage` を含む）
- `progress` — フック実行やエージェント進捗（`data.type`: `hook_progress`, `agent_progress`, `bash_progress`）
- `system` — コンパクション境界等
- `queue-operation` — キュー操作
- `file-history-snapshot` — ファイル履歴スナップショット

**共通フィールド（各エントリのトップレベル）:**
```
sessionId, uuid, parentUuid, timestamp, type,
cwd, gitBranch, version, userType, permissionMode
```

**assistant エントリの `message` 構造:**
```json
{
  "model": "claude-opus-4-6",
  "role": "assistant",
  "content": [{"type": "thinking", ...}, {"type": "text", ...}, {"type": "tool_use", ...}],
  "stop_reason": null,
  "usage": {
    "input_tokens": 3,
    "output_tokens": 14,
    "cache_creation_input_tokens": 1757,
    "cache_read_input_tokens": 18914
  }
}
```

**重要な実装ポイント:**
- `costUSD` フィールドは存在しない → コスト表示は実装しない
- `model` は `message.model` にある（トップレベルではない）
- `usage` は `message.usage` にある
- `tool_use` は `message.content[]` 内の要素として出現
- `tool_result` は次の `user` エントリの `message.content[]` 内に出現
- ファイル全体を毎回読むのではなく、末尾から逆順に必要な情報を取得（パフォーマンス）
- セッションタイトル: 最初の `user` タイプメッセージの `message.content` からテキストを抽出、先頭80文字

### 2.5 セッションエイリアス
- `tauri-plugin-store` で `session-aliases` として永続化
- エイリアスがあれば自動タイトルより優先して表示
- セッション詳細のタイトル横 ✏️ アイコン → インライン編集
- 空にするとエイリアス削除（自動タイトルに戻る）

### 3. 環境判別 (`process_detector.rs`)

JSONLデータには環境情報がないため、プロセス情報から判別する:

| 環境 | 判別方法 |
|---|---|
| Terminal (CLI) | `claude` プロセスが TTY を持つ（`ps -o tty` が `ttysXXX`） |
| Cursor | プロセスパスに `.cursor/extensions/` を含む |
| VS Code | プロセスパスに `.vscode/extensions/` を含む |
| Desktop | 上記いずれにも該当しないローカルセッション |
| Web | `web_client.rs` 経由で取得（API由来） |

`lsof -d cwd -p {PID}` でプロセスの CWD を取得し、JSONL の `cwd` フィールドとマッチングしてセッションと紐づける。

### 4. ジャンプ機能 (`launcher.rs`)

**全環境で実機検証済み。** macOS Accessibility API / CLI / ディープリンクを組み合わせ、可能な限り正確にセッション単位でジャンプする。

| 環境 | 方式 | 精度 | 検証 |
|---|---|---|---|
| Ghostty | Accessibility API: `AXTabGroup` → `AXRadioButton`(タブ名マッチ) → `AXPress` | **タブ単位** | 済 |
| iTerm2 | AppleScript: `variable named "path"` でCWDマッチ → `select` session/tab/window | **セッション単位** | 調査済 |
| Terminal.app | `lsof` でTTY特定 → AppleScript: `tty` マッチ → `set selected` | **タブ単位** | 調査済 |
| WezTerm | `wezterm cli list --format json` → CWDマッチ → `activate-tab` + `activate-pane` | **ペイン単位** | 調査済 |
| VS Code | `code {project_path}` | **プロジェクト単位** | 済 |
| Cursor | `cursor {project_path}` | **プロジェクト単位** | 済 |
| Desktop (Chat) | `open "claude://claude.ai/chat/{uuid}"` | **会話単位** | 済 |
| Desktop (Code) | `open "claude://claude.ai/claude-code-desktop/{sessionId}"` | **セッション単位** | 済 |
| Web | `open https://claude.ai/chat/{conversationId}` | **会話単位** | 調査済 |

#### Ghostty ジャンプ実装詳細
```
1. pgrep で Ghostty PID 取得
2. AXUIElementCreateApplication(pid) → AXWindows[0]
3. AXChildren から AXTabGroup を探索
4. AXTabGroup 内の AXRadioButton を列挙（各タブ）
5. AXTitle にプロジェクト名/セッション名を含むタブを検索
6. AXPress アクションでタブ切替
7. osascript で Ghostty を activate（前面化）
```
- Ghostty はタブを macOS ネイティブタブとして公開 → Accessibility API で列挙可能
- シェルがタイトルにCWDを設定している前提（一般的なデフォルト動作）
- フォールバック: タイトルマッチ失敗時は `tell application "Ghostty" to activate` のみ

#### iTerm2 ジャンプ実装詳細
```applescript
tell application "iTerm2"
    activate
    repeat with aWindow in windows
        repeat with aTab in tabs of aWindow
            repeat with aSession in sessions of aTab
                set sessionPath to variable named "path" of aSession
                if sessionPath starts with "{project_path}" then
                    select aSession → select aTab → select aWindow
                end if
            end repeat
        end repeat
    end repeat
end tell
```
- `variable named "path"` で各セッションの CWD を直接取得可能
- iTerm2 の `select` は session / tab / window の3レベルで動作

#### Terminal.app ジャンプ実装詳細
```
1. lsof +D {project_path} で該当CWDを持つプロセスの TTY を特定
2. AppleScript で全タブの tty プロパティを走査しマッチ
3. set selected of tab to true + set index of window to 1
```

#### WezTerm ジャンプ実装詳細
```bash
# 1. CWDでペイン検索
wezterm cli list --format json | jq '.[] | select(.cwd | contains("{project_path}"))'
# 2. タブ＆ペインをアクティベート
wezterm cli activate-tab --tab-id {TAB_ID}
wezterm cli activate-pane --pane-id {PANE_ID}
# 3. アプリを前面化
osascript -e 'tell application "WezTerm" to activate'
```
- `wezterm cli list` の JSON に `cwd` フィールドが含まれるため完璧にマッチ可能

#### Desktop ディープリンク詳細
Claude Desktop は `claude://` URLスキームを登録（Info.plist で確認済み）:
- **チャット会話**: `claude://claude.ai/chat/{conversation-uuid}` — UUID検証後に内部ナビゲーション
- **コードセッション**: `claude://claude.ai/claude-code-desktop/{sessionId}` — `local_` prefix 付きのセッションID
- **セッション再開**: `claude://resume?session={cliSessionId}&cwd={path}`
- UUID は `/^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$/i` で検証される
- Desktop のセッションメタデータは `~/Library/Application Support/Claude/claude-code-sessions/` に JSON で保存

### 5. Webセッション監視 (`web_client.rs`)

**claude.ai 非公式 API を利用:**

| メソッド | エンドポイント | 用途 |
|---|---|---|
| GET | `/api/organizations` | 組織ID取得 |
| GET | `/api/organizations/{orgId}/chat_conversations` | 会話一覧 |
| GET | `/api/organizations/{orgId}/chat_conversations/{chatId}` | 会話詳細 |

- 認証: ユーザーが Settings で入力する `sessionKey` Cookie（`sk-ant-sid01-*` 形式）
- ポーリング間隔: 30秒
- `sessionKey` は `tauri-plugin-store` で保存
- ジャンプ: `open https://claude.ai/chat/{conversationId}`

**注意事項:**
- 非公式API — Anthropic が変更すると動かなくなる可能性あり
- sessionKey の有効期限あり — 期限切れ時はUIで再入力を促す
- レートリミット — ポーリング間隔は30秒以上を維持

### 6. ステータス判定

| ステータス | 判定条件 |
|---|---|
| `working` | JSONL ファイルが直近数秒以内に更新されている |
| `needs_approval` | 最新の `assistant` エントリの `message.content` に `tool_use` があり、対応する `tool_result` が次の `user` エントリにまだない + ファイル更新が停止 |
| `idle` | ファイル更新が 1分以上停止 + 最新エントリが `assistant`（`tool_use` なし） |
| `done` | ファイル更新が 5分以上停止 |
| `error` | 最新エントリにエラー情報を含む |

### 7. 通知 (`notifier.rs`)
- `tauri-plugin-notification` 使用
- セッションが `needs_approval` になったときのみ発火
- 通知タップでメインウィンドウを表示

### 8. メニューバー常駐 (`tray.rs`)
- Tauri の SystemTray API 使用
- 22x22px テンプレートイメージ（単色白）
- クリックでウィンドウ show/hide
- 右クリックメニュー: Show/Hide, Settings, Quit

## UIデザインルール

### カラー
```css
--accent: #E8734A;           /* デフォルト、設定で変更可能 */
--bg-primary: #191919;       /* メイン背景 */
--bg-secondary: #1E1E1E;     /* サイドバー・ヘッダー */
--bg-elevated: #2A2A2A;      /* ホバー・選択状態 */
--text-primary: #D4D4D4;
--text-secondary: #888888;
--border: #2A2A2A;
```

### フォント
- UI: `-apple-system, BlinkMacSystemFont, "Segoe UI", "Helvetica Neue", sans-serif`
- コード要素（ブランチ名, パス, ツール名）: `SF Mono, Cascadia Code, Fira Code, Menlo, monospace`

### ステータスカラー
| ステータス | 色 |
|---|---|
| working | `var(--accent)` |
| needs_approval | `#F59E0B` |
| idle | `#888888` |
| done | `#22C55E` |
| error | `#EF4444` |

### レイアウト
- サイドバー: 320px 固定
- ヘッダー: 48px
- ステータスバー: 28px
- ウィンドウ: デフォルト 1200x800, 最小 800x500
- macOS トラフィックライト表示

## TypeScript 型定義

```typescript
type Environment = "terminal" | "vscode" | "cursor" | "desktop" | "web";
type SessionStatus = "working" | "needs_approval" | "idle" | "done" | "error";
type TerminalApp = "ghostty" | "iterm2" | "terminal" | "wezterm";

interface Session {
  id: string;
  projectPath: string;
  projectName: string;
  branchName?: string;
  title: string;                  // 自動: 最初のuserメッセージ先頭80文字
  alias?: string;                 // 手動エイリアス（設定済みならtitleより優先）
  environment: Environment;
  status: SessionStatus;
  model?: string;
  inputTokens: number;
  outputTokens: number;
  activeTools: string[];
  startedAt: string;
  lastActivityAt: string;
  approvalDetail?: { tool: string; description: string };
  errorMessage?: string;
}

interface Settings {
  accentColor: string;
  terminalApp: TerminalApp;
  launchAtLogin: boolean;
  notificationsEnabled: boolean;
  claudeSessionKey?: string;      // claude.ai Web版監視用（sk-ant-sid01-*）
}
```

## Tauri IPC

### コマンド（フロント → Rust）
```rust
#[tauri::command]
fn get_sessions() -> Vec<Session>;

#[tauri::command]
fn open_in_environment(session_id: String) -> Result<(), String>;

#[tauri::command]
fn get_settings() -> Settings;

#[tauri::command]
fn save_settings(settings: Settings) -> Result<(), String>;

#[tauri::command]
fn save_alias(session_id: String, alias: Option<String>) -> Result<(), String>;
```

### イベント（Rust → フロント）
```typescript
listen("session-update", (event: { payload: Session }) => {});
listen("session-removed", (event: { payload: { id: string } }) => {});
listen("approval-needed", (event: { payload: { sessionId: string; tool: string } }) => {});
```

## コーディング規約

### Rust
- `cargo clippy` でwarning 0を維持
- エラーハンドリング: `anyhow` or Tauri の Result 型
- ログ: `log` crate + `env_logger`
- コメント: 公開関数には `///` doc comment

### TypeScript / React
- 関数コンポーネント + hooks のみ（class component 不可）
- **ファイル名はケバブケース**: `session-list.tsx`, `use-session-store.ts`
- コンポーネントのエクスポート名は PascalCase: `export function SessionList()`
- `interface` > `type`（オブジェクト型の場合）
- コンポーネントファイルは1ファイル1エクスポート
- barrel export (`index.ts`) は使わない

### CSS
- CSS Modules (`.module.css`) を基本とする
- CSS変数は `styles/global.css` に集約
- `--accent` を変更するだけでテーマカラーが全体に反映される設計

## v1 スコープ外

以下は実装しない:
- アプリ上からの Allow/Deny 操作
- ライトテーマ
- Windows / Linux 対応
- モバイル対応
- セッションへのメッセージ送信
- セッション履歴の永続化

## 配布

- `.dmg` + GitHub Releases のみ
- `cc-pilot-{version}-macos-arm64.dmg` / `cc-pilot-{version}-macos-x64.dmg`
- CI/CD: GitHub Actions（tag push でビルド → Release 自動アップロード）

## 実装順序（推奨）

1. **Tauri v2 プロジェクト初期化** — `npm create tauri-app@latest`
2. **Rust: watcher + parser** — ファイル監視とJSONLパース（実データ構造確認済み）
3. **Rust: process_detector** — プロセス情報から環境判別
4. **React: SessionList + SessionDetail** — メインUI
5. **Rust: launcher** — ジャンプ機能
6. **Rust: web_client** — claude.ai APIポーリング
7. **React: Settings** — 設定画面（sessionKey入力含む）
8. **Rust: tray + notifier** — メニューバー常駐 + 通知
9. **StatusBar** — 下部ステータスバー
10. **CI/CD** — GitHub Actions
11. **README + 配布準備**
