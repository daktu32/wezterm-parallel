use super::Config;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime};
use crate::logging::LogContext;
use crate::{log_info, log_warn, log_error};

pub struct HotReloader {
    config_path: PathBuf,
    #[allow(dead_code)]
    last_modified: Option<SystemTime>,
    receiver: mpsc::Receiver<Config>,
    sender: mpsc::Sender<Config>,
}

impl HotReloader {
    pub fn new(config_path: PathBuf) -> Self {
        let (sender, receiver) = mpsc::channel();
        
        Self {
            config_path,
            last_modified: None,
            receiver,
            sender,
        }
    }

    pub fn start_watching(&mut self) -> Result<(), String> {
        let config_path = self.config_path.clone();
        let sender = self.sender.clone();

        thread::spawn(move || {
            let mut last_modified = None;
            
            loop {
                if let Ok(metadata) = std::fs::metadata(&config_path) {
                    if let Ok(modified) = metadata.modified() {
                        if last_modified.is_none() || last_modified.unwrap() != modified {
                            last_modified = Some(modified);
                            
                            match std::fs::read_to_string(&config_path)
                                .and_then(|content| 
                                    serde_yaml::from_str::<Config>(&content)
                                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                                ) {
                                Ok(config) => {
                                    let reload_context = LogContext::new("config", "hot_reload_success")
                                        .with_entity_id(&config_path.display().to_string());
                                    log_info!(reload_context, "Configuration reloaded from {:?}", config_path);
                                    if let Err(e) = sender.send(config) {
                                        let send_error_context = LogContext::new("config", "hot_reload_send_error");
                                        log_error!(send_error_context, "Failed to send reloaded config: {}", e);
                                        break;
                                    }
                                }
                                Err(e) => {
                                    let reload_error_context = LogContext::new("config", "hot_reload_error")
                                        .with_entity_id(&config_path.display().to_string());
                                    log_warn!(reload_error_context, "Failed to reload config: {}", e);
                                }
                            }
                        }
                    }
                }
                
                thread::sleep(Duration::from_millis(1000));
            }
        });

        Ok(())
    }

    pub fn try_recv_config(&self) -> Option<Config> {
        self.receiver.try_recv().ok()
    }
}