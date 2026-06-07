mod db;
mod handlers;
mod models;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

use crate::db::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").expect("Brak DATABASE_URL w .env");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::query("SELECT 1").execute(&pool).await?;
    println!("✓ Połączono z bazą");

    let state = AppState { pool };

    let app = Router::new()
        .route("/", get(handlers::events::list_events))
        .route("/events/:id", get(handlers::events::event_detail))
        .route("/partials/events", get(handlers::events::events_partial))
        .route(
            "/auth/login",
            get(handlers::auth::login_page).post(handlers::auth::login),
        )
        .route(
            "/auth/register",
            get(handlers::auth::register_page).post(handlers::auth::register),
        )
        .route("/auth/logout", post(handlers::auth::logout))
        .route(
            "/reservations",
            get(handlers::reservations::my_reservations)
                .post(handlers::reservations::create_reservation),
        )
        .route(
            "/reservations/:id/cancel",
            post(handlers::reservations::cancel_reservation),
        )
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("✓ Serwer na http://localhost:3000");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
