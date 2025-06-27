# コントリビューションガイド

## 概要

WezTermマルチプロセス開発補助ツールへの貢献を歓迎します！このガイドでは、プロジェクトへの貢献方法を説明します。

## 1. 貢献の方法

### 1.1 貢献の種類
- 🐛 **バグ報告**: 問題を発見したら報告してください
- ✨ **機能提案**: 新しいアイデアを共有してください
- 📝 **ドキュメント改善**: 誤字修正から新規ガイド作成まで
- 🔧 **コード貢献**: バグ修正や新機能の実装
- 🧪 **テスト追加**: テストカバレッジの向上
- 🌐 **翻訳**: ドキュメントの多言語化

### 1.2 はじめる前に
1. [Code of Conduct](CODE_OF_CONDUCT.md)を読む
2. 既存のIssueを確認する
3. 開発環境をセットアップする

## 2. 開発環境のセットアップ

### 2.1 必要なツール
```bash
# Rust（最新安定版）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 開発ツール
cargo install cargo-watch
cargo install cargo-tarpaulin  # カバレッジ
cargo install cargo-audit      # セキュリティ監査

# WezTerm
# https://wezfurlong.org/wezterm/installation.html
```

### 2.2 プロジェクトのクローン
```bash
git clone https://github.com/daktu32/wezterm-parallel.git
cd wezterm-parallel
```

### 2.3 ビルドとテスト
```bash
# ビルド
cargo build

# テスト実行
cargo test

# 開発モードで実行
cargo run
```

## 3. 開発ワークフロー

### 3.1 ブランチ戦略
```bash
# 機能開発
git checkout -b feature/your-feature-name

# バグ修正
git checkout -b fix/issue-description

# ドキュメント
git checkout -b docs/what-you-update
```

### 3.2 コミットメッセージ
```
<type>: <description>

[optional body]

[optional footer]
```

**タイプ**:
- `feat`: 新機能
- `fix`: バグ修正
- `docs`: ドキュメント
- `style`: フォーマット変更
- `refactor`: リファクタリング
- `test`: テスト追加・修正
- `chore`: ビルドプロセスやツールの変更

**例**:
```
feat: Add task priority filtering to ProcessCoordinator

Implement priority-based task filtering in the coordination system
to improve task distribution efficiency.

Closes #123
```

### 3.3 プルリクエスト

1. **変更前にIssueを作成**
   - 大きな変更の場合は事前に議論
   - 実装方針の合意を得る

2. **PRテンプレートに従う**
   ```markdown
   ## 概要
   変更の簡潔な説明

   ## 変更内容
   - [ ] 具体的な変更点1
   - [ ] 具体的な変更点2

   ## テスト
   - [ ] ユニットテスト追加
   - [ ] 統合テスト追加
   - [ ] 手動テスト実施

   ## 関連Issue
   Closes #123
   ```

## 4. コーディング規約

### 4.1 Rustコード

**命名規則**:
```rust
// 構造体: PascalCase
struct ProcessManager {}

// 関数・メソッド: snake_case
fn spawn_process() {}

// 定数: SCREAMING_SNAKE_CASE
const MAX_PROCESSES: usize = 16;

// モジュール: snake_case
mod process_manager;
```

**コードスタイル**:
```rust
// rustfmtの設定に従う
cargo fmt

// clippyの警告を修正
cargo clippy -- -D warnings
```

**エラーハンドリング**:
```rust
// Result型を使用
pub fn risky_operation() -> Result<String, Error> {
    // エラーコンテキストを追加
    do_something()
        .map_err(|e| Error::OperationFailed(e.to_string()))?;
    
    Ok("success".to_string())
}

// カスタムエラー型
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
```

### 4.2 Luaコード

**スタイルガイド**:
```lua
-- モジュール定義
local M = {}

-- ローカル関数は先頭に
local function helper_function(param)
    -- インデントは2スペース
    return param * 2
end

-- パブリック関数
function M.public_function(config)
    -- 早期リターン
    if not config then
        return nil, "Config is required"
    end
    
    -- 明確な変数名
    local result = helper_function(config.value)
    return result
end

return M
```

### 4.3 ドキュメント

**Rustドキュメント**:
```rust
/// プロセスを管理する構造体
/// 
/// # Examples
/// 
/// ```
/// let manager = ProcessManager::new();
/// let process_id = manager.spawn_process("test", "command", None)?;
/// ```
pub struct ProcessManager {
    /// アクティブなプロセスのマップ
    processes: HashMap<String, Process>,
}
```

**README更新**:
- 新機能は必ずREADMEに追加
- 使用例を含める
- 設定オプションを文書化

**ドキュメント体系の維持**:
- [DOCUMENTATION-MAP.md](DOCUMENTATION-MAP.md)のメンテナンスガイドに従う
- Single Source of Truthの原則を守る
- 更新時は関連ドキュメントも同時更新
- 実装状況は正確に記載（過大評価禁止）

## 5. テスト

### 5.1 テスト作成

**必須テスト**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_path() {
        // 正常系のテスト
    }

    #[test]
    fn test_error_cases() {
        // エラーケースのテスト
    }

    #[test]
    fn test_edge_cases() {
        // エッジケースのテスト
    }
}
```

**テストカバレッジ**:
- 新機能: 90%以上
- バグ修正: 修正箇所の100%

### 5.2 テスト実行
```bash
# 全テスト
cargo test

# 特定のテスト
cargo test test_process_coordination

# カバレッジ確認
cargo tarpaulin --out Html
```

## 6. レビュープロセス

### 6.1 レビュー基準
- [ ] コードが規約に従っている
- [ ] テストが十分に書かれている
- [ ] ドキュメントが更新されている
- [ ] パフォーマンスへの影響を考慮
- [ ] セキュリティの観点から問題ない

### 6.2 レビューへの対応
- レビューコメントには建設的に対応
- 変更理由を明確に説明
- 必要に応じて追加のコミット

## 7. リリースプロセス

### 7.1 バージョニング
セマンティックバージョニング（SemVer）に従う：
- MAJOR: 破壊的変更
- MINOR: 後方互換性のある機能追加
- PATCH: バグ修正

### 7.2 リリースノート
```markdown
## [0.2.0] - 2025-06-27

### Added
- Claude Code複数プロセス協調システム (#17)
- WezTermペイン分割テンプレート機能 (#18)

### Fixed
- プロセス再起動時のメモリリーク (#45)

### Changed
- テスト数を108から127に増加
```

## 8. コミュニティ

### 8.1 コミュニケーション
- **GitHub Issues**: バグ報告・機能提案
- **GitHub Discussions**: 一般的な議論
- **Pull Requests**: コード貢献

### 8.2 行動規範
- 建設的なフィードバック
- 相互尊重
- オープンな議論
- 初心者に優しい環境

## 9. ライセンス

貢献されたコードは、プロジェクトと同じMITライセンスの下でリリースされます。

## 10. 謝辞

すべての貢献者に感謝します！あなたの貢献がこのプロジェクトを成長させます。

---

質問がある場合は、Issueを作成してお気軽にお問い合わせください。