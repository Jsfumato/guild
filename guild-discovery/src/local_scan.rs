// 로컬 네트워크 스캔을 통한 피어 발견
use std::net::SocketAddr;

// Guild Home 기본 포트 설정
pub const DEFAULT_PORT: u16 = 42000;
pub const DEFAULT_PORT_RANGE: u16 = 100;  // 42000-42100 범위

pub struct LocalScanner {
    base_port: u16,
    port_range: u16,
}

impl LocalScanner {
    pub fn new(current_port: u16) -> Self {
        Self {
            base_port: current_port,
            port_range: DEFAULT_PORT_RANGE,
        }
    }
    
    /// 로컬 네트워크에서 피어 스캔
    pub async fn scan_local_peers(&self) -> Vec<SocketAddr> {
        let mut discovered = Vec::new();
        
        // 1. 기본 포트 우선 확인 (가장 일반적)
        if self.base_port != DEFAULT_PORT {
            let addr: SocketAddr = format!("127.0.0.1:{}", DEFAULT_PORT).parse().unwrap();
            discovered.push(addr);
        }
        
        // 2. 인접 포트 확인 (가장 가능성 높은 순서)
        // 기본 포트 근처부터 확인 (최대 3개 포트만 시도)
        for offset in 1..=3 {
            let port = DEFAULT_PORT + offset;
            if port != self.base_port && port < DEFAULT_PORT + 10 {
                let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
                if !discovered.contains(&addr) {
                    discovered.push(addr);
                }
            }
        }
        
        // 3. 현재 포트가 기본 범위 밖이면 추가
        if self.base_port > DEFAULT_PORT + 10 || self.base_port < DEFAULT_PORT {
            // 사용자 정의 포트도 스캔 범위에 포함
            for offset in 1..=2 {
                let port = self.base_port + offset;
                if port != self.base_port {
                    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
                    if !discovered.contains(&addr) {
                        discovered.push(addr);
                    }
                }
            }
        }
        
        if !discovered.is_empty() {
            println!("🔍 Scanning {} potential local peers", discovered.len());
        }
        
        discovered
    }
    
}