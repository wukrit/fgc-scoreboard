use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const WINDOW: Duration = Duration::from_secs(60);

pub struct RateLimiter {
    limit: u32,
    history: Mutex<HashMap<IpAddr, Vec<Instant>>>,
}

impl RateLimiter {
    pub fn new(limit: u32) -> Self {
        Self {
            limit,
            history: Mutex::new(HashMap::new()),
        }
    }

    pub async fn check(&self, ip: IpAddr) -> Result<(), u64> {
        if self.limit == 0 {
            return Ok(());
        }
        let now = Instant::now();
        let cutoff = now - WINDOW;
        let mut map = self.history.lock().await;
        let history = map.entry(ip).or_default();
        history.retain(|t| *t > cutoff);
        if history.len() >= self.limit as usize {
            let retry = history
                .first()
                .map(|t| WINDOW.saturating_sub(now - *t).as_secs())
                .unwrap_or(1)
                .max(1);
            return Err(retry);
        }
        history.push(now);
        Ok(())
    }
}
