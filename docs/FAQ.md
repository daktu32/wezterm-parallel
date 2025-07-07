# 🙋 よくある質問 (FAQ)

**WezTerm Parallelに関するよくある質問と回答集**

## 📋 基本的な質問

### Q1: WezTerm Parallelとは何ですか？
**A**: WezTerm Parallelは、WezTermでClaude Codeを複数プロセス実行するためのマルチプロセス管理ツールです。個人開発者の生産性向上を目的とした実験的なフレームワークです。

### Q2: どのような人が使うべきですか？
**A**: 以下のような方におすすめです：
- Claude Codeを日常的に使用する開発者
- 複数のプロジェクトを並行して進める方
- WezTermを主要なターミナルとして使用している方
- 開発環境の自動化・効率化に興味がある方

### Q3: 商用利用は可能ですか？
**A**: 現在、WezTerm Parallelは**個人利用・実験用途のみ**を想定しています。商用利用やチーム利用については将来の検討課題です。

## 🔧 インストール・セットアップ

### Q4: インストールに失敗します
**A**: 以下を順番に確認してください：

1. **システム要件の確認**：
   ```bash
   # WezTerm確認
   wezterm --version  # 20240203-110809-5046fc22以降
   
   # Rust確認  
   rustc --version    # 1.70.0以降
   ```

2. **自動セットアップスクリプトの使用**：
   ```bash
   git clone https://github.com/daktu32/wezterm-parallel.git
   cd wezterm-parallel
   ./setup.sh
   ```

3. **手動ビルド**：
   ```bash
   cargo clean
   cargo build --release
   ```

4. **詳細ログで原因確認**：
   ```bash
   RUST_LOG=debug cargo build --release
   ```

### Q5: 設定ファイルが見つからない
**A**: 設定ファイルは以下の順序で検索されます：

```bash
# 1. カレントディレクトリ
./wezterm-parallel.yaml

# 2. ユーザー設定ディレクトリ
~/.config/wezterm-parallel/config.yaml

# 3. デフォルト設定を生成
mkdir -p ~/.config/wezterm-parallel
cp config/quickstart-config.yaml ~/.config/wezterm-parallel/config.yaml
```

### Q6: Claude Codeが必要ですか？
**A**: **Claude Codeは必須ではありません**。基本的なプロセス管理・ワークスペース管理・ダッシュボード機能はClaude Codeなしでも動作します。Claude Codeがある場合、より高度な統合機能が利用できます。

## 🚀 使用方法

### Q7: ワークスペースとは何ですか？
**A**: ワークスペースは、関連するプロセス群とペイン構成をまとめた作業単位です：

```bash
# ワークスペース作成
curl -X POST http://localhost:8080/api/workspaces \
  -H "Content-Type: application/json" \
  -d '{"name": "my-project", "template": "basic"}'

# ワークスペース一覧
curl http://localhost:8080/api/workspaces | jq .

# ワークスペース削除
curl -X DELETE http://localhost:8080/api/workspaces/my-project
```

### Q8: テンプレートはどのように使いますか？
**A**: テンプレートは事前定義されたワークスペース構成です：

**内蔵テンプレート**：
- `basic`: 最小構成
- `claude-dev`: Claude Code統合開発
- `web-stack`: ウェブ開発スタック
- `microservices`: マイクロサービス開発

**カスタムテンプレート作成**：
```yaml
# ~/.config/wezterm-parallel/templates/my-template.yaml
name: "My Custom Template"
layout:
  type: "grid"
  panes:
    - id: "editor"
      command: "nvim"
    - id: "terminal"  
      command: "bash"
```

### Q9: キーバインドが効きません
**A**: WezTerm設定の確認：

```bash
# 1. 設定ファイル確認
ls -la ~/.config/wezterm/wezterm.lua

# 2. クイックスタート設定適用
cp config/quickstart-wezterm.lua ~/.config/wezterm/wezterm.lua

# 3. WezTerm設定リロード（WezTerm内で）
# Ctrl+Shift+R

# 4. 設定構文チェック
wezterm show-config
```

