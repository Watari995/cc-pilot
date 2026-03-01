# cc-pilot — 詳細仕様書

## 1. プロジェクト概要

### 1.1 プロダクト定義

**cc-pilot** は、Claude Codeの全セッション（CLI / VS Code拡張 / Cursor拡張 / Desktop）を1つのUIでリアルタイム監視し、タップで各環境にジャンプできるmacOSデスクトップアプリ。

### 1.2 解決する課題

- 複数のClaude Codeセッションを同時に走らせると、どのセッションが何をしているか把握しづらい
- セッションが承認待ち（Needs approval）になっても気づけない
- ターミナル、VS Code、Cursor、Desktopを行き来する手間

### 1.3 ターゲットユーザー

Claude Codeを日常的に使う開発者。特に複数セッションを並行実行するヘビーユーザー。

### 1.4 パッケージ情報

| 項目         | 値                             |
| ------------ | ------------------------------ |
| パッケージ名 | `cc-pilot`                     |
| リポジトリ   | `github.com/{user}/cc-pilot`   |
| コマンド     | `npx cc-pilot`                 |
| ライセンス   | MIT                            |
| 対象OS       | macOS（Apple Silicon + Intel） |

---

## 2. 技術スタック

### 2.1 コアテクノロジー

| レイヤー       | 技術                        | 役割                             |
| -------------- | --------------------------- | -------------------------------- |
| フレームワーク | Tauri v2                    | ネイティブアプリ基盤             |
| フロントエンド | React 19 + TypeScript 5     | UI描画                           |
| スタイリング   | CSS Modules or Tailwind CSS | コンポーネントスタイル           |
| 状態管理       | Zustand                     | 軽量グローバルステート           |
| バックエンド   | Rust                        | ファイル監視・プロセス管理・通知 |
| ビルドツール   | Vite                        | フロントエンドビルド             |

### 2.2 主要Rust crates

| crate                       | 用途                                       |
| --------------------------- | ------------------------------------------ |
| `notify` (v6+)              | `~/.claude/projects/` のファイル変更監視   |
| `serde` / `serde_json`      | セッションJSONパース                       |
| `tauri` v2                  | アプリフレームワーク                       |
| `tauri-plugin-notification` | macOS通知                                  |
| `tauri-plugin-shell`        | 外部コマンド実行 (osascript, code, cursor) |
| `tauri-plugin-store`        | ユーザー設定永続化                         |

### 2.3 ビルド成果物

- `.app` バンドル（macOS）
- `.dmg` インストーラー
- npm パッケージ（`npx cc-pilot` 経由で起動）

---

## 3. アーキテクチャ

### 3.1 全体構成

```
┌─────────────────────────────────────────────┐
│                  cc-pilot                    │
│                                             │
│  ┌─────────────┐     ┌──────────────────┐  │
│  │   Rust       │     │   React (WebView)│  │
│  │   Backend    │◄───►│   Frontend       │  │
│  │              │ IPC │                  │  │
│  │ • FileWatcher│     │ • SessionList    │  │
│  │ • Launcher   │     │ • SessionDetail  │  │
│  │ • Notifier   │     │ • Settings       │  │
│  │ • TrayIcon   │     │ • StatusBar      │  │
│  └──────┬───────┘     └──────────────────┘  │
│         │                                    │
│         ▼                                    │
│  ~/.claude/projects/  (read-only監視)         │
└─────────────────────────────────────────────┘
```

### 3.2 データフロー

```
~/.claude/projects/**/*.jsonl
        │
        ▼ (notify crate: ファイル変更検知)
   Rust FileWatcher
        │
        ▼ (JSONパース + 差分計算)
   SessionState (Rust構造体)
        │
        ▼ (Tauri IPC: emit event)
   React Frontend (Zustand store)
        │
        ▼ (React re-render)
   UI更新
```

### 3.3 ディレクトリ構造

