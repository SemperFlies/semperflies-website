use std::sync::LazyLock;
use telemetry::{get_subscriber, init_subscriber};
pub mod auth;
pub mod cert;
pub mod components;
pub mod database;
pub mod error;
pub mod routes;
pub mod state;
pub mod stripe;
pub mod telemetry;
pub mod util;

pub static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "main".to_string();

    if std::env::var("MAIN_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub const LOCALHOST: &str = "http://localhost";
pub const DEV_ENV: &str = "DEV";
pub const PROD_ENV: &str = "PROD";
