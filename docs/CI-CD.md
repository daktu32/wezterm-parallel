# CI/CD Pipeline Documentation

## 概要

WezTerm Parallelプロジェクトでは、GitHub Actionsを使用した包括的なCI/CDパイプラインを実装しています。

## ワークフロー一覧

### 1. メインCI/CDパイプライン (`.github/workflows/ci.yml`)

**トリガー**: 
- `main`, `develop` ブランチへのpush
- `main` ブランチへのPull Request

**ジョブ**:
- **テスト**: 複数Rustバージョンでのテスト実行
- **セキュリティ監査**: cargo-auditによる脆弱性チェック
- **コードカバレッジ**: Codecovによるカバレッジレポート
- **マルチプラットフォームビルド**: Linux/Windows/macOS
- **パフォーマンステスト**: ビルド時間・バイナリサイズ測定

### 2. リリース自動化 (`.github/workflows/release.yml`)

**トリガー**: `v*.*.*` タグのpush

**機能**:
- マルチプラットフォームバイナリの自動ビルド
- GitHub Releasesへの自動アップロード
- crates.ioへの自動公開

### 3. 依存関係管理 (`.github/workflows/dependencies.yml`)

**トリガー**: 
- 毎週月曜日 18:00 JST (自動)
- 手動実行

**機能**:
- 古い依存関係の検出
- セキュリティ監査
- 自動依存関係更新PR作成

### 4. ドキュメント生成 (`.github/workflows/docs.yml`)

**トリガー**: ドキュメント関連ファイルの変更

**機能**:
- Rustドキュメント生成
- リンク検証
- スペルチェック

### 5. GitHub Pages (`.github/workflows/pages.yml`)

**トリガー**: `main` ブランチへのpush

**機能**:
- API ドキュメントのWebサイト公開
- プロジェクトドキュメントサイト構築

## 品質チェック項目

### コード品質
- ✅ `cargo fmt` - コードフォーマット
- ✅ `cargo clippy` - リンティング (警告をエラーとして扱う)
- ✅ `cargo test` - 全テスト実行 (251個)
- ✅ `cargo audit` - セキュリティ監査
- ✅ `cargo deny` - ライセンス・依存関係チェック

### カバレッジ目標
- **プロジェクト全体**: 80%以上
- **パッチカバレッジ**: 70%以上
- **コアモジュール**: 90%以上

### パフォーマンス基準
- **ビルド時間**: 初回 < 60秒、増分 < 10秒
- **テスト実行時間**: < 45秒
- **バイナリサイズ**: < 20MB (release)

## ローカル開発での品質チェック

```bash
# 必須チェック（CI前に必ず実行）
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo audit

# 追加チェック
cargo deny check
cargo doc --no-deps --all-features
```

## リリースプロセス

### 1. バージョン更新
```bash
# Cargo.tomlのバージョンを更新
vim Cargo.toml

# CHANGELOGを更新
vim CHANGELOG.md
```

### 2. タグ作成とプッシュ
```bash
git add .
git commit -m "chore: bump version to v0.3.1"
git tag v0.3.1
git push origin main --tags
```

### 3. 自動リリース
- GitHub Actionsが自動でリリースを作成
- マルチプラットフォームバイナリを生成
- crates.ioに公開

## トラブルシューティング

### CI失敗時の対処

#### テスト失敗
```bash
# ローカルでテスト実行
cargo test --verbose

# 特定のテストのみ実行
cargo test test_name --verbose
```

#### Clippy警告
```bash
# 警告の詳細確認
cargo clippy --all-targets --all-features

# 自動修正（一部）
cargo clippy --fix
```

#### セキュリティ監査失敗
```bash
# 脆弱性の詳細確認
cargo audit

# 依存関係の更新
cargo update
```

### よくある問題

1. **テストタイムアウト**
   - 統合テストの実行時間が長い場合
   - 解決: タイムアウト値の調整、テストの分割

2. **依存関係の競合**
   - Cargo.lockの競合
   - 解決: `cargo update` で解決

3. **プラットフォーム固有のエラー**
   - Windows/macOS特有の問題
   - 解決: 条件付きコンパイル、プラットフォーム固有の実装

## セキュリティ

### 設定済みセキュリティ対策
- ✅ 依存関係の脆弱性監査 (cargo-audit)
- ✅ ライセンス・依存関係検証 (cargo-deny)
- ✅ Codecovによるセキュアなカバレッジレポート
- ✅ GitHub Actionsのキャッシュセキュリティ

### シークレット管理
- `CARGO_REGISTRY_TOKEN`: crates.io公開用
- `CODECOV_TOKEN`: カバレッジレポート用

## パフォーマンス最適化

### ビルド時間短縮
- 依存関係のキャッシュ化
- 増分ビルドの活用
- 並列テスト実行

### リソース使用量
- GitHub Actions使用時間の最適化
- キャッシュの効率的な利用
- 不要なジョブの削除

## 今後の改善計画

1. **カバレッジ向上**
   - 統合テストの拡充
   - エッジケースのテスト追加

2. **パフォーマンス監視**
   - ベンチマーク自動実行
   - パフォーマンス回帰の検出

3. **セキュリティ強化**
   - SAST (Static Application Security Testing)
   - コンテナセキュリティスキャン

## 関連リンク

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust CI Best Practices](https://doc.rust-lang.org/cargo/guide/continuous-integration.html)
- [Codecov Documentation](https://docs.codecov.com/)
- [cargo-audit](https://github.com/RustSec/rustsec/tree/main/cargo-audit)
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny)