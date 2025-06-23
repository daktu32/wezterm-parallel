use wezterm_parallel::commander::{SimpleBridge};

fn main() {
    let bridge = SimpleBridge::new("/tmp/wezterm_tasks.json");
    bridge.send_task("echo hello".into()).expect("send task");
    let tasks = bridge.fetch_tasks().expect("fetch");
    println!("tasks: {:?}", tasks);
}
