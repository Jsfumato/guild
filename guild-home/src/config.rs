use std::env;

#[derive(Debug)]
pub enum ConfigError {
    InvalidPort(String),
    InvalidBlockTime(String),
    InvalidBootstrap(String),
}

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub bootstrap: Vec<String>,
    pub data_dir: String,
    pub heartbeat_interval: u64,
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            port: 0,
            bootstrap: vec![],
            data_dir: "./data".to_string(),
            heartbeat_interval: 5,
            log_level: "info".to_string(),
        }
    }
}

impl Config {
    pub fn from_args() -> Result<Self, ConfigError> {
        let args: Vec<String> = env::args().collect();
        let mut config = Config::default();
        
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--port" | "-p" => {
                    if i + 1 < args.len() {
                        config.port = args[i + 1].parse()
                            .map_err(|_| ConfigError::InvalidPort(args[i + 1].clone()))?;
                        i += 2;
                    } else {
                        return Err(ConfigError::InvalidPort("Missing port value".to_string()));
                    }
                }
                "--bootstrap" | "-b" => {
                    if i + 1 < args.len() {
                        let bootstrap_str = &args[i + 1];
                        if bootstrap_str.is_empty() {
                            return Err(ConfigError::InvalidBootstrap("Empty bootstrap list".to_string()));
                        }
                        config.bootstrap = bootstrap_str.split(',')
                            .filter(|s| !s.trim().is_empty())
                            .map(|s| s.trim().to_string())
                            .collect();
                        i += 2;
                    } else {
                        return Err(ConfigError::InvalidBootstrap("Missing bootstrap value".to_string()));
                    }
                }
                "--data-dir" | "-d" => {
                    if i + 1 < args.len() {
                        config.data_dir = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--interval" | "-i" => {
                    if i + 1 < args.len() {
                        config.heartbeat_interval = args[i + 1].parse()
                            .map_err(|_| ConfigError::InvalidBlockTime(args[i + 1].clone()))?;
                        if config.heartbeat_interval == 0 {
                            return Err(ConfigError::InvalidBlockTime("Heartbeat interval must be greater than 0".to_string()));
                        }
                        i += 2;
                    } else {
                        return Err(ConfigError::InvalidBlockTime("Missing heartbeat interval value".to_string()));
                    }
                }
                "--log" | "-l" => {
                    if i + 1 < args.len() {
                        let log_level = &args[i + 1];
                        if !["error", "warn", "info", "debug"].contains(&log_level.as_str()) {
                            eprintln!("Warning: Invalid log level '{}', using 'info'", log_level);
                            config.log_level = "info".to_string();
                        } else {
                            config.log_level = log_level.clone();
                        }
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--help" | "-h" => {
                    crate::help::print_help();
                    std::process::exit(0);
                }
                _ => {
                    i += 1;
                }
            }
        }
        
        // 환경변수도 체크 (CLI가 우선순위 높음)
        config.load_from_env()?;
        
        Ok(config)
    }

    fn load_from_env(&mut self) -> Result<(), ConfigError> {
        if self.port == 0 {
            if let Ok(port_str) = env::var("GUILD_PORT") {
                self.port = port_str.parse()
                    .map_err(|_| ConfigError::InvalidPort(format!("Invalid GUILD_PORT: {}", port_str)))?;
            }
        }
        
        if self.bootstrap.is_empty() {
            if let Ok(bootstrap_str) = env::var("GUILD_BOOTSTRAP") {
                if !bootstrap_str.is_empty() {
                    self.bootstrap = bootstrap_str.split(',')
                        .filter(|s| !s.trim().is_empty())
                        .map(|s| s.trim().to_string())
                        .collect();
                }
            }
        }
        
        if let Ok(data_dir) = env::var("GUILD_DATA_DIR") {
            if self.data_dir == "./data" {
                self.data_dir = data_dir;
            }
        }

        if let Ok(interval_str) = env::var("GUILD_HEARTBEAT_INTERVAL") {
            let interval = interval_str.parse()
                .map_err(|_| ConfigError::InvalidBlockTime(format!("Invalid GUILD_HEARTBEAT_INTERVAL: {}", interval_str)))?;
            if interval == 0 {
                return Err(ConfigError::InvalidBlockTime("Heartbeat interval must be greater than 0".to_string()));
            }
            self.heartbeat_interval = interval;
        }

        if let Ok(log_level) = env::var("GUILD_LOG_LEVEL") {
            if ["error", "warn", "info", "debug"].contains(&log_level.as_str()) {
                self.log_level = log_level;
            }
        }
        
        Ok(())
    }
    
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = Config::default();
        config.load_from_env()?;
        Ok(config)
    }
}