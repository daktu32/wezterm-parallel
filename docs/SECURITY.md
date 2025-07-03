# セキュリティ設計書
---
**Last Updated**: 2025-07-03
**Version**: 0.1.0
**Next Review**: 2025-10-01
---


## 概要

WezTermマルチプロセス開発補助ツールのセキュリティ設計と実装ガイドラインです。

## 1. セキュリティ原則

### 1.1 基本原則
- **最小権限の原則**: 各コンポーネントは必要最小限の権限で動作
- **多層防御**: 複数のセキュリティレイヤーで保護
- **ゼロトラスト**: すべての入力を検証
- **監査可能性**: すべての操作をログに記録

### 1.2 対象範囲
- ローカル環境での個人利用を前提
- ネットワーク経由のアクセスは想定外
- 機密情報の取り扱いは最小限

## 2. 脅威モデル

### 2.1 想定される脅威

| 脅威 | リスクレベル | 対策 |
|------|------------|------|
| 不正なプロセス起動 | 中 | プロセスホワイトリスト |
| ファイルシステムアクセス | 高 | サンドボックス化 |
| IPC経由の攻撃 | 中 | 入力検証・権限チェック |
| リソース枯渇 | 低 | リソース制限 |
| ログ改ざん | 低 | ログローテーション |

### 2.2 攻撃ベクトル
1. Unix Domain Socket経由の不正なコマンド
2. 悪意のあるテンプレートファイル
3. プロセス間通信の傍受・改ざん
4. ファイルシステムへの不正アクセス

## 3. セキュリティ実装

### 3.1 認証・認可

#### Unix Socket権限
```rust
// src/main.rs
use std::os::unix::fs::PermissionsExt;

fn create_socket() -> Result<UnixListener> {
    let socket_path = "/tmp/wezterm-parallel.sock";
    
    // 既存ソケット削除
    if Path::new(socket_path).exists() {
        std::fs::remove_file(socket_path)?;
    }
    
    let listener = UnixListener::bind(socket_path)?;
    
    // 権限を600に設定（所有者のみ読み書き可能）
    let mut perms = std::fs::metadata(socket_path)?.permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(socket_path, perms)?;
    
    Ok(listener)
}
```

#### プロセス権限分離
```rust
// src/process/security.rs
pub struct ProcessSandbox {
    allowed_paths: Vec<PathBuf>,
    max_memory: usize,
    max_cpu_percent: f32,
}

impl ProcessSandbox {
    pub fn apply(&self, command: &mut Command) {
        // 作業ディレクトリ制限
        command.current_dir(&self.allowed_paths[0]);
        
        // 環境変数のサニタイズ
        command.env_clear();
        command.env("HOME", &self.allowed_paths[0]);
        command.env("PATH", "/usr/local/bin:/usr/bin:/bin");
        
        // リソース制限（Linux）
        #[cfg(target_os = "linux")]
        {
            command.env("RLIMIT_AS", self.max_memory.to_string());
        }
    }
}
```

### 3.2 入力検証

#### メッセージ検証
```rust
// src/lib.rs
impl Message {
    pub fn validate(&self) -> Result<(), ValidationError> {
        match self {
            Message::ProcessSpawn { id, command, .. } => {
                // IDフォーマット検証
                if !id.chars().all(|c| c.is_alphanumeric() || c == '-') {
                    return Err(ValidationError::InvalidId);
                }
                
                // コマンドホワイトリスト
                const ALLOWED_COMMANDS: &[&str] = &["claude-code", "npm", "cargo"];
                if !ALLOWED_COMMANDS.contains(&command.as_str()) {
                    return Err(ValidationError::UnauthorizedCommand);
                }
            }
            Message::WorkspaceCreate { name, .. } => {
                // ディレクトリトラバーサル防止
                if name.contains("..") || name.contains("/") {
                    return Err(ValidationError::InvalidWorkspaceName);
                }
            }
            _ => {}
        }
        Ok(())
    }
}
```

