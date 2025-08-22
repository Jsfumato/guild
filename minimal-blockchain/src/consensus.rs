// 간단한 컨센서스 엔진
use crate::types::{BlockVote, MinimalBlock};
use std::collections::HashMap;

/// 초간단 PBFT 컨센서스
pub struct SimpleConsensus {
    pub node_id: [u8; 32],
    validators: Vec<[u8; 32]>,
    current_height: u64,
    current_round: u32,
    votes: HashMap<u64, Vec<BlockVote>>,
    last_block: Option<MinimalBlock>,
    committed_blocks: Vec<MinimalBlock>,
}

impl SimpleConsensus {
    /// 새 컨센서스 엔진 생성
    pub fn new(node_id: [u8; 32]) -> Self {
        Self {
            node_id,
            validators: vec![node_id], // 처음엔 자신만
            current_height: 0,
            current_round: 0,
            votes: HashMap::new(),
            last_block: None,
            committed_blocks: Vec::new(),
        }
    }
    
    /// 검증자 추가
    pub fn add_validator(&mut self, validator: [u8; 32]) {
        if !self.validators.contains(&validator) {
            self.validators.push(validator);
            println!("✅ 검증자 추가됨. 총 {}명", self.validators.len());
        }
    }
    
    /// 검증자 제거
    pub fn remove_validator(&mut self, validator: [u8; 32]) {
        self.validators.retain(|v| *v != validator);
        println!("✅ 검증자 제거됨. 총 {}명", self.validators.len());
    }
    
    /// 내 차례인지 확인 (라운드 로빈)
    pub fn is_my_turn(&self) -> bool {
        if self.validators.is_empty() {
            return false;
        }
        
        let proposer = self.get_proposer(self.current_height);
        proposer == self.node_id
    }
    
    /// 현재 높이의 제안자 결정
    pub fn get_proposer(&self, height: u64) -> [u8; 32] {
        if self.validators.is_empty() {
            return [0u8; 32];
        }
        
        let index = (height as usize) % self.validators.len();
        self.validators[index]
    }
    
    /// 다음 블록 높이
    pub fn get_next_height(&self) -> u64 {
        self.current_height
    }
    
    /// 마지막 블록 해시
    pub fn get_last_hash(&self) -> [u8; 32] {
        self.last_block
            .as_ref()
            .map(|b| b.hash())
            .unwrap_or([0u8; 32])
    }
    
    /// 투표 생성
    pub fn create_vote(&self, block: &MinimalBlock) -> BlockVote {
        BlockVote::new(block, self.node_id)
    }
    
    /// 투표 추가
    pub fn add_vote(&mut self, vote: BlockVote) {
        let quorum_size = self.get_quorum_size();
        let vote_height = vote.height;
        let votes = self.votes.entry(vote.height).or_insert_with(Vec::new);
        
        // 중복 투표 방지
        if !votes.iter().any(|v| v.voter == vote.voter) {
            votes.push(vote);
            println!("📝 투표 수집: {}/{} (높이: {})", 
                votes.len(), 
                quorum_size,
                vote_height
            );
        }
    }
    
    /// 정족수 확인 (2/3 + 1)
    pub fn check_quorum(&self, height: u64) -> bool {
        let votes = self.votes.get(&height).map(|v| v.len()).unwrap_or(0);
        votes >= self.get_quorum_size()
    }
    
    /// 정족수 크기
    fn get_quorum_size(&self) -> usize {
        if self.validators.is_empty() {
            return 1;
        }
        (self.validators.len() * 2) / 3 + 1
    }
    
    /// 블록 확정
    pub fn finalize_block(&mut self, height: u64) {
        self.current_height = height + 1;
        self.votes.remove(&height);
        println!("⏫ 다음 블록 높이: {}", self.current_height);
    }
    
    /// 블록 커밋
    pub fn commit_block(&mut self, block: MinimalBlock) {
        self.last_block = Some(block.clone());
        self.committed_blocks.push(block);
        
        // 메모리 관리 (최근 100개만 유지)
        if self.committed_blocks.len() > 100 {
            self.committed_blocks.remove(0);
        }
    }
    
    /// 통계 출력
    pub fn print_stats(&self) {
        println!("📊 컨센서스 상태:");
        println!("  - 현재 높이: {}", self.current_height);
        println!("  - 검증자 수: {}", self.validators.len());
        println!("  - 커밋된 블록: {}", self.committed_blocks.len());
    }
}