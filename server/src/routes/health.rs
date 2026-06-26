use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

use crate::auth::{check_auth, unauthorized_response};
use crate::state::AppState;

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, r#"{"ok":true}"#)
}

pub async fn auth_check(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    if !check_auth(&headers, state.auth_token.as_deref().map(|s| s.as_ref())) {
        return unauthorized_response();
    }
    StatusCode::NO_CONTENT.into_response()
}
