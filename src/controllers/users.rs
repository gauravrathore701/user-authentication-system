use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use mongodb::bson::{self, doc, Bson};
use serde_json::{json, Map, Value};

use crate::AppState;

pub async fn register(
    State(state): State<AppState>,
    Json(mut body): Json<Map<String, Value>>,
) -> impl IntoResponse {
    let username = match body.remove("username").and_then(|v| v.as_str().map(String::from)) {
        Some(u) => u,
        None => return (StatusCode::BAD_REQUEST, Json(json!({"error": "username required"}))),
    };
    let password = match body.remove("password").and_then(|v| v.as_str().map(String::from)) {
        Some(p) => p,
        None => return (StatusCode::BAD_REQUEST, Json(json!({"error": "password required"}))),
    };
    let email = match body.remove("email").and_then(|v| v.as_str().map(String::from)) {
        Some(e) => e,
        None => return (StatusCode::BAD_REQUEST, Json(json!({"error": "email required"}))),
    };

    let existing = state.database_service.users
        .find_one(doc! { "username": &username })
        .await;
    if let Ok(Some(_)) = existing {
        return (StatusCode::CONFLICT, Json(json!({"error": "username already exists"})));
    }

    let hashed = match bcrypt::hash(&password, bcrypt::DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "password hashing failed"}))),
    };

    let mut doc = doc! {
        "username": &username,
        "email": &email,
        "password": &hashed,
    };

    // any extra fields from the body get stored as-is
    for (key, val) in &body {
        doc.insert(key, bson::to_bson(val).unwrap_or(Bson::Null));
    }

    match state.database_service.users.insert_one(doc).await {
        Ok(result) => (
            StatusCode::CREATED,
            Json(json!({"message": "user created", "id": result.inserted_id.to_string()})),
        ),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "failed to save user"}))),
    }
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let username = match body.get("username").and_then(|v| v.as_str()) {
        Some(u) => u.to_string(),
        None => return (StatusCode::BAD_REQUEST, Json(json!({"error": "username required"}))),
    };
    let password = match body.get("password").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return (StatusCode::BAD_REQUEST, Json(json!({"error": "password required"}))),
    };

    let user = match state.database_service.users
        .find_one(doc! { "username": &username })
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid credentials"}))),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "database error"}))),
    };

    let stored_hash = user.get_str("password").unwrap_or("");
    match bcrypt::verify(&password, stored_hash) {
        Ok(true) => match state.auth_service.create_token(&username) {
            Ok(token) => (StatusCode::OK, Json(json!({"token": token}))),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "failed to generate token"}))),
        },
        _ => (StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid credentials"}))),
    }
}
