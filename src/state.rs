use dotenv::dotenv;
use std::sync::Arc;
use tokio::sync::RwLock;

use sqlx::{Pool, Postgres};

pub type SharedState = Arc<RwLock<AppState>>;
#[derive(Debug, Clone)]
pub struct AppState {
    pub db: Pool<Postgres>,
    // shuld be a salted stirgn
    pub admin_session_id: Option<uuid::Uuid>,
    pub env: Config,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    // pub jwt_expires_in: String,
    // pub jwt_maxage: i32,
}

impl Config {
    pub fn init() -> Config {
        let database_url = std::env::var("DB_URL").expect("DATABASE_URL must be set");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        Config {
            database_url,
            jwt_secret,
        }
    }
}
