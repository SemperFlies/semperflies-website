use askama::Template;
use axum::body::Body;
use axum::extract::Request;
use axum::http::{StatusCode, Uri};
use axum::response::{Html, IntoResponse};

#[derive(Template, Debug)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub path: &'a str,
    pub params: String,
}

impl<'a> Default for IndexTemplate<'a> {
    fn default() -> Self {
        Self {
            path: "/landing",
            params: String::new(),
        }
    }
}

impl<'a> From<&'a Uri> for IndexTemplate<'a> {
    fn from(uri: &'a Uri) -> Self {
        let mut path = uri.path();
        if path == "/" {
            path = "/landing"
        }
        let params = match uri.query() {
            Some(p) => format!("?{}", p),
            None => String::new(),
        };

        Self { path, params }
    }
}

pub async fn index(req: Request<Body>) -> Html<String> {
    let tmpl = IndexTemplate::from(req.uri());
    match tmpl.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}

#[derive(Template)]
#[template(path = "404.html")]
pub struct FourOFourTemplate;

pub async fn custom_404() -> impl IntoResponse {
    let html = match FourOFourTemplate.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    };
    (StatusCode::NOT_FOUND, html)
}
