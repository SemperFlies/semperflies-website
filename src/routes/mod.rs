mod index;
mod middlware;
mod pages;
use crate::{
    auth::{
        handlers::{login_admin_handler, logout_handler},
        middleware::{admin_auth, soft_auth},
    },
    state::SharedState,
    AppState,
};
use axum::{
    http::StatusCode,
    middleware::{self},
    routing::{get, post},
    Router,
};
use middlware::htmx_request_check;
use std::sync::Arc;
use tower_http::services::ServeDir;

pub type HandlerResult<T> = Result<T, StatusCode>;

#[tracing::instrument(name = "create app router", skip_all)]
pub fn create_router(state: SharedState) -> Router {
    let admin_routes = Router::new()
        .route(
            "/upload",
            get(pages::admin::upload), // get(admin::upload::get_upload_form).post(admin::upload::post_upload_form),
        )
        .route_layer(middleware::from_fn_with_state(state.clone(), admin_auth))
        .route(
            "/status",
            get(pages::admin::login_logout)
                .route_layer(middleware::from_fn_with_state(state.clone(), soft_auth)),
        );

    let data_routes = Router::new()
        .route("/auth/login", post(login_admin_handler))
        .route("/auth/logout", post(logout_handler));

    // let private_dir_router = Router::new()
    //     .nest_service("/", ServeDir::new("private"))
    //     .route_layer(middleware::from_fn_with_state(state.clone(), auth));

    Router::new()
        .route("/", get(index::index))
        .route("/landing", get(pages::landing))
        // .route("/email", get(contact::send_email))
        //
        // .nest("/blog", blog_routes)
        // .nest("/admin", admin_routes)
        // .route("/contact", get(contact::index))
        // .route_layer(middleware::from_fn_with_state(state.clone(), soft_auth))
        .nest("/admin", admin_routes)
        .layer(middleware::from_fn(htmx_request_check))
        .nest("/data", data_routes)
        // .nest("/private", private_dir_router)
        .fallback(index::custom_404)
        .with_state(state)
        .nest_service("/public", ServeDir::new("public"))
}
