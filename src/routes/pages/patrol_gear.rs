use crate::stripe::get_products;
use askama::Template;
use axum::response::Html;
use std::collections::HashMap;
use tracing::warn;

#[derive(Template, Debug)]
#[template(path = "pages/patrol_gear.html")]
pub struct PatrolGearTemplate {
    products: CategorizedProducts,
}

type CategorizedProducts = HashMap<String, Vec<stripe::Product>>;
const CATEGORY_METADATA_KEY: &str = "category";
const UNCATEGORIZED_KEY: &str = "misc";

pub async fn patrol_gear() -> Html<String> {
    let mut categorized: CategorizedProducts = HashMap::new();
    categorized.insert(UNCATEGORIZED_KEY.to_string(), vec![]);

    for (id, prod) in get_products().into_iter() {
        if prod.name.as_ref().is_some_and(|s| s.as_str() != "Donation") {
            let category = prod
                .metadata
                .as_ref()
                .and_then(|map| {
                    map.get(CATEGORY_METADATA_KEY)
                        .and_then(|k| Some(k.as_str()))
                })
                .unwrap_or(UNCATEGORIZED_KEY);

            match categorized.get_mut(category) {
                Some(vec) => vec.push(prod),
                None => {
                    let _ = categorized.insert(category.to_string(), vec![prod]);
                }
            }
        }
    }

    categorized.iter_mut().for_each(|(_, vec)| {
        vec.sort_by(|a, b| {
            a.created
                .unwrap_or(i64::MAX)
                .cmp(&b.created.unwrap_or(i64::MAX))
        });
    });

    let template = PatrolGearTemplate {
        products: categorized,
    };
    warn!("got gear template: {:?}", template);
    match template.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}
