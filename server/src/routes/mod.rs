pub mod health;
pub mod scoreboard;

use axum::{
    extract::{Request, State},
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};

use crate::logging::access_log_level;
use crate::state::AppState;

pub async fn auth_header_middleware(
    State(state): axum::extract::State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let mut res = next.run(req).await;
    if state.auth_required() {
        res.headers_mut().insert(
            "X-FGC-Auth-Required",
            HeaderValue::from_static("1"),
        );
    }
    res
}

pub async fn access_log_middleware(
    State(state): axum::extract::State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let log_poll = state.log_poll;
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let res = next.run(req).await;
    let status = res.status().as_u16();
    let level = access_log_level(&method, &path, status, log_poll);
    log_access(&method, &path, status, level);
    res
}

fn log_access(method: &str, path: &str, status: u16, level: tracing::Level) {
    match level {
        tracing::Level::ERROR => {
            tracing::error!(target: "fgc_access", "{method} {path} {status}");
        }
        tracing::Level::WARN => {
            tracing::warn!(target: "fgc_access", "{method} {path} {status}");
        }
        tracing::Level::INFO => {
            tracing::info!(target: "fgc_access", "{method} {path} {status}");
        }
        tracing::Level::DEBUG => {
            tracing::debug!(target: "fgc_access", "{method} {path} {status}");
        }
        _ => {
            tracing::trace!(target: "fgc_access", "{method} {path} {status}");
        }
    }
}

pub fn static_router(state: &AppState) -> axum::Router<Arc<AppState>> {
    let index = state.asset_root.join("index.html");
    let css = state.asset_root.join("css");
    let overlay = state.asset_root.join("overlay");

    axum::Router::new()
        .route_service("/", ServeFile::new(index))
        .nest_service("/css", ServeDir::new(css))
        .nest_service("/overlay", ServeDir::new(overlay))
        .fallback(not_found)
}

async fn not_found() -> impl IntoResponse {
    StatusCode::NOT_FOUND
}

pub fn client_ip(req: &Request, trust_proxy: bool) -> std::net::IpAddr {
    if trust_proxy {
        if let Some(forwarded) = req.headers().get("x-forwarded-for") {
            if let Ok(s) = forwarded.to_str() {
                if let Some(first) = s.split(',').next() {
                    if let Ok(ip) = first.trim().parse() {
                        return ip;
                    }
                }
            }
        }
    }
    req.extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip())
        .unwrap_or(std::net::IpAddr::from([127, 0, 0, 1]))
}
