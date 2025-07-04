use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use uuid::Uuid;
use anyhow::{Result, anyhow};
use super::file_sync::ConflictResolution;

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictType {
    ContentConflict,
    StructuralConflict,
    MetadataConflict,
}

#[derive(Debug, Clone)]
pub struct ConflictInfo {
    pub conflict_type: ConflictType,
    pub file_path: PathBuf,
    pub base_content: String,
    pub version1_content: String,
    pub version2_content: String,
    pub version1_process: Uuid,
    pub version2_process: Uuid,
    pub detected_at: SystemTime,
}

#[derive(Debug, Clone)]
pub enum MergeResult {
    Success(String),
    Conflict(ConflictInfo),
}

pub struct MergeManager {
    conflict_resolution_strategy: ConflictResolution,
    process_priorities: HashMap<Uuid, u8>,
    merge_patterns: Vec<MergePattern>,
    auto_merge_enabled: bool,
}

#[derive(Debug, Clone)]
struct MergePattern {
    file_extension: String,
    merge_strategy: MergeStrategy,
}

#[derive(Debug, Clone)]
enum MergeStrategy {
    LineByLine,
    BlockBased,
    StructuralMerge,
    NoMerge,
}

impl MergeManager {
    pub fn new() -> Self {
        let mut manager = Self {
            conflict_resolution_strategy: ConflictResolution::PreferLatest,
            process_priorities: HashMap::new(),
            merge_patterns: Vec::new(),
            auto_merge_enabled: true,
        };
        
        // デフォルトのマージパターンを設定
        manager.setup_default_patterns();
        manager
    }
    
    pub fn merge_content(
        &self,
        file_path: &PathBuf,
        base_content: &str,
        version1: &str,
        version2: &str,
    ) -> Result<MergeResult> {
        // ファイル拡張子に基づいてマージ戦略を決定
        let merge_strategy = self.get_merge_strategy(file_path);
        
        match merge_strategy {
            MergeStrategy::LineByLine => {
                self.merge_line_by_line(file_path, base_content, version1, version2)
            }
            MergeStrategy::BlockBased => {
                self.merge_block_based(file_path, base_content, version1, version2)
            }
            MergeStrategy::StructuralMerge => {
                self.merge_structural(file_path, base_content, version1, version2)
            }
            MergeStrategy::NoMerge => {
                // マージ不可 - 競合として扱う
                Ok(MergeResult::Conflict(ConflictInfo {
                    conflict_type: ConflictType::ContentConflict,
                    file_path: file_path.clone(),
                    base_content: base_content.to_string(),
                    version1_content: version1.to_string(),
                    version2_content: version2.to_string(),
                    version1_process: Uuid::new_v4(),
                    version2_process: Uuid::new_v4(),
                    detected_at: SystemTime::now(),
                }))
            }
        }
    }
    
    pub fn merge_multiple_versions(
        &self,
        file_path: &PathBuf,
        base_content: &str,
        versions: &[(String, Uuid)],
    ) -> Result<MergeResult> {
        if versions.is_empty() {
            return Ok(MergeResult::Success(base_content.to_string()));
        }
        
        if versions.len() == 1 {
            return Ok(MergeResult::Success(versions[0].0.clone()));
        }
        
        // 段階的マージ：各バージョンをベースと比較してマージ
        let mut current_content = base_content.to_string();
        
        // 最初のバージョンから開始
        current_content = versions[0].0.clone();
        
        // 残りのバージョンを順次マージ
        for (version_content, _process_id) in versions.iter().skip(1) {
            match self.merge_content(file_path, base_content, &current_content, version_content)? {
                MergeResult::Success(merged) => {
                    current_content = merged;
                }
                MergeResult::Conflict(conflict) => {
                    return Ok(MergeResult::Conflict(conflict));
                }
            }
        }
        
        Ok(MergeResult::Success(current_content))
    }
    
    pub fn resolve_conflict(
        &self,
        file_path: &PathBuf,
        _base_content: &str,
        version1: &str,
        version2: &str,
        timestamp1: SystemTime,
        timestamp2: SystemTime,
    ) -> Result<String> {
        match self.conflict_resolution_strategy {
            ConflictResolution::PreferLatest => {
                if timestamp2 > timestamp1 {
                    Ok(version2.to_string())
                } else {
                    Ok(version1.to_string())
                }
            }
            ConflictResolution::PreferOldest => {
                if timestamp1 < timestamp2 {
                    Ok(version1.to_string())
                } else {
                    Ok(version2.to_string())
                }
            }
            ConflictResolution::PreferHighPriority => {
                // プロセス優先度が設定されていない場合は最新を選択
                Ok(version2.to_string())
            }
            ConflictResolution::Manual => {
                Err(anyhow!("Manual conflict resolution required for file: {:?}", file_path))
            }
        }
    }
    
