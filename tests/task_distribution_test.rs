use uuid::Uuid;
use wezterm_parallel::task::{
    distributor::{DistributedTask, ProcessLoad, TaskDependency, TaskDistributor},
    TaskPriority, TaskStatus,
};

#[test]
fn test_task_creation() {
    let task = DistributedTask::new("Test task".to_string(), TaskPriority::Medium, vec![]);

    assert_eq!(task.base_task.title, "Test task");
    assert_eq!(*task.priority(), TaskPriority::Medium);
    assert_eq!(*task.status(), TaskStatus::Todo);
    assert!(task.dependencies.is_empty());
}

#[test]
fn test_task_dependency_detection() {
    let task1_id = Uuid::new_v4();
    let task2_id = Uuid::new_v4();

    let mut task1 = DistributedTask::new("Task 1".to_string(), TaskPriority::High, vec![]);
    task1.distribution_id = task1_id;

    let mut task2 = DistributedTask::new(
        "Task 2".to_string(),
        TaskPriority::Medium,
        vec![TaskDependency::TaskCompletion(task1_id)],
    );
    task2.distribution_id = task2_id;

    assert!(task2.depends_on(&task1_id));
    assert!(!task1.depends_on(&task2_id));
}

#[test]
fn test_parallel_execution_possibility() {
    let distributor = TaskDistributor::new();

    // 独立したタスク
    let task1 = DistributedTask::new(
        "Independent task 1".to_string(),
        TaskPriority::Medium,
        vec![],
    );

    let task2 = DistributedTask::new(
        "Independent task 2".to_string(),
        TaskPriority::Medium,
        vec![],
    );

    assert!(distributor.can_run_parallel(&task1, &task2));

    // 依存関係のあるタスク
    let task3_id = Uuid::new_v4();
    let mut task3 = DistributedTask::new("Task 3".to_string(), TaskPriority::Medium, vec![]);
    task3.distribution_id = task3_id;

    let task4 = DistributedTask::new(
        "Task 4 depends on 3".to_string(),
        TaskPriority::Medium,
        vec![TaskDependency::TaskCompletion(task3_id)],
    );

    assert!(!distributor.can_run_parallel(&task3, &task4));
}

#[test]
fn test_optimal_process_assignment() {
    let mut distributor = TaskDistributor::new();

    // プロセスの負荷状態を設定
    let process1_id = Uuid::new_v4();
    let process2_id = Uuid::new_v4();
    let process3_id = Uuid::new_v4();

    distributor.update_process_load(
        process1_id,
        ProcessLoad {
            cpu_usage: 0.8,
            memory_usage: 0.6,
            active_tasks: 3,
        },
    );

    distributor.update_process_load(
        process2_id,
        ProcessLoad {
            cpu_usage: 0.2,
            memory_usage: 0.3,
            active_tasks: 1,
        },
    );

    distributor.update_process_load(
        process3_id,
        ProcessLoad {
            cpu_usage: 0.5,
            memory_usage: 0.4,
            active_tasks: 2,
        },
    );

    let task = DistributedTask::new("New task".to_string(), TaskPriority::Medium, vec![]);

    // 最も負荷の低いprocess2が選ばれるべき
    let assigned_process = distributor.assign_task(&task).unwrap();
    assert_eq!(assigned_process, process2_id);
}

#[test]
fn test_dependency_graph_resolution() {
    let mut distributor = TaskDistributor::new();

    // タスクグラフの作成
    // A -> B -> D
    //   -> C -> D
    let task_a_id = Uuid::new_v4();
    let task_b_id = Uuid::new_v4();
    let task_c_id = Uuid::new_v4();
    let task_d_id = Uuid::new_v4();

    let mut task_a = DistributedTask::new("Task A".to_string(), TaskPriority::High, vec![]);
    task_a.distribution_id = task_a_id;

    let mut task_b = DistributedTask::new(
        "Task B".to_string(),
        TaskPriority::Medium,
        vec![TaskDependency::TaskCompletion(task_a_id)],
    );
    task_b.distribution_id = task_b_id;

    let mut task_c = DistributedTask::new(
        "Task C".to_string(),
        TaskPriority::Medium,
        vec![TaskDependency::TaskCompletion(task_a_id)],
    );
    task_c.distribution_id = task_c_id;

    let mut task_d = DistributedTask::new(
        "Task D".to_string(),
        TaskPriority::Low,
        vec![
            TaskDependency::TaskCompletion(task_b_id),
            TaskDependency::TaskCompletion(task_c_id),
        ],
    );
    task_d.distribution_id = task_d_id;

    distributor.add_task(task_a);
    distributor.add_task(task_b);
    distributor.add_task(task_c);
    distributor.add_task(task_d);

    // 実行順序の検証
    let execution_order = distributor.resolve_execution_order().unwrap();

    // Aが最初
    assert_eq!(execution_order[0], task_a_id);

    // BとCは順不同だが、両方ともDより前
    let b_index = execution_order
        .iter()
        .position(|&id| id == task_b_id)
        .unwrap();
    let c_index = execution_order
        .iter()
        .position(|&id| id == task_c_id)
        .unwrap();
    let d_index = execution_order
        .iter()
        .position(|&id| id == task_d_id)
        .unwrap();

    assert!(b_index < d_index);
    assert!(c_index < d_index);
}

