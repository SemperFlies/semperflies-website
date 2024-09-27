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
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        // let jwt_expires_in = std::env::var("JWT_EXPIRED_IN").expect("JWT_EXPIRED_IN must be set");
        // let jwt_maxage = std::env::var("JWT_MAXAGE").expect("JWT_MAXAGE must be set");
        Config {
            database_url,
            jwt_secret,
            // jwt_expires_in,
            // jwt_maxage: jwt_maxage.parse::<i32>().unwrap(),
        }
    }
}
