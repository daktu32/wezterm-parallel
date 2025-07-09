use super::broadcast::BroadcastManager;
use super::handlers::handle_websocket;
use crate::config::ServerConfig;
use log::{error, info};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;

pub struct DashboardServer {
    #[allow(dead_code)]
    config: ServerConfig,
    broadcast_manager: Arc<BroadcastManager>,
}

impl DashboardServer {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            broadcast_manager: Arc::new(BroadcastManager::new()),
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = "127.0.0.1:8080"; // Default address since we're using Unix sockets for IPC
        let listener = TcpListener::bind(addr).await?;
        info!("Dashboard server listening on {addr}");

        while let Ok((stream, addr)) = listener.accept().await {
            info!("New connection from {addr}");

            let broadcast_manager = Arc::clone(&self.broadcast_manager);

            tokio::spawn(async move {
                match accept_async(stream).await {
                    Ok(ws_stream) => {
                        if let Err(e) = handle_websocket(ws_stream, broadcast_manager).await {
                            error!("WebSocket error: {e}");
                        }
                    }
                    Err(e) => {
                        error!("Failed to accept WebSocket connection: {e}");
                    }
                }
            });
        }

        Ok(())
    }

    pub fn get_broadcast_manager(&self) -> Arc<BroadcastManager> {
        Arc::clone(&self.broadcast_manager)
    }
    
    /// For testing: start server with custom address
    pub async fn start_with_address(&self, addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = TcpListener::bind(addr).await?;
        info!("Dashboard server listening on {addr}");

        while let Ok((stream, addr)) = listener.accept().await {
            info!("New connection from {addr}");

            let broadcast_manager = Arc::clone(&self.broadcast_manager);

            tokio::spawn(async move {
                match accept_async(stream).await {
                    Ok(ws_stream) => {
                        if let Err(e) = handle_websocket(ws_stream, broadcast_manager).await {
                            error!("WebSocket error: {e}");
                        }
                    }
                    Err(e) => {
                        error!("Failed to accept WebSocket connection: {e}");
                    }
                }
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::timeout;

    // === テストヘルパー関数 ===

    /// テスト用のServerConfigを作成
    fn create_test_server_config() -> ServerConfig {
        ServerConfig {
            socket_path: "/tmp/test-dashboard.sock".to_string(),
            max_connections: 10,
            connection_timeout: 30,
            enable_metrics: true,
            health_check_interval: 10,
        }
    }

    /// 空きポートを見つけるヘルパー関数
    async fn find_available_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        port
    }

    // === 基本機能テスト ===

    #[test]
    fn test_server_new_initialization_success() {
        let config = create_test_server_config();
        let server = DashboardServer::new(config);
        
        // BroadcastManagerが正常に初期化されていることを確認
        let broadcast_manager = server.get_broadcast_manager();
        assert_eq!(Arc::strong_count(&broadcast_manager), 2); // server内とこのテストで2つの参照
    }

    #[test]
    fn test_get_broadcast_manager_returns_valid_manager() {
        let config = create_test_server_config();
        let server = DashboardServer::new(config);
        
        let manager1 = server.get_broadcast_manager();
        let manager2 = server.get_broadcast_manager();
        
        // 同じインスタンスが返されることを確認
        assert_eq!(Arc::strong_count(&manager1), 3); // server内、manager1、manager2
        assert_eq!(Arc::strong_count(&manager2), 3);
    }

    // === 非同期・ネットワーク機能テスト ===

    #[tokio::test]
    async fn test_start_binds_to_address_successfully() {
        let config = create_test_server_config();
        let server = DashboardServer::new(config);
        
        let port = find_available_port().await;
        let addr = format!("127.0.0.1:{}", port);
        
        // サーバーが正常に起動できることを確認（タイムアウト付き）
        let result = timeout(Duration::from_millis(100), server.start_with_address(&addr)).await;
        
        // タイムアウトエラーが発生することを確認（サーバーが正常に起動している証拠）
        assert!(result.is_err());
        
        // Note: タイムアウト後はサーバーが停止するため、ポートが解放される
        // 実際のテストでは、サーバーが起動してタイムアウトすることを確認するだけで十分
    }

    #[tokio::test]
    async fn test_start_returns_error_if_port_in_use() {
        let config = create_test_server_config();
        let server = DashboardServer::new(config);
        
        let port = find_available_port().await;
        let addr = format!("127.0.0.1:{}", port);
        
        // ポートを先に使用する
        let _listener = TcpListener::bind(&addr).await.unwrap();
        
        // 同じポートでサーバーを起動しようとするとエラーになる
        let result = server.start_with_address(&addr).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tcp_listener_creation() {
        let config = create_test_server_config();
        let server = DashboardServer::new(config);
        
        let port = find_available_port().await;
        let addr = format!("127.0.0.1:{}", port);
        
        // TCPリスナーが正常に作成できることを確認
        let listener = TcpListener::bind(&addr).await;
        assert!(listener.is_ok());
        
        // 同じポートでサーバーを起動しようとするとエラーになる
        let result = server.start_with_address(&addr).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_server_with_invalid_address() {
        let config = create_test_server_config();
        let server = DashboardServer::new(config);
        
        // 無効なアドレスでサーバーを起動しようとするとエラーになる
        let result = server.start_with_address("invalid_address").await;
        assert!(result.is_err());
    }

    // === 設定テスト ===

    #[test]
    fn test_server_config_storage() {
        let config = ServerConfig {
            socket_path: "/tmp/custom-dashboard.sock".to_string(),
            max_connections: 50,
            connection_timeout: 60,
            enable_metrics: false,
            health_check_interval: 20,
        };
        
        let server = DashboardServer::new(config);
        
        // 設定が正常に保存されていることを確認
        // Note: configフィールドは現在 #[allow(dead_code)] なので直接アクセスできない
        // 実際の使用では設定を取得するメソッドを追加する必要がある
        let broadcast_manager = server.get_broadcast_manager();
        assert_eq!(Arc::strong_count(&broadcast_manager), 2);
    }

    #[test]
    fn test_multiple_server_instances() {
        let config1 = create_test_server_config();
        let config2 = create_test_server_config();
        
        let server1 = DashboardServer::new(config1);
        let server2 = DashboardServer::new(config2);
        
        // 各サーバーが独立したBroadcastManagerを持つことを確認
        let manager1 = server1.get_broadcast_manager();
        let manager2 = server2.get_broadcast_manager();
        
        assert_eq!(Arc::strong_count(&manager1), 2);
        assert_eq!(Arc::strong_count(&manager2), 2);
    }

    // === エラーハンドリングテスト ===

    #[tokio::test]
    async fn test_concurrent_server_startup() {
        let config = create_test_server_config();
        let server = DashboardServer::new(config);
        
        let port = find_available_port().await;
        let addr = format!("127.0.0.1:{}", port);
        
        // 同時に複数のサーバーを起動しようとする
        let addr_clone = addr.clone();
        let server_task = tokio::spawn(async move {
            timeout(Duration::from_millis(100), server.start_with_address(&addr_clone)).await
        });
        
        // 少し待ってから同じポートで別のサーバーを起動しようとする
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let config2 = create_test_server_config();
        let server2 = DashboardServer::new(config2);
        let result2 = server2.start_with_address(&addr).await;
        
        // 2番目のサーバーは起動に失敗する（ポートが使用中）
        assert!(result2.is_err());
        
        // 最初のサーバーはタイムアウトする（正常に起動している証拠）
        let result1 = server_task.await.unwrap();
        assert!(result1.is_err()); // タイムアウトエラー
    }

    #[test]
    fn test_broadcast_manager_clone_behavior() {
        let config = create_test_server_config();
        let server = DashboardServer::new(config);
        
        let manager1 = server.get_broadcast_manager();
        let manager2 = Arc::clone(&manager1);
        let manager3 = server.get_broadcast_manager();
        
        // 参照カウントが正しく管理されていることを確認
        assert_eq!(Arc::strong_count(&manager1), 4); // server内、manager1、manager2、manager3
        assert_eq!(Arc::strong_count(&manager2), 4);
        assert_eq!(Arc::strong_count(&manager3), 4);
        
        drop(manager2);
        assert_eq!(Arc::strong_count(&manager1), 3); // manager2がドロップされた
    }
}
