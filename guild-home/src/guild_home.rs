use std::sync::Arc;
use uuid::Uuid;

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
        
        // 메인 루프 - 피어 상태 모니터링
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
            
            // 연결된 피어들과 하트비트 메시지 교환
            if peers > 0 {
                let heartbeat_msg = format!("heartbeat-{}", Uuid::new_v4());
                self.network.broadcast(heartbeat_msg.as_bytes()).await;
            }
        }
    }
}