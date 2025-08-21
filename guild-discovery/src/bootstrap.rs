// Bootstrap 기반 피어 발견
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub addr: SocketAddr,
    pub node_id: [u8; 32],
    pub last_seen: u64,
}

pub struct Bootstrap {
    bootstrap_nodes: Vec<String>,
    known_peers: Arc<RwLock<Vec<PeerInfo>>>,
}

impl Bootstrap {
    pub fn new(bootstrap_nodes: Vec<String>) -> Self {
        Self {
            bootstrap_nodes,
            known_peers: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub fn with_defaults() -> Self {
        // 기본 부트스트랩 노드들 (실제 서비스에서는 여러 지역의 안정적인 노드들)
        let defaults = vec![
            "bootstrap1.guild.network:9000".to_string(),
            "bootstrap2.guild.network:9000".to_string(),
            "bootstrap3.guild.network:9000".to_string(),
        ];
        Self::new(defaults)
    }
    
    pub async fn connect_bootstrap(&self) -> Vec<SocketAddr> {
        let mut connected = Vec::new();
        
        for node in &self.bootstrap_nodes {
            if let Ok(addr) = node.parse::<SocketAddr>() {
                // 실제로는 여기서 연결 시도
                connected.push(addr);
                println!("🔗 Bootstrap node: {}", addr);
            }
        }
        
        connected
    }
    
    pub async fn exchange_peers(&self, _remote_addr: SocketAddr) -> Vec<PeerInfo> {
        // 부트스트랩 노드와 피어 목록 교환
        // 실제로는 네트워크 통신으로 구현
        let peers = self.known_peers.read().await;
        peers.clone()
    }
    
    pub async fn add_peer(&self, peer: PeerInfo) {
        let mut peers = self.known_peers.write().await;
        
        // 중복 체크
        if !peers.iter().any(|p| p.addr == peer.addr) {
            peers.push(peer);
            
            // 최대 1000개 피어 유지
            if peers.len() > 1000 {
                peers.remove(0);
            }
        }
    }
    
    pub async fn get_peers(&self) -> Vec<PeerInfo> {
        self.known_peers.read().await.clone()
    }
    
    pub async fn cleanup_stale_peers(&self, max_age_secs: u64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut peers = self.known_peers.write().await;
        peers.retain(|p| now - p.last_seen < max_age_secs);
    }
}