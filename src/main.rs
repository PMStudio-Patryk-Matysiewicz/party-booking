mod db;
mod models;

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use uuid::Uuid;

use crate::db::AppState;
use crate::models::EventWithLocation;

// Handler zwracający listę wszystkich wydarzeń jako JSON
// State(state) — Axum automatycznie wstrzykuje AppState
// Json(...) — Axum automatycznie serializuje strukturę do JSON
async fn list_events(
    State(state): State<AppState>,
) -> Result<Json<Vec<EventWithLocation>>, String> {
    let events = sqlx::query_as!(
        EventWithLocation,
        r#"
        SELECT
            e.id,
            e.title,
            e.description,
            e.date,
            e.location_id,
            e.created_at,
            l.name    AS location_name,
            l.address AS location_address
        FROM events e
        JOIN locations l ON e.location_id = l.id
        ORDER BY e.date ASC
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Json(events))
}

// Handler zwracający jedno wydarzenie po ID
// Path(id) — Axum wycina :id z URL i parsuje jako Uuid
async fn get_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<EventWithLocation>, String> {
    let event = sqlx::query_as!(
        EventWithLocation,
        r#"
        SELECT
            e.id,
            e.title,
            e.description,
            e.date,
            e.location_id,
            e.created_at,
            l.name    AS location_name,
            l.address AS location_address
        FROM events e
        JOIN locations l ON e.location_id = l.id
        WHERE e.id = $1
        "#,
        id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Nie znaleziono wydarzenia".to_string())?;

    Ok(Json(event))
}

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
        .route("/events", get(list_events))
        .route("/events/:id", get(get_event))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("✓ Serwer na http://localhost:3000");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
