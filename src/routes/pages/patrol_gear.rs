use crate::stripe::{get_products, CachedProducts, STRIPE_CLIENT};
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
    let mut products: Vec<stripe::Product> = get_products()
        .into_iter()
        .filter_map(|(_id, p)| {
            if p.name.as_ref().is_some_and(|s| s.as_str() != "Donation") {
                Some(p)
            } else {
                None
            }
        })
        .collect();
    products.sort_by(|a, b| {
        a.created
            .unwrap_or(i64::MAX)
            .cmp(&b.created.unwrap_or(i64::MAX))
    });

    let template = PatrolGearTemplate {
        products, // gear: crate::database::builtins::builtin_gear(),
    };
    warn!("got gear template: {:?}", template);
    match template.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}
