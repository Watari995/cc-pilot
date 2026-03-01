# cc-pilot — 詳細仕様書

## 1. プロジェクト概要

### 1.1 プロダクト定義

**cc-pilot** は、Claude Codeの全セッション（CLI / VS Code拡張 / Cursor拡張 / Desktop / Web）を1つのUIでリアルタイム監視し、タップで各環境にジャンプできるmacOSデスクトップアプリ。

### 1.2 解決する課題

- 複数のClaude Codeセッションを同時に走らせると、どのセッションが何をしているか把握しづらい
- セッションが承認待ち（Needs approval）になっても気づけない
- ターミナル、VS Code、Cursor、Desktop、Webを行き来する手間

### 1.3 ターゲットユーザー

Claude Codeを日常的に使う開発者。特に複数セッションを並行実行するヘビーユーザー。

### 1.4 パッケージ情報

| 項目         | 値                             |
| ------------ | ------------------------------ |
| パッケージ名 | `cc-pilot`                     |
| リポジトリ   | `github.com/Watari995/cc-pilot` |
| ライセンス   | MIT                            |
| 対象OS       | macOS（Apple Silicon + Intel） |

---

## 2. 技術スタック

### 2.1 コアテクノロジー

| レイヤー       | 技術                        | 役割                             |
| -------------- | --------------------------- | -------------------------------- |
| フレームワーク | Tauri v2                    | ネイティブアプリ基盤             |
| フロントエンド | React 19 + TypeScript 5     | UI描画                           |
| スタイリング   | CSS Modules                 | コンポーネントスタイル           |
| 状態管理       | Zustand                     | 軽量グローバルステート           |
| バックエンド   | Rust                        | ファイル監視・プロセス管理・通知 |
| ビルドツール   | Vite                        | フロントエンドビルド             |

### 2.2 主要Rust crates

| crate                       | 用途                                       |
| --------------------------- | ------------------------------------------ |
| `notify` (v6+)              | `~/.claude/projects/` のファイル変更監視   |
| `serde` / `serde_json`      | セッションJSONパース                       |
| `reqwest`                   | claude.ai API クライアント（Web版監視）    |
| `tauri` v2                  | アプリフレームワーク                       |
| `tauri-plugin-notification` | macOS通知                                  |
| `tauri-plugin-shell`        | 外部コマンド実行 (osascript, code, cursor) |
| `tauri-plugin-store`        | ユーザー設定永続化                         |

### 2.3 ビルド成果物

- `.app` バンドル（macOS）
- `.dmg` インストーラー

---

## 3. アーキテクチャ

### 3.1 全体構成

```
┌──────────────────────────────────────────────────┐
│                    cc-pilot                       │
│                                                  │
│  ┌──────────────────┐     ┌──────────────────┐  │
│  │   Rust Backend    │     │ React (WebView)  │  │
│  │                   │◄───►│ Frontend         │  │
│  │                   │ IPC │                  │  │
│  │ • FileWatcher     │     │ • SessionList    │  │
│  │ • ProcessDetector │     │ • SessionDetail  │  │
│  │ • WebClient       │     │ • Settings       │  │
│  │ • Launcher        │     │ • StatusBar      │  │
│  │ • Notifier        │     │                  │  │
│  │ • TrayIcon        │     │                  │  │
│  └───────┬───────────┘     └──────────────────┘  │
│          │                                        │
│    ┌─────┴──────┐                                │
│    │            │                                │
│    ▼            ▼                                │
│  ~/.claude/   claude.ai API                      │
│  projects/    (reqwest polling)                   │
└──────────────────────────────────────────────────┘
```

### 3.2 データフロー

**ローカルセッション（CLI / Cursor / VS Code / Desktop）:**
```
~/.claude/projects/**/*.jsonl
        │
        ▼ (notify crate: ファイル変更検知)
   Rust FileWatcher
        │
        ▼ (JSONパース + プロセス情報で環境判別)
   SessionState (Rust構造体)
        │
        ▼ (Tauri IPC: emit event)
   React Frontend (Zustand store)
        │
        ▼ (React re-render)
   UI更新
```

