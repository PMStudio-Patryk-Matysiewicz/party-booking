use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use uuid::Uuid;

use crate::db::AppState;
use crate::handlers::{get_session_user, render_guest_nav, render_nav};
use crate::models::EventWithLocation;

#[derive(Debug, Deserialize)]
pub struct EventFilter {
    pub category: Option<String>,
    pub location: Option<String>,
}

pub async fn list_events(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(filter): Query<EventFilter>,
) -> Response {
    let events = fetch_events(&state, &filter).await;

    let nav = match get_session_user(&jar) {
        Some(user) => render_nav(&user),
        None => render_guest_nav(),
    };

    let grid_html = render_event_cards(&events);

    let locations = fetch_location_names(&state).await;
    let categories = fetch_category_names(&state).await;

    Html(render_page(
        &nav,
        &grid_html,
        &locations,
        &categories,
        &filter,
    ))
    .into_response()
}

pub async fn event_detail(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<Uuid>,
) -> Response {
    let nav = match get_session_user(&jar) {
        Some(ref user) => render_nav(user),
        None => render_guest_nav(),
    };

    let event = sqlx::query_as!(
        EventWithLocation,
        r#"SELECT e.id, e.title, e.description, e.date, e.location_id,
                  e.created_at, l.name AS location_name, l.address AS location_address
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
        None => return Html(render_404()).into_response(),
    };

    let session_user = get_session_user(&jar);
    let has_reservation = match &session_user {
        Some(u) => sqlx::query!(
            "SELECT id FROM reservations WHERE user_id = $1 AND event_id = $2",
            u.id,
            id
        )
        .fetch_optional(&state.pool)
        .await
        .unwrap_or(None)
        .is_some(),
        None => false,
    };

    Html(render_detail_page(
        &nav,
        &event,
        &session_user,
        has_reservation,
    ))
    .into_response()
}

pub async fn events_partial(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(filter): Query<EventFilter>,
) -> Response {
    let events = fetch_events(&state, &filter).await;
    Html(render_event_cards(&events)).into_response()
}