```
cc-pilot/
├── src/                          # React フロントエンド
│   ├── app.tsx
│   ├── main.tsx
│   ├── components/
│   │   ├── session-list/         # サイドバー: セッション一覧
│   │   │   ├── session-list.tsx
│   │   │   ├── session-card.tsx
│   │   │   └── environment-filter.tsx
│   │   ├── session-detail/       # メインパネル: セッション詳細
│   │   │   ├── session-detail.tsx
│   │   │   ├── token-usage.tsx
│   │   │   ├── active-tools.tsx
│   │   │   └── approval-banner.tsx
│   │   ├── settings/             # 設定画面
│   │   │   ├── settings.tsx
│   │   │   ├── accent-color-picker.tsx
│   │   │   └── terminal-selector.tsx
│   │   ├── status-bar/           # 下部ステータスバー
│   │   │   └── status-bar.tsx
│   │   └── common/               # 共通コンポーネント
│   │       ├── badge.tsx
│   │       ├── spinner.tsx
│   │       └── icon.tsx
│   ├── hooks/
│   │   ├── use-session-store.ts  # Zustand store
│   │   ├── use-settings.ts       # 設定管理
│   │   └── use-tauri-events.ts   # Tauriイベントリスナー
│   ├── lib/
│   │   ├── types.ts              # TypeScript型定義
│   │   ├── constants.ts          # 定数
│   │   └── formatters.ts         # 時間・コスト表示フォーマッタ
│   └── styles/
│       ├── global.css            # CSS変数・グローバルスタイル
│       └── theme.css             # アクセントカラー定義
│
├── src-tauri/                    # Rust バックエンド
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   ├── src/
│   │   ├── main.rs               # エントリーポイント
│   │   ├── lib.rs                 # Tauriプラグイン登録
│   │   ├── watcher.rs             # ファイル監視ロジック
│   │   ├── parser.rs              # セッションJSONパーサー
│   │   ├── session.rs             # セッションデータ構造体
│   │   ├── launcher.rs            # 外部アプリ起動
│   │   ├── notifier.rs            # macOS通知
│   │   └── tray.rs                # メニューバーアイコン
│   └── icons/                     # アプリアイコン
│       ├── 32x32.png
│       ├── 128x128.png
│       ├── 128x128@2x.png
│       ├── icon.icns
│       └── icon.png
│
├── assets/
│   └── icons/                     # アイコンカラーバリエーション
│       ├── orange.png
│       ├── cyan.png
│       ├── purple.png
│       ├── green.png
│       ├── blue.png
│       ├── pink.png
│       └── red.png
│
├── docs/
│   ├── PRD.md
│   └── SPEC.md                    # この仕様書
│
├── CLAUDE.md                      # Claude Code用プロジェクトガイド
├── package.json
├── tsconfig.json
├── vite.config.ts
├── index.html
└── README.md
```

---

## 4. データソース

### 4.1 監視対象

Claude Codeのセッションデータは `~/.claude/projects/` 配下に保存される。

```
~/.claude/
├── projects/
│   ├── {project-path-hash}/
│   │   ├── {session-id}.jsonl     # セッションログ（JSONL形式）
│   │   └── ...
│   └── ...
├── settings.json                   # Claude Code設定
└── ...
```

### 4.2 セッションファイル構造

> ⚠️ **要確認**: 実機で `~/.claude/projects/` の実際のファイル内容を確認し、以下のフィールドマッピングを検証する必要がある。

**想定されるJSONLフィールド:**

```typescript
// 各行がJSON object
interface SessionLogEntry {
  type: string; // "user", "assistant", "tool_use", "tool_result" 等
  timestamp: string; // ISO 8601
  model?: string; // "claude-sonnet-4-5-20250514" 等
  message?: {
    role: string;
    content: string | ContentBlock[];
  };
  usage?: {
    input_tokens: number;
    output_tokens: number;
    cache_read_input_tokens?: number;
    cache_creation_input_tokens?: number;
  };
  costUSD?: number;
  tool_name?: string; // ツール使用時
  status?: string; // セッションステータス
}
```