#[test]
fn test_file_dependency_detection() {
    let task1 = DistributedTask::new(
        "Modify file1.rs".to_string(),
        TaskPriority::Medium,
        vec![TaskDependency::FileAccess("src/file1.rs".to_string())],
    );

    let task2 = DistributedTask::new(
        "Read file1.rs".to_string(),
        TaskPriority::Medium,
        vec![TaskDependency::FileAccess("src/file1.rs".to_string())],
    );

    let task3 = DistributedTask::new(
        "Modify file2.rs".to_string(),
        TaskPriority::Medium,
        vec![TaskDependency::FileAccess("src/file2.rs".to_string())],
    );

    let distributor = TaskDistributor::new();

    // 同じファイルにアクセスするタスクは並列実行不可
    assert!(!distributor.can_run_parallel(&task1, &task2));

    // 異なるファイルにアクセスするタスクは並列実行可能
    assert!(distributor.can_run_parallel(&task1, &task3));
}

#[test]
fn test_priority_based_scheduling() {
    let mut distributor = TaskDistributor::new();

    let low_priority =
        DistributedTask::new("Low priority task".to_string(), TaskPriority::Low, vec![]);

    let normal_priority = DistributedTask::new(
        "Normal priority task".to_string(),
        TaskPriority::Medium,
        vec![],
    );

    let high_priority =
        DistributedTask::new("High priority task".to_string(), TaskPriority::High, vec![]);

    distributor.add_task(low_priority.clone());
    distributor.add_task(normal_priority.clone());
    distributor.add_task(high_priority.clone());

    let next_task = distributor.get_next_task().unwrap();
    assert_eq!(*next_task.priority(), TaskPriority::High);
}

#[test]
fn test_circular_dependency_detection() {
    let mut distributor = TaskDistributor::new();

    // 循環依存: A -> B -> C -> A
    let task_a_id = Uuid::new_v4();
    let task_b_id = Uuid::new_v4();
    let task_c_id = Uuid::new_v4();

    let mut task_a = DistributedTask::new(
        "Task A".to_string(),
        TaskPriority::Medium,
        vec![TaskDependency::TaskCompletion(task_c_id)],
    );
    task_a.distribution_id = task_a_id;

    let mut task_b = DistributedTask::new(
        "Task B".to_string(),
        TaskPriority::Medium,
        vec![TaskDependency::TaskCompletion(task_a_id)],
    );
    task_b.distribution_id = task_b_id;

    let mut task_c = DistributedTask::new(
        "Task C".to_string(),
        TaskPriority::Medium,
        vec![TaskDependency::TaskCompletion(task_b_id)],
    );
    task_c.distribution_id = task_c_id;

    distributor.add_task(task_a);
    distributor.add_task(task_b);
    distributor.add_task(task_c);

    // 循環依存がある場合、実行順序の解決は失敗すべき
    assert!(distributor.resolve_execution_order().is_err());
}

#[test]
fn test_resource_based_assignment() {
    let mut distributor = TaskDistributor::new();

    // CPU集約的なタスク
    let cpu_task = DistributedTask::new_with_resources(
        "CPU intensive task".to_string(),
        TaskPriority::Medium,
        vec![],
        0.8, // CPU要求
        0.2, // メモリ要求
    );

    // メモリ集約的なタスク
    let memory_task = DistributedTask::new_with_resources(
        "Memory intensive task".to_string(),
        TaskPriority::Medium,
        vec![],
        0.2, // CPU要求
        0.8, // メモリ要求
    );

    // プロセスの負荷状態を設定
    let cpu_process_id = Uuid::new_v4();
    let memory_process_id = Uuid::new_v4();

    distributor.update_process_load(
        cpu_process_id,
        ProcessLoad {
            cpu_usage: 0.1,    // CPU余裕あり
            memory_usage: 0.7, // メモリ不足気味
            active_tasks: 1,
        },
    );

    distributor.update_process_load(
        memory_process_id,
        ProcessLoad {
            cpu_usage: 0.7,    // CPU不足気味
            memory_usage: 0.1, // メモリ余裕あり
            active_tasks: 1,
        },
    );

    // CPU集約的タスクはCPUに余裕があるプロセスへ
    let cpu_assignment = distributor.assign_task(&cpu_task).unwrap();
    assert_eq!(cpu_assignment, cpu_process_id);

    // メモリ集約的タスクはメモリに余裕があるプロセスへ
    let memory_assignment = distributor.assign_task(&memory_task).unwrap();
    assert_eq!(memory_assignment, memory_process_id);
}
