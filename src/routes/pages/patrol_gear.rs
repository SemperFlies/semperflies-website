use askama::Template;
use axum::response::Html;

#[derive(Template, Debug)]
#[template(path = "pages/patrol_gear.html")]
pub struct PatrolGearTemplate;

pub async fn patrol_gear() -> Html<String> {
    match PatrolGearTemplate.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}
