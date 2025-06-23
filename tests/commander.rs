use wezterm_parallel::commander::SimpleBridge;

#[test]
fn add_and_fetch_task() {
    let path = tempfile::NamedTempFile::new().unwrap();
    let bridge = SimpleBridge::new(path.path());
    bridge.send_task("ls".into()).unwrap();
    let tasks = bridge.fetch_tasks().unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].command, "ls");
}
