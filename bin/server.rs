use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use core::panic;
use reqwest::StatusCode;
use semperflies::state::AppState;
use semperflies::telemetry::{get_subscriber, init_subscriber};
use semperflies::{
    cert::{get_cert_config, redirect_http_to_https, Ports},
    TRACING,
};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::{
    net::SocketAddr,
    sync::{Arc, LazyLock},
};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::warn;

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

    let app_config = semperflies::state::Config::init();
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
        format!("{}:{}", semperflies::LOCALHOST, 3000)
    });

    if allowed_origin != format!("{}:{}", semperflies::LOCALHOST, 3000) {
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

    let app = semperflies::routes::create_router(Arc::new(RwLock::new(AppState {
        db: pool.clone(),
        admin_session_id: None,
        env: app_config.clone(),
    })))
    .layer(cors);

    match env_.as_str() {
        semperflies::DEV_ENV => {
            let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", 3000))
                .await
                .unwrap();

            tracing::debug!("listening on {listener:#?}");
            axum::serve(listener, app).await.unwrap();
        }
        semperflies::PROD_ENV => {
            let addr = SocketAddr::from(([0, 0, 0, 0], ports.https));
            tracing::debug!("listening on {}", addr);
            axum_server::bind_rustls(addr, cert_config)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
        _ => panic!("unexpected env: {env_}"),
    }
    // }
}
