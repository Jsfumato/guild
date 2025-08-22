// 최소 블록체인 - Guild-Home 네트워크 사용
mod types;
mod consensus;
mod ipc;

use consensus::SimpleConsensus;
use ipc::{IPCClient, IPCMessage};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{interval, Duration};
use types::{ConsensusMessage, MinimalBlock};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    guild_logger::init_logger(false);
    
    println!("⛓️ 최소 블록체인 시작...");
    
    // 명령줄 인자 파싱
    let args: Vec<String> = env::args().collect();
    let mut ipc_port: u16 = 9000; // 기본 포트
    
    // --port 플래그 처리
    for i in 0..args.len() {
        if args[i] == "--port" || args[i] == "-p" {
            if i + 1 < args.len() {
                ipc_port = args[i + 1].parse().unwrap_or_else(|_| {
                    eprintln!("❌ 잘못된 포트 번호: {}", args[i + 1]);
                    eprintln!("사용법: {} [--port <PORT>]", args[0]);
                    std::process::exit(1);
                });
            } else {
                eprintln!("❌ --port 플래그 뒤에 포트 번호가 필요합니다");
                eprintln!("사용법: {} [--port <PORT>]", args[0]);
                std::process::exit(1);
            }
        }
        if args[i] == "--help" || args[i] == "-h" {
            println!("사용법: {} [옵션]", args[0]);
            println!("옵션:");
            println!("  -p, --port <PORT>    IPC 포트 설정 (기본값: 9000)");
            println!("  -h, --help           도움말 표시");
            std::process::exit(0);
        }
    }
    
    // 환경 변수 우선순위 (환경 변수가 있으면 덮어씀)
    if let Ok(env_port) = env::var("IPC_PORT") {
        if let Ok(port) = env_port.parse::<u16>() {
            ipc_port = port;
        }
    }
    
    // 노드 ID 생성
    let node_id = generate_node_id();
    println!("📍 노드 ID: {}", hex::encode(&node_id));
    
    // IPC 연결 (Guild-Home에 연결)
    println!("🔌 Guild-Home 연결 중... (포트: {})", ipc_port);
    let mut ipc = IPCClient::connect(ipc_port).await?;
    println!("✅ Guild-Home 연결 성공!");
    
    // 컨센서스 엔진 초기화
    let mut consensus = SimpleConsensus::new(node_id);
    
    // 블록 생성 타이머 (1초마다)
    let mut block_timer = interval(Duration::from_secs(1));
    
    println!("🚀 블록체인 실행 중...\n");
    
    loop {
        tokio::select! {
            // 블록 생성 시간
            _ = block_timer.tick() => {
                // 내 차례인지 확인
                if consensus.is_my_turn() {
                    let block = create_block(&consensus);
                    println!("📦 블록 제안: #{}", block.height);
                    
                    let msg = ConsensusMessage::Propose(block);
                    let data = bincode::serialize(&msg)?;
                    
                    // Guild-Home을 통해 브로드캐스트
                    ipc.broadcast(data).await?;
                }
            }
            
            // Guild-Home으로부터 메시지 수신
            Ok(msg) = ipc.recv() => {
                handle_ipc_message(&mut consensus, &mut ipc, msg).await?;
            }
        }
    }
}

// IPC 메시지 처리
async fn handle_ipc_message(
    consensus: &mut SimpleConsensus,
    ipc: &mut IPCClient,
    msg: IPCMessage,
) -> Result<(), Box<dyn std::error::Error>> {
    match msg {
        IPCMessage::PeerMessage { from, data } => {
            // 컨센서스 메시지 역직렬화
            if let Ok(consensus_msg) = bincode::deserialize::<ConsensusMessage>(&data) {
                match consensus_msg {
                    ConsensusMessage::Propose(block) => {
                        println!("📨 블록 제안 받음: #{} from {:?}", block.height, from);
                        
                        // 블록 검증
                        if validate_block(&block, consensus) {
                            // 투표 생성
                            let vote = consensus.create_vote(&block);
                            let vote_msg = ConsensusMessage::Vote(vote);
                            let vote_data = bincode::serialize(&vote_msg)?;
                            
                            // 투표 브로드캐스트
                            ipc.broadcast(vote_data).await?;
                            println!("✅ 블록 #{} 투표 완료", block.height);
                        }
                    }
                    
                    ConsensusMessage::Vote(vote) => {
                        // 투표 수집
                        consensus.add_vote(vote.clone());
                        
                        // 정족수 확인
                        if consensus.check_quorum(vote.height) {
                            println!("🎉 블록 #{} 확정! (2/3 투표 달성)", vote.height);
                            consensus.finalize_block(vote.height);
                        }
                    }
                    
                    ConsensusMessage::Commit(block) => {
                        println!("📗 블록 #{} 커밋됨", block.height);
                        consensus.commit_block(block);
                    }
                }
            }
        }
        
        IPCMessage::PeerJoined(peer) => {
            println!("👋 새 피어 참가: {:?}", peer);
            consensus.add_validator(peer);
        }
        
        IPCMessage::PeerLeft(peer) => {
            println!("👋 피어 떠남: {:?}", peer);
            consensus.remove_validator(peer);
        }
        
        _ => {}
    }
    
    Ok(())
}

// 블록 생성
fn create_block(consensus: &SimpleConsensus) -> MinimalBlock {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    MinimalBlock {
        height: consensus.get_next_height(),
        timestamp,
        prev_hash: consensus.get_last_hash(),
        proposer: consensus.node_id,
        data: vec![], // 빈 데이터 (필요시 추가)
    }
}

// 블록 검증
fn validate_block(block: &MinimalBlock, consensus: &SimpleConsensus) -> bool {
    // 기본 검증
    if block.height != consensus.get_next_height() {
        println!("❌ 잘못된 높이: {} (예상: {})", block.height, consensus.get_next_height());
        return false;
    }
    
    if block.prev_hash != consensus.get_last_hash() {
        println!("❌ 잘못된 이전 해시");
        return false;
    }
    
    // 제안자 검증
    let expected_proposer = consensus.get_proposer(block.height);
    if block.proposer != expected_proposer {
        println!("❌ 잘못된 제안자");
        return false;
    }
    
    true
}

// 노드 ID 생성
fn generate_node_id() -> [u8; 32] {
    use sha2::{Digest, Sha256};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    
    let mut hasher = Sha256::new();
    hasher.update(timestamp.to_be_bytes());
    
    let result = hasher.finalize();
    let mut id = [0u8; 32];
    id.copy_from_slice(&result);
    id
}