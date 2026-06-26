use crate::rate_limit::RateLimiter;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub auth_token: Option<Arc<str>>,
    pub asset_root: PathBuf,
    pub scoreboard_path: PathBuf,
    pub rate_limiter: Arc<RateLimiter>,
    pub trust_proxy: bool,
    pub log_poll: bool,
}

impl AppState {
    pub fn auth_required(&self) -> bool {
        self.auth_token.is_some()
    }
}
