# CLAUDE.md — cc-pilot

## プロジェクト概要

cc-pilot は Claude Code の全セッション（CLI / VS Code / Cursor / Desktop）を1つのUIでリアルタイム監視する macOS デスクトップアプリ。Tauri v2 + React + TypeScript + Rust で構築。

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
| 主要 crates | `notify` (ファイル監視), `serde`/`serde_json`, `tauri-plugin-notification`, `tauri-plugin-shell`, `tauri-plugin-store` |

## アーキテクチャ

### データフロー
```
~/.claude/projects/**/*.jsonl
  → Rust FileWatcher (notify crate)
  → JSONパース (serde)
  → Tauri IPC (emit event)
  → React Zustand store
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
├── launcher.rs          # 外部アプリ起動 (osascript, code, cursor)
├── notifier.rs          # macOS通知
└── tray.rs              # メニューバーアイコン
```

## コア機能と実装方針

### 1. ファイル監視 (`watcher.rs`)
- `notify` crate v6+ で `~/.claude/projects/` を RecursiveMode::Recursive で監視
- ファイル変更イベント発生時に対象の `.jsonl` を再パースして差分をフロントに配信
- デバウンス: 同一ファイルの変更は 100ms でバッチ処理

### 2. セッションパース (`parser.rs`)
- JSONL形式（1行1JSON）を行単位で読み取り
- 最新の数行からステータス・モデル・トークン数・コストを抽出
- セッションタイトル: 最初の `user` タイプメッセージから先頭80文字を自動抽出
- **重要**: ファイル全体を毎回読むのではなく、末尾から逆順に必要な情報を取得（パフォーマンス）

### 2.5 セッションエイリアス
- `tauri-plugin-store` で `session-aliases` として永続化
- エイリアスがあれば自動タイトルより優先して表示
- セッション詳細のタイトル横 ✏️ アイコン → インライン編集
- 空にするとエイリアス削除（自動タイトルに戻る）

### 3. ジャンプ機能 (`launcher.rs`)
| 環境 | 起動コマンド |
|---|---|
| Ghostty | `osascript -e 'tell application "Ghostty" to activate'` |
| iTerm2 | `osascript -e 'tell application "iTerm2" to activate'` |
| VS Code | `code {project_path}` |
| Cursor | `cursor {project_path}` |
| Desktop | `open -a "Claude"` |

### 4. 通知 (`notifier.rs`)
- `tauri-plugin-notification` 使用
- セッションが `needs_approval` になったときのみ発火
- 通知タップでメインウィンドウを表示

### 5. メニューバー常駐 (`tray.rs`)
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
- コード要素（ブランチ名, パス, コスト, ツール名）: `SF Mono, Cascadia Code, Fira Code, Menlo, monospace`

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
type Environment = "terminal" | "vscode" | "cursor" | "desktop";
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
  costUSD: number;
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

## セッションデータの実機確認

> **開発開始前に必ず確認すること:**
>
> 1. `ls -la ~/.claude/projects/` の構造
> 2. 任意のセッションファイル（`.jsonl`）の中身を `head -50` で確認
> 3. 以下のフィールドの有無を検証:
>    - モデル名
>    - トークン数（input/output）
>    - コスト
>    - ツール名
>    - ステータス（特に承認待ちの判別方法）
>    - 環境判別に使える情報
>
> 確認結果に基づいて `parser.rs` の実装を決定する。

## v1 スコープ外

以下は実装しない:
- アプリ上からの Allow/Deny 操作
- ライトテーマ
- Windows / Linux 対応
- モバイル対応
- セッションへのメッセージ送信
- セッション履歴の永続化

## 実装順序（推奨）

1. **Tauri v2 プロジェクト初期化** — `npm create tauri-app@latest`
2. **`~/.claude/projects/` の実データ確認** — パーサー仕様確定
3. **Rust: watcher + parser** — ファイル監視とJSONLパース
4. **React: SessionList + SessionDetail** — メインUI
5. **Rust: launcher** — ジャンプ機能
6. **React: Settings** — 設定画面
7. **Rust: tray + notifier** — メニューバー常駐 + 通知
8. **StatusBar** — 下部ステータスバー
9. **CI/CD** — GitHub Actions
10. **README + 配布準備**