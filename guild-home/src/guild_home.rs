use std::sync::Arc;

use crate::blockchain_bridge::BlockchainBridge;
use crate::config::Config;
use crate::network::Network;
use crate::log_network;
use guild_discovery::{Discovery, DiscoveryConfig};

pub struct GuildHome {
    pub config: Config,
    pub network: Arc<Network>,
    pub blockchain_bridge: Option<BlockchainBridge>,
}

impl GuildHome {
    pub async fn new(config: Config) -> Self {
        let network = Arc::new(Network::with_port(config.port).await);
        let blockchain_bridge = Some(BlockchainBridge::new(network.clone()));

        GuildHome { 
            config, 
            network,
            blockchain_bridge,
        }
    }

    pub async fn start(&mut self) {
        // Start blockchain bridge
        log_network!("Starting blockchain bridge...");
        if let Some(ref mut bridge) = self.blockchain_bridge {
            match bridge.start().await {
                Ok(_) => log_network!("Blockchain bridge started successfully"),
                Err(e) => {
                    let err_msg = e.to_string();
                    log_network!("Failed to start blockchain bridge: {}", err_msg);
                }
            }
        } else {
            log_network!("No blockchain bridge configured");
        }
        log_network!("Blockchain bridge initialization complete");
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
            let mut discovery_interval =
                tokio::time::interval(tokio::time::Duration::from_secs(30));
            discovery_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            // 즉시 첫 탐색 실행
            loop {
                let peers = discovery.start().await;

                for peer_addr in peers {
                    // 발견된 피어에 연결 시도
                    let connect_result = network.connect(peer_addr).await;

                    if connect_result.is_ok() {
                        log_network!("✅ Connected to peer: {}", peer_addr);
                        // 연결 성공한 피어를 discovery에 추가
                        discovery.add_peer(peer_addr).await;
                    } else if let Err(e) = connect_result {
                        // 연결 실패는 조용히 처리 (피어가 실제로 없을 수 있음)
                        let error_msg = e.to_string();
                        if !error_msg.contains("refused")
                            && !error_msg.contains("Connection refused")
                        {
                            log_network!("⚠️ Failed to connect to {}: {}", peer_addr, error_msg);
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
            let mut ping_interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
            loop {
                ping_interval.tick().await;
                network_ping.send_ping().await;
            }
        });

        // 피어 헬스 체크 루프 (10초마다)
        let network_health = self.network.clone();
        tokio::spawn(async move {
            let mut health_interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
            loop {
                health_interval.tick().await;
                network_health.check_peer_health().await;
            }
        });

        // 메인 모니터링 루프 (백그라운드에서 실행)
        let network_monitor = self.network.clone();
        let heartbeat_interval = self.config.heartbeat_interval;
        let log_level = self.config.log_level.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                heartbeat_interval,
            ));

            loop {
                interval.tick().await;

                let _peers = network_monitor.peer_count().await;

                // Log level check for future use
                if log_level != "error" {
                    // Future monitoring output can go here
                }
            }
        });
    }
}
