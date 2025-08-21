// Guild Home Network - QUIC 기반 초고속 P2P
use quinn::{ClientConfig, Connection, Endpoint, ServerConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Ping { id: String, timestamp: u64 },
    Pong { id: String, timestamp: u64 },
    Data(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub connection: Connection,
    pub last_ping: Instant,
    pub last_pong: Instant,
    pub latency_ms: u64,
}

pub struct Network {
    endpoint: Endpoint,
    peers: Arc<RwLock<HashMap<SocketAddr, PeerInfo>>>,
}

impl Network {
    pub async fn new() -> Self {
        Self::with_port(0).await
    }

    pub async fn with_port(port: u16) -> Self {
        // QUIC 서버 설정 (자체 서명 인증서)
        let server_config = Self::make_server_config();
        let client_config = Self::make_client_config();

        let mut endpoint = None;
        let mut current_port = port;
        let max_attempts = 100; // 최대 100번 시도
        
        // Address already in use 에러 시 포트를 1씩 증가시키며 재시도
        for attempt in 0..max_attempts {
            let addr = format!("0.0.0.0:{}", current_port);
            match Endpoint::server(server_config.clone(), addr.parse().unwrap()) {
                Ok(ep) => {
                    endpoint = Some(ep);
                    if attempt > 0 {
                        println!("✅ Found available port {} after {} attempts", current_port, attempt + 1);
                    }
                    break;
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("Address already in use") || error_msg.contains("already being used") {
                        println!("⚠️ Port {} already in use, trying port {}", current_port, current_port + 1);
                        current_port += 1;
                    } else {
                        panic!("Failed to create endpoint: {:?}", e);
                    }
                }
            }
        }

        let mut endpoint = endpoint.expect("Failed to find available port after maximum attempts");
        endpoint.set_default_client_config(client_config);

        let addr = endpoint.local_addr().unwrap();
        println!("📡 Listening on {}", addr);

        let network = Self {
            endpoint: endpoint.clone(),
            peers: Arc::new(RwLock::new(HashMap::new())),
        };

        // 연결 수락 루프
        let peers = network.peers.clone();
        let endpoint_clone = endpoint.clone();
        tokio::spawn(async move {
            while let Some(conn) = endpoint_clone.accept().await {
                let peers = peers.clone();
                tokio::spawn(async move {
                    if let Ok(conn) = conn.await {
                        let addr = conn.remote_address();
                        println!("✅ New peer: {}", addr);

                        let peer_info = PeerInfo {
                            connection: conn.clone(),
                            last_ping: Instant::now(),
                            last_pong: Instant::now(),
                            latency_ms: 0,
                        };

                        peers.write().await.insert(addr, peer_info);

                        // 이 피어로부터 메시지 수신 처리
                        Self::handle_peer_messages(conn, addr, peers.clone()).await;
                    }
                });
            }
        });

        network
    }

    pub async fn connect(&self, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.endpoint.connect(addr, "localhost")?.await?;
        println!("🔗 Connected to {}", addr);

        let peer_info = PeerInfo {
            connection: conn.clone(),
            last_ping: Instant::now(),
            last_pong: Instant::now(),
            latency_ms: 0,
        };

        self.peers.write().await.insert(addr, peer_info);

        // 이 피어로부터 메시지 수신 처리
        let peers = self.peers.clone();
        tokio::spawn(async move {
            Self::handle_peer_messages(conn, addr, peers).await;
        });

        Ok(())
    }

    pub async fn broadcast(&self, data: &[u8]) {
        let msg = Message::Data(data.to_vec());
        let serialized = bincode::serialize(&msg).unwrap();

        let peers = self.peers.read().await;
        for (_addr, peer_info) in peers.iter() {
            if let Ok(mut send) = peer_info.connection.open_uni().await {
                let _ = send.write_all(&serialized).await;
                let _ = send.finish().await;
            }
        }
    }

    pub async fn send_ping(&self) {
        let peers = self.peers.read().await;
        println!("📍 Sending ping to {} peers", peers.len());
        
        for (addr, peer_info) in peers.iter() {
            let ping_id = uuid::Uuid::new_v4().to_string();
            let msg = Message::Ping {
                id: ping_id.clone(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            };

            let serialized = bincode::serialize(&msg).unwrap();
            println!("📤 Sending Ping {} to {} ({} bytes)", ping_id, addr, serialized.len());
            
            match peer_info.connection.open_uni().await {
                Ok(mut send) => {
                    match send.write_all(&serialized).await {
                        Ok(_) => {
                            match send.finish().await {
                                Ok(_) => println!("✅ Ping {} sent to {}", ping_id, addr),
                                Err(e) => println!("❌ Failed to finish send to {}: {:?}", addr, e),
                            }
                        }
                        Err(e) => println!("❌ Failed to write ping to {}: {:?}", addr, e),
                    }
                }
                Err(e) => println!("❌ Failed to open stream to {}: {:?}", addr, e),
            }
        }
    }

    pub async fn check_peer_health(&self) {
        let mut dead_peers = Vec::new();
        let timeout = Duration::from_secs(10); // 10초 타임아웃

        {
            let peers = self.peers.read().await;
            for (addr, peer_info) in peers.iter() {
                if peer_info.last_pong.elapsed() > timeout {
                    println!(
                        "💀 Peer timeout: {} (no response for {:?})",
                        addr,
                        peer_info.last_pong.elapsed()
                    );
                    dead_peers.push(*addr);
                }
            }
        }

        // 응답하지 않는 피어 제거
        if !dead_peers.is_empty() {
            let mut peers = self.peers.write().await;
            for addr in dead_peers {
                peers.remove(&addr);
                println!("❌ Removed dead peer: {}", addr);
            }
        }
    }

    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    pub fn local_port(&self) -> u16 {
        self.endpoint.local_addr().unwrap().port()
    }

    async fn handle_peer_messages(
        conn: Connection,
        addr: SocketAddr,
        peers: Arc<RwLock<HashMap<SocketAddr, PeerInfo>>>,
    ) {
        println!("👂 Starting message handler for {}", addr);
        loop {
            match conn.accept_uni().await {
                Ok(mut recv) => {
                    let buf = match recv.read_to_end(1024 * 1024).await {
                        Ok(data) => data,
                        Err(e) => {
                            println!("⚠️ Failed to read from {}: {:?}", addr, e);
                            continue;
                        }
                    };
                    
                    println!("📨 Received {} bytes from {}", buf.len(), addr);
                    
                    if buf.is_empty() {
                        println!("⚠️ Empty message from {}", addr);
                        continue;
                    }
                    
                    match bincode::deserialize::<Message>(&buf) {
                        Ok(msg) => {
                            match msg {
                                Message::Ping { id, timestamp } => {
                                    println!("🏓 Got Ping {} from {}", id, addr);

                                    // Ping 받으면 Pong 응답
                                    let pong = Message::Pong { id: id.clone(), timestamp };
                                    let serialized = bincode::serialize(&pong).unwrap();

                                    if let Ok(mut send) = conn.open_uni().await {
                                        let _ = send.write_all(&serialized).await;
                                        let _ = send.finish().await;
                                        println!("🏓 Sent Pong {} to {}", id, addr);
                                    }

                                    // last_ping 업데이트
                                    if let Some(peer) = peers.write().await.get_mut(&addr) {
                                        peer.last_ping = Instant::now();
                                    }
                                }
                                Message::Pong { id, timestamp } => {
                                    // Pong 받으면 latency 계산하고 last_pong 업데이트
                                    let now = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap()
                                        .as_millis()
                                        as u64;

                                    let latency = now.saturating_sub(timestamp);

                                    if let Some(peer) = peers.write().await.get_mut(&addr) {
                                        peer.last_pong = Instant::now();
                                        peer.latency_ms = latency;
                                        println!("🏓 Got Pong {} from {} ({}ms)", id, addr, latency);
                                    }
                                }
                                Message::Data(data) => {
                                    // 일반 데이터 메시지 처리
                                    println!("📦 Data from {}: {} bytes", addr, data.len());
                                }
                            }
                        }
                        Err(e) => {
                            println!("⚠️ Failed to deserialize message from {}: {:?}", addr, e);
                            println!("⚠️ Buffer content (first 100 bytes): {:?}", &buf[..buf.len().min(100)]);
                        }
                    }
                }
                Err(e) => {
                    // 연결 상태 확인
                    let error_msg = e.to_string();
                    if error_msg.contains("Closed") || error_msg.contains("closed") {
                        println!("🔌 Connection closed: {} ({})", addr, error_msg);
                        peers.write().await.remove(&addr);
                        break;
                    } else {
                        // 일시적인 에러는 로그만 출력하고 계속 시도
                        println!("⚠️ Stream error from {}: {} (retrying...)", addr, error_msg);
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            }
        }
    }

    fn make_server_config() -> ServerConfig {
        // 자체 서명 인증서 생성
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
        let cert_der = cert.serialize_der().unwrap();
        let key_der = cert.serialize_private_key_der();

        let priv_key = rustls::PrivateKey(key_der);
        let cert_chain = vec![rustls::Certificate(cert_der)];

        // Keep-alive 설정으로 연결 유지
        let mut transport_config = quinn::TransportConfig::default();
        transport_config.keep_alive_interval(Some(Duration::from_secs(5))); // 5초마다 keep-alive
        transport_config.max_idle_timeout(Some(Duration::from_secs(30).try_into().unwrap())); // 30초 타임아웃
        
        let mut config = ServerConfig::with_single_cert(cert_chain, priv_key).unwrap();
        config.transport_config(Arc::new(transport_config));
        config
    }

    fn make_client_config() -> ClientConfig {
        // 모든 인증서 허용 (개발용)
        let crypto = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(SkipServerVerification::new())
            .with_no_client_auth();

        // Keep-alive 설정으로 연결 유지
        let mut transport_config = quinn::TransportConfig::default();
        transport_config.keep_alive_interval(Some(Duration::from_secs(5))); // 5초마다 keep-alive
        transport_config.max_idle_timeout(Some(Duration::from_secs(30).try_into().unwrap())); // 30초 타임아웃
        
        let mut config = ClientConfig::new(Arc::new(crypto));
        config.transport_config(Arc::new(transport_config));
        config
    }
}

// 인증서 검증 스킵 (개발용)
struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl rustls::client::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}
