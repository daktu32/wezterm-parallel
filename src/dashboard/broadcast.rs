use tokio::sync::broadcast;

pub struct BroadcastManager {
    sender: broadcast::Sender<String>,
}

impl BroadcastManager {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self { sender }
    }

    pub async fn broadcast(&self, message: String) -> Result<(), String> {
        self.sender
            .send(message)
            .map_err(|e| format!("Failed to broadcast message: {}", e))?;
        Ok(())
    }

    pub async fn subscribe(&self) -> broadcast::Receiver<String> {
        self.sender.subscribe()
    }
}