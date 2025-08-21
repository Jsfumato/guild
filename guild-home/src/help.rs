pub fn print_help() {
    println!(r#"
🏰 Guild Home - P2P Networking Module

USAGE:
    guild-home [OPTIONS]

OPTIONS:
    -p, --port <PORT>             Port to listen on (0 = auto)
    -b, --bootstrap <PEERS>       Bootstrap peers (comma separated)
    -d, --data-dir <DIR>          Data directory (default: ./data)
    -i, --interval <SECONDS>      Heartbeat interval (default: 5)
    -l, --log <LEVEL>             Log level (error/warn/info/debug)
    -h, --help                    Show this help message

ENVIRONMENT VARIABLES:
    GUILD_PORT                    Same as --port
    GUILD_BOOTSTRAP               Same as --bootstrap
    GUILD_DATA_DIR                Same as --data-dir
    GUILD_HEARTBEAT_INTERVAL      Same as --interval
    GUILD_LOG_LEVEL               Same as --log

EXAMPLES:
    # Run with auto-discovery
    guild-home

    # Connect to specific peers
    guild-home --bootstrap 192.168.1.10:8000,192.168.1.11:8000

    # Run on specific port with 10-second heartbeat
    guild-home --port 8080 --interval 10
"#);
}