mod controllers;
mod services;

use axum::{routing::post, Router};
use services::{auth_service::AuthService, database_service::DatabaseService};
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub database_service: DatabaseService,
    pub auth_service: AuthService,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let database_service = DatabaseService::new().await;
    let auth_service = AuthService::new();

    let state = AppState { database_service, auth_service };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/users/register", post(controllers::users::register))
        .route("/users/login", post(controllers::users::login))
        .route("/progress", post(controllers::progress::save).get(controllers::progress::list))
        .layer(cors)
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "4183".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    println!("User Auth API running on port {}", port);
    axum::serve(listener, app).await.unwrap();
}
