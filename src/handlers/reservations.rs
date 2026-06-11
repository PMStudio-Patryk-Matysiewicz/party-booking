use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Form, Path, State};
use axum::response::Redirect;
use axum_extra::extract::CookieJar;
use uuid::Uuid;

use crate::db::AppState;
use crate::handlers::require_user;
use crate::models::{CreateReservationForm, ReservationWithDetails, SessionUser};

#[derive(Template)]
#[template(path = "reservations/list.html")]
pub struct ReservationsTemplate {
    pub current_user: Option<SessionUser>,
    pub reservations: Vec<ReservationWithDetails>,
}

pub async fn my_reservations(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    let user = match require_user(&jar) {
        Ok(u) => u,
        Err(r) => return r,
    };

    let reservations = sqlx::query_as!(
        ReservationWithDetails,
        r#"SELECT r.id, r.user_id, r.created_at, u.username,
                  e.id AS event_id, e.title AS event_title,
                  e.date AS event_date, l.name AS location_name
           FROM reservations r
           JOIN users u     ON r.user_id    = u.id
           JOIN events e    ON r.event_id   = e.id
           JOIN locations l ON e.location_id = l.id
           WHERE r.user_id = $1
           ORDER BY e.date ASC"#,
        user.id
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let current_user = Some(user);
    ReservationsTemplate {
        current_user,
        reservations,
    }
    .into_response()
}

pub async fn create_reservation(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<CreateReservationForm>,
) -> impl IntoResponse {
    let user = match require_user(&jar) {
        Ok(u) => u,
        Err(r) => return r,
    };
    sqlx::query!(
        "INSERT INTO reservations (user_id, event_id) VALUES ($1, $2)",
        user.id,
        form.event_id
    )
    .execute(&state.pool)
    .await
    .ok();
    Redirect::to(&format!("/events/{}", form.event_id)).into_response()
}

pub async fn cancel_reservation(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
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
