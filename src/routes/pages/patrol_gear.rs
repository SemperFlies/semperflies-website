use crate::stripe::{get_products, STRIPE_CLIENT};
use askama::Template;
use axum::extract::Path;
use axum::response::Html;
use std::sync::LazyLock;
use stripe::{Expandable, List, ListProducts, Price};
use tracing::warn;

#[derive(Template, Debug)]
#[template(path = "pages/patrol_gear.html")]
pub struct PatrolGearTemplate {
    products: Vec<stripe::Product>,
    // gear: HashMap<String, Vec<Gear>>,
}

pub async fn patrol_gear() -> Html<String> {
    let products: Vec<stripe::Product> = get_products()
        .into_iter()
        .filter(|p| p.name.as_ref().and_then(|s| Some(s.as_str())) != Some("Donation"))
        .collect();

    let template = PatrolGearTemplate {
        products, // gear: crate::database::builtins::builtin_gear(),
    };
    warn!("got gear template: {:?}", template);
    match template.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}
