mod db;

use axum::{routing::get, Router};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;

async fn hello() -> &'static str {
    "Klub działa!"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // Logi w konsoli
    tracing_subscriber::fmt::init();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL musi być ustawione w .env");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::query("SELECT 1").execute(&pool).await?;
    println!("✓ Połączono z bazą danych");

    let state = db::AppState { pool };

    let app = Router::new().route("/", get(hello)).with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("✓ Serwer na http://localhost:3000");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
