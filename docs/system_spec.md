# System Specification: Local Dev Insights

## 1. 構成要素
### 1.1 MCP Server Core
- **言語**: Rust Edition 2024
- **ランタイム**: Tokio (非同期処理)
- **プロトコル**: MCP (Model Context Protocol) 1.0
- **通信方式**: Standard Input/Output (stdio)

### 1.2 データストレージ
- **SQLite**: `dev_insights.db`
  - `memos` テーブル: `id`, `content`, `tags`, `created_at`

## 2. インターフェース定義
### 2.1 Resources
- `db://memos`: 
  - 説明: 保存されたすべての開発メモをJSON形式で返す。
- `env://vars`:
  - 説明: プロジェクトルートの `.env` から読み取った環境変数のリスト。

### 2.2 Tools
- `add_memo(content: string, tags: string[])`:
  - 説明: 新しいメモをデータベースに保存する。
- `get_system_stats()`:
  - 説明: 現在のCPU使用率、メモリ使用率、特定ポート（3000, 8080等）のListen状態を返す。
- `list_files_by_extension(extension: string)`:
  - 説明: カレントディレクトリ以下の指定された拡張子のファイルを再帰的にリストアップする。

### 2.3 Prompts
- `analyze-health`:
  - 説明: `get_system_stats` の結果を元に、開発環境が正常か診断させるプロンプトテンプレート。

## 3. 制約事項
- ファイル操作ツールは、プロジェクトルート外へのアクセスを禁止する（パスのバリデーション）。
- DB操作は `rusqlite` または `sqlx` を使用し、SQLインジェクション対策を行う。