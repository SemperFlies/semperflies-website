use askama::Template;
use axum::response::Html;

#[derive(Template, Debug)]
#[template(path = "pages/fallen_brothers.html")]
pub struct DedicationsTemplate;

pub async fn dedications() -> Html<String> {
    match DedicationsTemplate.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}