async fn fetch_events(state: &AppState, filter: &EventFilter) -> Vec<EventWithLocation> {
    sqlx::query_as!(
        EventWithLocation,
        r#"SELECT e.id, e.title, e.description, e.date, e.location_id,
                  e.created_at, l.name AS location_name, l.address AS location_address
           FROM events e
           JOIN locations l ON e.location_id = l.id
           WHERE ($1::text IS NULL OR l.name ILIKE $1)
             AND ($2::text IS NULL OR EXISTS (
                   SELECT 1 FROM event_categories ec
                   JOIN categories c ON ec.category_id = c.id
                   WHERE ec.event_id = e.id AND c.name ILIKE $2
                 ))
           ORDER BY e.date ASC"#,
        match &filter.location {
            Some(loc) if !loc.is_empty() => Some(loc),
            _ => None,
        },
        match &filter.category {
            Some(cat) if !cat.is_empty() => Some(cat),
            _ => None,
        },
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

pub fn render_event_cards(events: &[EventWithLocation]) -> String {
    if events.is_empty() {
        return r#"<div class="empty-state">
            <p>Brak wydarzeń spełniających kryteria.</p>
        </div>"#
            .to_string();
    }

    events
        .iter()
        .map(|e| {
            let date_fmt = e.date.format("%d.%m.%Y · %H:%M").to_string();
            format!(
                r#"<article class="event-card">
  <div class="event-card-body">
    <h3 class="event-card-title">{title}</h3>
    <p class="event-card-desc">{desc}</p>
    <div class="event-card-meta">
      <span>📅 {date}</span>
      <span>📍 {loc}</span>
    </div>
  </div>
  <div class="event-card-footer">
    <a href="/events/{id}" class="btn btn-primary btn-sm">Szczegóły →</a>
  </div>
</article>"#,
                title = e.title,
                desc = truncate(&e.description, 100),
                date = date_fmt,
                loc = e.location_name,
                id = e.id,
            )
        })
        .collect()
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

fn render_page(
    nav: &str,
    grid_html: &str,
    locations: &[String],
    categories: &[String],
    filter: &EventFilter,
) -> String {
    let loc_options = build_options(locations, filter.location.as_deref());
    let cat_options = build_options(categories, filter.category.as_deref());

    let filters_bar = format!(
        r##"<div class="filters-bar"
     hx-get="/partials/events"
     hx-trigger="change from:select"
     hx-target="#events-grid"
     hx-include="[name='location'],[name='category']"
     hx-indicator="#spinner">
  <select name="location" class="filter-select">
    <option value="">Wszystkie lokalizacje</option>
    {loc_options}
  </select>
  <select name="category" class="filter-select">
    <option value="">Wszystkie kategorie</option>
    {cat_options}
  </select>
  <span id="spinner" class="htmx-indicator">⟳</span>
</div>"##
    );

    format!(
        r#"<!DOCTYPE html>
<html lang="pl">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>KlubEvents</title>
  <link rel="stylesheet" href="/static/css/app.css">
</head>
<body>
  {nav}
  <main class="container">
    <div class="page-hero">
      <h1>Nadchodzące wydarzenia</h1>
      <p>Znajdź coś dla siebie i zarezerwuj miejsce</p>
    </div>
    {filters_bar}
    <div id="events-grid" class="events-grid">
      {grid_html}
    </div>
  </main>
  <script src="/static/js/htmx.min.js"></script>
</body>
</html>"#
    )
}

fn render_detail_page(
    nav: &str,
    event: &EventWithLocation,
    session_user: &Option<crate::models::SessionUser>,
    has_reservation: bool,
) -> String {
    let date_fmt = event.date.format("%d.%m.%Y · %H:%M").to_string();

    let action_html = match (session_user, has_reservation) {
        (None, _) => r#"<a href="/auth/login" class="btn btn-primary">
                 Zaloguj się, aby zarezerwować
               </a>"#
            .to_string(),
        (Some(_), true) => r#"<div class="reservation-badge">
                 ✓ Masz rezerwację na to wydarzenie
               </div>"#
            .to_string(),
        (Some(_), false) => {
            format!(
                r#"<form method="POST" action="/reservations">
                     <input type="hidden" name="event_id" value="{id}">
                     <button type="submit" class="btn btn-primary">
                       Zarezerwuj miejsce
                     </button>
                   </form>"#,
                id = event.id
            )
        }
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="pl">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{title} – KlubEvents</title>
  <link rel="stylesheet" href="/static/css/app.css">
</head>
<body>
  {nav}
  <main class="container">
    <a href="/" class="back-link">← Wróć do listy</a>
    <article class="event-detail">
      <h1>{title}</h1>
      <div class="event-meta">
        <span>📅 {date}</span>
        <span>📍 {loc} · {addr}</span>
      </div>
      <p class="event-description">{desc}</p>
      <div class="event-action">
        {action_html}
      </div>
    </article>
  </main>
  <script src="/static/js/htmx.min.js"></script>
</body>
</html>"#,
        title = event.title,
        date = date_fmt,
        loc = event.location_name,
        addr = event.location_address,
        desc = event.description,
    )
}

fn render_404() -> String {
    r#"<!DOCTYPE html>
<html lang="pl">
<head>
  <meta charset="UTF-8">
  <title>404 – KlubEvents</title>
  <link rel="stylesheet" href="/static/css/app.css">
</head>
<body>
  <main class="container" style="text-align:center;padding-top:4rem">
    <h1>404</h1>
    <p>Nie znaleziono wydarzenia.</p>
    <a href="/" class="btn btn-primary">Wróć na stronę główną</a>
  </main>
</body>
</html>"#
        .to_string()
}

fn build_options(items: &[String], selected: Option<&str>) -> String {
    items
        .iter()
        .map(|item| {
            let sel = if selected == Some(item.as_str()) {
                " selected"
            } else {
                ""
            };
            format!(r#"<option value="{item}"{sel}>{item}</option>"#)
        })
        .collect()
}
