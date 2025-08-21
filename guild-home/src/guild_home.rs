use std::sync::Arc;

use crate::config::Config;
use crate::network::Network;
use guild_discovery::{Discovery, DiscoveryConfig};

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
        // Discovery 설정
        let discovery_config = DiscoveryConfig {
            bootstrap_nodes: self.config.bootstrap.clone(),
            enable_dht: true,
            max_peers: 100,
            port: self.network.local_port(),
        };
        
        let discovery = Discovery::new(discovery_config);
        let network = self.network.clone();
        
        // 피어 탐색 루프 (즉시 시작, 30초마다 재시도)
        tokio::spawn(async move {
            let mut discovery_interval = tokio::time::interval(
                tokio::time::Duration::from_secs(30)
            );
            discovery_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            
            // 즉시 첫 탐색 실행
            loop {
                let peers = discovery.start().await;
                
                for peer_addr in peers {
                    // 발견된 피어에 연결 시도
                    let connect_result = network.connect(peer_addr).await;
                    
                    if connect_result.is_ok() {
                        println!("✅ Connected to peer: {}", peer_addr);
                        // 연결 성공한 피어를 discovery에 추가
                        discovery.add_peer(peer_addr).await;
                    } else if let Err(e) = connect_result {
                        // 연결 실패는 조용히 처리 (피어가 실제로 없을 수 있음)
                        let error_msg = e.to_string();
                        if !error_msg.contains("refused") && !error_msg.contains("Connection refused") {
                            println!("⚠️ Failed to connect to {}: {}", peer_addr, error_msg);
                        }
                    }
                }
                
                // 다음 탐색까지 대기
                discovery_interval.tick().await;
            }
        });
        
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