mod index;
mod middlware;
pub mod pages;
use crate::{
    auth::{
        handlers::{
            delete::delete_item_handler,
            login_admin_handler, logout_handler,
            upload::{upload_form_handler, upload_multipart_handler},
        },
        middleware::soft_auth,
    },
    state::SharedState,
};
use axum::{
    extract::DefaultBodyLimit,
    http::StatusCode,
    middleware::{self},
    routing::{delete, get, post},
    Router,
};
use middlware::htmx_request_check;
use tower_http::services::ServeDir;

pub type HandlerResult<T> = Result<T, StatusCode>;

#[tracing::instrument(name = "create app router", skip_all)]
pub fn create_router(state: SharedState) -> Router {
    let admin_routes = Router::new()
        .route("/upload", get(pages::admin::upload))
        // .route_layer(middleware::from_fn_with_state(state.clone(), admin_auth))
        .route("/status", get(pages::admin::login_logout))
        .layer(middleware::from_fn_with_state(state.clone(), soft_auth));

    let data_routes = Router::new()
        .route("/auth/login", post(login_admin_handler))
        .route("/auth/logout", post(logout_handler))
        .route("/auth/upload_form/:item", post(upload_form_handler))
        .route("/auth/delete/:item/:id", delete(delete_item_handler))
        .route(
            "/auth/upload_multipart/:item",
            post(upload_multipart_handler),
        )
        .layer(DefaultBodyLimit::disable());

    Router::new()
        .route("/", get(index::index))
        .route("/landing", get(pages::landing::landing))
        .route("/about_us", get(pages::about_us::about_us))
        .route("/support", get(pages::support::support))
        .layer(middleware::from_fn_with_state(state.clone(), soft_auth))
        .route("/patrol_gear", get(pages::patrol_gear::patrol_gear))
        .route("/patrol_log", get(pages::patrol_log::logs::patrol_log))
        .layer(middleware::from_fn_with_state(state.clone(), soft_auth))
        .route("/dedications", get(pages::dedications::dedications))
        .layer(middleware::from_fn_with_state(state.clone(), soft_auth))
        .route("/debriefs", get(pages::debriefs::debriefs))
        .layer(middleware::from_fn_with_state(state.clone(), soft_auth))
        .route("/videos", get(pages::patrol_log::videos::videos))
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
