//! Guild Home - P2P 네트워킹 모듈

pub mod config;
pub mod guild_home;
pub mod help;
pub mod network;
pub mod tui;

// Re-export main types for convenience
pub use config::{Config, ConfigError};
pub use guild_home::GuildHome;

// Re-export other core types
pub use network::Network;

// Re-export logging macros
pub use guild_logger::{
    log_connection, log_discovery, log_error, log_info, 
    log_network, log_ping, log_success, log_warning
};