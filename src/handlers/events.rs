use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Path, Query, State};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use uuid::Uuid;

use crate::db::AppState;
use crate::handlers::get_session_user;
use crate::models::{EventWithLocation, SessionUser};

#[derive(Template)]
#[template(path = "events/list.html")]
pub struct EventsListTemplate {
    pub current_user: Option<SessionUser>,
    pub events: Vec<EventWithLocation>,
    pub locations: Vec<String>,
    pub categories: Vec<String>,
    pub selected_location: Option<String>,
    pub selected_category: Option<String>,
}

#[derive(Template)]
#[template(path = "events/_cards.html")]
pub struct EventsCardsTemplate {
    pub events: Vec<EventWithLocation>,
}

#[derive(Template)]
#[template(path = "events/detail.html")]
pub struct EventDetailTemplate {
    pub current_user: Option<SessionUser>,
    pub event: EventWithLocation,
    pub has_reservation: bool,
}

#[derive(Debug, Deserialize)]
pub struct EventFilter {
    pub location: Option<String>,
    pub category: Option<String>,
}

pub async fn list_events(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(filter): Query<EventFilter>,
) -> impl IntoResponse {
    let events = fetch_events(&state, &filter).await;
    let locations = fetch_location_names(&state).await;
    let categories = fetch_category_names(&state).await;

    EventsListTemplate {
        current_user: get_session_user(&jar),
        events,
        locations,
        categories,
        selected_location: filter.location.clone(),
        selected_category: filter.category.clone(),
    }
    .into_response()
}

pub async fn events_partial(
    State(state): State<AppState>,
    Query(filter): Query<EventFilter>,
) -> impl IntoResponse {
    let events = fetch_events(&state, &filter).await;
    EventsCardsTemplate { events }.into_response()
}

pub async fn event_detail(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let session_user = get_session_user(&jar);

    let event = sqlx::query_as!(
        EventWithLocation,
        r#"SELECT e.id, e.title, e.description, e.date, e.location_id,
                  e.created_at, l.name AS location_name,
                  l.address AS location_address
           FROM events e
           JOIN locations l ON e.location_id = l.id
           WHERE e.id = $1"#,
        id
    )
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    let event = match event {
        Some(e) => e,
        None => return axum::response::Redirect::to("/").into_response(),
    };

    let has_reservation = match &session_user {
        Some(u) => sqlx::query!(
            "SELECT id FROM reservations
                 WHERE user_id = $1 AND event_id = $2",
            u.id,
            id
        )
        .fetch_optional(&state.pool)
        .await
        .unwrap_or(None)
        .is_some(),
        None => false,
    };

    EventDetailTemplate {
        current_user: session_user,
        event,
        has_reservation,
    }
    .into_response()
}

async fn fetch_events(state: &AppState, filter: &EventFilter) -> Vec<EventWithLocation> {
    let location = filter.location.as_deref().filter(|s| !s.is_empty());
    let category = filter.category.as_deref().filter(|s| !s.is_empty());

    sqlx::query_as!(
        EventWithLocation,
        r#"SELECT e.id, e.title, e.description, e.date, e.location_id,
                  e.created_at, l.name AS location_name,
                  l.address AS location_address
           FROM events e
           JOIN locations l ON e.location_id = l.id
           WHERE ($1::text IS NULL OR l.name ILIKE $1)
             AND ($2::text IS NULL OR EXISTS (
                   SELECT 1 FROM event_categories ec
                   JOIN categories c ON ec.category_id = c.id
                   WHERE ec.event_id = e.id AND c.name ILIKE $2
                 ))
           ORDER BY e.date ASC"#,
        location,
        category,
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default()
}

async fn fetch_location_names(state: &AppState) -> Vec<String> {
    sqlx::query!("SELECT name FROM locations ORDER BY name")
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| r.name)
        .collect()
}

async fn fetch_category_names(state: &AppState) -> Vec<String> {
    sqlx::query!("SELECT name FROM categories ORDER BY name")
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| r.name)
        .collect()
}
