pub mod auth;
pub mod events;
pub mod reservations;

use crate::models::SessionUser;
use axum_extra::extract::CookieJar;

pub fn get_session_user(jar: &CookieJar) -> Option<SessionUser> {
    let cookie = jar.get("session")?;
    serde_json::from_str(cookie.value()).ok()
}

pub fn render_nav(user: &SessionUser) -> String {
    let admin_link = if user.is_admin() {
        r#"<a href="/admin">Panel admina</a>"#
    } else {
        ""
    };

    format!(
        r#"<nav class="navbar">
  <a href="/" class="nav-brand">◆ KlubEvents</a>
  <div class="nav-links">
    <a href="/">Wydarzenia</a>
    <a href="/reservations">Moje rezerwacje</a>
    {admin_link}
  </div>
  <div class="nav-user">
    <span>👤 {}</span>
    <form method="POST" action="/auth/logout">
      <button type="submit" class="btn btn-ghost btn-sm">Wyloguj</button>
    </form>
  </div>
</nav>"#,
        user.username
    )
}

pub fn render_guest_nav() -> String {
    r#"<nav class="navbar">
  <a href="/" class="nav-brand">◆ KlubEvents</a>
  <div class="nav-links">
    <a href="/">Wydarzenia</a>
  </div>
  <div class="nav-user">
    <a href="/auth/login" class="btn btn-ghost btn-sm">Zaloguj się</a>
    <a href="/auth/register" class="btn btn-primary btn-sm">Rejestracja</a>
  </div>
</nav>"#
        .to_string()
}

use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};

pub fn require_user(jar: &CookieJar) -> Result<crate::models::SessionUser, Response> {
    get_session_user(jar).ok_or_else(|| Redirect::to("/auth/login").into_response())
}