**Webセッション（claude.ai）:**
```
claude.ai /api/organizations/{orgId}/chat_conversations
        │
        ▼ (reqwest: 30秒間隔ポーリング)
   Rust WebClient
        │
        ▼ (Session構造体に変換, environment: "web")
   SessionState に統合
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
│   │   └── formatters.ts         # 時間表示フォーマッタ
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
│   │   ├── process_detector.rs    # プロセス情報から環境判別
│   │   ├── web_client.rs          # claude.ai API クライアント
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
│
├── CLAUDE.md                      # Claude Code用プロジェクトガイド
├── cc-pilot-spec.md               # この仕様書
├── package.json
├── tsconfig.json
├── vite.config.ts
└── index.html
```

---

## 4. データソース

### 4.1 ローカルセッション（JSONL監視）

Claude Codeのセッションデータは `~/.claude/projects/` 配下に保存される。

```
~/.claude/
├── projects/
│   ├── -Users-username-project-name/  # パスのハイフン区切り表現
│   │   ├── {session-id}.jsonl          # セッションログ（JSONL形式）
│   │   └── ...
│   └── ...
└── ...
```

### 4.2 セッションファイル構造（実機確認済み）

各行がJSON objectのJSONL形式。エントリのトップレベル `type` フィールドで種別を判定:

| type | 説明 | 出現頻度 |
|---|---|---|
| `user` | ユーザーメッセージ | 多 |
| `assistant` | アシスタント応答（model, usage含む） | 多 |
| `progress` | フック実行・エージェント進捗 | 多 |
| `system` | コンパクション境界等 | 少 |
| `queue-operation` | キュー操作（enqueue/dequeue） | 少 |
| `file-history-snapshot` | ファイル履歴スナップショット | 少 |

**共通フィールド（各エントリのトップレベル）:**

```typescript
interface BaseEntry {
  type: string;
  sessionId: string;
  uuid: string;
  parentUuid: string | null;
  timestamp: string;           // ISO 8601
  cwd: string;                 // プロジェクトパス（例: "/Users/user/myproject"）
  gitBranch: string;           // ブランチ名
  version: string;             // Claude Codeバージョン（例: "2.1.63"）
  userType: string;            // 常に "external"
  permissionMode?: string;     // "bypassPermissions" | "default" | "acceptEdits"
  isSidechain: boolean;
}
```

**`assistant` エントリの `message` 構造:**

```typescript
interface AssistantMessage {
  model: string;               // 例: "claude-opus-4-6"
  id: string;
  type: "message";
  role: "assistant";
  content: ContentBlock[];     // text, thinking, tool_use 等
  stop_reason: string | null;  // 実データでは常に null
  usage: {
    input_tokens: number;
    output_tokens: number;
    cache_creation_input_tokens: number;
    cache_read_input_tokens: number;
  };
}

type ContentBlock =
  | { type: "text"; text: string }
  | { type: "thinking"; thinking: string }
  | { type: "tool_use"; id: string; name: string; input: object }
  ;
```

**`user` エントリの `message` 構造:**

```typescript
interface UserMessage {
  role: "user";
  content: UserContentBlock[];
}

type UserContentBlock =
  | { type: "text"; text: string }
  | { type: "tool_result"; tool_use_id: string; content: string; is_error?: boolean }
  ;
```

**重要な発見事項:**
- `costUSD` フィールドは存在しない → コスト表示は実装しない
- `model` は `message.model` にある（トップレベルではない）
- `usage` は `message.usage` にある
- `userType` は常に `"external"` — 環境判別には使えない
- IDE環境のセッションは `<ide_opened_file>` タグが user メッセージに含まれる
- `stop_reason` は実データでは常に `null`（API直接のレスポンスとは異なる）

### 4.3 Webセッション（claude.ai API）

**エンドポイント:**

