// Guild Discovery - P2P 노드 탐색 라이브러리
pub mod bootstrap;
pub mod dht;
pub mod discovery;
pub mod local_scan;

pub use bootstrap::{Bootstrap, PeerInfo};
pub use dht::{Kademlia, NodeId};
pub use discovery::{Discovery, DiscoveryConfig};
pub use local_scan::{LocalScanner, DEFAULT_PORT, DEFAULT_PORT_RANGE};

// Re-export logging macros
pub use guild_logger::{
    log_connection, log_discovery, log_error, log_info, 
    log_network, log_ping, log_success, log_warning
};