#### テンプレート検証
```lua
-- lua/room/template_validator.lua
local validator = {}

function validator.validate_template(template)
    -- YAMLインジェクション防止
    if template:match("[;&|`$]") then
        return false, "Invalid characters in template"
    end
    
    -- パス検証
    for _, pane in ipairs(template.panes or {}) do
        if pane.working_dir and pane.working_dir:match("%.%.") then
            return false, "Directory traversal detected"
        end
    end
    
    -- コマンド検証
    local allowed_commands = {
        ["claude-code"] = true,
        ["npm"] = true,
        ["cargo"] = true,
        ["python"] = true
    }
    
    for _, pane in ipairs(template.panes or {}) do
        local cmd = pane.command:match("^(%S+)")
        if not allowed_commands[cmd] then
            return false, "Unauthorized command: " .. cmd
        end
    end
    
    return true
end

return validator
```

### 3.3 データ保護

#### ファイル暗号化（将来実装）
```rust
// src/security/encryption.rs
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

pub struct FileEncryption {
    cipher: Aes256Gcm,
}

impl FileEncryption {
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        Self { cipher }
    }
    
    pub fn encrypt_file(&self, path: &Path) -> Result<()> {
        let plaintext = std::fs::read(path)?;
        let nonce = generate_nonce();
        
        let ciphertext = self.cipher.encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| Error::Encryption(e))?;
            
        // 暗号化ファイル保存
        let encrypted_path = path.with_extension("enc");
        std::fs::write(encrypted_path, ciphertext)?;
        
        // 元ファイル安全削除
        secure_delete(path)?;
        
        Ok(())
    }
}
```

#### セキュアな一時ファイル
```rust
// src/security/tempfile.rs
use tempfile::{NamedTempFile, TempDir};

pub fn create_secure_tempfile() -> Result<NamedTempFile> {
    let file = NamedTempFile::new_in("/tmp")?;
    
    // 権限を600に設定
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = file.as_file().metadata()?.permissions();
        perms.set_mode(0o600);
        file.as_file().set_permissions(perms)?;
    }
    
    Ok(file)
}
```

### 3.4 監査・ログ

#### セキュリティイベントログ
```rust
// src/security/audit.rs
#[derive(Debug, Serialize)]
pub struct SecurityEvent {
    timestamp: u64,
    event_type: SecurityEventType,
    user: String,
    source_ip: Option<String>,
    details: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub enum SecurityEventType {
    Authentication,
    Authorization,
    ProcessSpawn,
    FileAccess,
    ConfigChange,
    SecurityViolation,
}

pub struct AuditLogger {
    writer: Arc<Mutex<File>>,
}

impl AuditLogger {
    pub fn log_event(&self, event: SecurityEvent) {
        let mut writer = self.writer.lock().unwrap();
        let json = serde_json::to_string(&event).unwrap();
        writeln!(writer, "{}", json).unwrap();
        writer.sync_all().unwrap();
    }
}
```

## 4. セキュアコーディング

### 4.1 Rustセキュリティ

```rust
// メモリ安全性
// - 所有権システムによる自動的なメモリ管理
// - 借用チェッカーによるデータ競合防止

// バッファオーバーフロー防止
fn safe_string_copy(input: &str, max_len: usize) -> String {
    input.chars().take(max_len).collect()
}

// SQLインジェクション防止（将来のDB実装）
use sqlx::query;
let result = query!("SELECT * FROM tasks WHERE id = ?", task_id)
    .fetch_one(&pool)
    .await?;

// 整数オーバーフロー防止
let result = a.checked_add(b).ok_or(Error::IntegerOverflow)?;
```

### 4.2 エラーハンドリング

```rust
// センシティブ情報を含まないエラー
#[derive(Debug, thiserror::Error)]
pub enum PublicError {
    #[error("Invalid request")]
    InvalidRequest,
    
    #[error("Resource not found")]
    NotFound,
    
