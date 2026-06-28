use axum::{
    body::Bytes,
    extract::{Request, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::sync::Arc;

use crate::auth::{check_auth, unauthorized_response};
use crate::routes::client_ip;
use crate::state::AppState;

pub const MAX_BODY_SIZE: usize = 65536;
pub const SCOREBOARD_FILE: &str = "scoreboard.json";
pub const MAX_COUNTERS: usize = 8;
pub const MAX_COUNTER_LABEL: usize = 64;
pub const MAX_COUNTER_ID: usize = 32;
pub const MAX_SCORE_VALUE: u32 = 999;

pub static ALLOWED_KEYS: [&str; 9] = [
    "p1Name",
    "p1Team",
    "p1Score",
    "p2Name",
    "p2Team",
    "p2Score",
    "round",
    "game",
    "timestamp",
];

pub fn default_data() -> Value {
    json!({
        "p1Name": "",
        "p1Team": "",
        "p1Score": "0",
        "p2Name": "",
        "p2Team": "",
        "p2Score": "0",
        "round": "",
        "game": "",
        "timestamp": "",
        "counters": {}
    })
}

pub fn ensure_scoreboard_file(path: &std::path::Path) -> std::io::Result<()> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = default_data();
    std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}

pub async fn get_scoreboard(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let path = &state.scoreboard_path;
    let body = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(_) => serde_json::to_string_pretty(&default_data()).unwrap_or_default(),
    };
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        body,
    )
}

pub async fn post_scoreboard(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Request,
) -> impl IntoResponse {
    if !check_auth(&headers, state.auth_token.as_deref().map(|s| s.as_ref())) {
        return unauthorized_response();
    }

    let ip = client_ip(&req, state.trust_proxy);
    if let Err(retry) = state.rate_limiter.check(ip).await {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [
                (header::RETRY_AFTER, retry.to_string()),
                (header::CONTENT_TYPE, "application/json".to_string()),
            ],
            r#"{"error":"rate_limit_exceeded"}"#,
        )
            .into_response();
    }

    let (_parts, body) = req.into_parts();
    let bytes = match axum::body::to_bytes(body, MAX_BODY_SIZE + 1).await {
        Ok(b) => b,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Invalid body").into_response();
        }
    };

    if bytes.is_empty() || bytes.len() > MAX_BODY_SIZE {
        let status = if bytes.len() > MAX_BODY_SIZE {
            StatusCode::PAYLOAD_TOO_LARGE
        } else {
            StatusCode::BAD_REQUEST
        };
        return status.into_response();
    }

    let data: Value = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Invalid JSON").into_response();
        }
    };

    if !validate_schema(&data) {
        return (StatusCode::BAD_REQUEST, "Invalid schema").into_response();
    }

    let path = &state.scoreboard_path;
    let tmp_path = path.with_extension("json.tmp");
    if let Err(e) = atomic_write(path, &tmp_path, &bytes) {
        tracing::error!("Failed to write scoreboard: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        r#"{"ok":true}"#,
    )
        .into_response()
}

fn validate_schema(data: &Value) -> bool {
    if !data.is_object() {
        return false;
    }
    let obj = data.as_object().unwrap();
    let mut allowed: HashSet<&str> = ALLOWED_KEYS.iter().copied().collect();
    allowed.insert("counters");
    if !obj.keys().all(|k| allowed.contains(k.as_str())) {
        return false;
    }
    for (key, value) in obj {
        if key == "counters" {
            if !validate_counters(value) {
                return false;
            }
        } else if key == "p1Score" || key == "p2Score" {
            let s = match value.as_str() {
                Some(s) => s,
                None => return false,
            };
            if !validate_numeric_score(s) {
                return false;
            }
        } else if !value.is_string() || value.as_str().map(|s| s.len() > 128).unwrap_or(true) {
            return false;
        }
    }
    true
}

fn validate_counters(value: &Value) -> bool {
    let counters = match value.as_object() {
        Some(o) => o,
        None => return false,
    };
    if counters.len() > MAX_COUNTERS {
        return false;
    }
    for (id, entry) in counters {
        if id.is_empty() || id.len() > MAX_COUNTER_ID {
            return false;
        }
        let entry_obj = match entry.as_object() {
            Some(o) => o,
            None => return false,
        };
        if entry_obj.len() != 2
            || !entry_obj.contains_key("label")
            || !entry_obj.contains_key("value")
        {
            return false;
        }
        for (field, field_val) in entry_obj {
            if field != "label" && field != "value" {
                return false;
            }
            let s = match field_val.as_str() {
                Some(s) => s,
                None => return false,
            };
            match field.as_str() {
                "label" if s.len() > MAX_COUNTER_LABEL => return false,
                "value" if !validate_numeric_score(s) => return false,
                _ => {}
            }
        }
    }
    true
}

fn validate_numeric_score(s: &str) -> bool {
    if s.is_empty() || s.len() > 3 || !s.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    s.parse::<u32>().map(|n| n <= MAX_SCORE_VALUE).unwrap_or(false)
}

fn atomic_write(
    path: &std::path::Path,
    tmp_path: &std::path::Path,
    body: &Bytes,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(tmp_path, body)?;
    std::fs::rename(tmp_path, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accepts_triple_digit_counter() {
        let data = json!({
            "p1Name": "",
            "p1Team": "",
            "p1Score": "0",
            "p2Name": "",
            "p2Team": "",
            "p2Score": "0",
            "round": "",
            "game": "",
            "timestamp": "1",
            "counters": {
                "abc123": { "label": "Stock", "value": "100" }
            }
        });
        assert!(validate_schema(&data), "expected triple-digit counter to pass");
    }

    #[test]
    fn accepts_exact_post_payload() {
        let raw = r#"{"p1Name":"","p1Team":"","p1Score":"110","p2Name":"","p2Team":"","p2Score":"0","round":"","game":"","timestamp":"1","counters":{"abc123":{"label":"Stock","value":"100"}}}"#;
        let data: Value = serde_json::from_str(raw).unwrap();
        assert!(validate_numeric_score("100"));
        assert!(validate_numeric_score("110"));
        assert!(validate_schema(&data), "exact POST payload should pass");
    }

    #[test]
    fn rejects_four_digit_counter() {
        let data = json!({
            "p1Name": "",
            "p1Team": "",
            "p1Score": "0",
            "p2Name": "",
            "p2Team": "",
            "p2Score": "0",
            "round": "",
            "game": "",
            "timestamp": "1",
            "counters": {
                "x": { "label": "Bad", "value": "1000" }
            }
        });
        assert!(!validate_schema(&data));
    }
}
