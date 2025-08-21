// Guild Home - P2P 네트워킹 실행 파일
use guild_home::{Config, GuildHome};

#[tokio::main]
async fn main() {
    let config = Config::from_args().unwrap_or_else(|e| {
        eprintln!("Configuration error: {:?}", e);
        std::process::exit(1);
    });
    
    // 로그 레벨 설정
    match config.log_level.as_str() {
        "debug" => println!("🔍 Debug mode enabled"),
        "error" => println!("🔴 Error logging only"),
        _ => {}
    }
    
    println!("🏰 Guild Home Starting...");
    println!("📁 Data directory: {}", config.data_dir);
    if !config.bootstrap.is_empty() {
        println!("🌐 Bootstrap peers: {:?}", config.bootstrap);
    }
    
    // Guild Home 인스턴스 생성 및 시작
    let guild = GuildHome::new(config).await;
    guild.start().await;
}