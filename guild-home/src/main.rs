// Guild Home - P2P 네트워킹 실행 파일
use guild_home::{Config, GuildHome};
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let use_tui = args.contains(&"--tui".to_string());

    // 로거 초기화
    guild_logger::init_logger(use_tui);

    let config = Config::from_args().unwrap_or_else(|e| {
        eprintln!("Configuration error: {:?}", e);
        std::process::exit(1);
    });

    if use_tui {
        // TUI 모드로 실행 - 초기 메시지만 출력하고 나머지는 TUI에서 처리
        println!("🎨 Starting Guild Home in TUI mode...");
        println!("📁 Data directory: {}", config.data_dir);
        if !config.bootstrap.is_empty() {
            println!("🌐 Bootstrap peers: {:?}", config.bootstrap);
        }
        println!("Press 'q' or 'Esc' to quit");
        println!("Initializing...");

        // 잠시 후 화면을 지우고 TUI 시작
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        // Guild Home 인스턴스 생성
        println!("Opening Guild Home...");
        let mut guild = GuildHome::new(config).await;
        println!("Opening network...");
        let network = guild.network.clone();

        // 백그라운드에서 Guild Home 시작
        println!("Starting Guild Home...");
        guild.start().await;
        println!("Guild Home is running...");

        // Get IPC port from blockchain bridge
        let ipc_port = guild
            .blockchain_bridge
            .as_ref()
            .map(|b| b.get_ipc_port())
            .unwrap_or(0);

        // TUI 실행
        if let Err(e) = guild_home::tui::run_tui(network, ipc_port).await {
            eprintln!("TUI error: {:?}", e);
        }
    } else {
        // 기존 콘솔 모드로 실행
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
        println!("💡 Tip: Use --tui for dashboard mode");

        // Guild Home 인스턴스 생성 및 시작
        let mut guild = GuildHome::new(config).await;
        guild.start().await;
    }
}
