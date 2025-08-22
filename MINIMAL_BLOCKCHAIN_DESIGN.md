# 최소 블록체인 설계

## 개요

Guild-Home의 네트워크 레이어를 활용하는 초경량 블록체인 시스템

## 핵심 설계 원칙

1. **네트워크 격리**: Guild-Home만 외부 네트워크 접근
2. **프로세스 분리**: 블록체인은 독립 프로세스로 실행
3. **최소 기능**: 지갑 없음, 스토리지 없음, 순수 컨센서스만
4. **IPC 통신**: TCP localhost를 통한 프로세스 간 통신

## 컴포넌트 설계

### 1. 최소 블록 구조

```rust
// 지갑, 트랜잭션 없는 순수 블록
pub struct MinimalBlock {
    pub height: u64,           // 블록 높이
    pub timestamp: u64,        // 생성 시간
    pub prev_hash: [u8; 32],   // 이전 블록 해시
    pub proposer: [u8; 32],    // 제안자 ID
    pub data: Vec<u8>,         // 임의 데이터 (옵션)
}

// 컨센서스 메시지
pub enum ConsensusMessage {
    Propose(MinimalBlock),     // 블록 제안
    Vote(BlockVote),          // 투표
    Commit(MinimalBlock),     // 확정
}

pub struct BlockVote {
    pub height: u64,
    pub block_hash: [u8; 32],
    pub voter: [u8; 32],
    pub signature: Vec<u8>,   // 간단한 서명
}
```

### 2. IPC 프로토콜

```rust
// Guild-Home <-> Blockchain 통신
pub enum IPCMessage {
    // Guild-Home -> Blockchain
    PeerMessage {
        from: PeerId,
        data: Vec<u8>,
    },
    PeerJoined(PeerId),
    PeerLeft(PeerId),
    
    // Blockchain -> Guild-Home
    Broadcast(Vec<u8>),
    SendTo {
        peer: PeerId,
        data: Vec<u8>,
    },
}
```

### 3. 컨센서스 엔진 (초간단 PBFT)

```rust
pub struct SimpleConsensus {
    node_id: [u8; 32],
    validators: Vec<[u8; 32]>,
    current_height: u64,
    current_round: u32,
    votes: HashMap<u64, Vec<BlockVote>>,
}

impl SimpleConsensus {
    // 라운드 로빈 제안자 선택
    fn get_proposer(&self, height: u64) -> [u8; 32] {
        let index = (height as usize) % self.validators.len();
        self.validators[index]
    }
    
    // 2/3 + 1 투표 확인
    fn check_quorum(&self, height: u64) -> bool {
        let votes = self.votes.get(&height).unwrap_or(&vec![]);
        votes.len() > (self.validators.len() * 2) / 3
    }
}
```

## 구현 구조

### Guild-Home 측 (네트워크 브리지)

```rust
// guild-home/src/blockchain_bridge.rs

pub struct BlockchainBridge {
    network: Arc<Network>,
    ipc_client: IPCConnection,
}

impl BlockchainBridge {
    pub async fn start(&mut self) {
        // 블록체인 프로세스 시작
        let child = Command::new("./minimal-blockchain")
            .env("IPC_PORT", "9000")
            .spawn()?;
        
        // IPC 연결
        self.ipc_client = IPCConnection::connect(9000).await?;
        
        // 메시지 라우팅 시작
        self.route_messages().await;
    }
    
    async fn route_messages(&mut self) {
        loop {
            tokio::select! {
                // 네트워크 -> 블록체인
                Some(msg) = self.network.recv() => {
                    let ipc_msg = IPCMessage::PeerMessage {
                        from: msg.sender,
                        data: msg.data,
                    };
                    self.ipc_client.send(ipc_msg).await;
                }
                
                // 블록체인 -> 네트워크
                Some(msg) = self.ipc_client.recv() => {
                    match msg {
                        IPCMessage::Broadcast(data) => {
                            self.network.broadcast(data).await;
                        }
                        IPCMessage::SendTo { peer, data } => {
                            self.network.send_to(peer, data).await;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
```

### 블록체인 프로세스 (별도 실행)

```rust
// minimal-blockchain/src/main.rs

#[tokio::main]
async fn main() {
    // IPC 서버 시작
    let ipc_port = env::var("IPC_PORT").unwrap_or("9000".to_string());
    let mut ipc = IPCServer::bind(ipc_port).await?;
    
    // 컨센서스 엔진 초기화
    let mut consensus = SimpleConsensus::new(node_id());
    
    // 블록 생성 타이머 (1초마다)
    let mut interval = interval(Duration::from_secs(1));
    
    loop {
        tokio::select! {
            // 블록 생성 시간
            _ = interval.tick() => {
                if consensus.is_my_turn() {
                    let block = consensus.create_block();
                    let msg = ConsensusMessage::Propose(block);
                    
                    // 모든 피어에게 제안 전송
                    ipc.broadcast(serialize(&msg)).await;
                }
            }
            
            // 피어 메시지 처리
            Some(msg) = ipc.recv() => {
                match msg {
                    IPCMessage::PeerMessage { from, data } => {
                        if let Ok(consensus_msg) = deserialize(&data) {
                            handle_consensus_message(consensus_msg).await;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn handle_consensus_message(msg: ConsensusMessage) {
    match msg {
        ConsensusMessage::Propose(block) => {
            // 블록 검증
            if validate_block(&block) {
                // 투표 생성 및 전송
                let vote = create_vote(&block);
                ipc.broadcast(serialize(&vote)).await;
            }
        }
        
        ConsensusMessage::Vote(vote) => {
            // 투표 수집
            consensus.add_vote(vote);
            
            // 정족수 확인
            if consensus.check_quorum(vote.height) {
                // 블록 확정
                let block = consensus.finalize_block(vote.height);
                println!("✅ 블록 #{} 확정!", block.height);
            }
        }
        
        _ => {}
    }
}
```

## 실행 방법

### 1. Guild-Home 시작 (네트워크 레이어)

```bash
# Terminal 1
cd guild-home
cargo run -- --enable-blockchain-bridge
```

### 2. 블록체인 프로세스 실행

```bash
# Terminal 2
cd minimal-blockchain
cargo build --release
./target/release/minimal-blockchain
```

### 3. 추가 노드 연결

```bash
# Terminal 3 (다른 머신)
cd guild-home
cargo run -- --bootstrap 192.168.1.100:8000 --enable-blockchain-bridge

# Terminal 4
cd minimal-blockchain
./target/release/minimal-blockchain
```

## 메시지 흐름

```
1. 블록 제안:
   Proposer -> IPC -> Guild-Home -> Network -> 모든 피어

2. 투표:
   Validator -> IPC -> Guild-Home -> Network -> 모든 피어

3. 블록 확정:
   2/3 투표 수집 -> 블록 확정 -> 로컬 상태 업데이트
```

## 장점

1. **격리**: 네트워크와 컨센서스 로직 완전 분리
2. **단순함**: 지갑, 스토리지 없는 순수 컨센서스
3. **확장성**: 다른 컨센서스 알고리즘으로 쉽게 교체
4. **안정성**: 블록체인 크래시가 네트워크에 영향 없음
5. **디버깅**: 각 컴포넌트 독립적 테스트 가능

## 최소 요구사항

- Rust 1.70+
- tokio 1.0+
- 로컬 TCP 포트 (9000-9100)
- 메모리: ~50MB per 프로세스