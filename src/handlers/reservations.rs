use axum::{
    extract::{Form, Path, State},
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use uuid::Uuid;

use crate::{
    db::AppState,
    handlers::{render_nav, require_user},
    models::{CreateReservationForm, ReservationWithDetails},
};

pub async fn my_reservations(State(state): State<AppState>, jar: CookieJar) -> Response {
    let user = match require_user(&jar) {
        Ok(u) => u,
        Err(r) => return r,
    };

    let reservations = sqlx::query_as!(
        ReservationWithDetails,
        r#"SELECT
               r.id,
               r.user_id,
               r.created_at,
               u.username,
               e.id   AS event_id,
               e.title AS event_title,
               e.date  AS event_date,
               l.name  AS location_name
           FROM reservations r
           JOIN users  u ON r.user_id   = u.id
           JOIN events e ON r.event_id  = e.id
           JOIN locations l ON e.location_id = l.id
           WHERE r.user_id = $1
           ORDER BY e.date ASC"#,
        user.id
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let nav = render_nav(&user);
    Html(render_reservations_page(&nav, &reservations)).into_response()
}

pub async fn create_reservation(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<CreateReservationForm>,
) -> Response {
    let user = match require_user(&jar) {
        Ok(u) => u,
        Err(r) => return r,
    };

    let result = sqlx::query!(
        "INSERT INTO reservations (user_id, event_id)
         VALUES ($1, $2)",
        user.id,
        form.event_id
    )
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => Redirect::to(&format!("/events/{}", form.event_id)).into_response(),
        Err(e) if e.to_string().contains("unique") => {
            Redirect::to(&format!("/events/{}", form.event_id)).into_response()
        }
        Err(_) => Redirect::to("/").into_response(),
    }
}

pub async fn cancel_reservation(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<Uuid>,
) -> Response {
    let user = match require_user(&jar) {
        Ok(u) => u,
        Err(r) => return r,
    };

    sqlx::query!(
        "DELETE FROM reservations WHERE id = $1 AND user_id = $2",
        id,
        user.id
    )
    .execute(&state.pool)
    .await
    .ok();

    Redirect::to("/reservations").into_response()
}

fn render_reservations_page(nav: &str, reservations: &[ReservationWithDetails]) -> String {
    let content = if reservations.is_empty() {
        r#"<div class="empty-state">
             <p>Nie masz jeszcze żadnych rezerwacji.</p>
             <a href="/" class="btn btn-primary" style="margin-top:1rem">
               Przeglądaj wydarzenia
             </a>
           </div>"#
            .to_string()
    } else {
        let rows: String = reservations
            .iter()
            .map(|r| {
                let date = r.event_date.format("%d.%m.%Y · %H:%M").to_string();
                let cancel_url = format!("/reservations/{}/cancel", r.id);
                format!(
                    r#"<tr>
  <td><a href="/events/{event_id}">{title}</a></td>
  <td>{date}</td>
  <td>{loc}</td>
  <td>
    <form method="POST" action="{cancel_url}">
      <button type="submit" class="btn btn-danger btn-sm"
              onclick="return confirm('Anulować rezerwację?')">
        Anuluj
      </button>
    </form>
  </td>
</tr>"#,
                    event_id = r.event_id,
                    title = r.event_title,
                    loc = r.location_name,
                )
            })
            .collect();

        format!(
            r#"<div class="card">
  <table class="table">
    <thead>
      <tr>
        <th>Wydarzenie</th>
        <th>Data</th>
        <th>Lokalizacja</th>
        <th>Akcja</th>
      </tr>
    </thead>
    <tbody>{rows}</tbody>
  </table>
</div>"#
        )
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="pl">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Moje rezerwacje – KlubEvents</title>
  <link rel="stylesheet" href="/static/css/app.css">
</head>
<body>
  {nav}
  <main class="container">
    <h1 class="page-title">Moje rezerwacje</h1>
    {content}
  </main>
</body>
</html>"#
    )
}
