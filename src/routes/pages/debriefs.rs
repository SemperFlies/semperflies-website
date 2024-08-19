use askama::Template;
use axum::response::Html;

#[derive(Template, Debug)]
#[template(path = "pages/debriefs.html")]
pub struct DebriefsTemplate;

pub async fn debriefs() -> Html<String> {
    match DebriefsTemplate.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}
