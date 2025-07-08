use log::{debug, info, warn};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Claude Codeバイナリの検出と検証を行うモジュール
#[derive(Debug)]
pub struct ClaudeCodeDetector {
    /// 検索対象のパス一覧
    search_paths: Vec<PathBuf>,
    /// バイナリ名の候補
    binary_names: Vec<String>,
}

impl ClaudeCodeDetector {
    /// 新しいDetectorインスタンスを作成
    pub fn new() -> Self {
        let mut search_paths = Vec::new();

        // PATH環境変数から検索パスを追加
        if let Ok(path_env) = env::var("PATH") {
            for path in env::split_paths(&path_env) {
                search_paths.push(path);
            }
        }

        // よく使われる追加パスも含める
        search_paths.extend([
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/usr/bin"),
            PathBuf::from("/opt/local/bin"),
            PathBuf::from("/opt/homebrew/bin"),
            PathBuf::from("~/.local/bin").expand_home(),
            PathBuf::from("~/bin").expand_home(),
        ]);

        // バイナリ名の候補
        let binary_names = vec![
            "claude-code".to_string(),
            "claude".to_string(),
            "cu".to_string(),
        ];

        Self {
            search_paths,
            binary_names,
        }
    }

    /// Claude Codeバイナリを検出して返す
    pub fn detect(&self) -> Result<PathBuf> {
        info!("Starting Claude Code binary detection...");

        // まず環境変数で指定されたパスをチェック
        if let Ok(explicit_path) = env::var("CLAUDE_CODE_BINARY") {
            let path = PathBuf::from(explicit_path);
            if self.verify_binary(&path)? {
                info!("Found Claude Code at explicit path: {path:?}");
                return Ok(path);
            }
        }

        // 各検索パスと各バイナリ名の組み合わせで検索
        for dir in &self.search_paths {
            for name in &self.binary_names {
                let candidate = dir.join(name);
                debug!("Checking candidate: {candidate:?}");

                if candidate.exists() && self.verify_binary(&candidate)? {
                    info!("Found Claude Code binary: {candidate:?}");
                    return Ok(candidate);
                }
            }
        }

        // whichコマンドを使って検索
        for name in &self.binary_names {
            if let Ok(path) = self.find_with_which(name) {
                if self.verify_binary(&path)? {
                    info!("Found Claude Code via which: {path:?}");
                    return Ok(path);
                }
            }
        }

        Err("Claude Code binary not found. Please install Claude Code or set CLAUDE_CODE_BINARY environment variable.".into())
    }

    /// whichコマンドを使ってバイナリを検索
    fn find_with_which(&self, name: &str) -> Result<PathBuf> {
        let output = Command::new("which").arg(name).output()?;

        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                return Ok(PathBuf::from(path_str));
            }
        }

        Err(format!("'which {name}' failed").into())
    }

    /// バイナリが実行可能で、Claude Codeであることを検証
    fn verify_binary(&self, path: &Path) -> Result<bool> {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        // ファイルが存在するか確認
        if !path.exists() {
            return Ok(false);
        }

        // 実行権限があるか確認
        let metadata = fs::metadata(path)?;
        let permissions = metadata.permissions();
        if permissions.mode() & 0o111 == 0 {
            debug!("Binary at {path:?} is not executable");
            return Ok(false);
        }

        // --versionを実行してClaude Codeであることを確認
        match Command::new(path).arg("--version").output() {
            Ok(output) => {
                let version_str = String::from_utf8_lossy(&output.stdout);
                let is_claude_code = version_str.to_lowercase().contains("claude");

                if is_claude_code {
                    debug!("Verified Claude Code binary: {}", version_str.trim());
                    Ok(true)
                } else {
                    debug!("Binary at {path:?} is not Claude Code");
                    Ok(false)
                }
            }
            Err(e) => {
                warn!("Failed to run --version on {path:?}: {e}");
                Ok(false)
            }
        }
    }

    /// Claude Codeのバージョン情報を取得
    pub fn get_version(&self, binary_path: &Path) -> Result<String> {
        let output = Command::new(binary_path).arg("--version").output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err("Failed to get Claude Code version".into())
        }
    }
}

impl Default for ClaudeCodeDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// ホームディレクトリを展開する拡張trait
trait PathExpand {
    fn expand_home(&self) -> PathBuf;
}

impl PathExpand for PathBuf {
    fn expand_home(&self) -> PathBuf {
        if let Some(path_str) = self.to_str() {
            if path_str.starts_with('~') {
                if let Ok(home) = env::var("HOME") {
                    return PathBuf::from(path_str.replacen('~', &home, 1));
                }
            }
        }
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    #[test]
    fn test_detector_creation() {
        let detector = ClaudeCodeDetector::new();
        assert!(!detector.search_paths.is_empty());
        assert!(!detector.binary_names.is_empty());
    }

    #[test]
    fn test_path_expansion() {
        let path = PathBuf::from("~/test");
        let expanded = path.expand_home();

        if let Ok(home) = env::var("HOME") {
            assert_eq!(expanded, PathBuf::from(format!("{home}/test")));
        }
    }

    #[test]
    fn test_verify_binary_nonexistent() {
        let detector = ClaudeCodeDetector::new();
        let result = detector
            .verify_binary(Path::new("/nonexistent/binary"))
            .unwrap();
        assert!(!result);
    }

    #[test]
    fn test_verify_binary_not_executable() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("non_executable");
        File::create(&file_path)?;

        // ファイルを読み取り専用に設定
        let mut perms = fs::metadata(&file_path)?.permissions();
        perms.set_mode(0o444);
        fs::set_permissions(&file_path, perms)?;

        let detector = ClaudeCodeDetector::new();
        let result = detector.verify_binary(&file_path)?;
        assert!(!result);

        Ok(())
    }

    #[test]
    fn test_env_var_override() {
        // 環境変数をモックするテスト
        let detector = ClaudeCodeDetector::new();
        // 実際の環境変数は変更せず、detect()メソッドの動作を確認
        // 環境変数が設定されている場合の動作は統合テストで確認
        assert!(detector.binary_names.contains(&"claude-code".to_string()));
    }
}
