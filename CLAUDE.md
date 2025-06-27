# CLAUDE.md

このファイルは、このリポジトリでのコード作業時にClaude Code (claude.ai/code) へのガイダンスを提供します。

## プロジェクト概要

**WezTerm マルチプロセス開発補助ツール** - WezTermでClaude Codeを複数プロセス実行するための実験的なツールです。

### 主要機能
- 複数Claude Codeプロセスの管理
- ワークスペース単位でのプロセス整理
- 基本的なタスク管理
- シンプルな監視機能

## アーキテクチャ概要

### コンポーネント構成
1. **フロントエンドレイヤー (WezTerm + Lua)**
   - WezTerm Terminal: ユーザーインターフェース
   - Lua Configuration: 設定管理とイベントハンドリング
   - Workspace Management: ワークスペースとペインの管理

2. **バックエンドレイヤー (Rust)**
   - Process Manager: Claude Codeプロセスの管理
   - Communication Hub: プロセス間通信の仲介
   - State Management: アプリケーション状態の永続化

### 技術スタック
- **フロントエンド**: WezTerm、Lua (7,175行実装済み)
- **バックエンド**: Rust (19,335行実装済み)
- **IPC**: Unix Domain Socket
- **設定**: YAML/TOML
- **状態管理**: JSON/YAML、SQLite（オプション）

## 実装状況

### ✅ MVP機能 (Issue #17 & #18)
- **Claude Code複数プロセス協調システム**: プロセス間通信・タスク分散・ファイル同期・成果物統合
- **WezTermペイン分割・レイアウトテンプレート**: YAMLテンプレート・動的レイアウト・4種実用テンプレート

### ✅ 実装済み機能
- 基本的なプロセス管理 (起動・停止・監視)
- ワークスペース作成・切り替え
- Unix Socket経由のIPC通信
- タスク管理・キューイング・スケジューリング
- 基本的な時間追跡
- WebSocketダッシュボード・メトリクス収集
- カンバンボード風UI

### ⚠️ 制限事項
- 個人開発・実験用途のみ
- 本格運用には不向き
- エラーハンドリングが不完全
- テストカバレッジが限定的

## 開発ワークフロー

