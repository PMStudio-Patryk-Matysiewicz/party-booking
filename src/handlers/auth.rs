use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Form, State};
use axum::response::Redirect;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use bcrypt::{hash, verify, DEFAULT_COST};
use time::Duration;

use crate::db::AppState;
use crate::handlers::get_session_user;
use crate::models::{LoginForm, RegisterForm, SessionUser};

#[derive(Template)]
#[template(path = "auth/login.html")]
pub struct LoginTemplate {
    pub current_user: Option<SessionUser>,
    pub error:        Option<String>,
}

#[derive(Template)]
#[template(path = "auth/register.html")]
pub struct RegisterTemplate {
    pub current_user: Option<SessionUser>,
    pub error:        Option<String>,
}

pub async fn login_page(jar: CookieJar) -> impl IntoResponse {
    if get_session_user(&jar).is_some() {
        return Redirect::to("/").into_response();
    }
    LoginTemplate { current_user: None, error: None }.into_response()
}

pub async fn register_page(jar: CookieJar) -> impl IntoResponse {
    if get_session_user(&jar).is_some() {
        return Redirect::to("/").into_response();
    }
    RegisterTemplate { current_user: None, error: None }.into_response()
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        r#"SELECT u.id, u.username, u.email, u.password_hash, r.name AS role_name
           FROM users u JOIN roles r ON u.role_id = r.id
           WHERE u.email = $1"#,
        form.email
    )
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    match result {
        Some(row) if verify(&form.password, &row.password_hash).unwrap_or(false) => {
            let session = SessionUser {
                id: row.id, username: row.username,
                email: row.email, role_name: row.role_name,
            };
            let cookie = build_session_cookie(&session);
            (jar.add(cookie), Redirect::to("/")).into_response()
        }
        _ => LoginTemplate {
            current_user: None,
            error: Some("Nieprawidłowy email lub hasło".into()),
        }
        .into_response(),
    }
}

pub async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<RegisterForm>,
) -> impl IntoResponse {
    // Walidacja
    if form.password != form.confirm_password {
        return RegisterTemplate {
            current_user: None,
            error: Some("Hasła nie są zgodne".into()),
        }
        .into_response();
    }
    if form.password.len() < 6 {
        return RegisterTemplate {
            current_user: None,
            error: Some("Hasło musi mieć co najmniej 6 znaków".into()),
        }
        .into_response();
    }

    let password_hash = match hash(&form.password, DEFAULT_COST) {
        Ok(h)  => h,
        Err(_) => return RegisterTemplate {
            current_user: None,
            error: Some("Błąd serwera".into()),
        }.into_response(),
    };

    let role = sqlx::query!("SELECT id FROM roles WHERE name = 'user'")
        .fetch_one(&state.pool).await;

    let role_id = match role {
        Ok(r)  => r.id,
        Err(_) => return RegisterTemplate {
            current_user: None,
            error: Some("Błąd serwera".into()),
        }.into_response(),
    };

    let result = sqlx::query!(
        "INSERT INTO users (username, email, password_hash, role_id)
         VALUES ($1, $2, $3, $4) RETURNING id",
        form.username, form.email, password_hash, role_id
    )
    .fetch_one(&state.pool).await;

    match result {
        Ok(row) => {
            let session = SessionUser {
                id: row.id, username: form.username,
                email: form.email, role_name: "user".into(),
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
            RegisterTemplate {
                current_user: None,
                error: Some(msg.into()),
            }
            .into_response()
        }
    }
}

pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    let cookie = Cookie::build(("session", ""))
        .path("/")
        .same_site(SameSite::Lax)
        .http_only(true)
        .max_age(Duration::ZERO)
        .build();
    (jar.remove(cookie), Redirect::to("/auth/login")).into_response()
}

fn build_session_cookie(user: &SessionUser) -> Cookie<'static> {
    let json = serde_json::to_string(user).unwrap();
    Cookie::build(("session", json))
        .path("/")
        .same_site(SameSite::Lax)
        .http_only(true)
        .build()
}