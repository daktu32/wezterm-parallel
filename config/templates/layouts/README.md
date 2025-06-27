# WezTerm レイアウトテンプレート ガイド

このディレクトリには、WezTermマルチプロセス開発フレームワーク用のレイアウトテンプレートが含まれています。

## 概要

レイアウトテンプレートは、異なる開発環境や作業フローに最適化されたペイン配置とプロセス設定を定義するYAMLファイルです。各テンプレートは特定の開発シナリオに合わせて設計されており、ワンクリックで理想的な開発環境を構築できます。

## 利用可能なテンプレート

### 1. Claude開発用テンプレート (`claude-dev.yaml`)
**用途**: AI支援開発（Claude Code使用）
- **ペイン構成**: 3ペイン（エディタ、ターミナル、監視）
- **特徴**: Claude Code統合、開発ログ監視
- **推奨用途**: AI支援プログラミング、コード生成

### 2. Web開発用テンプレート (`web-dev.yaml`)
**用途**: フロントエンド・フルスタック開発
- **ペイン構成**: 4ペイン（エディタ、開発サーバー、ターミナル、ブラウザ）
- **特徴**: ホットリロード、ライブプレビュー
- **推奨用途**: React/Vue/Angular開発、Web アプリケーション

### 3. Rust開発用テンプレート (`rust-dev.yaml`)
**用途**: Rustシステムプログラミング
- **ペイン構成**: 4ペイン（エディタ、ビルド、テスト、ターミナル）
- **特徴**: cargo watch統合、継続的テスト
- **推奨用途**: Rustライブラリ、CLI ツール、サーバー開発

### 4. リサーチ用テンプレート (`research.yaml`)
**用途**: 技術調査・学習・ドキュメント作成
- **ペイン構成**: 4ペイン（ノート、ブラウザ、ツール、参照）
- **特徴**: Markdown支援、文献管理
- **推奨用途**: 技術調査、学術研究、ドキュメント作成

## テンプレートの使用方法

### 基本的な使用方法

1. **WezTermでテンプレート適用**:
   ```lua
   -- WezTerm設定内で
   wezterm.on('apply-template', function(window, pane)
     local template_manager = require('template_manager')
     template_manager.apply_template("web-dev", window, pane)
   end)
   ```

2. **キーバインド経由**:
   - `Ctrl+Shift+W`: Web開発テンプレート
   - `Ctrl+Shift+R`: Rust開発テンプレート  
   - `Ctrl+Shift+N`: リサーチテンプレート
   - `Ctrl+Shift+T`: Claude開発テンプレート

3. **Luaスクリプト経由**:
   ```lua
   local TemplateManager = require('template_manager')
   TemplateManager.apply_template("template-name", window, pane)
   ```

### 高度な使用方法

#### カスタマイズオプション
```lua
local options = {
  working_directory = "/path/to/project",
  auto_start_processes = true,
  custom_environment = {
    NODE_ENV = "development",
    DEBUG = "true"
  }
}

TemplateManager.apply_template("web-dev", window, pane, options)
```

#### テンプレート一覧・検索
```lua
-- 利用可能なテンプレート一覧
local templates = TemplateManager.list_templates({
  details = true,
  sort_by = "usage"
})

-- テンプレート検索
local matches = TemplateManager.search_templates("rust", {
  case_sensitive = false,
  fields = {"name", "description"}
})
```

## カスタムテンプレートの作成

### 基本構造

```yaml
# テンプレート基本情報
name: "My Custom Layout"
description: "カスタム開発環境"
version: "1.0.0"
author: "Your Name"
created: "2025-01-27"

# レイアウト設定
layout:
  type: "dynamic"
  description: "カスタムペイン配置"
  
  panes:
    - id: "main_editor"
      position:
        row: 0
        col: 0
        span_rows: 2
        span_cols: 2
      size: 0.6
      title: "Main Editor"
      command: "your-editor"
      working_directory: "."
      description: "メインエディタペイン"

# ワークスペース設定
workspace:
  name: "custom-workspace"
  description: "カスタムワークスペース"
  auto_start: true
  
  processes:
    - name: "custom-process"
      command: "your-command"
      auto_start: true
      working_directory: "."

# オプション設定
options:
  sync_enabled: false
  auto_save_layout: true
  save_interval: 300
```

### 必須フィールド