| メソッド | エンドポイント | 用途 |
|---|---|---|
| GET | `/api/organizations` | 組織ID取得 |
| GET | `/api/organizations/{orgId}/chat_conversations` | 会話一覧 |
| GET | `/api/organizations/{orgId}/chat_conversations/{chatId}` | 会話詳細 |

**認証:** ブラウザの `sessionKey` Cookie（`sk-ant-sid01-*` 形式）

**ディープリンク:** `https://claude.ai/chat/{conversationId}` で特定の会話に直接ジャンプ可能

**制約事項:**
- 非公式API — Anthropic が変更すると動かなくなる可能性あり
- sessionKey の有効期限 — 期限切れ時はUIで再入力を促す
- レートリミット — ポーリング間隔は30秒以上を維持

### 4.4 環境判別ロジック（実機確認済み）

JSONLデータには環境情報がないため、プロセス情報から判別する:

| 環境 | 判別方法 | 確認済み |
|---|---|---|
| Terminal (CLI) | `claude` プロセスが TTY を持つ（`ps -o tty` が `ttysXXX`） | Yes |
| Cursor | プロセスパスに `.cursor/extensions/anthropic.claude-code-` を含む | Yes |
| VS Code | プロセスパスに `.vscode/extensions/anthropic.claude-code-` を含む | 未確認（要テスト） |
| Desktop | 上記いずれにも該当しないローカルセッション | 推定 |
| Web | `web_client.rs` 経由で取得（API由来） | - |

**PID → セッション紐づけ:**
1. `ps` でClaude Codeプロセス一覧を取得（PID, TTY, コマンドパス）
2. `lsof -d cwd -p {PID}` で各プロセスのCWDを取得
3. JSONL エントリの `cwd` フィールドとマッチング

### 4.5 ステータス判定

| ステータス | 判定条件 |
|---|---|
| `working` | JONLファイルが直近数秒以内に更新されている |
| `needs_approval` | 最新の `assistant` エントリの `message.content` に `tool_use` があり、対応する `tool_result` が次の `user` エントリにまだない + ファイル更新が停止 |
| `idle` | ファイル更新が1分以上停止 + 最新エントリが `assistant`（`tool_use` なし） |
| `done` | ファイル更新が5分以上停止 |
| `error` | 最新エントリにエラー情報を含む |

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
│  |C|D|W] │  Model: claude-opus-4-6           │
│          │  Status: ● Working                │
│ ┌──────┐ │                                   │
│ │card 1│ │  Tokens: 12.4K in / 8.2K out     │
│ │● work│ │  Duration: 23m                    │
│ └──────┘ │                                   │
│ ┌──────┐ │  Active Tools:                    │
│ │card 2│ │  • Read  • Write  • Bash         │
│ │⏳ wait│ │                                   │
│ └──────┘ │  ┌─────────────────────────────┐  │
│ ┌──────┐ │  │  Open in Ghostty →          │  │
│ │card 3│ │  └─────────────────────────────┘  │
│ │● idle│ │                                   │
│ └──────┘ │                                   │
│          │                                   │
├──────────┴───────────────────────────────────┤
│ 5 sessions │ 2 active │ Web: connected       │ ← ステータスバー (28px)
└──────────────────────────────────────────────┘
```

### 5.3 サイドバー（320px）

#### 環境フィルター

セッション一覧上部にタブ形式で配置:

```
[ All ] [ T ] [ V ] [ C ] [ D ] [ W ]
```

| ラベル | 意味           |
| ------ | -------------- |
| All    | 全環境         |
| T      | Terminal (CLI) |
| V      | VS Code        |
| C      | Cursor         |
| D      | Desktop        |
| W      | Web (claude.ai)|

アクティブなタブにアクセントカラーの下線を表示。

#### セッションカード

各セッションを縦積みカードで表示:

```
┌────────────────────────────┐
│ T  my-app                  │  ← 環境バッジ + プロジェクト名
│    feature/auth            │  ← ブランチ名（mono）
│    認証機能を実装して...     │  ← セッションタイトル（グレー）
│ ● Working         23m ago  │  ← ステータスドット + 経過時間
└────────────────────────────┘
```

Webセッションの場合:
```
┌────────────────────────────┐
│ W  Claude Code セッション   │  ← 環境バッジ + 会話名
│    新機能の設計について...   │  ← セッションタイトル（グレー）
│ ● Working          5m ago  │  ← ステータスドット + 経過時間
└────────────────────────────┘
```

#### セッションタイトル

セッションの内容を示すタイトル行。以下の優先順で決定:

1. **手動エイリアス**（設定済みの場合）
2. **自動取得**: ローカルはJSONLの最初の `user` メッセージから先頭80文字。Webは会話名。

手動エイリアスは `tauri-plugin-store` で永続化。

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
- ブランチ名（mono、グレー）— Webセッションでは非表示
- ステータスバッジ

**メトリクスセクション:**

```
Model            claude-opus-4-6
Input Tokens     12,438
Output Tokens     8,201
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

