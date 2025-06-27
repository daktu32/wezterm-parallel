use wezterm_parallel::sync::{
    file_sync::{FileSyncManager, FileChange, ChangeType, ConflictResolution},
    merger::{MergeManager, MergeResult, ConflictType},
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use tempfile::TempDir;
use uuid::Uuid;

#[test]
fn test_file_change_detection() {
    let sync_manager = FileSyncManager::new();
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_file.txt");
    
    // ファイル作成
    std::fs::write(&file_path, "Initial content").unwrap();
    
    let change = FileChange::new(
        file_path.clone(),
        ChangeType::Created,
        "Initial content".to_string(),
        SystemTime::now(),
        Uuid::new_v4(),
    );
    
    assert_eq!(change.file_path, file_path);
    assert_eq!(change.change_type, ChangeType::Created);
    assert_eq!(change.content, "Initial content");
}

#[test]
fn test_file_conflict_detection() {
    let mut sync_manager = FileSyncManager::new();
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("conflict_test.txt");
    
    let process1_id = Uuid::new_v4();
    let process2_id = Uuid::new_v4();
    
    // プロセス1からの変更
    let change1 = FileChange::new(
        file_path.clone(),
        ChangeType::Modified,
        "Content from process 1".to_string(),
        SystemTime::now(),
        process1_id,
    );
    
    // プロセス2からの変更（同時期）
    let change2 = FileChange::new(
        file_path.clone(),
        ChangeType::Modified,
        "Content from process 2".to_string(),
        SystemTime::now(),
        process2_id,
    );
    
    sync_manager.apply_change(change1).unwrap();
    
    // 競合が検出されるはず
    let conflict_result = sync_manager.apply_change(change2);
    assert!(conflict_result.is_err());
}

#[test]
fn test_automatic_conflict_resolution() {
    let mut merge_manager = MergeManager::new();
    
    let file_path = PathBuf::from("test.txt");
    let base_content = "Line 1\nLine 2\nLine 3";
    let version1 = "Line 1 modified\nLine 2\nLine 3";
    let version2 = "Line 1\nLine 2\nLine 3 modified";
    
    let result = merge_manager.merge_content(
        &file_path,
        base_content,
        version1,
        version2,
    ).unwrap();
    
    match result {
        MergeResult::Success(merged_content) => {
            assert!(merged_content.contains("Line 1 modified"));
            assert!(merged_content.contains("Line 3 modified"));
        }
        MergeResult::Conflict(_) => panic!("Should be able to auto-merge non-conflicting changes"),
    }
}

#[test]
fn test_manual_conflict_resolution() {
    let mut merge_manager = MergeManager::new();
    
    let file_path = PathBuf::from("conflict.txt");
    let base_content = "Original line";
    let version1 = "Modified by process 1";
    let version2 = "Modified by process 2";
    
    let result = merge_manager.merge_content(
        &file_path,
        base_content,
        version1,
        version2,
    ).unwrap();
    
    match result {
        MergeResult::Conflict(conflict_info) => {
            assert_eq!(conflict_info.conflict_type, ConflictType::ContentConflict);
            assert_eq!(conflict_info.base_content, base_content);
            assert_eq!(conflict_info.version1_content, version1);
            assert_eq!(conflict_info.version2_content, version2);
        }
        MergeResult::Success(_) => panic!("Should detect conflict in overlapping changes"),
    }
}

#[test]
fn test_file_watch_system() {
    let mut sync_manager = FileSyncManager::new();
    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();
    
    // ディレクトリ監視開始
    sync_manager.start_watching(&watch_path).unwrap();
    
    // ファイル作成
    let test_file = watch_path.join("watched_file.txt");
    std::fs::write(&test_file, "Test content").unwrap();
    
    // 少し待機（ファイルシステム監視のため）
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    // 変更が検出されているか確認
    let changes = sync_manager.get_pending_changes();
    assert!(!changes.is_empty());
    
    let change = &changes[0];
    assert_eq!(change.file_path, test_file);
    assert_eq!(change.change_type, ChangeType::Created);
}

#[test]
fn test_versioned_file_tracking() {
    let mut sync_manager = FileSyncManager::new();
    let file_path = PathBuf::from("versioned_file.txt");
    
    let process_id = Uuid::new_v4();
    
    // バージョン1
    let change1 = FileChange::new(
        file_path.clone(),
        ChangeType::Created,
        "Version 1".to_string(),
        SystemTime::now(),
        process_id,
    );
    
    sync_manager.apply_change(change1).unwrap();
    
    // バージョン2
    let change2 = FileChange::new(
        file_path.clone(),
        ChangeType::Modified,
        "Version 2".to_string(),
        SystemTime::now(),
        process_id,
    );
    
    sync_manager.apply_change(change2).unwrap();
    
    // ファイル履歴の確認
    let history = sync_manager.get_file_history(&file_path).unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].content, "Version 1");
    assert_eq!(history[1].content, "Version 2");
}

