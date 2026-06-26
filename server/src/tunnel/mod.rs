mod client;

use client::{register_tunnel, spawn_tunnel_cluster, TunnelInfo};
use tokio::task::JoinHandle;

pub use client::normalize_tunnel_host;

pub struct TunnelHandle {
    shutdown_tx: tokio::sync::watch::Sender<bool>,
    workers: Vec<JoinHandle<()>>,
}

impl TunnelHandle {
    pub async fn shutdown(self) {
        let _ = self.shutdown_tx.send(true);
        for worker in self.workers {
            let _ = worker.await;
        }
    }
}

pub async fn start_tunnel(
    host: String,
    subdomain: Option<String>,
    local_port: u16,
) -> Result<TunnelHandle, String> {
    let info = register_tunnel(&host, subdomain.as_deref()).await?;
    log_tunnel_urls(&info);

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let workers = spawn_tunnel_cluster(info.clone(), local_port, shutdown_rx);

    Ok(TunnelHandle {
        shutdown_tx,
        workers,
    })
}

fn log_tunnel_urls(info: &TunnelInfo) {
    let base = info.url.trim_end_matches('/');
    tracing::info!("  Tunnel:     {}", info.url);
    tracing::info!("  Controller: {base}/");
    tracing::info!("  Overlay:    {base}/overlay/scoreboard.html");
    if let Some(cached) = &info.cached_url {
        tracing::info!("  Cached URL: {cached}");
    }
}
