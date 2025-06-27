use crate::stripe::products::*;
use askama::Template;
use axum::response::Html;
use std::collections::HashMap;

#[derive(Template, Debug)]
#[template(path = "pages/patrol_gear.html")]
pub struct PatrolGearTemplate {
    products: HashMap<String, Vec<Product>>,
}

pub async fn patrol_gear() -> Html<String> {
    let products = {
        let prods = get_categorized_products();
        let mut map = HashMap::new();
        for (c, p) in prods {
            map.insert(c.as_ref().to_string(), p);
        }
        map
    };
    let template = PatrolGearTemplate { products };
    // warn!("got gear template: {:?}", template);
    match template.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}