**主要キーバインド**：
- `Ctrl+Shift+N`: 新しいワークスペース作成
- `Ctrl+Shift+D`: ダッシュボードを開く
- `Ctrl+Alt+S`: フレームワーク状態確認

## 🔧 トラブルシューティング

### Q10: ポート8080/8081が使用中エラー
**A**: ポート競合の解決：

```bash
# 1. 使用中プロセス確認
sudo lsof -i :8080 -i :8081

# 2. プロセス停止
kill -9 <PID>

# 3. または設定でポート変更
# ~/.config/wezterm-parallel/config.yaml:
server:
  port: 9080
  websocket_port: 9081
```

### Q11: ダッシュボードに接続できません
**A**: 接続問題の診断：

```bash
# 1. サービス状態確認
curl http://localhost:8080/api/status

# 2. ネットワーク確認
netstat -tlnp | grep -E "(8080|8081)"

# 3. ファイアウォール確認（Linux）
sudo ufw status

# 4. ブラウザで直接アクセス
# http://localhost:8081
# http://127.0.0.1:8081
```

### Q12: プロセスが起動しません
**A**: プロセス起動問題の診断：

```bash
# 1. ログ確認
tail -f ~/.config/wezterm-parallel/logs/application.log

# 2. プロセス別ログ確認
tail -f ~/.config/wezterm-parallel/logs/processes/workspace-name/process-name.log

# 3. 権限確認
ls -la $(which claude-code)

# 4. 環境変数確認
env | grep PATH
```

### Q13: システムが重い・遅い
**A**: パフォーマンス最適化：

```bash
# 1. リソース使用量確認
curl -s http://localhost:8080/api/system/resources | jq .

# 2. プロセス数制限
# config.yaml:
process_management:
  max_processes_per_workspace: 4

# 3. ログレベル調整
logging:
  level: "warn"  # debug から変更

# 4. ダッシュボード更新頻度調整
dashboard:
  update_interval: 10000  # 10秒間隔
```

## 🔐 セキュリティ

### Q14: セキュリティ上の注意点はありますか？
**A**: ローカル開発用として設計されているため、以下に注意：

- **APIキー認証**: デフォルトでは無効（ローカル開発用）
- **CORS設定**: デフォルトで`localhost`のみ許可
- **ネットワーク接続**: `127.0.0.1`のみでリスニング
- **ログファイル**: プレーンテキストで保存

**本番環境での使用は推奨されません**。

### Q15: データはどこに保存されますか？
**A**: 以下の場所にデータを保存：

```bash
# 設定ファイル
~/.config/wezterm-parallel/

# データファイル
~/.local/share/wezterm-parallel/

# ログファイル  
~/.config/wezterm-parallel/logs/

# バックアップ（設定により）
~/.local/share/wezterm-parallel/backups/
```

## 💡 機能・制限

### Q16: 同時に実行できるプロセス数の制限は？
**A**: 設定により調整可能：

```yaml
# デフォルト設定
process_management:
  max_processes_per_workspace: 8
  max_total_processes: 32

# 推奨設定（システム性能に応じて）
process_management:
  max_processes_per_workspace: 4  # 軽量
  max_processes_per_workspace: 12 # 高性能
```

### Q17: バックアップ機能はありますか？
**A**: はい、自動バックアップ機能があります：

```yaml
# 設定例
backup:
  enabled: true
  interval: 3600          # 1時間間隔
  max_backups: 24         # 24個まで保持
  include:
    - workspaces          # ワークスペース状態
    - templates           # カスタムテンプレート
    - preferences         # 個人設定
```

### Q18: 他のターミナルでも使えますか？
**A**: 現在は**WezTerm専用**です。ただし、以下の機能はWezTerm以外でも利用可能：

- REST API
- WebSocketダッシュボード
- プロセス管理機能
- ワークスペース管理

