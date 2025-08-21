// Guild Home Discovery - mDNS 기반 로컬 네트워크 탐색
use mdns_sd::{ServiceDaemon, ServiceInfo, ServiceEvent};
use std::sync::Arc;
use std::collections::HashSet;
use tokio::sync::RwLock;
use super::network::Network;

const SERVICE_TYPE: &str = "_guildhome._udp.local.";

pub struct Discovery {
    network: Arc<Network>,
    discovered: Arc<RwLock<HashSet<String>>>,
}

impl Discovery {
    pub fn new(network: Arc<Network>) -> Self {
        Self::with_bootstrap(network, vec![])
    }
    
    pub fn with_bootstrap(network: Arc<Network>, bootstrap: Vec<String>) -> Self {
        let mut discovered = HashSet::new();
        // 부트스트랩 노드를 미리 추가
        for peer in &bootstrap {
            discovered.insert(peer.clone());
        }
        
        Self {
            network,
            discovered: Arc::new(RwLock::new(discovered)),
        }
    }
    
    pub async fn start(self) {
        // mDNS 서비스 시작
        let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");
        
        // 자신을 광고
        let my_service = ServiceInfo::new(
            SERVICE_TYPE,
            "guildhome",
            &format!("guild-{}", uuid::Uuid::new_v4()),
            "",
            8000,  // 포트 (실제로는 QUIC가 자동 할당)
            None,
        ).unwrap();
        
        mdns.register(my_service).expect("Failed to register mDNS service");
        
        // 브라우저 시작 (다른 노드 찾기)
        let receiver = mdns.browse(SERVICE_TYPE).expect("Failed to browse");
        
        // 발견된 서비스 처리
        let network = self.network.clone();
        let discovered = self.discovered.clone();
        
        tokio::spawn(async move {
            while let Ok(event) = receiver.recv_async().await {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        let addr = format!("{}:{}", 
                            info.get_addresses().iter().next().unwrap(),
                            info.get_port()
                        );
                        
                        // 중복 연결 방지
                        if discovered.read().await.contains(&addr) {
                            continue;
                        }
                        
                        println!("🔍 Discovered peer: {}", addr);
                        discovered.write().await.insert(addr.clone());
                        
                        // 연결 시도
                        if let Ok(socket_addr) = addr.parse() {
                            let _ = network.connect(socket_addr).await;
                        }
                    }
                    ServiceEvent::ServiceRemoved(_, _) => {
                        // 피어가 오프라인
                    }
                    _ => {}
                }
            }
        });
        
        // 부트스트랩 노드 연결
        self.connect_bootstrap().await;
    }
    
    async fn connect_bootstrap(&self) {
        // 사용자가 제공한 부트스트랩 노드
        let discovered = self.discovered.read().await;
        for peer in discovered.iter() {
            if let Ok(addr) = peer.parse() {
                let _ = self.network.connect(addr).await;
            }
        }
    }
}

// 간단한 DHT 구현 (Kademlia 스타일)
pub struct SimpleDHT {
    node_id: [u8; 32],
    routing_table: Vec<Vec<String>>,  // K-buckets
}

impl SimpleDHT {
    pub fn new() -> Self {
        // 노드 ID = 공개키 해시
        let node_id = blake3::hash(b"random_seed").as_bytes().clone();
        
        Self {
            node_id,
            routing_table: vec![Vec::new(); 256],  // 256 buckets
        }
    }
    
    pub fn add_peer(&mut self, peer_id: [u8; 32], addr: String) {
        let distance = self.xor_distance(&self.node_id, &peer_id);
        let bucket = distance.leading_zeros() as usize;
        
        if bucket < 256 {
            self.routing_table[bucket].push(addr);
            // K=20 제한 (버킷당 최대 20개 노드)
            if self.routing_table[bucket].len() > 20 {
                self.routing_table[bucket].pop();
            }
        }
    }
    
    fn xor_distance(&self, a: &[u8; 32], b: &[u8; 32]) -> u128 {
        let mut result = 0u128;
        for i in 0..16 {
            result |= ((a[i] ^ b[i]) as u128) << (i * 8);
        }
        result
    }
}