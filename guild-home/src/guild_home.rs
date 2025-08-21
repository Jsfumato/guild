use std::sync::Arc;

use crate::config::Config;
use crate::network::Network;
use crate::discovery::Discovery;

pub struct GuildHome {
    pub config: Config,
    pub network: Arc<Network>,
}

impl GuildHome {
    pub async fn new(config: Config) -> Self {
        let network = Arc::new(Network::with_port(config.port).await);
        
        GuildHome {
            config,
            network,
        }
    }
    
    pub async fn start(&self) {
        // 자동 피어 탐색 시작
        let discovery = Discovery::with_bootstrap(
            self.network.clone(), 
            self.config.bootstrap.clone()
        );
        tokio::spawn(discovery.start());
        
        // Ping 전송 루프 (5초마다)
        let network_ping = self.network.clone();
        tokio::spawn(async move {
            let mut ping_interval = tokio::time::interval(
                tokio::time::Duration::from_secs(5)
            );
            loop {
                ping_interval.tick().await;
                network_ping.send_ping().await;
            }
        });
        
        // 피어 헬스 체크 루프 (10초마다)
        let network_health = self.network.clone();
        tokio::spawn(async move {
            let mut health_interval = tokio::time::interval(
                tokio::time::Duration::from_secs(10)
            );
            loop {
                health_interval.tick().await;
                network_health.check_peer_health().await;
            }
        });
        
        // 메인 모니터링 루프
        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.heartbeat_interval)
        );
        
        loop {
            interval.tick().await;
            
            let peers = self.network.peer_count().await;
            
            if self.config.log_level != "error" {
                println!("⚡ Guild Home | Peers: {} | Port: {}", 
                    peers, 
                    self.network.local_port()
                );
            }
        }
    }
}