Webセッションの場合:
```
┌────────────────────────────────┐
│  ↗  Open in Browser           │
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

**Web版連携:**

- `sessionKey` 入力フィールド（パスワードマスク表示）
- 取得方法のヘルプテキスト: 「claude.ai にログイン → DevTools → Application → Cookies → `sessionKey` の値をコピー」
- 接続テストボタン
- ステータス表示: Connected / Disconnected / Invalid Key

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
5 sessions  │  2 active  │  Web: connected
```

- 総セッション数
- アクティブセッション数
- Web接続ステータス（sessionKey 設定時のみ表示）

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

### 7.2 ProcessDetector (`process_detector.rs`)

```rust
// 擬似コード
fn detect_environment(session_cwd: &str) -> Environment {
    // 1. ps で claude プロセス一覧取得
    // 2. lsof -d cwd でCWDマッチング
    // 3. プロセスパス・TTYで環境判別

    if process_path.contains(".cursor/extensions/") {
        Environment::Cursor
    } else if process_path.contains(".vscode/extensions/") {
        Environment::VsCode
    } else if has_tty {
        Environment::Terminal
    } else {
        Environment::Desktop
    }
}
```

### 7.3 WebClient (`web_client.rs`)

```rust
// 擬似コード
async fn poll_web_sessions(session_key: &str) -> Result<Vec<Session>> {
    let client = reqwest::Client::new();

    // 1. 組織ID取得
    let orgs = client.get("https://claude.ai/api/organizations")
        .header("Cookie", format!("sessionKey={}", session_key))
        .send().await?;

    // 2. 会話一覧取得
    let conversations = client.get(format!(
        "https://claude.ai/api/organizations/{}/chat_conversations", org_id
    )).send().await?;

    // 3. Session構造体に変換
    conversations.iter().map(|c| Session {
        id: c.id,
        environment: Environment::Web,
        // ...
    }).collect()
}
```

### 7.4 Launcher (`launcher.rs`)

| 環境         | 起動コマンド | 精度 |
| ------------ | -------------------------------------------------------- | --- |
| Ghostty      | Accessibility API タブ名マッチ → クリック。フォールバック: `osascript activate` | 中 |
| iTerm2       | `osascript -e 'tell application "iTerm2" to activate'`   | 低 |
| Terminal.app | `osascript -e 'tell application "Terminal" to activate'` | 低 |
| WezTerm      | `osascript -e 'tell application "WezTerm" to activate'`  | 低 |
| VS Code      | `code {project_path}`                                    | 高 |
| Cursor       | `cursor {project_path}`                                  | 高 |
| Desktop      | `open -a "Claude"`                                       | 低 |
| Web          | `open https://claude.ai/chat/{conversation_id}`          | 高 |

### 7.5 Tauri IPC Commands

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

  // Web接続テスト
  test_web_connection(sessionKey: string): Promise<{ success: boolean; error?: string }>;
}

