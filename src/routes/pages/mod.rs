pub mod admin;
pub mod videos;
use askama::Template;
use axum::response::Html;

#[derive(Template, Debug)]
#[template(path = "pages/landing.html")]
pub struct LandingTemplate;

pub async fn landing() -> Html<String> {
    match LandingTemplate.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}

#[derive(Template, Debug)]
#[template(path = "pages/about_us.html")]
pub struct AboutUsTemplate;

pub async fn about_us() -> Html<String> {
    match AboutUsTemplate.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}
