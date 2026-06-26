use crate::config::Config;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_logging(config: &Config) {
    let level = std::env::var("FGC_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    let filter = EnvFilter::try_new(level.to_lowercase()).unwrap_or_else(|_| EnvFilter::new("info"));

    if config.use_json_logs() {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().json().with_target(false))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_target(false))
            .init();
    }
}

pub fn access_log_level(method: &str, path: &str, status: u16, log_poll: bool) -> tracing::Level {
    let is_poll = method == "GET" && (path == "/health" || path == "/scoreboard.json");
    let is_post_save = method == "POST" && path == "/scoreboard.json";
    let is_static = method == "GET"
        && !is_poll
        && (path == "/" || path.starts_with("/css/") || path.starts_with("/overlay/"));

    if status >= 500 {
        return tracing::Level::ERROR;
    }
    if status >= 400 {
        return tracing::Level::WARN;
    }
    if is_post_save {
        return tracing::Level::INFO;
    }
    if is_poll {
        return if log_poll {
            tracing::Level::INFO
        } else {
            tracing::Level::DEBUG
        };
    }
    if is_static {
        return tracing::Level::DEBUG;
    }
    tracing::Level::DEBUG
}