    pub fn resolve_conflict_with_process(
        &self,
        file_path: &PathBuf,
        _base_content: &str,
        version1: (&str, Uuid),
        version2: (&str, Uuid),
    ) -> Result<String> {
        match self.conflict_resolution_strategy {
            ConflictResolution::PreferHighPriority => {
                let priority1 = self.process_priorities.get(&version1.1).unwrap_or(&5);
                let priority2 = self.process_priorities.get(&version2.1).unwrap_or(&5);
                
                if priority1 > priority2 {
                    Ok(version1.0.to_string())
                } else {
                    Ok(version2.0.to_string())
                }
            }
            _ => {
                // 他の戦略では最初のバージョンを優先
                Ok(version1.0.to_string())
            }
        }
    }
    
    pub fn set_resolution_strategy(&mut self, strategy: ConflictResolution) {
        self.conflict_resolution_strategy = strategy;
    }
    
    pub fn set_process_priority(&mut self, process_id: Uuid, priority: u8) {
        self.process_priorities.insert(process_id, priority);
    }
    
    pub fn add_merge_pattern(&mut self, extension: String, strategy: MergeStrategy) {
        self.merge_patterns.push(MergePattern {
            file_extension: extension,
            merge_strategy: strategy,
        });
    }
    
    fn setup_default_patterns(&mut self) {
        // ソースコードファイル
        self.merge_patterns.push(MergePattern {
            file_extension: "rs".to_string(),
            merge_strategy: MergeStrategy::LineByLine,
        });
        
        self.merge_patterns.push(MergePattern {
            file_extension: "py".to_string(),
            merge_strategy: MergeStrategy::LineByLine,
        });
        
        self.merge_patterns.push(MergePattern {
            file_extension: "js".to_string(),
            merge_strategy: MergeStrategy::LineByLine,
        });
        
        // 設定ファイル
        self.merge_patterns.push(MergePattern {
            file_extension: "toml".to_string(),
            merge_strategy: MergeStrategy::BlockBased,
        });
        
        self.merge_patterns.push(MergePattern {
            file_extension: "yaml".to_string(),
            merge_strategy: MergeStrategy::StructuralMerge,
        });
        
        self.merge_patterns.push(MergePattern {
            file_extension: "yml".to_string(),
            merge_strategy: MergeStrategy::StructuralMerge,
        });
        
        // バイナリファイル
        self.merge_patterns.push(MergePattern {
            file_extension: "png".to_string(),
            merge_strategy: MergeStrategy::NoMerge,
        });
        
        self.merge_patterns.push(MergePattern {
            file_extension: "jpg".to_string(),
            merge_strategy: MergeStrategy::NoMerge,
        });
    }
    
    fn get_merge_strategy(&self, file_path: &PathBuf) -> MergeStrategy {
        if let Some(extension) = file_path.extension().and_then(|e| e.to_str()) {
            for pattern in &self.merge_patterns {
                if pattern.file_extension == extension {
                    return pattern.merge_strategy.clone();
                }
            }
        }
        
        // デフォルトは行単位マージ
        MergeStrategy::LineByLine
    }
    
    fn merge_line_by_line(
        &self,
        file_path: &PathBuf,
        base_content: &str,
        version1: &str,
        version2: &str,
    ) -> Result<MergeResult> {
        let base_lines: Vec<&str> = base_content.lines().collect();
        let v1_lines: Vec<&str> = version1.lines().collect();
        let v2_lines: Vec<&str> = version2.lines().collect();
        
        // 3-way mergeの簡易実装
        let mut merged_lines = Vec::new();
        let mut i = 0;
        let max_len = base_lines.len().max(v1_lines.len()).max(v2_lines.len());
        
        while i < max_len {
            let base_line = base_lines.get(i).unwrap_or(&"");
            let v1_line = v1_lines.get(i).unwrap_or(&"");
            let v2_line = v2_lines.get(i).unwrap_or(&"");
            
            if v1_line == v2_line {
                // 両方が同じ変更 or 変更なし
                merged_lines.push(*v1_line);
            } else if v1_line == base_line {
                // v1は変更なし、v2が変更
                merged_lines.push(*v2_line);
            } else if v2_line == base_line {
                // v2は変更なし、v1が変更
                merged_lines.push(*v1_line);
            } else {
                // 両方が異なる変更 - 競合
                return Ok(MergeResult::Conflict(ConflictInfo {
                    conflict_type: ConflictType::ContentConflict,
                    file_path: file_path.clone(),
                    base_content: base_content.to_string(),
                    version1_content: version1.to_string(),
                    version2_content: version2.to_string(),
                    version1_process: Uuid::new_v4(),
                    version2_process: Uuid::new_v4(),
                    detected_at: SystemTime::now(),
                }));
            }
            
            i += 1;
        }
        
        Ok(MergeResult::Success(merged_lines.join("\n")))
    }
    
