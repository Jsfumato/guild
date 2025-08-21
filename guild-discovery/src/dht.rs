// Kademlia DHT 구현
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

pub const K_BUCKET_SIZE: usize = 20;  // 각 버킷의 최대 노드 수
pub const ALPHA: usize = 3;           // 동시 조회 수
pub const NODE_ID_LENGTH: usize = 32; // 256 bits

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub [u8; NODE_ID_LENGTH]);

impl NodeId {
    pub fn new(data: [u8; NODE_ID_LENGTH]) -> Self {
        NodeId(data)
    }
    
    pub fn random() -> Self {
        let mut id = [0u8; NODE_ID_LENGTH];
        rand::Rng::fill(&mut rand::thread_rng(), &mut id);
        NodeId(id)
    }
    
    pub fn from_addr(addr: &SocketAddr) -> Self {
        let hash = blake3::hash(addr.to_string().as_bytes());
        NodeId(*hash.as_bytes())
    }
    
    pub fn distance(&self, other: &NodeId) -> Distance {
        let mut dist = [0u8; NODE_ID_LENGTH];
        for i in 0..NODE_ID_LENGTH {
            dist[i] = self.0[i] ^ other.0[i];
        }
        Distance(dist)
    }
    
    pub fn common_prefix_len(&self, other: &NodeId) -> usize {
        let dist = self.distance(other);
        dist.leading_zeros()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Distance([u8; NODE_ID_LENGTH]);

impl Distance {
    pub fn leading_zeros(&self) -> usize {
        for (i, &byte) in self.0.iter().enumerate() {
            if byte != 0 {
                return i * 8 + byte.leading_zeros() as usize;
            }
        }
        NODE_ID_LENGTH * 8
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub addr: SocketAddr,
    pub last_seen: u64,
}

pub struct KBucket {
    nodes: Vec<Node>,
    max_size: usize,
}

impl KBucket {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            max_size: K_BUCKET_SIZE,
        }
    }
    
    pub fn add_node(&mut self, node: Node) -> bool {
        // 이미 있으면 업데이트
        if let Some(pos) = self.nodes.iter().position(|n| n.id == node.id) {
            self.nodes.remove(pos);
            self.nodes.push(node);
            return true;
        }
        
        // 공간이 있으면 추가
        if self.nodes.len() < self.max_size {
            self.nodes.push(node);
            return true;
        }
        
        // 버킷이 가득 차면 가장 오래된 노드 교체 고려
        // 실제로는 핑을 보내서 응답 없으면 교체
        false
    }
    
    pub fn get_nodes(&self) -> Vec<Node> {
        self.nodes.clone()
    }
    
    pub fn remove_node(&mut self, id: &NodeId) {
        self.nodes.retain(|n| n.id != *id);
    }
}

pub struct Kademlia {
    node_id: NodeId,
    k_buckets: Arc<RwLock<Vec<KBucket>>>,
}

impl Kademlia {
    pub fn new(node_id: NodeId) -> Self {
        let mut buckets = Vec::with_capacity(NODE_ID_LENGTH * 8);
        for _ in 0..NODE_ID_LENGTH * 8 {
            buckets.push(KBucket::new());
        }
        
        Self {
            node_id,
            k_buckets: Arc::new(RwLock::new(buckets)),
        }
    }
    
    pub async fn add_node(&self, node: Node) {
        let distance = self.node_id.distance(&node.id);
        let bucket_idx = distance.leading_zeros();
        
        if bucket_idx < NODE_ID_LENGTH * 8 {
            let mut buckets = self.k_buckets.write().await;
            buckets[bucket_idx].add_node(node);
        }
    }
    
    pub async fn find_closest_nodes(&self, target: &NodeId, count: usize) -> Vec<Node> {
        let mut all_nodes = Vec::new();
        let buckets = self.k_buckets.read().await;
        
        for bucket in buckets.iter() {
            all_nodes.extend(bucket.get_nodes());
        }
        
        // 타겟과의 거리로 정렬
        all_nodes.sort_by_key(|node| {
            let dist = node.id.distance(target);
            dist.0
        });
        
        all_nodes.truncate(count);
        all_nodes
    }
    
    pub async fn lookup(&self, target: NodeId) -> Vec<Node> {
        // FIND_NODE 연산 구현
        // 1. 자신의 k-bucket에서 가장 가까운 α개 노드 선택
        // 2. 이 노드들에게 FIND_NODE 요청
        // 3. 응답으로 받은 노드들 중 더 가까운 노드가 있으면 반복
        
        self.find_closest_nodes(&target, ALPHA).await
    }
    
    pub fn get_node_id(&self) -> NodeId {
        self.node_id
    }
}