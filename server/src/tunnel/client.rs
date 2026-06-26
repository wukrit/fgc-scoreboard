use reqwest::Url;
use serde::Deserialize;
use std::time::Duration;
use tokio::io;
use tokio::net::TcpStream;
use tokio::sync::watch;
use tokio::task::JoinHandle;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct TunnelInfo {
    pub id: String,
    pub ip: String,
    pub port: u16,
    pub url: String,
    pub cached_url: Option<String>,
    pub max_conn_count: Option<u32>,
}

impl TunnelInfo {
    pub fn max_conn(&self) -> u32 {
        self.max_conn_count.unwrap_or(1).max(1)
    }
}

const REGISTRATION_ATTEMPTS: u32 = 3;
const REGISTRATION_TIMEOUT: Duration = Duration::from_secs(5);
const REGISTRATION_RETRY_DELAY: Duration = Duration::from_secs(1);

pub async fn register_tunnel(host: &str, subdomain: Option<&str>) -> Result<TunnelInfo, String> {
    let base = host.trim_end_matches('/');
    let uri = match subdomain {
        Some(name) if !name.is_empty() => format!("{base}/{name}"),
        _ => format!("{base}/?new"),
    };

    let client = reqwest::Client::builder()
        .timeout(REGISTRATION_TIMEOUT)
        .build()
        .map_err(|e| format!("failed to build HTTP client: {e}"))?;

    let mut last_err = String::from("tunnel server unreachable");

    for attempt in 1..=REGISTRATION_ATTEMPTS {
        match client.get(&uri).send().await {
            Ok(response) => {
                let status = response.status();
                let body: serde_json::Value = response
                    .json()
                    .await
                    .map_err(|e| format!("invalid tunnel response: {e}"))?;

                if !status.is_success() {
                    let message = body
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("localtunnel server returned an error");
                    return Err(message.to_string());
                }

                return serde_json::from_value(body)
                    .map_err(|e| format!("unexpected tunnel response: {e}"));
            }
            Err(err) => {
                last_err = err.to_string();
                if attempt < REGISTRATION_ATTEMPTS {
                    tracing::warn!(
                        "tunnel server unreachable (attempt {attempt}/{REGISTRATION_ATTEMPTS}): {last_err}"
                    );
                    tokio::time::sleep(REGISTRATION_RETRY_DELAY).await;
                }
            }
        }
    }

    Err(format!(
        "tunnel server unreachable after {REGISTRATION_ATTEMPTS} attempts ({last_err})"
    ))
}

pub fn spawn_tunnel_cluster(
    info: TunnelInfo,
    local_port: u16,
    shutdown: watch::Receiver<bool>,
) -> Vec<JoinHandle<()>> {
    let max_conn = info.max_conn() as usize;
    (0..max_conn)
        .map(|_| {
            let info = info.clone();
            let mut shutdown = shutdown.clone();
            tokio::spawn(async move {
                loop {
                    if *shutdown.borrow() {
                        return;
                    }

                    let remote = match connect_remote(&info).await {
                        Ok(stream) => stream,
                        Err(err) => {
                            tracing::warn!("tunnel remote connect failed: {err}");
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                    };

                    if remote_connect_once(remote, local_port, &mut shutdown).await {
                        return;
                    }

                    if *shutdown.borrow() {
                        return;
                    }

                    tracing::debug!("tunnel connection closed; reconnecting");
                    tokio::time::sleep(Duration::from_millis(250)).await;
                }
            })
        })
        .collect()
}

async fn connect_remote(info: &TunnelInfo) -> Result<TcpStream, io::Error> {
    let addr = format!("{}:{}", info.ip, info.port);
    TcpStream::connect(addr).await
}

async fn remote_connect_once(
    mut remote: TcpStream,
    local_port: u16,
    shutdown: &mut watch::Receiver<bool>,
) -> bool {
    let local_addr = format!("127.0.0.1:{local_port}");

    loop {
        if *shutdown.borrow() {
            return true;
        }

        match TcpStream::connect(&local_addr).await {
            Ok(mut local) => {
                if pipe_bidirectional(&mut remote, &mut local, shutdown).await {
                    return true;
                }
                return false;
            }
            Err(err) if err.kind() == io::ErrorKind::ConnectionRefused => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Err(err) => {
                tracing::warn!("tunnel local connect failed: {err}");
                return false;
            }
        }
    }
}

async fn pipe_bidirectional(
    remote: &mut TcpStream,
    local: &mut TcpStream,
    shutdown: &mut watch::Receiver<bool>,
) -> bool {
    tokio::select! {
        biased;
        changed = shutdown.changed() => {
            changed.is_ok() && *shutdown.borrow()
        }
        result = io::copy_bidirectional(remote, local) => {
            if let Err(err) = result {
                tracing::debug!("tunnel pipe error: {err}");
            }
            false
        }
    }
}

pub fn normalize_tunnel_host(host: &str) -> Result<String, String> {
    let trimmed = host.trim();
    if trimmed.is_empty() {
        return Err("tunnel host must not be empty".to_string());
    }

    let with_scheme = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };

    Url::parse(&with_scheme).map_err(|e| format!("invalid tunnel host: {e}"))?;
    Ok(with_scheme.trim_end_matches('/').to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_registration_json() {
        let json = r#"{
            "id": "abc123",
            "ip": "159.203.27.74",
            "port": 12345,
            "url": "https://abc123.loca.lt",
            "max_conn_count": 2
        }"#;

        let info: TunnelInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.id, "abc123");
        assert_eq!(info.ip, "159.203.27.74");
        assert_eq!(info.port, 12345);
        assert_eq!(info.url, "https://abc123.loca.lt");
        assert_eq!(info.max_conn(), 2);
    }

    #[test]
    fn normalizes_tunnel_host() {
        assert_eq!(
            normalize_tunnel_host("localtunnel.me").unwrap(),
            "https://localtunnel.me"
        );
        assert_eq!(
            normalize_tunnel_host("https://localtunnel.me/").unwrap(),
            "https://localtunnel.me"
        );
    }
}
