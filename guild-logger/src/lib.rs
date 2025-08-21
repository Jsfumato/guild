// Guild Logger - 공유 로깅 시스템
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Logger {
    logs: Arc<RwLock<VecDeque<String>>>,
    tui_mode: bool,
}

impl Logger {
    pub fn new(tui_mode: bool) -> Self {
        Self {
            logs: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
            tui_mode,
        }
    }

    pub async fn log(&self, message: String) {
        if self.tui_mode {
            // TUI 모드에서는 로그를 메모리에 저장
            let mut logs = self.logs.write().await;
            let now = chrono::Local::now();
            let formatted_message = format!("{} {}", now.format("%H:%M:%S"), message);
            
            logs.push_back(formatted_message);
            if logs.len() > 100 {
                logs.pop_front();
            }
        } else {
            // 콘솔 모드에서는 직접 출력
            println!("{}", message);
        }
    }

    pub async fn info(&self, message: &str) {
        self.log(message.to_string()).await;
    }

    pub async fn success(&self, message: &str) {
        self.log(format!("✅ {}", message)).await;
    }

    pub async fn warning(&self, message: &str) {
        self.log(format!("⚠️ {}", message)).await;
    }

    pub async fn error(&self, message: &str) {
        self.log(format!("❌ {}", message)).await;
    }

    pub async fn ping(&self, message: &str) {
        self.log(format!("🏓 {}", message)).await;
    }

    pub async fn network(&self, message: &str) {
        self.log(format!("📡 {}", message)).await;
    }

    pub async fn discovery(&self, message: &str) {
        self.log(format!("🔍 {}", message)).await;
    }

    pub async fn connection(&self, message: &str) {
        self.log(format!("🔗 {}", message)).await;
    }

    pub async fn get_recent_logs(&self) -> Vec<String> {
        self.logs.read().await.iter().cloned().collect()
    }
}

// 전역 로거 인스턴스
static mut GLOBAL_LOGGER: Option<Logger> = None;
static INIT: std::sync::Once = std::sync::Once::new();

pub fn init_logger(tui_mode: bool) {
    INIT.call_once(|| {
        unsafe {
            GLOBAL_LOGGER = Some(Logger::new(tui_mode));
        }
    });
}

pub fn get_logger() -> &'static Logger {
    unsafe {
        GLOBAL_LOGGER.as_ref().expect("Logger not initialized")
    }
}

// 편의 매크로들
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            rt.spawn(async move {
                guild_logger::get_logger().info(&format!($($arg)*)).await;
            });
        }
    };
}

#[macro_export]
macro_rules! log_success {
    ($($arg:tt)*) => {
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            rt.spawn(async move {
                guild_logger::get_logger().success(&format!($($arg)*)).await;
            });
        }
    };
}

#[macro_export]
macro_rules! log_warning {
    ($($arg:tt)*) => {
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            rt.spawn(async move {
                guild_logger::get_logger().warning(&format!($($arg)*)).await;
            });
        }
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            rt.spawn(async move {
                guild_logger::get_logger().error(&format!($($arg)*)).await;
            });
        }
    };
}

#[macro_export]
macro_rules! log_ping {
    ($($arg:tt)*) => {
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            rt.spawn(async move {
                guild_logger::get_logger().ping(&format!($($arg)*)).await;
            });
        }
    };
}

#[macro_export]
macro_rules! log_network {
    ($($arg:tt)*) => {
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            rt.spawn(async move {
                guild_logger::get_logger().network(&format!($($arg)*)).await;
            });
        }
    };
}

#[macro_export]
macro_rules! log_discovery {
    ($($arg:tt)*) => {
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            rt.spawn(async move {
                guild_logger::get_logger().discovery(&format!($($arg)*)).await;
            });
        }
    };
}

#[macro_export]
macro_rules! log_connection {
    ($($arg:tt)*) => {
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            rt.spawn(async move {
                guild_logger::get_logger().connection(&format!($($arg)*)).await;
            });
        }
    };
}