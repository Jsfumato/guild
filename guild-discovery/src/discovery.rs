// Discovery - 통합 피어 발견 시스템
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::bootstrap::{Bootstrap, PeerInfo};
use crate::dht::{Kademlia, Node, NodeId};
use crate::local_scan::LocalScanner;
use crate::log_network;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    pub bootstrap_nodes: Vec<String>,
    pub enable_dht: bool,
    pub max_peers: usize,
    pub port: u16,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            bootstrap_nodes: vec![],
            enable_dht: true,
            max_peers: 100,
            port: 42000,
        }
    }
}

#[async_trait]
pub trait DiscoveryMethod: Send + Sync {
    async fn discover_peers(&self) -> Vec<SocketAddr>;
    async fn announce(&self, addr: SocketAddr);
}

pub struct Discovery {
    config: DiscoveryConfig,
    bootstrap: Arc<Bootstrap>,
    dht: Option<Arc<Kademlia>>,
    discovered_peers: Arc<RwLock<Vec<PeerInfo>>>,
    node_id: NodeId,
}

impl Discovery {
    pub fn new(config: DiscoveryConfig) -> Self {
        let node_id = NodeId::random();
        let bootstrap = Arc::new(Bootstrap::new(config.bootstrap_nodes.clone()));

        let dht = if config.enable_dht {
            Some(Arc::new(Kademlia::new(node_id)))
        } else {
            None
        };

        Self {
            config,
            bootstrap,
            dht,
            discovered_peers: Arc::new(RwLock::new(Vec::new())),
            node_id,
        }
    }

    pub async fn start(&self) -> Vec<SocketAddr> {
        let mut peers = Vec::new();

        // 1. 로컬 네트워크 스캔 (Bootstrap 없이도 동작)
        log_network!("🔍 Scanning local network for peers...");
        let scanner = LocalScanner::new(self.config.port);
        let local_peers = scanner.scan_local_peers().await;

        for addr in local_peers {
            if addr.port() != self.config.port {
                // 자기 자신 제외
                peers.push(addr);

                // DHT에 추가
                if let Some(dht) = &self.dht {
                    let node = Node {
                        id: NodeId::from_addr(&addr),
                        addr,
                        last_seen: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    };
                    dht.add_node(node).await;
                }
            }
        }

        // 2. Bootstrap 노드 연결 (있는 경우에만)
        if !self.config.bootstrap_nodes.is_empty() {
            log_network!("🚀 Connecting to bootstrap nodes");
            let bootstrap_peers = self.bootstrap.connect_bootstrap().await;
            peers.extend(bootstrap_peers.clone());

            // Bootstrap 노드로부터 피어 목록 받기
            for addr in &bootstrap_peers {
                let more_peers = self.bootstrap.exchange_peers(*addr).await;
                for peer in more_peers {
                    if !peers.contains(&peer.addr) {
                        peers.push(peer.addr);

                        // DHT에 추가
                        if let Some(dht) = &self.dht {
                            let node = Node {
                                id: NodeId::from_addr(&peer.addr),
                                addr: peer.addr,
                                last_seen: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                            };
                            dht.add_node(node).await;
                        }
                    }
                }
            }
        }

        // 3. DHT를 통한 추가 발견
        if let Some(dht) = &self.dht {
            log_network!("🔍 Using DHT for peer discovery");
            let closest = dht.find_closest_nodes(&self.node_id, 10).await;
            for node in closest {
                if !peers.contains(&node.addr) {
                    peers.push(node.addr);
                }
            }
        }

        // 최대 피어 수 제한
        peers.truncate(self.config.max_peers);

        let peer_count = peers.len();
        log_network!("✅ Discovered {} peers", peer_count);
        peers
    }

    pub async fn add_peer(&self, addr: SocketAddr) {
        let peer = PeerInfo {
            addr,
            node_id: NodeId::from_addr(&addr).0,
            last_seen: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Bootstrap에 추가
        self.bootstrap.add_peer(peer.clone()).await;

        // DHT에 추가
        if let Some(dht) = &self.dht {
            let node = Node {
                id: NodeId::from_addr(&addr),
                addr,
                last_seen: peer.last_seen,
            };
            dht.add_node(node).await;
        }

        // 로컬 목록에 추가
        let mut peers = self.discovered_peers.write().await;
        if !peers.iter().any(|p| p.addr == addr) {
            peers.push(peer);
        }
    }

    pub async fn get_peers(&self) -> Vec<PeerInfo> {
        self.discovered_peers.read().await.clone()
    }

    pub async fn find_node(&self, target: NodeId) -> Vec<SocketAddr> {
        if let Some(dht) = &self.dht {
            let nodes = dht.lookup(target).await;
            nodes.into_iter().map(|n| n.addr).collect()
        } else {
            // DHT가 없으면 bootstrap 피어 반환
            self.bootstrap
                .get_peers()
                .await
                .into_iter()
                .map(|p| p.addr)
                .collect()
        }
    }

    pub fn get_node_id(&self) -> NodeId {
        self.node_id
    }
}
