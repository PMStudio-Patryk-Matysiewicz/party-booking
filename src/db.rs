use sqlx::PgPool;

#[derive(Clone)] // klonowanie stanu aplikacji ze względu na wielowątkowość (async)
pub struct AppState {
    pub pool: PgPool,
}