// Rustからフロントエンドへのイベント
interface TauriEvents {
  "session-update": Session;
  "session-removed": { id: string };
  "approval-needed": { sessionId: string; tool: string; detail: string };
  "web-connection-status": { connected: boolean; error?: string };
}
```

---

## 8. 型定義

### 8.1 セッション

```typescript
interface Session {
  id: string;                    // セッションID（ローカル: UUID, Web: conversation ID）
  projectPath: string;           // プロジェクトパス（Web: 空文字列）
  projectName: string;           // パスから抽出したプロジェクト名（Web: 会話名）
  branchName?: string;           // Gitブランチ名（Web: なし）
  title: string;                 // セッションタイトル
  alias?: string;                // 手動エイリアス（設定済みの場合titleより優先）
  environment: Environment;      // 起動元環境
  status: SessionStatus;         // 現在のステータス
  model?: string;                // 使用モデル
  inputTokens: number;           // 累計入力トークン
  outputTokens: number;          // 累計出力トークン
  activeTools: string[];         // 使用中のツール一覧
  startedAt: string;             // セッション開始時刻 (ISO 8601)
  lastActivityAt: string;        // 最終アクティビティ (ISO 8601)
  approvalDetail?: ApprovalDetail; // 承認待ち時の詳細
  errorMessage?: string;         // エラー時のメッセージ
}

type Environment = "terminal" | "vscode" | "cursor" | "desktop" | "web";
type SessionStatus = "working" | "needs_approval" | "idle" | "done" | "error";

interface ApprovalDetail {
  tool: string;                  // ツール名
  description: string;           // 実行しようとしている内容
}
```

### 8.2 設定

```typescript
interface Settings {
  accentColor: string;           // HEXカラー
  terminalApp: TerminalApp;      // 使用ターミナル
  launchAtLogin: boolean;        // 自動起動
  notificationsEnabled: boolean; // 通知
  claudeSessionKey?: string;     // claude.ai Web版監視用（sk-ant-sid01-*）
}

type TerminalApp = "ghostty" | "iterm2" | "terminal" | "wezterm";
```

---

## 9. 配布

### 9.1 GitHub Releases

- `cc-pilot-{version}-macos-arm64.dmg`
- `cc-pilot-{version}-macos-x64.dmg`
- Universal binary も検討

### 9.2 CI/CD

GitHub Actionsワークフロー:

1. `main` ブランチへのtag pushでトリガー
2. macOS runner でビルド（arm64 + x64）
3. `.app` を `.dmg` にパッケージング
4. GitHub Releasesに自動アップロード

### 9.3 README

- 英語 + 日本語
- スクリーンショット付き
- インストール手順

---

## 10. 制限事項・前提条件

### 10.1 v1 スコープ外

- アプリ上からのAllow/Deny操作（表示のみ）
- ライトテーマ
- Windows / Linux対応
- モバイル対応
- セッションへのメッセージ送信
- セッション履歴の永続化（リアルタイム監視のみ）
- コスト表示（JONLにcostUSD情報なし）

### 10.2 前提条件

- macOS 12 (Monterey) 以上
- Claude Codeがインストール済み
- `~/.claude/projects/` にセッションデータが存在すること
- Web版監視: claude.ai の有効なsessionKeyが必要

---

## 11. マイルストーン

### Phase 1: 基盤

- [ ] Tauri v2プロジェクト初期化
- [ ] 基本ディレクトリ構造

### Phase 2: コアバックエンド

- [ ] watcher + parser（ファイル監視・JSONLパース）
- [ ] process_detector（環境判別）
- [ ] session 構造体

### Phase 3: コアUI

- [ ] SessionList（サイドバー）
- [ ] SessionDetail（メインパネル）
- [ ] 環境フィルター
- [ ] ステータス判定ロジック

### Phase 4: ジャンプ＆統合

- [ ] launcher（ジャンプ機能）
- [ ] web_client（claude.ai APIポーリング）
- [ ] Settings画面（sessionKey入力含む）

### Phase 5: 仕上げ

- [ ] tray + notifier（メニューバー常駐 + 通知）
- [ ] StatusBar
- [ ] CI/CD設定
- [ ] README作成
