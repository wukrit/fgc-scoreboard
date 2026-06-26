use reqwest::Url;
use serde::Deserialize;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io;
use tokio::net::TcpStream;
use tokio::sync::watch;
use tokio::task::JoinHandle;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
struct TunnelRegistration {
    id: String,
    ip: Option<String>,
    port: u16,
    url: String,
    cached_url: Option<String>,
    max_conn_count: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TunnelInfo {
    pub id: String,
    pub remote_host: String,
    pub port: u16,
    pub url: String,
    pub cached_url: Option<String>,
    pub max_conn_count: Option<u32>,
}

impl TunnelInfo {
    pub fn max_conn(&self) -> u32 {
        self.max_conn_count.unwrap_or(1).max(1)
    }

    fn remote_addr(&self) -> String {
        format!("{}:{}", self.remote_host, self.port)
    }
}

fn tunnel_host_hostname(host: &str) -> String {
    Url::parse(host)
        .ok()
        .and_then(|u| u.host_str().map(str::to_string))
        .unwrap_or_else(|| "localtunnel.me".to_string())
}

fn parse_registration(host: &str, body: serde_json::Value) -> Result<TunnelInfo, String> {
    let reg: TunnelRegistration = serde_json::from_value(body)
        .map_err(|e| format!("unexpected tunnel response: {e}"))?;

    Ok(TunnelInfo {
        id: reg.id,
        remote_host: tunnel_host_hostname(host),
        port: reg.port,
        url: reg.url,
        cached_url: reg.cached_url,
        max_conn_count: reg.max_conn_count,
    })
}

pub(crate) fn log_tunnel_target(info: &TunnelInfo) {
    tracing::info!(
        "tunnel TCP target: {} (id={})",
        info.remote_addr(),
        info.id
    );
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

                return parse_registration(host, body);
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

                    if serve_tunnel_connection(remote, local_port, &mut shutdown).await {
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

fn tunnel_connect_addrs(addrs: impl IntoIterator<Item = SocketAddr>) -> Vec<SocketAddr> {
    let mut v4 = Vec::new();
    let mut v6 = Vec::new();
    for addr in addrs {
        if addr.is_ipv4() {
            v4.push(addr);
        } else if !is_ipv4_mapped_v6(&addr) {
            // Skip ::ffff:x.x.x.x — duplicates IPv4 and can fail on macOS (EADDRNOTAVAIL).
            v6.push(addr);
        }
    }
    v4.extend(v6);
    v4
}

fn is_ipv4_mapped_v6(addr: &SocketAddr) -> bool {
    matches!(addr, SocketAddr::V6(a) if a.ip().to_ipv4_mapped().is_some())
}

async fn connect_remote(info: &TunnelInfo) -> Result<TcpStream, io::Error> {
    let target = info.remote_addr();
    let addrs = tunnel_connect_addrs(tokio::net::lookup_host(&target).await?);
    let mut last_err = None;

    for addr in addrs {
        match TcpStream::connect(addr).await {
            Ok(stream) => {
                let _ = stream.set_nodelay(true);
                tracing::debug!("tunnel connected to {addr}");
                return Ok(stream);
            }
            Err(err) => {
                tracing::debug!("tunnel connect to {addr} failed: {err}");
                last_err = Some(err);
            }
        }
    }

    Err(last_err.unwrap_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotConnected,
            format!("could not connect to {target}"),
        )
    }))
}

enum PipeExit {
    Continue,
    Shutdown,
}

async fn serve_tunnel_connection(
    mut remote: TcpStream,
    local_port: u16,
    shutdown: &mut watch::Receiver<bool>,
) -> bool {
    loop {
        if *shutdown.borrow() {
            return true;
        }

        match pipe_to_local(&mut remote, local_port, shutdown).await {
            Ok(PipeExit::Continue) => continue,
            Ok(PipeExit::Shutdown) => return true,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => return true,
            Err(err) if err.kind() == io::ErrorKind::ConnectionRefused => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Err(err) => {
                tracing::debug!("tunnel proxy error: {err}");
                return false;
            }
        }
    }
}

async fn pipe_to_local(
    remote: &mut TcpStream,
    local_port: u16,
    shutdown: &mut watch::Receiver<bool>,
) -> io::Result<PipeExit> {
    let local_addr = format!("127.0.0.1:{local_port}");
    let mut local = TcpStream::connect(&local_addr).await?;

    if pipe_bidirectional(remote, &mut local, shutdown).await {
        return Ok(PipeExit::Shutdown);
    }

    Ok(PipeExit::Continue)
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
    fn parses_current_registration_json() {
        let json = r#"{
            "id": "abc123",
            "port": 12345,
            "url": "https://abc123.loca.lt",
            "max_conn_count": 2
        }"#;

        let info = parse_registration(
            "https://localtunnel.me",
            serde_json::from_str(json).unwrap(),
        )
        .unwrap();

        assert_eq!(info.id, "abc123");
        assert_eq!(info.remote_host, "localtunnel.me");
        assert_eq!(info.port, 12345);
        assert_eq!(info.url, "https://abc123.loca.lt");
        assert_eq!(info.max_conn(), 2);
        assert_eq!(info.remote_addr(), "localtunnel.me:12345");
    }

    #[test]
    fn parses_legacy_registration_json_with_ip_in_body() {
        let json = r#"{
            "id": "abc123",
            "ip": "159.203.27.74",
            "port": 12345,
            "url": "https://abc123.loca.lt",
            "max_conn_count": 2
        }"#;

        let info = parse_registration(
            "https://localtunnel.me",
            serde_json::from_str(json).unwrap(),
        )
        .unwrap();

        // TCP connect uses tunnel host hostname, matching the official localtunnel client.
        assert_eq!(info.remote_addr(), "localtunnel.me:12345");
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

    #[test]
    fn prefers_ipv4_for_tunnel_connect() {
        use std::net::{Ipv4Addr, Ipv6Addr};
        let addrs = vec![
            SocketAddr::from((Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x7f00, 1), 8080)),
            SocketAddr::from((Ipv4Addr::new(193, 34, 76, 44), 8080)),
        ];
        let ordered = tunnel_connect_addrs(addrs);
        assert_eq!(ordered.len(), 1);
        assert!(ordered[0].is_ipv4());
    }
}
