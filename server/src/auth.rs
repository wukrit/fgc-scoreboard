use axum::{
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use subtle::ConstantTimeEq;

pub const AUTH_REALM: &str = "FGC Scoreboard";

pub fn parse_bearer(auth_header: Option<&str>) -> Option<String> {
    let header = auth_header?;
    let rest = header.strip_prefix("Bearer ")?;
    let token = rest.trim();
    if token.is_empty() {
        return None;
    }
    Some(token.to_string())
}

pub fn check_auth(headers: &HeaderMap, expected: Option<&str>) -> bool {
    match expected {
        None => true,
        Some(expected) => {
            let presented = parse_bearer(headers.get(header::AUTHORIZATION).and_then(|h| h.to_str().ok()));
            match presented {
                None => false,
                Some(token) => {
                    if token.len() != expected.len() {
                        return false;
                    }
                    token.as_bytes().ct_eq(expected.as_bytes()).into()
                }
            }
        }
    }
}

pub fn unauthorized_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [
            (
                header::WWW_AUTHENTICATE,
                format!("Bearer realm=\"{AUTH_REALM}\", error=\"invalid_token\""),
            ),
            (header::CONTENT_TYPE, "application/json".to_string()),
        ],
        r#"{"error":"invalid_token"}"#,
    )
        .into_response()
}
