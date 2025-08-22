// 블록체인 브리지 - Guild-Home과 블록체인 프로세스 연결
use crate::network::Network;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};

pub type PeerId = [u8; 32];

/// IPC 메시지
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IPCMessage {
    // Guild-Home -> Blockchain
    PeerMessage { from: PeerId, data: Vec<u8> },
    PeerJoined(PeerId),
    PeerLeft(PeerId),
    
    // Blockchain -> Guild-Home
    Broadcast(Vec<u8>),
    SendTo { peer: PeerId, data: Vec<u8> },
}

/// 블록체인 브리지
pub struct BlockchainBridge {
    network: Arc<Network>,
    ipc_listener: Option<TcpListener>,
    peer_map: Arc<RwLock<HashMap<SocketAddr, PeerId>>>,
    ipc_port: u16,
}

impl BlockchainBridge {
    /// IPC 포트 가져오기
    pub fn get_ipc_port(&self) -> u16 {
        self.ipc_port
    }
    /// 새 브리지 생성
    pub fn new(network: Arc<Network>) -> Self {
        Self {
            network,
            ipc_listener: None,
            peer_map: Arc::new(RwLock::new(HashMap::new())),
            ipc_port: 0,
        }
    }
    
    /// 브리지 시작
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        guild_logger::log_info!("🌉 블록체인 브리지 시작...");
        
        // IPC 서버 시작 (P2P 포트 + 1 사용)
        let p2p_port = self.network.local_port();
        let ipc_port = p2p_port + 1;
        self.ipc_port = ipc_port;
        self.ipc_listener = Some(TcpListener::bind(("127.0.0.1", ipc_port)).await?);
        guild_logger::log_info!("📡 IPC 서버 대기 중: 포트 {}", ipc_port);
        guild_logger::log_info!("💡 minimal-blockchain을 다음 명령으로 실행하세요:");
        guild_logger::log_info!("   cargo run --bin minimal-blockchain -- --port {}", ipc_port);
        
        // IPC 연결 대기 및 메시지 라우팅
        if let Some(listener) = self.ipc_listener.take() {
            let network = self.network.clone();
            let peer_map = self.peer_map.clone();
            
            tokio::spawn(async move {
                Self::handle_ipc_connections(listener, network, peer_map).await;
            });
        }
        
        Ok(())
    }
    
    /// IPC 연결 처리
    async fn handle_ipc_connections(
        listener: TcpListener,
        network: Arc<Network>,
        peer_map: Arc<RwLock<HashMap<SocketAddr, PeerId>>>,
    ) {
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    guild_logger::log_info!("📞 블록체인 프로세스 연결: {}", addr);
                    
                    let net = network.clone();
                    let map = peer_map.clone();
                    
                    tokio::spawn(async move {
                        Self::handle_blockchain_connection(stream, net, map).await;
                    });
                }
                Err(e) => {
                    guild_logger::log_error!("IPC 연결 실패: {}", e);
                }
            }
        }
    }
    
    /// 블록체인 연결 처리
    async fn handle_blockchain_connection(
        mut stream: TcpStream,
        network: Arc<Network>,
        peer_map: Arc<RwLock<HashMap<SocketAddr, PeerId>>>,
    ) {
        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(100);
        
        // 네트워크 메시지 수신 태스크
        let _net_clone = network.clone();
        let _tx_clone = tx.clone();
        tokio::spawn(async move {
            // 네트워크에서 메시지 수신하여 블록체인으로 전달
            // (실제 구현 필요)
        });
        
        loop {
            tokio::select! {
                // 블록체인에서 메시지 수신
                result = Self::read_message(&mut stream) => {
                    match result {
                        Ok(msg) => {
                            Self::handle_blockchain_message(msg, &network, &peer_map).await;
                        }
                        Err(e) => {
                            let err_msg = e.to_string();
                    guild_logger::log_error!("블록체인 연결 끊김: {}", err_msg);
                            break;
                        }
                    }
                }
                
                // 네트워크 메시지를 블록체인으로 전달
                Some(data) = rx.recv() => {
                    if let Err(e) = Self::write_message(&mut stream, &data).await {
                        let err_msg = e.to_string();
                        guild_logger::log_error!("블록체인 전송 실패: {}", err_msg);
                        break;
                    }
                }
            }
        }
    }
    
    /// 메시지 읽기
    async fn read_message(stream: &mut TcpStream) -> Result<IPCMessage, Box<dyn std::error::Error + Send + Sync>> {
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        
        let mut buffer = vec![0u8; len];
        stream.read_exact(&mut buffer).await?;
        
        let msg = bincode::deserialize(&buffer)?;
        Ok(msg)
    }
    
    /// 메시지 쓰기
    async fn write_message(stream: &mut TcpStream, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let len = data.len() as u32;
        stream.write_all(&len.to_be_bytes()).await?;
        stream.write_all(data).await?;
        stream.flush().await?;
        Ok(())
    }
    
    /// 블록체인 메시지 처리
    async fn handle_blockchain_message(
        msg: IPCMessage,
        network: &Arc<Network>,
        _peer_map: &Arc<RwLock<HashMap<SocketAddr, PeerId>>>,
    ) {
        match msg {
            IPCMessage::Broadcast(data) => {
                let data_len = data.len();
                guild_logger::log_network!("📢 블록체인 브로드캐스트: {} bytes", data_len);
                
                // 모든 피어에게 전송
                network.broadcast(&data).await;
            }
            
            IPCMessage::SendTo { peer, data } => {
                let data_len = data.len();
                guild_logger::log_network!("📤 블록체인 메시지 전송: {} bytes", data_len);
                
                // 특정 피어에게 전송 (현재는 브로드캐스트로 대체)
                // TODO: 특정 피어에게만 전송하는 기능 구현 필요
                let _ = peer; // unused warning 제거
                network.broadcast(&data).await;
            }
            
            _ => {}
        }
    }
    
    /// 피어 ID로 주소 찾기
    async fn find_peer_address(
        peer_map: &Arc<RwLock<HashMap<SocketAddr, PeerId>>>,
        peer_id: PeerId,
    ) -> Option<SocketAddr> {
        let map = peer_map.read().await;
        map.iter()
            .find(|(_, id)| **id == peer_id)
            .map(|(addr, _)| *addr)
    }
    
}