use axum::{
    extract::{Query, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use futures_util::TryStreamExt;
use mongodb::bson::{doc, Document};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashSet;

use crate::AppState;

// An episode counts as finished once the playhead passes this share of runtime.
const FINISHED_RATIO: f64 = 0.9;

#[derive(Deserialize)]
pub struct ShowQuery {
    pub show: Option<String>,
}

fn bearer(headers: &HeaderMap) -> Option<String> {
    headers
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::to_string)
}

/// Resolves the caller's username from a verified JWT, or the error to return.
fn caller(state: &AppState, headers: &HeaderMap) -> Result<String, (StatusCode, Json<Value>)> {
    let token = bearer(headers).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(json!({"error": "missing bearer token"})),
    ))?;
    state
        .auth_service
        .verify_token(&token)
        .map(|claims| claims.sub)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "invalid or expired token"})),
            )
        })
}

fn to_json(doc: &Document) -> Value {
    json!({
        "show": doc.get_str("show").unwrap_or(""),
        "path": doc.get_str("path").unwrap_or(""),
        "position": doc.get_f64("position").unwrap_or(0.0),
        "duration": doc.get_f64("duration").unwrap_or(0.0),
        "finished": doc.get_bool("finished").unwrap_or(false),
        "updatedAt": doc.get_i64("updatedAt").unwrap_or(0),
    })
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// POST /progress — upsert one episode's playback position for the caller.
pub async fn save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let username = match caller(&state, &headers) {
        Ok(u) => u,
        Err(e) => return e,
    };

    let show = body.get("show").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    let path = body.get("path").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    if show.is_empty() || path.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "show and path required"})),
        );
    }

    let position = body.get("position").and_then(|v| v.as_f64()).unwrap_or(0.0).max(0.0);
    let duration = body.get("duration").and_then(|v| v.as_f64()).unwrap_or(0.0).max(0.0);
    // An explicit finished flag from the client (fired on `ended`) wins over the ratio.
    let finished = body
        .get("finished")
        .and_then(|v| v.as_bool())
        .unwrap_or(duration > 0.0 && position / duration >= FINISHED_RATIO);

    let filter = doc! { "username": &username, "show": &show, "path": &path };
    let update = doc! {
        "$set": {
            "position": position,
            "duration": duration,
            "finished": finished,
            "updatedAt": now_ms(),
        },
        "$setOnInsert": { "username": &username, "show": &show, "path": &path },
    };

    match state
        .database_service
        .watch_progress
        .update_one(filter, update)
        .upsert(true)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "show": show,
                "path": path,
                "position": position,
                "duration": duration,
                "finished": finished
            })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "failed to save progress"})),
        ),
    }
}

/// GET /progress          → most recent episode per show (one row each)
/// GET /progress?show=X   → every episode watched in show X, newest first
pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ShowQuery>,
) -> impl IntoResponse {
    let username = match caller(&state, &headers) {
        Ok(u) => u,
        Err(e) => return e,
    };

    let mut filter = doc! { "username": &username };
    let single_show = q.show.as_deref().map(str::trim).filter(|s| !s.is_empty());
    if let Some(show) = single_show {
        filter.insert("show", show);
    }

    let cursor = state
        .database_service
        .watch_progress
        .find(filter)
        .sort(doc! { "updatedAt": -1 })
        .await;

    let docs: Vec<Document> = match cursor {
        Ok(c) => match c.try_collect().await {
            Ok(d) => d,
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "database error"})),
                )
            }
        },
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "database error"})),
            )
        }
    };

    // Already sorted newest-first, so the first row seen for a show is its latest.
    let items: Vec<Value> = if single_show.is_some() {
        docs.iter().map(to_json).collect()
    } else {
        let mut seen = HashSet::new();
        docs.iter()
            .filter(|d| seen.insert(d.get_str("show").unwrap_or("").to_string()))
            .map(to_json)
            .collect()
    };

    (StatusCode::OK, Json(json!({ "items": items })))
}