    fn merge_block_based(
        &self,
        file_path: &PathBuf,
        base_content: &str,
        version1: &str,
        version2: &str,
    ) -> Result<MergeResult> {
        // ブロック単位のマージ（セクション単位）
        // 実装簡略化のため、行単位マージを使用
        self.merge_line_by_line(file_path, base_content, version1, version2)
    }
    
    fn merge_structural(
        &self,
        file_path: &PathBuf,
        base_content: &str,
        version1: &str,
        version2: &str,
    ) -> Result<MergeResult> {
        // 構造的マージ（YAML/JSONの場合）
        // 実装簡略化のため、行単位マージを使用
        self.merge_line_by_line(file_path, base_content, version1, version2)
    }
    
    pub fn create_merge_conflict_markers(
        &self,
        base_content: &str,
        version1: &str,
        version2: &str,
        process1: Uuid,
        process2: Uuid,
    ) -> String {
        format!(
            "<<<<<<< Process {} (Version 1)\n{}\n=======\n{}\n>>>>>>> Process {} (Version 2)\n",
            process1, version1, version2, process2
        )
    }
    
    pub fn validate_merge_result(&self, file_path: &PathBuf, merged_content: &str) -> Result<bool> {
        // マージ結果の検証
        // 基本的な構文チェックなど
        
        if let Some(extension) = file_path.extension().and_then(|e| e.to_str()) {
            match extension {
                "rs" => self.validate_rust_syntax(merged_content),
                "json" => self.validate_json_syntax(merged_content),
                "yaml" | "yml" => self.validate_yaml_syntax(merged_content),
                _ => Ok(true), // 不明な拡張子は検証スキップ
            }
        } else {
            Ok(true)
        }
    }
    
    fn validate_rust_syntax(&self, content: &str) -> Result<bool> {
        // Rustの基本的な構文チェック
        // 実装簡略化：括弧の対応をチェック
        let mut stack = Vec::new();
        
        for ch in content.chars() {
            match ch {
                '(' | '[' | '{' => stack.push(ch),
                ')' => {
                    if stack.pop() != Some('(') {
                        return Ok(false);
                    }
                }
                ']' => {
                    if stack.pop() != Some('[') {
                        return Ok(false);
                    }
                }
                '}' => {
                    if stack.pop() != Some('{') {
                        return Ok(false);
                    }
                }
                _ => {}
            }
        }
        
        Ok(stack.is_empty())
    }
    
    fn validate_json_syntax(&self, content: &str) -> Result<bool> {
        // JSON構文チェック
        match serde_json::from_str::<serde_json::Value>(content) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    fn validate_yaml_syntax(&self, content: &str) -> Result<bool> {
        // YAML構文チェックの簡易実装
        // 実際の実装ではserde_yamlなどを使う
        Ok(!content.trim().is_empty())
    }
}

impl Default for MergeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_merge_manager_creation() {
        let manager = MergeManager::new();
        assert!(manager.auto_merge_enabled);
        assert!(!manager.merge_patterns.is_empty());
    }
    
    #[test]
    fn test_simple_line_merge() {
        let manager = MergeManager::new();
        let file_path = PathBuf::from("test.txt");
        
        let base = "Line 1\nLine 2\nLine 3";
        let version1 = "Line 1 modified\nLine 2\nLine 3";
        let version2 = "Line 1\nLine 2\nLine 3 modified";
        
        let result = manager.merge_content(&file_path, base, version1, version2).unwrap();
        
        match result {
            MergeResult::Success(merged) => {
                assert!(merged.contains("Line 1 modified"));
                assert!(merged.contains("Line 3 modified"));
            }
            MergeResult::Conflict(_) => panic!("Should merge successfully"),
        }
    }
    
    #[test]
    fn test_conflict_detection() {
        let manager = MergeManager::new();
        let file_path = PathBuf::from("test.txt");
        
        let base = "Original line";
        let version1 = "Modified by process 1";
        let version2 = "Modified by process 2";
        
        let result = manager.merge_content(&file_path, base, version1, version2).unwrap();
        
        match result {
            MergeResult::Conflict(conflict) => {
                assert_eq!(conflict.conflict_type, ConflictType::ContentConflict);
            }
            MergeResult::Success(_) => panic!("Should detect conflict"),
        }
    }
    
    #[test]
    fn test_rust_syntax_validation() {
        let manager = MergeManager::new();
        
        let valid_rust = "fn main() { println!(\"Hello\"); }";
        let invalid_rust = "fn main() { println!(\"Hello\"; }"; // 括弧不一致
        
        assert!(manager.validate_rust_syntax(valid_rust).unwrap());
        assert!(!manager.validate_rust_syntax(invalid_rust).unwrap());
    }
}