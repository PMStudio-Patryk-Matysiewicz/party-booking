use axum::{
    extract::{Form, State},
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use bcrypt::{hash, verify, DEFAULT_COST};

use crate::{
    db::AppState,
    handlers::{get_session_user, render_guest_nav, render_nav},
    models::{LoginForm, RegisterForm, SessionUser},
};

pub async fn login_page(jar: CookieJar) -> Response {
    if get_session_user(&jar).is_some() {
        return Redirect::to("/").into_response();
    }
    Html(render_login_form(None)).into_response()
}

pub async fn register_page(jar: CookieJar) -> Response {
    if get_session_user(&jar).is_some() {
        return Redirect::to("/").into_response();
    }
    Html(render_register_form(None)).into_response()
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> Response {
    let result = sqlx::query!(
        r#"SELECT u.id, u.username, u.email, u.password_hash, r.name AS role_name
           FROM users u
           JOIN roles r ON u.role_id = r.id
           WHERE u.email = $1"#,
        form.email
    )
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    match result {
        Some(row) => {
            let valid = verify(&form.password, &row.password_hash).unwrap_or(false);

            if valid {
                let session = SessionUser {
                    id: row.id,
                    username: row.username,
                    email: row.email,
                    role_name: row.role_name,
                };
                let cookie = build_session_cookie(&session);
                (jar.add(cookie), Redirect::to("/")).into_response()
            } else {
                Html(render_login_form(Some("Nieprawidłowy email lub hasło"))).into_response()
            }
        }
        None => Html(render_login_form(Some("Nieprawidłowy email lub hasło"))).into_response(),
    }
}

pub async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<RegisterForm>,
) -> Response {
    if form.password != form.confirm_password {
        return Html(render_register_form(Some("Hasła nie są zgodne"))).into_response();
    }
    if form.password.len() < 6 {
        return Html(render_register_form(Some(
            "Hasło musi mieć co najmniej 6 znaków",
        )))
        .into_response();
    }

    let password_hash = match hash(&form.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return Html(render_register_form(Some("Błąd serwera"))).into_response(),
    };

    let role = sqlx::query!("SELECT id FROM roles WHERE name = 'user'")
        .fetch_one(&state.pool)
        .await;

    let role_id = match role {
        Ok(r) => r.id,
        Err(_) => return Html(render_register_form(Some("Błąd serwera"))).into_response(),
    };

    let result = sqlx::query!(
        "INSERT INTO users (username, email, password_hash, role_id)
         VALUES ($1, $2, $3, $4)
         RETURNING id",
        form.username,
        form.email,
        password_hash,
        role_id
    )
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(row) => {
            let session = SessionUser {
                id: row.id,
                username: form.username,
                email: form.email,
                role_name: "user".to_string(),
            };
            let cookie = build_session_cookie(&session);
            (jar.add(cookie), Redirect::to("/")).into_response()
        }
        Err(e) => {
            let msg = if e.to_string().contains("unique") {
                "Użytkownik z tym emailem lub nazwą już istnieje"
            } else {
                "Błąd podczas rejestracji"
            };
            Html(render_register_form(Some(msg))).into_response()
        }
    }
}

pub async fn logout(jar: CookieJar) -> Response {
    let jar = jar.remove(
        Cookie::build(("session", ""))
            .path("/")
            .same_site(SameSite::Lax)
            .http_only(true)
            .build(),
    );
    (jar, Redirect::to("/auth/login")).into_response()
}

fn build_session_cookie(user: &SessionUser) -> Cookie<'static> {
    let json = serde_json::to_string(user).unwrap();
    Cookie::build(("session", json))
        .path("/")
        .same_site(SameSite::Lax)
        .http_only(true)
        .build()
}

fn render_login_form(error: Option<&str>) -> String {
    let error_html = render_error(error);
    format!(
        r#"<!DOCTYPE html>
<html lang="pl">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Logowanie – KlubEvents</title>
  <link rel="stylesheet" href="/static/css/app.css">
</head>
<body class="auth-page">
  <div class="auth-card">
    <h1 class="auth-logo">◆ KlubEvents</h1>
    <h2>Zaloguj się</h2>
    {error_html}
    <form method="POST" action="/auth/login">
      <div class="form-group">
        <label for="email">Email</label>
        <input type="email" id="email" name="email"
               required placeholder="jan@example.com">
      </div>
      <div class="form-group">
        <label for="password">Hasło</label>
        <input type="password" id="password" name="password"
               required placeholder="••••••••">
      </div>
      <button type="submit" class="btn btn-primary btn-full">
        Zaloguj się
      </button>
    </form>
    <p class="auth-link">
      Nie masz konta? <a href="/auth/register">Zarejestruj się</a>
    </p>
  </div>
</body>
</html>"#
    )
}

fn render_register_form(error: Option<&str>) -> String {
    let error_html = render_error(error);
    format!(
        r#"<!DOCTYPE html>
<html lang="pl">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Rejestracja – KlubEvents</title>
  <link rel="stylesheet" href="/static/css/app.css">
</head>
<body class="auth-page">
  <div class="auth-card">
    <h1 class="auth-logo">◆ KlubEvents</h1>
    <h2>Utwórz konto</h2>
    {error_html}
    <form method="POST" action="/auth/register">
      <div class="form-group">
        <label for="username">Nazwa użytkownika</label>
        <input type="text" id="username" name="username"
               required placeholder="jan_kowalski">
      </div>
      <div class="form-group">
        <label for="email">Email</label>
        <input type="email" id="email" name="email"
               required placeholder="jan@example.com">
      </div>
      <div class="form-group">
        <label for="password">Hasło</label>
        <input type="password" id="password" name="password"
               required placeholder="min. 6 znaków">
      </div>
      <div class="form-group">
        <label for="confirm_password">Potwierdź hasło</label>
        <input type="password" id="confirm_password" name="confirm_password"
               required placeholder="••••••••">
      </div>
      <button type="submit" class="btn btn-primary btn-full">
        Zarejestruj się
      </button>
    </form>
    <p class="auth-link">
      Masz już konto? <a href="/auth/login">Zaloguj się</a>
    </p>
  </div>
</body>
</html>"#
    )
}

fn render_error(error: Option<&str>) -> String {
    match error {
        Some(msg) => format!(r#"<div class="alert alert-error">{msg}</div>"#),
        None => String::new(),
    }
}
