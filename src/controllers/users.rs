use axum::{
    extract::State,
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use mongodb::bson::{self, doc, Bson};
use serde_json::{json, Map, Value};

use crate::middlewares::client::ClientId;
use crate::AppState;

/// Cookie the browser file server uses. `/users/login` sets it so a plain
/// `<img>`/link request carries the token — a file listing can't add an
/// Authorization header. HttpOnly so a script on any cursedshrine subdomain
/// cannot read it.
pub const TOKEN_COOKIE: &str = "auth_token";

/// A master account: logs in to every allowed client without being listed
/// under it. The `clients` array is ignored for these users — the password is
/// the only check. Set `isUniversalUser: true` on the user document.
///
/// This does not bypass the `X-Client-Id` middleware: an unknown client id is
/// still a 403 for everyone. It only skips the per-user membership test.
fn is_universal(user: &bson::Document) -> bool {
    user.get_bool("isUniversalUser").unwrap_or(false)
}

/// The client ids a user is registered under.
fn clients_of(user: &bson::Document) -> Vec<String> {
    user.get_array("clients")
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

pub async fn register(
    State(state): State<AppState>,
    Extension(ClientId(client)): Extension<ClientId>,
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

    // A username is global, but its client list is not. Registering an existing
    // user under a new project grants that project instead of failing — the
    // password is still checked, so this can't be used to hijack the account.
    if let Ok(Some(user)) = existing {
        if clients_of(&user).contains(&client) {
            return (StatusCode::CONFLICT, Json(json!({"error": "username already exists"})));
        }
        let stored_hash = user.get_str("password").unwrap_or("");
        if !matches!(bcrypt::verify(&password, stored_hash), Ok(true)) {
            return (StatusCode::CONFLICT, Json(json!({"error": "username already exists"})));
        }
        return match state.database_service.users
            .update_one(doc! { "username": &username }, doc! { "$addToSet": { "clients": &client } })
            .await
        {
            Ok(_) => (StatusCode::OK, Json(json!({"message": "client granted", "client": client}))),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "failed to save user"}))),
        };
    }

    let hashed = match bcrypt::hash(&password, bcrypt::DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "password hashing failed"}))),
    };

    let mut doc = doc! {
        "username": &username,
        "email": &email,
        "password": &hashed,
        "clients": vec![client.clone()],
    };

    // any extra fields from the body get stored as-is
    for (key, val) in &body {
        doc.insert(key, bson::to_bson(val).unwrap_or(Bson::Null));
    }

    match state.database_service.users.insert_one(doc).await {
        Ok(result) => {
            let id = match result.inserted_id {
                Bson::ObjectId(oid) => oid.to_hex(),
                other => other.to_string(),
            };
            (StatusCode::CREATED, Json(json!({"message": "user created", "id": id})))
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "failed to save user"}))),
    }
}

pub async fn login(
    State(state): State<AppState>,
    Extension(ClientId(client)): Extension<ClientId>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let username = match body.get("username").and_then(|v| v.as_str()) {
        Some(u) => u.to_string(),
        None => return (StatusCode::BAD_REQUEST, HeaderMap::new(), Json(json!({"error": "username required"}))),
    };
    let password = match body.get("password").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return (StatusCode::BAD_REQUEST, HeaderMap::new(), Json(json!({"error": "password required"}))),
    };

    let unauthorized = || {
        (StatusCode::UNAUTHORIZED, HeaderMap::new(), Json(json!({"error": "invalid credentials"})))
    };

    let user = match state.database_service.users
        .find_one(doc! { "username": &username })
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => return unauthorized(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new(), Json(json!({"error": "database error"}))),
    };

    // Wrong project reads as bad credentials on purpose: telling a caller the
    // account exists but belongs elsewhere leaks which users are on which app.
    // A universal user skips this: every allowed client is theirs.
    if !is_universal(&user) && !clients_of(&user).contains(&client) {
        return unauthorized();
    }

    let stored_hash = user.get_str("password").unwrap_or("");
    match bcrypt::verify(&password, stored_hash) {
        Ok(true) => match state.auth_service.create_token(&username, &client) {
            Ok(token) => {
                let mut headers = HeaderMap::new();
                let cookie = format!(
                    "{}={}; Path=/; Max-Age=86400; HttpOnly; Secure; SameSite=Lax",
                    TOKEN_COOKIE, token
                );
                if let Ok(v) = cookie.parse() {
                    headers.insert(axum::http::header::SET_COOKIE, v);
                }
                (StatusCode::OK, headers, Json(json!({"token": token, "client": client})))
            }
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new(), Json(json!({"error": "failed to generate token"}))),
        },
        _ => unauthorized(),
    }
}

/// Reads the JWT from `Authorization: Bearer …`, else the auth cookie.
pub fn token_from(headers: &HeaderMap) -> Option<String> {
    if let Some(t) = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        return Some(t.to_string());
    }
    headers
        .get(axum::http::header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .filter_map(|c| c.trim().split_once('='))
        .find(|(k, _)| *k == TOKEN_COOKIE)
        .map(|(_, v)| v.to_string())
}

/// GET /users/verify — 200 if the caller holds a live token for *this* client.
///
/// This is what Caddy's `forward_auth` calls before serving a file. It has no
/// body worth reading; the status is the answer.
pub async fn verify(
    State(state): State<AppState>,
    Extension(ClientId(client)): Extension<ClientId>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let token = match token_from(&headers) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "missing token"}))),
    };
    match state.auth_service.verify_token(&token) {
        // A token minted for another project is as good as no token here.
        Ok(claims) if claims.client == client => (
            StatusCode::OK,
            Json(json!({"username": claims.sub, "client": claims.client})),
        ),
        Ok(_) => (StatusCode::FORBIDDEN, Json(json!({"error": "token issued for another client"}))),
        Err(_) => (StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid or expired token"}))),
    }
}

/// POST /users/logout — clears the cookie. The JWT itself stays valid until it
/// expires; there is no revocation list yet.
pub async fn logout() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    let cookie = format!(
        "{}=; Path=/; Max-Age=0; HttpOnly; Secure; SameSite=Lax",
        TOKEN_COOKIE
    );
    if let Ok(v) = cookie.parse() {
        headers.insert(axum::http::header::SET_COOKIE, v);
    }
    (StatusCode::OK, headers, Json(json!({"message": "logged out"})))
}
