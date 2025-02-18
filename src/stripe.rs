use std::{borrow::BorrowMut, collections::HashMap, sync::LazyLock};

use stripe::{Product, ProductId};

pub type CachedProducts = HashMap<ProductId, Product>;
pub const STRIPE_CLIENT: LazyLock<stripe::Client> = LazyLock::new(|| {
    dotenv::dotenv().ok();
    let key = std::env::var("STRIPE_SECRET").expect("STRIPE_SECRET env var must not exist");
    stripe::Client::new(key)
});

pub fn shopping_cart_to_line_items(
    product_ids: HashMap<ProductId, usize>,
) -> Vec<stripe::CreateCheckoutSessionLineItems> {
    let mut items = vec![];
    let all_products = get_products();

    for (id, quantity) in product_ids {
        let product = all_products
            .get(&id)
            .expect("got a product ID for a product that does not exist");

        let price = match product
            .default_price
            .as_ref()
            .expect("product did not have price")
        {
            stripe::Expandable::Id(id) => id.to_string(),
            stripe::Expandable::Object(obj) => obj.id.to_string(),
        };
        let item = stripe::CreateCheckoutSessionLineItems {
            quantity: Some(quantity as u64),
            price: Some(price),
            ..Default::default()
        };
        items.push(item);
    }
    items
}

pub fn products_path() -> std::path::PathBuf {
    match std::env::var("ENVIRONMENT")
        .expect("No ENVIRONMENT env variable")
        .to_lowercase()
        .as_str()
    {
        "prod" => {
            let home = std::env::var("HOME").expect("No HOME variable?");
            let pathstr = format!("{home}/semperflies_products.json");
            std::path::Path::new(&pathstr).to_owned()
        }
        _other => std::path::Path::new("./semperflies_products.json").to_owned(),
    }
}

/// The server expects a .json file that contains all available products
/// In production, this is maintained by a cron job that runs the `save_products` binary
/// In development, this json file is written to manually by running the `save_products` binary
pub fn get_products() -> CachedProducts {
    let str = std::fs::read_to_string(products_path()).expect("could not read path to string");
    let products: CachedProducts = serde_json::from_str(&str).expect("could not coerce to json");
    products
}