### 4.3 環境判別ロジック

| 環境          | 判別方法（想定）                                              |
| ------------- | ------------------------------------------------------------- |
| CLI (Ghostty) | プロセスリストに `claude` コマンドが存在 + 起動元がターミナル |
| VS Code       | プロセスリストに `code` 経由のClaude Code拡張                 |
| Cursor        | プロセスリストに `cursor` 経由のClaude Code拡張               |
| Desktop       | プロセスリストに `Claude.app`                                 |

> ⚠️ セッションファイル内に起動元環境の情報が含まれるかは要確認。含まれない場合はプロセス情報から推測するか、`claude_desktop_config.json` 等を参照する。

### 4.4 ステータス判定

セッションのステータスはログエントリの最新状態から推定:

| ステータス       | 判定条件                                                    |
| ---------------- | ----------------------------------------------------------- |
| `working`        | 最新エントリが `assistant` type かつ進行中                  |
| `needs_approval` | `tool_use` が発行されたが `tool_result` が未着              |
| `idle`           | 最新エントリの `assistant` に `turn_end` あり、一定時間経過 |
| `done`           | セッションファイルが更新されなくなった（閾値: 5分）         |
| `error`          | エラー情報を含むエントリが最新                              |

---

## 5. UI仕様

### 5.1 ウィンドウ

| 項目             | 値                                        |
| ---------------- | ----------------------------------------- |
| デフォルトサイズ | 1200 x 800                                |
| 最小サイズ       | 800 x 500                                 |
| タイトルバー     | macOSネイティブ（トラフィックライト付き） |
| ウィンドウ装飾   | `decorations: true`, `transparent: false` |
| 閉じるボタン挙動 | ウィンドウを隠す（トレイに常駐継続）      |

### 5.2 レイアウト

```
┌──────────────────────────────────────────────┐
│ ● ● ●              cc-pilot            ⚙    │ ← ヘッダー (48px)
├──────────┬───────────────────────────────────┤
│          │                                   │
│ Session  │        Session Detail             │
│ List     │                                   │
│          │  Project: my-app                  │
│ [All|T|V │  Branch: feature/auth             │
│  |C|D]   │  Model: claude-sonnet-4.5         │
│          │  Status: ● Working                │
│ ┌──────┐ │                                   │
│ │card 1│ │  Tokens: 12.4K in / 8.2K out     │
│ │● work│ │  Cost: $0.42                      │
│ └──────┘ │  Duration: 23m                    │
│ ┌──────┐ │                                   │
│ │card 2│ │  Active Tools:                    │
│ │⏳ wait│ │  • Read  • Write  • Bash         │
│ └──────┘ │                                   │
│ ┌──────┐ │  ┌─────────────────────────────┐  │
│ │card 3│ │  │  Open in Ghostty →          │  │
│ │● idle│ │  └─────────────────────────────┘  │
│ └──────┘ │                                   │
│          │                                   │
├──────────┴───────────────────────────────────┤
│ ~/.claude/projects/  │ 5 sessions │ 2 active │ ← ステータスバー (28px)
└──────────────────────────────────────────────┘
```

### 5.3 サイドバー（320px）

#### 環境フィルター

セッション一覧上部にタブ形式で配置:

```
[ All ] [ T ] [ V ] [ C ] [ D ]
```

| ラベル | 意味           |
| ------ | -------------- |
| All    | 全環境         |
| T      | Terminal (CLI) |
| V      | VS Code        |
| C      | Cursor         |
| D      | Desktop        |

アクティブなタブにアクセントカラーの下線を表示。

#### セッションカード

各セッションを縦積みカードで表示:

