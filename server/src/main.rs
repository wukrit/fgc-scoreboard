mod auth;
mod config;
mod logging;
mod rate_limit;
mod routes;
mod state;
mod tunnel;

use axum::{
    middleware,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tower_http::trace::TraceLayer;

use config::{generate_token, Config};
use logging::init_logging;
use rate_limit::RateLimiter;
use routes::health::{auth_check, health};
use routes::scoreboard::{ensure_scoreboard_file, get_scoreboard, post_scoreboard, SCOREBOARD_FILE};
use routes::{access_log_middleware, auth_header_middleware, static_router};
use state::AppState;

fn get_lan_ip() -> String {
    use std::net::UdpSocket;
    match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => {
            if socket.connect("8.8.8.8:80").is_ok() {
                if let Ok(addr) = socket.local_addr() {
                    return addr.ip().to_string();
                }
            }
        }
        Err(_) => {}
    }
    "127.0.0.1".to_string()
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

fn validate_token(token: Option<&String>) -> Result<(), String> {
    if let Some(t) = token {
        if t.len() < 32 {
            return Err("Auth token must be at least 32 characters".to_string());
        }
    }
    Ok(())
}

fn build_app(state: Arc<AppState>) -> Router {
    let api = Router::new()
        .route("/health", get(health))
        .route("/auth/check", get(auth_check))
        .route(
            "/scoreboard.json",
            get(get_scoreboard).post(post_scoreboard),
        );

    let static_routes = static_router(&state);

    Router::new()
        .merge(api)
        .merge(static_routes)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            access_log_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_header_middleware,
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[tokio::main]
async fn main() {
    let config = Config::parse_args();

    if config.generate_token {
        println!("{}", generate_token());
        return;
    }

    if let Err(msg) = validate_token(config.token.as_ref()) {
        eprintln!("{msg}");
        std::process::exit(1);
    }

    init_logging(&config);

    let asset_root = config.asset_root.canonicalize().unwrap_or(config.asset_root.clone());
    let index_path = asset_root.join("index.html");
    if !index_path.is_file() {
        eprintln!(
            "Missing controller: {} (set FGC_ASSET_ROOT if needed)",
            index_path.display()
        );
        std::process::exit(1);
    }

    let data_dir = config.data_dir.clone();
    std::fs::create_dir_all(&data_dir).unwrap_or_else(|e| {
        eprintln!("Failed to create data dir {}: {e}", data_dir.display());
        std::process::exit(1);
    });

    let scoreboard_path = data_dir.join(SCOREBOARD_FILE);
    ensure_scoreboard_file(&scoreboard_path).unwrap_or_else(|e| {
        eprintln!("Failed to initialize scoreboard file: {e}");
        std::process::exit(1);
    });

    let auth_token: Option<Arc<str>> = config.token.as_ref().map(|t| Arc::from(t.as_str()));

    let state = Arc::new(AppState {
        auth_token,
        asset_root,
        scoreboard_path,
        rate_limiter: Arc::new(RateLimiter::new(config.rate_limit)),
        trust_proxy: config.trust_proxy,
        log_poll: config.log_poll(),
    });

    let lan_ip = get_lan_ip();
    let port = config.port;
    let bind = config.bind.clone();

    tracing::info!("FGC Scoreboard Server");
    if state.auth_required() {
        tracing::info!("  Auth:       enabled (token from FGC_AUTH_TOKEN or --token)");
    } else {
        tracing::info!("  Auth:       disabled");
    }
    tracing::info!(
        "  Data:       {} (ephemeral on redeploy when hosted)",
        state.scoreboard_path.display()
    );
    if config.tunnel_enabled() {
        tracing::info!(
            "  LAN:        http://{}:{}/ (local fallback)",
            lan_ip,
            port
        );
    } else {
        tracing::info!("  Controller: http://{}:{}/", lan_ip, port);
        tracing::info!(
            "  Overlay:    http://{}:{}/overlay/scoreboard.html",
            lan_ip,
            port
        );
    }
    tracing::info!("Listening on {}:{} (Ctrl+C to stop)", bind, port);
    tracing::info!("  Generate token: fgc-server --generate-token");

    let app = build_app(state);
    let addr: SocketAddr = format!("{}:{}", bind, port)
        .parse()
        .unwrap_or_else(|e| {
            eprintln!("Invalid bind address: {e}");
            std::process::exit(1);
        });

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap_or_else(|e| {
        eprintln!("Failed to bind {addr}: {e}");
        std::process::exit(1);
    });

    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
    });

    // Accept loop must be running before tunnel workers proxy to localhost.
    tokio::task::yield_now().await;

    let tunnel = if config.tunnel_enabled() {
        let host = tunnel::normalize_tunnel_host(&config.tunnel_host).unwrap_or_else(|err| {
            eprintln!("{err}");
            std::process::exit(1);
        });
        match tunnel::start_tunnel(host, config.tunnel_subdomain.clone(), port).await {
            Ok(handle) => Some(handle),
            Err(err) => {
                tracing::warn!("Tunnel unavailable: {err}");
                tracing::info!("Falling back to LAN-only mode");
                tracing::info!("  Controller: http://{}:{}/", lan_ip, port);
                tracing::info!(
                    "  Overlay:    http://{}:{}/overlay/scoreboard.html",
                    lan_ip,
                    port
                );
                None
            }
        }
    } else {
        None
    };

    match server.await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            eprintln!("Server error: {e}");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Server task failed: {e}");
            std::process::exit(1);
        }
    }

    if let Some(handle) = tunnel {
        handle.shutdown().await;
    }

    tracing::info!("Server stopped.");
}
