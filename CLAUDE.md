# CLAUDE.md

WezTerm マルチプロセス開発補助ツールでのClaude Code作業ガイド

## 🚀 プロジェクト概要

**WezTerm マルチプロセス開発補助ツール** - WezTermでClaude Codeを複数プロセス実行するための実験的なツールです。

- **技術スタック**: Rust (19,335行) + WezTerm/Lua (7,175行)
- **アーキテクチャ**: Unix Domain Socket IPC、WebSocketダッシュボード
- **実装状況**: MVP機能完了 (Issue #17, #18)、127個のライブラリテスト通過

## 📋 重要な開発ルール

### 1. ドキュメント管理（最優先）

**必読**: [docs/DOCUMENTATION-MAP.md](docs/DOCUMENTATION-MAP.md) のメンテナンスガイドに従う

#### Single Source of Truth原則
```yaml
情報の管理責任:
  実装規模: PROGRESS.md
  テスト状況: TESTING.md  
  機能仕様: FEATURE-SPEC.md
  API仕様: API.md
```

#### 必須更新タイミング
- 機能実装完了時
- フェーズ移行時
- バグ修正・改善後

### 2. 品質管理チェック

#### 実装前チェック
- [ ] FEATURE-SPEC.mdに機能ID付きで仕様記載
- [ ] テストケース設計
- [ ] 既存機能への影響評価

#### コミット前必須チェック
```bash
cargo clippy -- -D warnings  # 警告をエラーとして扱う
cargo test                    # 全テスト実行
cargo fmt --check            # フォーマットチェック
cargo build --release        # リリースビルド確認
```

### 3. 開発ワークフロー

1. **テストファースト開発**: 実装前にテストを作成
2. **ブランチ戦略**: `feature/*` or `fix/*` ブランチで作業
3. **進捗追跡**: PROGRESS.md、DEVELOPMENT_ROADMAP.mdを更新
4. **GitHub Issues**: [#8-18](https://github.com/daktu32/wezterm-parallel/issues) で進捗管理

## 📊 実装状況

### ✅ 完了済み (MVP)
- Claude Code複数プロセス協調システム
- WezTermペイン分割・レイヤウトテンプレート
- Unix Socket経由のIPC通信
- WebSocketダッシュボード・メトリクス収集

### 🔧 開発優先順位
```yaml
最優先: MVP機能の改善・バグ修正
高優先: コア機能の安定化
中優先: 拡張機能の改善
低優先: 新機能の追加
```

## 🛠️ 基本コマンド

```bash
# 開発用コマンド
cargo build          # ビルド
cargo test           # テスト実行 (127個)
cargo run            # フレームワーク起動
cargo doc --open     # ドキュメント生成

# 品質チェック
cargo clippy         # 静的解析
cargo fmt            # フォーマット
```

## 🚨 避けるべき行為

- ❌ 未確認の数値データを記載
- ❌ テスト結果の改ざん・誇張
- ❌ 実装していない機能を「完了」とマーク
- ❌ ドキュメント間の矛盾を放置
- ❌ メインブランチでの直接作業

## 📚 参考資料

### プロジェクト文書
- [DOCUMENTATION-MAP.md](docs/DOCUMENTATION-MAP.md) - ドキュメント体系
- [FEATURE-SPEC.md](docs/FEATURE-SPEC.md) - 機能仕様
- [PROGRESS.md](PROGRESS.md) - 開発進捗
- [TESTING.md](docs/TESTING.md) - テスト指針

### 外部リソース
- [WezTerm公式](https://wezfurlong.org/wezterm/)
- [Rust公式](https://doc.rust-lang.org/)
- [GitHub Issues](https://github.com/daktu32/wezterm-parallel/issues)

## 🎯 不明点の解決

質問がある場合は、上記の参考資料を確認するか、該当するIssueで議論してください。

---

**重要**: このファイルはClaude Code作業時の最重要ガイドです。すべての開発作業前に必ず確認してください。