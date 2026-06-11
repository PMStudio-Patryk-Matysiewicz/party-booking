use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Event {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub date: DateTime<Utc>,
    pub location_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct EventWithLocation {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub date: DateTime<Utc>,
    pub location_id: Uuid,
    pub location_name: String,
    pub location_address: String,
    pub created_at: DateTime<Utc>,
}
