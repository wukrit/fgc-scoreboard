use clap::Parser;
use rand::RngCore;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "fgc-server", about = "FGC Scoreboard LAN Server")]
pub struct Config {
    #[arg(long, env = "PORT", default_value = "8080")]
    pub port: u16,

    #[arg(long, env = "FGC_BIND", default_value = "0.0.0.0")]
    pub bind: String,

    #[arg(long, env = "FGC_AUTH_TOKEN")]
    pub token: Option<String>,

    #[arg(long)]
    pub generate_token: bool,

    #[arg(long)]
    pub trust_proxy: bool,

    #[arg(long, env = "FGC_RATE_LIMIT", default_value = "60")]
    pub rate_limit: u32,

    #[arg(long, env = "FGC_ASSET_ROOT", default_value = "web")]
    pub asset_root: PathBuf,

    #[arg(long, env = "FGC_DATA_DIR", default_value = "data")]
    pub data_dir: PathBuf,
}

impl Config {
    pub fn parse_args() -> Self {
        Config::parse()
    }

    pub fn log_poll(&self) -> bool {
        env_truthy("FGC_LOG_POLL")
    }

    pub fn use_json_logs(&self) -> bool {
        let override_val = std::env::var("FGC_LOG_JSON")
            .unwrap_or_default()
            .trim()
            .to_lowercase();
        if override_val == "1" || override_val == "true" || override_val == "yes" {
            return true;
        }
        if override_val == "0" || override_val == "false" || override_val == "no" {
            return false;
        }
        std::env::var("RAILWAY_ENVIRONMENT").is_ok()
    }
}

pub fn env_truthy(name: &str) -> bool {
    std::env::var(name)
        .map(|v| {
            let v = v.trim().to_lowercase();
            v == "1" || v == "true" || v == "yes"
        })
        .unwrap_or(false)
}

pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, bytes)
}