- `name`: テンプレート名（一意である必要があります）
- `version`: バージョン情報（セマンティックバージョニング推奨）
- `layout`: レイアウト設定
- `layout.panes`: ペイン配列（最低1つ必要）

### ペイン設定の詳細

```yaml
panes:
  - id: "unique_pane_id"           # 必須: 一意のペインID
    position:                      # オプション: ペイン位置
      row: 0                       # 行位置（0から開始）
      col: 0                       # 列位置（0から開始）  
      span_rows: 1                 # 行スパン
      span_cols: 1                 # 列スパン
    size: 0.5                      # オプション: サイズ比率（0.1-1.0）
    title: "Pane Title"            # オプション: ペインタイトル
    command: "shell-command"       # オプション: 実行コマンド
    working_directory: "."         # オプション: 作業ディレクトリ
    description: "ペインの説明"      # オプション: 説明文
```

### サイズ調整のコツ

1. **合計サイズ**: 全ペインのサイズ合計は1.0になるよう設計
2. **最小サイズ**: 各ペインのサイズは0.1以上を推奨
3. **レスポンシブ**: 異なる画面サイズを考慮

### ベストプラクティス

#### 1. 明確な命名
```yaml
name: "React TypeScript Development"  # 良い例
name: "dev"                          # 悪い例
```

#### 2. 適切な説明
```yaml
description: "React TypeScript開発用の4ペイン構成。エディタ、開発サーバー、テスト、ターミナルを含む"
```

#### 3. バージョン管理
```yaml
changelog:
  "1.0.0":
    date: "2025-01-27"
    changes:
      - "初期版作成"
      - "基本4ペイン構成"
```

#### 4. 関連テンプレート
```yaml
related_templates:
  - "react-native-dev.yaml"
  - "node-backend-dev.yaml"
```

## トラブルシューティング

### よくある問題

#### 1. テンプレートが読み込まれない
- ファイルパスが正しいか確認
- YAML文法が正しいか確認
- 必須フィールドが全て設定されているか確認

#### 2. ペインサイズが正しくない
- サイズ値が0.1-1.0の範囲内か確認
- 合計サイズが1.0を大幅に超えていないか確認

#### 3. コマンドが実行されない
- コマンドが実際に存在するか確認
- パス設定が正しいか確認
- 権限に問題がないか確認

### デバッグ方法

#### 1. テンプレート検証
```lua
local TemplateManager = require('template_manager')
local details = TemplateManager.get_template_details("template-name")
print(details.validation_result)
```

#### 2. ログ確認
```bash
tail -f ~/.local/share/wezterm-multi-dev/logs/framework.log
```

#### 3. 手動テスト
```lua
-- テンプレートを段階的にテスト
local template = TemplateLoader.load_template("path/to/template.yaml")
local layout = LayoutEngine.generate_from_template(template)
local is_valid = LayoutEngine.validate_layout(layout)
```

## 高度な機能

### 1. 条件分岐
```yaml
panes:
  - id: "editor"
    command: "${EDITOR:-code}"  # 環境変数の使用
    working_directory: "${PROJECT_ROOT:-.}"
```

### 2. 動的設定
```yaml
workspace:
  processes:
    - name: "dev-server"
      command: "npm run dev"
      auto_start: "${AUTO_START_DEV:-true}"
```

### 3. フレームワーク別設定
```yaml
framework_configs:
  react:
    dev_command: "npm start"
    test_command: "npm test"
  vue:
    dev_command: "npm run serve"
    test_command: "npm run test:unit"
```

## 貢献方法

新しいテンプレートを作成した場合：

1. このディレクトリに`.yaml`ファイルを配置
2. テストを実行して動作確認
3. このREADMEに説明を追加
4. Pull Requestを送信

### テスト方法
```bash
# テンプレート構造テスト
lua test_templates_standalone.lua

# Rustテンプレート統合テスト  
cargo test template
```

## ライセンスと利用規約

これらのテンプレートは個人利用・学習目的での使用を想定しています。商用利用や再配布については、プロジェクトのライセンスに従ってください。

## サポート

- Issue: GitHub Issues
- ドキュメント: `docs/` ディレクトリ
- 例: `test_*.lua` ファイル

---

**最終更新**: 2025年1月27日  
**バージョン**: Issue #18 Phase 3 完了版