将来的に他ターミナルのサポートも検討中です。

## 🛠️ 開発・カスタマイズ

### Q19: カスタムテンプレートの作り方は？
**A**: YAMLファイルでカスタムテンプレートを作成：

```yaml
# ~/.config/wezterm-parallel/templates/my-template.yaml
name: "My Development Setup"
description: "個人用開発環境"
version: "1.0"

layout:
  type: "grid"
  panes:
    - id: "editor"
      position: { row: 0, col: 0, width: 0.6, height: 0.8 }
      command: "nvim"
    - id: "terminal"
      position: { row: 0, col: 1, width: 0.4, height: 0.8 }
      command: "bash"

processes:
  - name: "dev-server"
    command: "npm run dev"
    auto_restart: true
```

### Q20: プラグインは作れますか？
**A**: プラグインシステムは開発中（Issue #37）です。現在は以下で拡張可能：

- カスタムテンプレート
- 設定ファイルカスタマイズ
- 外部スクリプト統合
- API経由での機能拡張

## 📊 パフォーマンス

### Q21: 推奨システム要件は？
**A**: 
**最小要件**：
- OS: Linux (Ubuntu 20.04+), macOS (11.0+), Windows (WSL2)
- RAM: 512MB
- CPU: 2コア
- ディスク: 100MB

**推奨要件**：
- RAM: 1GB以上
- CPU: 4コア以上
- SSD: 1GB以上

### Q22: メモリ使用量を削減するには？
**A**: 以下の設定で最適化：

```yaml
# 軽量設定
process_management:
  max_processes_per_workspace: 2
  memory_limits:
    per_process_mb: 256

logging:
  level: "warn"
  console_enabled: false

dashboard:
  update_interval: 10000
  
claude_code:
  auto_start: false
```

## 🔄 アップデート・メンテナンス

### Q23: アップデート方法は？
**A**: 
```bash
# 1. 最新版取得
git pull origin main

# 2. 再ビルド
cargo build --release

# 3. 設定互換性確認
./target/release/wezterm-parallel --check-config

# 4. 自動セットアップ（必要に応じて）
./setup.sh
```

### Q24: 設定ファイルの移行は必要ですか？
**A**: 通常は自動で互換性が保たれますが、大きな変更時は移行が必要な場合があります：

```bash
# 設定バックアップ
cp ~/.config/wezterm-parallel/config.yaml ~/config-backup.yaml

# 設定検証
wezterm-parallel --validate-config

# 問題がある場合
wezterm-parallel --migrate-config
```

## 🆘 サポート

### Q25: バグを見つけたらどうすればいいですか？
**A**: [GitHub Issues](https://github.com/daktu32/wezterm-parallel/issues)で報告してください。以下の情報を含めると助かります：

```bash
# システム情報
uname -a
rustc --version
wezterm --version

# エラーログ
RUST_LOG=debug wezterm-parallel 2>&1

# 設定ファイル
cat ~/.config/wezterm-parallel/config.yaml
```

### Q26: 機能要求はどこで？
**A**: [GitHub Discussions](https://github.com/daktu32/wezterm-parallel/discussions)で機能要求や提案を投稿してください。

### Q27: 貢献したいのですが？
**A**: 大歓迎です！詳細は[CONTRIBUTING.md](CONTRIBUTING.md)をご覧ください。

- バグ修正
- 機能追加
- ドキュメント改善
- テスト追加
- 翻訳

---

## 📚 関連ドキュメント

- [クイックスタートガイド](../QUICKSTART.md)
- [詳細セットアップガイド](../SETUP-GUIDE.md)
- [ユーザーガイド](USER-GUIDE.md)
- [トラブルシューティング](QUICKSTART-TROUBLESHOOTING.md)

---

❓ **その他の質問がある場合**は、[GitHub Discussions](https://github.com/daktu32/wezterm-parallel/discussions)でお気軽にお聞きください！