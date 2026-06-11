use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Reservation {
    pub id: Uuid,
    pub user_id: Uuid,
    pub event_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ReservationWithDetails {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub event_id: Uuid,
    pub event_title: String,
    pub event_date: DateTime<Utc>,
    pub location_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateReservationForm {
    pub event_id: Uuid,
}
