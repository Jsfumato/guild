// Guild Home Network - QUIC 기반 초고속 P2P
use quinn::{Endpoint, ServerConfig, ClientConfig, Connection};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

pub struct Network {
    endpoint: Endpoint,
    peers: Arc<RwLock<HashMap<SocketAddr, Connection>>>,
}

impl Network {
    pub async fn new() -> Self {
        Self::with_port(0).await
    }
    
    pub async fn with_port(port: u16) -> Self {
        // QUIC 서버 설정 (자체 서명 인증서)
        let server_config = Self::make_server_config();
        let client_config = Self::make_client_config();
        
        // 포트 설정 (0 = 자동 할당)
        let addr = format!("0.0.0.0:{}", port);
        let mut endpoint = Endpoint::server(
            server_config,
            addr.parse().unwrap()
        ).unwrap();
        
        endpoint.set_default_client_config(client_config);
        
        let addr = endpoint.local_addr().unwrap();
        println!("📡 Listening on {}", addr);
        
        let network = Self {
            endpoint: endpoint.clone(),
            peers: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // 연결 수락 루프
        let peers = network.peers.clone();
        tokio::spawn(async move {
            while let Some(conn) = endpoint.accept().await {
                let peers = peers.clone();
                tokio::spawn(async move {
                    if let Ok(conn) = conn.await {
                        let addr = conn.remote_address();
                        println!("✅ New peer: {}", addr);
                        peers.write().await.insert(addr, conn);
                    }
                });
            }
        });
        
        network
    }
    
    pub async fn connect(&self, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.endpoint.connect(addr, "localhost")?.await?;
        println!("🔗 Connected to {}", addr);
        self.peers.write().await.insert(addr, conn);
        Ok(())
    }
    
    pub async fn broadcast(&self, data: &[u8]) {
        let peers = self.peers.read().await;
        for (_addr, conn) in peers.iter() {
            if let Ok(mut send) = conn.open_uni().await {
                let _ = send.write_all(data).await;
                let _ = send.finish().await;
            }
        }
    }
    
    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }
    
    pub fn local_port(&self) -> u16 {
        self.endpoint.local_addr().unwrap().port()
    }
    
    fn make_server_config() -> ServerConfig {
        // 자체 서명 인증서 생성
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
        let cert_der = cert.serialize_der().unwrap();
        let key_der = cert.serialize_private_key_der();
        
        let priv_key = rustls::PrivateKey(key_der);
        let cert_chain = vec![rustls::Certificate(cert_der)];
        
        ServerConfig::with_single_cert(cert_chain, priv_key).unwrap()
    }
    
    fn make_client_config() -> ClientConfig {
        // 모든 인증서 허용 (개발용)
        let crypto = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(SkipServerVerification::new())
            .with_no_client_auth();
        
        ClientConfig::new(Arc::new(crypto))
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