```
┌────────────────────────────┐
│ T  my-app                  │  ← 環境バッジ + プロジェクト名
│    feature/auth            │  ← ブランチ名（mono）
│    認証機能を実装して...     │  ← セッションタイトル（グレー）
│ ● Working         23m ago  │  ← ステータスドット + 経過時間
│                     $0.42  │  ← コスト（mono）
└────────────────────────────┘
```

#### セッションタイトル

セッションの内容を示すタイトル行。以下の優先順で決定:

1. **手動エイリアス**（設定済みの場合）
2. **自動取得**: セッションJSONLの最初の `user` メッセージから先頭80文字を切り出し、末尾 `...` で省略

手動エイリアスは `tauri-plugin-store` で永続化:

```json
{
  "session-aliases": {
    "session-abc123": "認証機能の実装",
    "session-def456": "DB設計リファクタ"
  }
}
```

編集UI: セッション詳細のタイトル横に ✏️ アイコン → クリックでインライン編集。空にするとエイリアス削除（自動タイトルに戻る）。

- ステータスドットの色:
  - Working: アクセントカラー（デフォルト: `#E8734A`）
  - Needs approval: `#F59E0B`（イエロー）
  - Idle: `#888`（グレー）
  - Done: `#22C55E`（グリーン）
  - Error: `#EF4444`（レッド）
- 選択中のカード: `#2A2A2A` 背景 + 左辺にアクセントカラーのボーダー
- ホバー: `#222` 背景

### 5.4 メインパネル

選択されたセッションの詳細を表示。

**ヘッダーセクション:**

- プロジェクト名（大きめ）
- セッションタイトル + ✏️ 編集アイコン（クリックでインライン編集）
- ブランチ名（mono、グレー）
- ステータスバッジ

**メトリクスセクション:**

```
Model            claude-sonnet-4.5
Input Tokens     12,438
Output Tokens     8,201
Cost              $0.42
Duration          23m 15s
Last Activity     2s ago
```

**Active Tools セクション:**
ツール名をチップス形式で横並び:

```
[ Read ] [ Write ] [ Bash ] [ WebSearch ]
```

**アクションセクション:**
環境に応じたジャンプボタン:

```
┌────────────────────────────────┐
│  ↗  Open in Ghostty           │
└────────────────────────────────┘
```

**Needs Approval 表示:**
承認待ち時、承認内容をハイライト:

```
┌────────────────────────────────────┐
│ ⚠️ Waiting for Approval            │
│                                    │
│ Tool: Execute bash command         │
│ Command: rm -rf node_modules       │
│                                    │
│ (表示のみ — 操作は各環境で行ってください) │
└────────────────────────────────────┘
```

### 5.5 設定画面

ヘッダー右端のギアアイコンから開く（メインパネルを差し替え or モーダル）。

**アクセントカラー:**

- プリセットカラー（丸いスウォッチ5色 + カスタム）
- プリセット: Orange `#E8734A` / Cyan `#22D3EE` / Purple `#A855F7` / Green `#22C55E` / Blue `#3B82F6`
- カラーピッカーでカスタム選択可能
- 選択時にリアルタイムプレビュー

**ターミナル:**

- ドロップダウン: Ghostty / iTerm2 / Terminal.app / WezTerm
- デフォルト: Ghostty

**その他:**

- ログイン時に自動起動: トグル（デフォルト: ON）
- 通知: トグル（デフォルト: ON）

### 5.6 メニューバー（トレイ）

macOSメニューバーに常駐するアイコン:

- アイコン: 22x22px 単色（白 or テンプレートイメージ）
- クリックでメインウィンドウのshow/hide をトグル
- 右クリックメニュー:
  - Show/Hide Window
  - Settings
  ***
  - Quit cc-pilot

### 5.7 ステータスバー（28px）

ウィンドウ最下部に固定:

```
~/.claude/projects/  │  5 sessions  │  2 active
```

- 監視パス（mono、グレー）
- 総セッション数
- アクティブセッション数

---

## 6. デザインシステム

### 6.1 カラーパレット