### 開発プラクティス
1. **テストファースト開発**: 実装前にテストを作成する (現在47個実装済み)
2. **進捗追跡**: PROGRESS.mdとDEVELOPMENT_ROADMAP.mdを更新する
3. **品質チェック**: コミット前にlint、型チェック、テストを通す
4. **GitHub Issues**: [Issue #8-16](https://github.com/daktu32/wezterm-parallel/issues) で進捗管理

### ブランチ戦略
- `main`: 動作する最新バージョン
- `develop`: 開発中の機能
- `feature/*`: 新機能ブランチ
- `fix/*`: バグ修正ブランチ

## コマンド

### 開発用コマンド（実装済み）
```bash
# Rustプロジェクトのビルド
cargo build

# 全テストの実行 (47個のテスト)
cargo test

# フレームワークの起動
cargo run

# リリースビルド
cargo build --release

# ドキュメント生成
cargo doc --open
```

### WezTerm設定関連（Phase 2で実装予定）
```bash
# WezTerm設定のリロード
# Ctrl+Shift+R (設定内で定義予定)

# 新規ワークスペース作成
# Ctrl+Shift+N (設定内で定義予定)

# ワークスペース切り替え
# Ctrl+Shift+W (設定内で定義予定)
```

## ディレクトリ構造（実装済み）

```
wezterm-parallel/
├── src/                    # Rustソースコード (6,734行)
│   ├── workspace/          # ワークスペース管理 (完全実装)
│   ├── process/            # プロセス管理 (完全実装)
│   ├── config/             # 設定管理 (完全実装)
│   ├── metrics/            # メトリクス収集 (完全実装)
│   ├── dashboard/          # ダッシュボード基盤
│   ├── lib.rs              # ライブラリエントリ
│   └── main.rs             # メインエントリ
├── lua/                    # WezTerm Lua統合 (3,239行)
│   ├── config/             # 基本設定・キーバインド
│   ├── ui/                 # UI機能 (ダッシュボード等)
│   ├── utils/              # ユーティリティ
│   └── workspace/          # ワークスペース統合
├── config/                 # 設定テンプレート
├── tests/                  # テスト (47個実装済み)
└── docs/                   # ドキュメント
```

## 開発進捗管理

### GitHub Issues（実装済み）
- [Issue #8](https://github.com/daktu32/wezterm-parallel/issues/8): ProcessManager-WorkspaceManager統合
- [Issue #9](https://github.com/daktu32/wezterm-parallel/issues/9): WezTerm Lua統合実装
- [Issue #11](https://github.com/daktu32/wezterm-parallel/issues/11): WebSocketダッシュボード
- [Issue #12](https://github.com/daktu32/wezterm-parallel/issues/12): ペイン管理システム

### 次のマイルストーン
1. **Issue #8**: ProcessManager-WorkspaceManager 統合強化
2. **Issue #9**: 基本的なWezTerm Lua統合
3. **Issue #11**: WebSocketダッシュボード実装

## 想定される性能

- ワークスペース起動: 数秒
- 基本操作: レスポンシブ
- メモリ使用量: 軽量（プロセス数に依存）
- CPU使用率: 低負荷

## セキュリティ注意事項

- ローカル環境での使用を想定
- プロセス間通信は平文
- 本格的なセキュリティ対策は未実装

## 進捗管理ルール

### 必須ファイル更新
AIエージェントは以下のファイルを最新に保つ必要があります：

1. **PROGRESS.md** - 開発進捗の追跡
   - 各タスク完了後に更新
   - 完了したタスク、現在の作業、次のタスクを文書化
   - 日付とタイムスタンプを含める

2. **DEVELOPMENT_ROADMAP.md** - 開発ロードマップ
   - フェーズの進行に応じて更新
   - 完了したマイルストーンにチェックマークを付ける
   - 新しい課題や変更を反映

### 更新タイミング
- 機能実装完了時
- 重要な設定変更後
- フェーズ移行時
- バグ修正や改善後
- 新しい技術的決定時

### 更新方法
1. 作業完了直後に該当ファイルを更新
2. 具体的な成果物と変更を文書化
3. 次のステップを明確化
4. コミットメッセージに進捗更新を含める

### 実装後チェックリスト
- [ ] 基本テストが通過
- [ ] Rustコンパイルが成功
- [ ] 基本動作確認
- [ ] ドキュメント更新
- [ ] 進捗記録更新

## AIエージェント（Claude Code）向け開発ガイドライン

### 📋 必須遵守事項

#### 1. ドキュメント体系の維持
- **必読**: [docs/DOCUMENTATION-MAP.md](docs/DOCUMENTATION-MAP.md) のメンテナンスガイドに従う
- **Single Source of Truth**: 各情報は1つのドキュメントで管理、他は参照のみ
- **更新の伝播**: 情報更新時は関連ドキュメントも同時更新

#### 2. 数値データの正確性
```yaml
情報の管理責任:
  実装規模: PROGRESS.md (現在: Rust 19,335行, Lua 7,175行)
  テスト状況: TESTING.md (現在: 127個のライブラリテスト通過)
  MVP機能: FEATURE-SPEC.md (Issue #17, #18)
  API仕様: API.md
```

#### 3. 実装状況の正確な記載
- ❌ **禁止**: 未確認情報の記載、過大評価、虚偽記載
- ✅ **推奨**: 実測ベース、現実的な表現、進行中の課題も明記

#### 4. ドキュメント更新ワークフロー

**機能追加時**:
```
1. FEATURE-SPEC.md → 機能仕様追加・更新
2. API.md → インターフェース仕様追加
3. TESTING.md → テスト要件追加
4. PROGRESS.md → 進捗状況更新
5. README.md → ユーザー向け説明更新
```

**実装完了時**:
```
1. FEATURE-SPEC.md → 実装状況更新 (❌→✅)
2. TESTING.md → テスト結果更新
3. PROGRESS.md → 完了タスク記録
4. README.md → 機能説明更新
```

### 🔧 技術的な実装指針

#### 1. 機能開発の優先順位
```yaml
最優先: MVP機能 (Issue #17, #18) の改善・バグ修正
高優先: コア機能の安定化
中優先: 拡張機能の改善
低優先: 新機能の追加
```

#### 2. テスト戦略
- **ライブラリテスト**: 127個（全て通過維持）
- **統合テスト**: 部分実装中（8通過/4失敗→改善必要）
- **新機能**: テストファースト開発（TDD）

#### 3. コード品質基準
```rust
// 必須: エラーハンドリング
pub fn operation() -> Result<T, Error> {
    // 処理...
}

// 必須: ドキュメント
/// 機能の説明
/// 
/// # Examples
/// ```
/// let result = operation()?;
/// ```

// 推奨: 型安全性
#[derive(Debug, Clone, PartialEq)]
pub struct SafeType {
    validated_field: String,
}
```

### 📊 品質管理チェック

#### 実装前チェック
- [ ] FEATURE-SPEC.mdに機能ID付きで仕様記載
- [ ] 関連するテストケース設計
- [ ] 既存機能への影響評価

#### 実装後チェック
- [ ] テスト実行・結果確認
- [ ] ドキュメント更新完了
- [ ] コードレビュー基準満足
- [ ] 実装状況の正確な記録

#### コミット前チェック
```bash
# 必須実行
cargo test          # テスト確認
cargo clippy        # 静的解析
cargo fmt          # フォーマット

# 推奨実行
cargo build --release  # リリースビルド確認
```

### 🚨 重要な注意事項

#### 絶対に避けるべき行為
- ❌ 未確認の数値データを記載する
- ❌ テスト結果を改ざん・誇張する
- ❌ 実装していない機能を「完了」とマークする
- ❌ 他のドキュメントとの矛盾を放置する
- ❌ Single Source of Truth原則を破る

#### 推奨される行動
- ✅ 実測値に基づいた正確な情報更新
- ✅ 課題・制限事項の率直な記載
- ✅ 段階的・継続的な改善
- ✅ ドキュメント間の整合性維持

### 📞 不明点の解決方法

1. **ドキュメント体系**: [DOCUMENTATION-MAP.md](docs/DOCUMENTATION-MAP.md)
2. **機能仕様**: [FEATURE-SPEC.md](docs/FEATURE-SPEC.md)
3. **API詳細**: [API.md](docs/API.md)
4. **テスト指針**: [TESTING.md](docs/TESTING.md)
5. **開発プロセス**: [CONTRIBUTING.md](docs/CONTRIBUTING.md)

これらのガイドラインに従って、高品質で整合性のあるプロジェクト開発を行ってください。

## プロジェクト固有の開発ルール

### Gitワークフロー

#### ブランチ戦略
- **メインブランチ**: `main` (Phase 1完了)
- **機能ブランチ**: `feature/task-description`
- **バグ修正ブランチ**: `fix/bug-description`

#### 必須作業手順
すべての開発作業で以下の手順に従ってください：

1. 機能要件を定義し、GitHub Issueで管理
2. **作業ブランチを作成し、git worktreeで分離**
3. 期待される入力と出力に基づいてテストを作成
4. テストを実行し、失敗を確認
5. テストを通過するコードを実装
6. すべてのテストが通過したらリファクタリング
7. 進捗ファイル（PROGRESS.md、DEVELOPMENT_ROADMAP.md）を更新

### 品質チェックリスト
実装完了前に以下を確認：
- `cargo build` (Rustコンパイル)
- `cargo test` (全テスト実行)
- `cargo clippy` (リンティング)
- ドキュメントの更新

### 避けるべき実践

以下は避けましょう：
- 動作確認なしでの機能追加
- メインブランチでの直接作業
- 認証情報のハードコーディング
- 既存機能の破壊
- 重い外部依存の追加
- ドキュメント更新の怠慢

## 参考資料

### プロジェクト文書
- [開発ロードマップ](DEVELOPMENT_ROADMAP.md) - 段階的開発計画
- [プロジェクト要求仕様書](docs/prd.md) - 機能要求定義
- [アーキテクチャ仕様書](docs/ARCHITECTURE.md) - 技術設計
- [進捗レポート](PROGRESS.md) - 開発進捗状況

### 外部リソース
- [WezTerm公式ドキュメント](https://wezfurlong.org/wezterm/)
- [Lua公式ドキュメント](https://www.lua.org/docs.html)
- [Rust公式ドキュメント](https://doc.rust-lang.org/)
- [GitHub Issues](https://github.com/daktu32/wezterm-parallel/issues) - 開発進捗

## 実装ガイドライン
- 基本的な動作テストを実行
- 既存の基本機能を維持
- Unix Domain Socket IPCを維持
- 簡素で理解しやすいコードを心がける