use futures_util::StreamExt;
use semperflies::stripe::{products_path, CachedProducts, STRIPE_CLIENT};
use std::{collections::HashMap, sync::LazyLock};
use stripe::ListProducts;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let c = &STRIPE_CLIENT;

    let client = LazyLock::force(c);
    let params = ListProducts {
        active: Some(true),
        ..Default::default()
    };

    let mut all_products: CachedProducts = HashMap::new();
    let paginator = stripe::Product::list(client, &params)
        .await
        .expect("could not get products")
        .paginate(params);

    let mut stream = paginator.stream(&client);

    while let Some(Ok(mut product)) = stream.next().await {
        if let Some(stripe::Expandable::Id(id)) = product.default_price.as_ref() {
            tracing::warn!("getting price of {product:#?}");
            let obj = stripe::Price::retrieve(&client, &id, &[])
                .await
                .expect("failed to retrieve price");
            product.default_price = Some(stripe::Expandable::Object(Box::new(obj)));
        }

        all_products.insert(product.id.to_owned(), product);
    }

    let path = products_path();

    println!("saving products: {all_products:#?} to {path:#?}");

    let contents = serde_json::to_string(&all_products).expect("failed to serialize products");
    match std::fs::write(path, contents) {
        Ok(_) => println!("successfully wrote products to path"),
        Err(e) => panic!("failed to write to products path: {e:#?}"),
    }
}
