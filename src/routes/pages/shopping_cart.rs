use askama::Template;
use axum::response::Html;

#[derive(Template, Debug)]
#[template(path = "pages/shopping_cart.html")]
pub struct ShoppingCartTemplate {}

pub async fn shopping_cart() -> Html<String> {
    let template = ShoppingCartTemplate {};
    match template.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}
