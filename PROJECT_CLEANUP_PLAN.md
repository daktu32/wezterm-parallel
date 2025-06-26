# プロジェクト構造最適化計画

## 📋 現状の問題点

### 1. 容量の問題
- **合計サイズ**: 2.1GB (target/ 1.5GB + releases/ 2.3MB が大部分)
- **実際のソースコード**: 364K のみ

### 2. 混乱を招く構造
- Node.js と Rust の混在
- 重複ファイル・定義
- 未実装機能のドキュメント

### 3. 不要なファイル
- 空ディレクトリ
- 過去のリリース成果物
- 未使用スクリプト

## 🎯 最適化目標

### 1. シンプル・クリーンな構造
- Rustプロジェクトとして一貫性
- 実装済み機能のみのドキュメント
- 重複排除

### 2. 開発効率向上
- 不要ファイル削除による高速化
- 明確なプロジェクト構造
- Phase 1開発に集中

## 🗑️ 削除対象ファイル・ディレクトリ

### 高優先度削除
```
releases/                    # 2.3MB - 過去のリリース成果物
package.json                 # Node.js設定 (Rustプロジェクトに不要)
package-lock.json            # Node.js lockfile
test_wezterm_config.lua      # 重複ファイル
install.sh                   # 未完成インストールスクリプト
release.sh                   # 未完成リリーススクリプト
integration_test.sh          # 未実装テストスクリプト
scripts/                     # 未使用スクリプトディレクトリ
```

### 空ディレクトリ削除
```
src/communication/           # 空ディレクトリ
tests/integration/           # 空ディレクトリ
```

### ドキュメント整理
```
docs/features/               # 未実装機能ドキュメント
docs/installation.md         # 実装と乖離
docs/troubleshooting.md      # 実装と乖離
docs/tutorial.md             # 実装と乖離
```

### 重複コード整理
```
src/message.rs               # lib.rs と重複定義
```

## 📁 最適化後の推奨構造

```
wezterm-parallel/
├── Cargo.toml              # Rustプロジェクト設定
├── Cargo.lock              # 依存関係ロック
├── README.md               # プロジェクト概要
├── DEVELOPMENT_ROADMAP.md  # 開発計画
├── CLAUDE.md               # Claude Code設定
├── .gitignore              # Git設定
│
├── src/                    # Rustソースコード
│   ├── lib.rs              # ライブラリエントリ
│   ├── main.rs             # メインエントリ
│   ├── config/             # 設定管理
│   ├── workspace/          # ワークスペース管理
│   ├── process/            # プロセス管理
│   ├── metrics/            # メトリクス管理
│   └── dashboard/          # ダッシュボード機能
│
├── lua/                    # WezTerm統合Lua
│   ├── config/             # 基本設定
│   ├── ui/                 # UI機能
│   ├── utils/              # ユーティリティ
│   └── workspace/          # ワークスペース統合
│
├── config/                 # 設定テンプレート
│   └── templates/          # YAML/Lua テンプレート
│
├── tests/                  # テストコード
│   └── unit/               # ユニットテスト
│
└── docs/                   # 実装済み機能ドキュメント
    ├── ARCHITECTURE.md     # アーキテクチャ
    ├── prd.md             # 要求仕様
    └── adr/               # 設計決定記録
```

## 🎯 削除の効果

### 容量削減
- **削除予定**: 約2.5MB
- **最終サイズ**: ~364K (ソースコードのみ)
- **85%以上の容量削減**

### 開発効率向上
- クリーンなプロジェクト構造
- Phase 1開発に集中可能
- 混乱要素の排除

### 保守性向上
- 一貫したRustプロジェクト構造
- 重複コードの排除
- 実装に即したドキュメント

## 🚀 実行計画

### Phase 1: 安全な削除
1. 大容量ディレクトリ削除 (releases/)
2. Node.js関連ファイル削除
3. 空ディレクトリ削除

### Phase 2: コード整理
1. 重複ファイル削除
2. lib.rs の Message定義統一
3. 未使用スクリプト削除

### Phase 3: ドキュメント整理
1. 未実装機能ドキュメント削除
2. 実装済み機能のドキュメント更新
3. 開発ガイド整備

## ⚠️ 注意事項

### バックアップ
- 削除前にgitコミット確実実行
- feature ブランチは保持

### 段階実行
- 一度にすべて削除せず段階的実行
- 各段階でビルド・テスト確認

### Phase 1準備
- クリーンアップ後すぐにPhase 1開発開始可能
- 最適化された環境で効率的開発