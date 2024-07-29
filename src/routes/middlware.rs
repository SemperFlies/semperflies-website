use super::index::IndexTemplate;
use askama::Template;
use axum::{
    extract::Request,
    http::HeaderMap,
    middleware::Next,
    response::{Html, IntoResponse, Response},
};

pub(super) async fn htmx_request_check(headers: HeaderMap, req: Request, next: Next) -> Response {
    let uri = req.uri();

    if headers.get("Hx-Request").is_none() {
        let template = IndexTemplate::from(uri);
        tracing::info!(
            "HxRequest header not present, middleware returning HTML template: {:?}",
            template
        );
        return Html(template.render().unwrap()).into_response();
    }

    tracing::info!("HxRequest header present, passing through middleware...");
    next.run(req).await.into()
}