```css
:root {
  /* アクセント（CSS変数で動的変更） */
  --accent: #e8734a;
  --accent-dim: rgba(232, 115, 74, 0.15); /* hover背景等 */

  /* 背景 */
  --bg-primary: #191919;
  --bg-secondary: #1e1e1e;
  --bg-tertiary: #222222;
  --bg-elevated: #2a2a2a;

  /* テキスト */
  --text-primary: #d4d4d4;
  --text-secondary: #888888;
  --text-tertiary: #555555;

  /* ボーダー */
  --border: #2a2a2a;

  /* ステータス */
  --status-working: var(--accent);
  --status-approval: #f59e0b;
  --status-idle: #888888;
  --status-done: #22c55e;
  --status-error: #ef4444;
}
```

### 6.2 タイポグラフィ

| 用途               | フォント                                                                      | サイズ                                  |
| ------------------ | ----------------------------------------------------------------------------- | --------------------------------------- |
| UI テキスト        | `-apple-system, BlinkMacSystemFont, "Segoe UI", "Helvetica Neue", sans-serif` | 13px / 14px                             |
| コード要素         | `SF Mono, Cascadia Code, Fira Code, Menlo, monospace`                         | 12px / 13px                             |
| セクションヘッダー | system font, semibold                                                         | 11px (uppercase, letter-spacing: 0.5px) |

### 6.3 スペーシング

| トークン     | 値   |
| ------------ | ---- |
| `--space-xs` | 4px  |
| `--space-sm` | 8px  |
| `--space-md` | 12px |
| `--space-lg` | 16px |
| `--space-xl` | 24px |

### 6.4 角丸

| 要素     | 値          |
| -------- | ----------- |
| カード   | 8px         |
| バッジ   | 4px         |
| ボタン   | 6px         |
| チップス | 12px (pill) |

---

## 7. Rust バックエンド詳細

### 7.1 FileWatcher (`watcher.rs`)

```rust
// 擬似コード
fn start_watching(app_handle: AppHandle) {
    let path = dirs::home_dir().join(".claude/projects");
    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(&path, RecursiveMode::Recursive)?;

    // ファイル変更を検知したらパースしてフロントに送信
    for event in rx {
        match event {
            Ok(event) => {
                if let Some(session) = parse_session_file(&event.paths) {
                    app_handle.emit("session-update", &session)?;
                }
            }
            Err(e) => log::error!("Watch error: {}", e),
        }
    }
}
```

### 7.2 Launcher (`launcher.rs`)

| 環境         | 起動コマンド                                             |
| ------------ | -------------------------------------------------------- |
| Ghostty      | `osascript -e 'tell application "Ghostty" to activate'`  |
| iTerm2       | `osascript -e 'tell application "iTerm2" to activate'`   |
| Terminal.app | `osascript -e 'tell application "Terminal" to activate'` |
| WezTerm      | `osascript -e 'tell application "WezTerm" to activate'`  |
| VS Code      | `code {project_path}`                                    |
| Cursor       | `cursor {project_path}`                                  |
| Desktop      | `open -a "Claude"`                                       |

### 7.3 Tauri IPC Commands

```typescript
// フロントエンドから呼べるコマンド
interface TauriCommands {
  // セッション取得
  get_sessions(): Promise<Session[]>;

  // 環境にジャンプ
  open_in_environment(sessionId: string): Promise<void>;

  // 設定
  get_settings(): Promise<Settings>;
  save_settings(settings: Settings): Promise<void>;

  // セッションエイリアス
  save_alias(sessionId: string, alias: string | null): Promise<void>;
}

// Rustからフロントエンドへのイベント
interface TauriEvents {
  "session-update": Session;
  "session-removed": { id: string };
  "approval-needed": { sessionId: string; tool: string; detail: string };
}
```

---

## 8. 型定義

### 8.1 セッション

