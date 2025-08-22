// IPC 통신 (Guild-Home과 통신)
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub type PeerId = [u8; 32];

/// IPC 메시지 (Guild-Home <-> Blockchain)
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// IPC 클라이언트 (블록체인이 Guild-Home에 연결)
pub struct IPCClient {
    stream: TcpStream,
}

impl IPCClient {
    /// Guild-Home IPC 서버에 연결
    pub async fn connect(port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        println!("🔌 IPC 연결 시도: localhost:{}", port);
        let stream = TcpStream::connect(("127.0.0.1", port)).await?;
        Ok(Self { stream })
    }
    
    /// 메시지 전송
    pub async fn send(&mut self, msg: IPCMessage) -> Result<(), Box<dyn std::error::Error>> {
        let data = bincode::serialize(&msg)?;
        let len = data.len() as u32;
        
        // 길이 프리픽스 + 데이터
        self.stream.write_all(&len.to_be_bytes()).await?;
        self.stream.write_all(&data).await?;
        self.stream.flush().await?;
        
        Ok(())
    }
    
    /// 메시지 수신
    pub async fn recv(&mut self) -> Result<IPCMessage, Box<dyn std::error::Error>> {
        // 메시지 길이 읽기
        let mut len_bytes = [0u8; 4];
        self.stream.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        
        // 메시지 데이터 읽기
        let mut buffer = vec![0u8; len];
        self.stream.read_exact(&mut buffer).await?;
        
        let msg = bincode::deserialize(&buffer)?;
        Ok(msg)
    }
    
    /// 브로드캐스트 요청
    pub async fn broadcast(&mut self, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let msg = IPCMessage::Broadcast(data);
        self.send(msg).await
    }
    
    /// 특정 피어에게 전송
    pub async fn send_to(&mut self, peer: PeerId, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let msg = IPCMessage::SendTo { peer, data };
        self.send(msg).await
    }
}