#[test]
fn test_cross_process_synchronization() {
    let mut sync_manager = FileSyncManager::new();
    let file_path = PathBuf::from("shared_file.txt");
    
    let process1_id = Uuid::new_v4();
    let process2_id = Uuid::new_v4();
    
    sync_manager.register_process(process1_id);
    sync_manager.register_process(process2_id);
    
    // プロセス1からファイル作成
    let change1 = FileChange::new(
        file_path.clone(),
        ChangeType::Created,
        "Initial content".to_string(),
        SystemTime::now(),
        process1_id,
    );
    
    sync_manager.apply_change(change1).unwrap();
    
    // プロセス2に同期
    let sync_changes = sync_manager.get_changes_for_process(process2_id);
    assert_eq!(sync_changes.len(), 1);
    assert_eq!(sync_changes[0].content, "Initial content");
    
    // プロセス2からファイル修正
    let change2 = FileChange::new(
        file_path.clone(),
        ChangeType::Modified,
        "Modified by process 2".to_string(),
        SystemTime::now(),
        process2_id,
    );
    
    sync_manager.apply_change(change2).unwrap();
    
    // プロセス1に同期
    let sync_changes = sync_manager.get_changes_for_process(process1_id);
    assert_eq!(sync_changes.len(), 1);
    assert_eq!(sync_changes[0].content, "Modified by process 2");
}

#[test]
fn test_merge_multiple_changes() {
    let mut merge_manager = MergeManager::new();
    
    let file_path = PathBuf::from("multi_change.txt");
    let base_content = "Line 1\nLine 2\nLine 3\nLine 4";
    
    // 複数の変更
    let changes = vec![
        ("Line 1 modified\nLine 2\nLine 3\nLine 4".to_string(), Uuid::new_v4()),
        ("Line 1\nLine 2 modified\nLine 3\nLine 4".to_string(), Uuid::new_v4()),
        ("Line 1\nLine 2\nLine 3\nLine 4 modified".to_string(), Uuid::new_v4()),
    ];
    
    let result = merge_manager.merge_multiple_versions(
        &file_path,
        base_content,
        &changes,
    ).unwrap();
    
    match result {
        MergeResult::Success(merged_content) => {
            assert!(merged_content.contains("Line 1 modified"));
            assert!(merged_content.contains("Line 2 modified"));
            assert!(merged_content.contains("Line 4 modified"));
        }
        MergeResult::Conflict(_) => panic!("Should be able to merge non-overlapping changes"),
    }
}

#[test]
fn test_backup_and_recovery() {
    let mut sync_manager = FileSyncManager::new();
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("backup_test.txt");
    
    let process_id = Uuid::new_v4();
    
    // 元ファイル作成
    let original_change = FileChange::new(
        file_path.clone(),
        ChangeType::Created,
        "Original content".to_string(),
        SystemTime::now(),
        process_id,
    );
    
    sync_manager.apply_change(original_change).unwrap();
    
    // バックアップ作成
    sync_manager.create_backup(&file_path).unwrap();
    
    // ファイル変更
    let modify_change = FileChange::new(
        file_path.clone(),
        ChangeType::Modified,
        "Modified content".to_string(),
        SystemTime::now(),
        process_id,
    );
    
    sync_manager.apply_change(modify_change).unwrap();
    
    // バックアップから復元
    sync_manager.restore_from_backup(&file_path).unwrap();
    
    // 内容確認
    let restored_content = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(restored_content, "Original content");
}

#[test]
fn test_conflict_resolution_strategies() {
    let mut merge_manager = MergeManager::new();
    
    let file_path = PathBuf::from("strategy_test.txt");
    let base_content = "Conflicted line";
    let version1 = "Version 1 content";
    let version2 = "Version 2 content";
    
    // 戦略: 最新を優先
    merge_manager.set_resolution_strategy(ConflictResolution::PreferLatest);
    let result = merge_manager.resolve_conflict(
        &file_path,
        base_content,
        version1,
        version2,
        SystemTime::now(),
        SystemTime::now() + std::time::Duration::from_secs(1),
    ).unwrap();
    
    assert_eq!(result, "Version 2 content");
    
    // 戦略: 高優先度プロセスを優先
    merge_manager.set_resolution_strategy(ConflictResolution::PreferHighPriority);
    let high_priority_process = Uuid::new_v4();
    merge_manager.set_process_priority(high_priority_process, 10);
    
    let result = merge_manager.resolve_conflict_with_process(
        &file_path,
        base_content,
        (version1, high_priority_process),
        (version2, Uuid::new_v4()),
    ).unwrap();
    
    assert_eq!(result, "Version 1 content");
}

#[test]
fn test_sync_performance_monitoring() {
    let mut sync_manager = FileSyncManager::new();
    
    let file_path = PathBuf::from("performance_test.txt");
    let process_id = Uuid::new_v4();
    
    // 大量の変更を適用
    for i in 0..100 {
        let change = FileChange::new(
            file_path.clone(),
            ChangeType::Modified,
            format!("Content {}", i),
            SystemTime::now(),
            process_id,
        );
        
        sync_manager.apply_change(change).unwrap();
    }
    
    // パフォーマンス統計取得
    let stats = sync_manager.get_performance_stats();
    assert_eq!(stats.total_changes_applied, 100);
    assert!(stats.average_apply_time.as_millis() > 0);
    assert_eq!(stats.total_conflicts_detected, 0);
}

#[test]
fn test_large_file_handling() {
    let mut sync_manager = FileSyncManager::new();
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("large_file.txt");
    
    let process_id = Uuid::new_v4();
    
    // 大きなファイル内容を作成（1MB程度）
    let large_content = "A".repeat(1024 * 1024);
    
    let change = FileChange::new(
        file_path.clone(),
        ChangeType::Created,
        large_content.clone(),
        SystemTime::now(),
        process_id,
    );
    
    // 大きなファイルでも適切に処理されるか確認
    let result = sync_manager.apply_change(change);
    assert!(result.is_ok());
    
    // ファイルが正しく作成されているか確認
    let written_content = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(written_content, large_content);
}