```typescript
interface Session {
  id: string; // セッションID
  projectPath: string; // プロジェクトパス
  projectName: string; // パスから抽出したプロジェクト名
  branchName?: string; // Gitブランチ名
  title: string; // セッションタイトル（自動: 最初のユーザーメッセージ先頭80文字）
  alias?: string; // 手動エイリアス（設定済みの場合titleより優先）
  environment: Environment; // 起動元環境
  status: SessionStatus; // 現在のステータス
  model?: string; // 使用モデル
  inputTokens: number; // 累計入力トークン
  outputTokens: number; // 累計出力トークン
  costUSD: number; // 累計コスト（USD）
  activeTools: string[]; // 使用中のツール一覧
  startedAt: string; // セッション開始時刻 (ISO 8601)
  lastActivityAt: string; // 最終アクティビティ (ISO 8601)
  approvalDetail?: ApprovalDetail; // 承認待ち時の詳細
  errorMessage?: string; // エラー時のメッセージ
}

type Environment = "terminal" | "vscode" | "cursor" | "desktop";
type SessionStatus = "working" | "needs_approval" | "idle" | "done" | "error";

interface ApprovalDetail {
  tool: string; // ツール名
  description: string; // 実行しようとしている内容
}
```

### 8.2 設定

```typescript
interface Settings {
  accentColor: string; // HEXカラー
  terminalApp: TerminalApp; // 使用ターミナル
  launchAtLogin: boolean; // 自動起動
  notificationsEnabled: boolean; // 通知
}

type TerminalApp = "ghostty" | "iterm2" | "terminal" | "wezterm";
```

---

## 9. 配布

### 9.1 GitHub Releases

- `cc-pilot-{version}-macos-arm64.dmg`
- `cc-pilot-{version}-macos-x64.dmg`
- Universal binary も検討

### 9.2 npm

```bash
npx cc-pilot  # Tauriバイナリをダウンロード＆実行
```

### 9.3 CI/CD

GitHub Actionsワークフロー:

1. `main` ブランチへのpushまたはtagでトリガー
2. macOS runner でビルド（arm64 + x64）
3. `.app` を `.dmg` にパッケージング
4. GitHub Releasesに自動アップロード
5. npm publish

### 9.4 README

- 英語 + 日本語
- スクリーンショット付き
- インストール手順
- 競合ツールとの比較

---

## 10. 制限事項・前提条件

### 10.1 v1 スコープ外

- アプリ上からのAllow/Deny操作（表示のみ）
- ライトテーマ
- Windows / Linux対応
- モバイル対応
- セッションへのメッセージ送信
- セッション履歴の永続化（リアルタイム監視のみ）

### 10.2 前提条件

- macOS 12 (Monterey) 以上
- Claude Codeがインストール済み
- `~/.claude/projects/` にセッションデータが存在すること

### 10.3 未確認事項（実機検証が必要）

1. `~/.claude/projects/` 配下のJSONL構造の正確なスキーマ
2. セッションファイルから環境（CLI/VSCode/Cursor/Desktop）を判別する方法
3. セッションステータス（working/needs_approval等）の正確な判定方法
4. コスト情報がセッションファイルに含まれるか

---

## 11. マイルストーン

### Phase 1: 基盤（1-2日）

- [ ] リポジトリ作成
- [ ] Tauri v2プロジェクト初期化
- [ ] 基本ディレクトリ構造
- [ ] `~/.claude/projects/` の実データ確認・パーサー実装

### Phase 2: コア機能（2-3日）

- [ ] ファイルウォッチャー実装
- [ ] セッション一覧UI
- [ ] セッション詳細UI
- [ ] ステータス判定ロジック

### Phase 3: 統合（1-2日）

- [ ] ジャンプ機能（osascript / code / cursor）
- [ ] macOS通知
- [ ] メニューバー常駐
- [ ] 設定画面

### Phase 4: 配布準備（1日）

- [ ] アイコン統合
- [ ] CI/CD設定
- [ ] README作成
- [ ] npm パッケージ準備
