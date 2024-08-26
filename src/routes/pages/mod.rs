pub mod about_us;
pub mod admin;
pub mod debriefs;
pub mod dedications;
pub mod patrol_gear;
pub mod patrol_log;
pub mod support;
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
