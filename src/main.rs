mod auth;
mod cert;
mod components;
mod database;
mod error;
mod routes;
mod state;
mod telemetry;
mod util;
use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use cert::{get_cert_config, redirect_http_to_https, Ports};
use core::panic;
use reqwest::StatusCode;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
pub(crate) use state::AppState;
use std::{
    net::SocketAddr,
    sync::{Arc, LazyLock},
};
use telemetry::{get_subscriber, init_subscriber};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::warn;

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

const LOCALHOST: &str = "http://localhost";
const DEV_ENV: &str = "DEV";
const PROD_ENV: &str = "PROD";

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    LazyLock::force(&TRACING);
    tracing::info!("Tracing Initialized");
    // let port = std::env::var("PORT").expect("Failed to get port env variable");
    let env_ = std::env::var("ENVIRONMENT").unwrap();

    let ports = Ports {
        http: 7878,
        https: 443,
    };

    let cert_config = get_cert_config().await;

    tokio::spawn(redirect_http_to_https(ports));

    let app_config = state::Config::init();
    tracing::info!(
        "attempting to connect to database: {:?}",
        &app_config.database_url
    );

    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&app_config.database_url)
        .await
    {
        Ok(pool) => {
            tracing::info!("✅Connection to the database is successful!");
            pool
        }
        Err(err) => {
            tracing::error!("🔥 Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    let allowed_origin = std::env::var("ALLOWED_ORIGIN").unwrap_or_else(|_| {
        warn!("No allowed origin env var, falling back to localhost");
        format!("{}:{}", LOCALHOST, 3000)
    });

    if allowed_origin != format!("{}:{}", LOCALHOST, 3000) {
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("failed to migrate database");
    }

    let cors = CorsLayer::new()
        .allow_origin(allowed_origin.parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = routes::create_router(Arc::new(RwLock::new(AppState {
        db: pool.clone(),
        admin_session_id: None,
        env: app_config.clone(),
    })))
    .layer(cors);

    let addr = match env_.as_str() {
        DEV_ENV => SocketAddr::from(([127, 0, 0, 1], ports.http)),
        PROD_ENV => SocketAddr::from(([0, 0, 0, 0], ports.https)),
        _ => panic!("unexpected env: {env_}"),
    };

    tracing::debug!("listening on {}", addr);
    axum_server::bind_rustls(addr, cert_config)
        .serve(app.into_make_service())
        .await
        .unwrap();
    // }
    // let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", ports.https))
    //     .await
    //     .unwrap();
    //
    //
    //
    // axum::serve(listener, app).await.unwrap();
}
