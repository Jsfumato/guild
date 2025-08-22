// 최소 블록체인 타입 정의
use serde::{Deserialize, Serialize};

/// 최소 블록 구조 (지갑, 트랜잭션 없음)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimalBlock {
    pub height: u64,           // 블록 높이
    pub timestamp: u64,        // 생성 시간
    pub prev_hash: [u8; 32],   // 이전 블록 해시
    pub proposer: [u8; 32],    // 제안자 ID
    pub data: Vec<u8>,         // 임의 데이터 (옵션)
}

impl MinimalBlock {
    /// 블록 해시 계산
    pub fn hash(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        
        let mut hasher = Sha256::new();
        hasher.update(self.height.to_be_bytes());
        hasher.update(self.timestamp.to_be_bytes());
        hasher.update(self.prev_hash);
        hasher.update(self.proposer);
        hasher.update(&self.data);
        
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

/// 컨센서스 메시지
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusMessage {
    /// 블록 제안
    Propose(MinimalBlock),
    /// 블록 투표
    Vote(BlockVote),
    /// 블록 확정
    Commit(MinimalBlock),
}

/// 블록 투표
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockVote {
    pub height: u64,            // 블록 높이
    pub block_hash: [u8; 32],   // 블록 해시
    pub voter: [u8; 32],        // 투표자 ID
    pub signature: Vec<u8>,     // 간단한 서명 (실제로는 더미)
}

impl BlockVote {
    /// 투표 생성
    pub fn new(block: &MinimalBlock, voter: [u8; 32]) -> Self {
        let block_hash = block.hash();
        
        // 간단한 서명 (실제로는 암호화 필요)
        let mut signature = vec![];
        signature.extend_from_slice(&block_hash);
        signature.extend_from_slice(&voter);
        
        Self {
            height: block.height,
            block_hash,
            voter,
            signature,
        }
    }
}

/// 피어 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: [u8; 32],
    pub address: String,
}