    #[error("Operation not permitted")]
    Forbidden,
}

// 内部エラーから公開エラーへの変換
impl From<InternalError> for PublicError {
    fn from(err: InternalError) -> Self {
        // ログに詳細を記録
        error!("Internal error: {:?}", err);
        
        // ユーザーには一般的なエラーを返す
        match err {
            InternalError::Database(_) => PublicError::InvalidRequest,
            InternalError::FileSystem(_) => PublicError::NotFound,
            _ => PublicError::InvalidRequest,
        }
    }
}
```

## 5. 依存関係セキュリティ

### 5.1 依存関係監査

```bash
# cargo-auditによる脆弱性チェック
cargo install cargo-audit
cargo audit

# 依存関係の更新
cargo update
cargo tree --duplicates
```

### 5.2 最小依存の原則

```toml
# Cargo.toml
[dependencies]
# 必要最小限の機能のみ有効化
tokio = { version = "1", features = ["rt-multi-thread", "net", "sync"] }
serde = { version = "1", features = ["derive"] }

# セキュリティ関連の依存関係
sodiumoxide = "0.2"  # 暗号化
zeroize = "1.5"      # メモリクリア
```

## 6. セキュリティテスト

### 6.1 ペネトレーションテスト

```rust
#[cfg(test)]
mod security_tests {
    #[test]
    fn test_command_injection() {
        let malicious_inputs = vec![
            "claude-code; rm -rf /",
            "claude-code && cat /etc/passwd",
            "claude-code`whoami`",
            "../../../etc/passwd",
        ];
        
        for input in malicious_inputs {
            let result = validate_command(input);
            assert!(result.is_err(), "Failed to block: {}", input);
        }
    }
    
    #[test]
    fn test_path_traversal() {
        let paths = vec![
            "../secret.txt",
            "/etc/passwd",
            "~/.ssh/id_rsa",
        ];
        
        for path in paths {
            let result = validate_room_path(path);
            assert!(result.is_err());
        }
    }
}
```

### 6.2 ファジング

```rust
// fuzz/fuzz_targets/message_parsing.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = serde_json::from_str::<Message>(s);
    }
});
```

## 7. インシデント対応

### 7.1 セキュリティインシデント検知

```rust
// src/security/detection.rs
pub struct AnomalyDetector {
    baseline: MetricsBaseline,
    threshold: f64,
}

impl AnomalyDetector {
    pub fn check_anomaly(&self, current: &SystemMetrics) -> Option<SecurityAlert> {
        // 異常なプロセス起動頻度
        if current.process_spawn_rate > self.baseline.avg_spawn_rate * 2.0 {
            return Some(SecurityAlert::AbnormalProcessSpawn);
        }
        
        // 異常なファイルアクセス
        if current.file_access_rate > self.threshold {
            return Some(SecurityAlert::ExcessiveFileAccess);
        }
        
        None
    }
}
```

### 7.2 自動対応

```rust
pub async fn handle_security_alert(alert: SecurityAlert) {
    match alert {
        SecurityAlert::AbnormalProcessSpawn => {
            // 新規プロセス起動を一時的に無効化
            PROCESS_SPAWN_ENABLED.store(false, Ordering::SeqCst);
            
            // アラート通知
            notify_admin("Abnormal process spawn detected");
            
            // 30秒後に自動復旧
            tokio::time::sleep(Duration::from_secs(30)).await;
            PROCESS_SPAWN_ENABLED.store(true, Ordering::SeqCst);
        }
        _ => {}
    }
}
```

## 8. セキュリティチェックリスト

### 開発時
- [ ] 入力検証の実装
- [ ] エラーメッセージの確認（機密情報を含まない）
- [ ] 権限チェックの実装
- [ ] セキュアなデフォルト設定

### デプロイ時
- [ ] ファイル権限の確認
- [ ] 不要なポートが開いていないか
- [ ] ログローテーションの設定
- [ ] 依存関係の脆弱性チェック

### 運用時
- [ ] セキュリティパッチの適用
- [ ] ログの定期的な監査
- [ ] 異常検知アラートの確認
- [ ] バックアップの暗号化
## 関連ドキュメント
- [プロジェクト概要](../README.md)
- [ドキュメント体系](DOCUMENTATION-MAP.md)
- [アーキテクチャ](ARCHITECTURE.md)
- [貢献ガイド](CONTRIBUTING.md)
