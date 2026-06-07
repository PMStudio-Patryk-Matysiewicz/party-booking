use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role_id: Uuid,
    pub created_at: DateTime<Utc>,
}

// Formularz logowania
// Deserialize - odczytywanie z POST
#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

// Formularz rejestracji
#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    pub username: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
}

// Cookies i sesje
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role_name: String,
}

impl SessionUser {
    pub fn is_admin(&self) -> bool {
        self.role_name == "admin"
    }
}
