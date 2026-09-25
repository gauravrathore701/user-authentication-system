//! Client (tenant) gate.
//!
//! Every request must name the project it belongs to in the `X-Client-Id`
//! header. A user registered under `shows_project` cannot log in to
//! `files_project` and vice versa — the check runs here, once, before any
//! controller sees the request.
//!
//! The allowlist is hardcoded on purpose for now: there are two callers and
//! no admin UI to manage a table with. Moving it to Mongo later only means
//! swapping `is_allowed()`.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub const CLIENT_HEADER: &str = "X-Client-Id";

pub const ALLOWED_CLIENTS: [&str; 2] = ["shows_project", "files_project"];

/// The client id of the current request, put in the extensions by the
/// middleware. Cloned into controllers via `Extension<ClientId>`.
#[derive(Clone, Debug)]
pub struct ClientId(pub String);

pub fn is_allowed(client: &str) -> bool {
    ALLOWED_CLIENTS.contains(&client)
}

/// Rejects anything without a known `X-Client-Id`, then hands the id on.
///
/// 400 when the header is missing or unreadable, 403 when it names a client
/// that does not exist — a caller that forgot the header and one that guessed
/// a tenant name are different mistakes and worth telling apart in logs.
pub async fn require_client(mut req: Request, next: Next) -> Response {
    let raw = req
        .headers()
        .get(CLIENT_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim().to_string())
        .unwrap_or_default();

    if raw.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("{} header required", CLIENT_HEADER)})),
        )
            .into_response();
    }
    if !is_allowed(&raw) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "unknown client"})),
        )
            .into_response();
    }

    req.extensions_mut().insert(ClientId(raw));
    next.run(req).await
}
