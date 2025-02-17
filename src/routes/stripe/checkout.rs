use std::sync::LazyLock;

use axum::{http::Response, response::IntoResponse, Json};

use crate::{error::DataApiReturn, stripe::STRIPE_CLIENT};

pub async fn create_checkout(
    Json(items_payload): Json<Vec<stripe::CreateCheckoutSessionLineItems>>,
    // Json(items_payload): Json<Vec<stripe::Product>>,
    // Should accept a hashmap of productIds and their quantities
    // The server should have a static instance of all available products
    // Json(items_payload): Json<HashMap<ProductId, usize>>,
) -> anyhow::Result<impl IntoResponse, DataApiReturn> {
    let c = STRIPE_CLIENT;
    let client = LazyLock::force(&c);
    let customer = stripe::Customer::create(
        &client,
        stripe::CreateCustomer {
            name: Some("Alexander Lyon"),
            email: Some("test@async-stripe.com"),
            description: Some(
                "A fake customer that is used to illustrate the examples in async-stripe.",
            ),
            metadata: Some(std::collections::HashMap::from([(
                String::from("async-stripe"),
                String::from("true"),
            )])),

            ..Default::default()
        },
    )
    .await
    .unwrap();

    println!(
        "created a customer at https://dashboard.stripe.com/test/customers/{}",
        customer.id
    );

    // create a new example project
    let product = {
        let mut create_product = stripe::CreateProduct::new("T-Shirt");
        create_product.metadata = Some(std::collections::HashMap::from([(
            String::from("async-stripe"),
            String::from("true"),
        )]));
        stripe::Product::create(&client, create_product)
            .await
            .unwrap()
    };

    // and add a price for it in USD
    let price = {
        let mut create_price = stripe::CreatePrice::new(stripe::Currency::USD);
        create_price.product = Some(stripe::IdOrCreate::Id(&product.id));
        create_price.metadata = Some(std::collections::HashMap::from([(
            String::from("async-stripe"),
            String::from("true"),
        )]));
        create_price.unit_amount = Some(1000);
        create_price.expand = &["product"];
        stripe::Price::create(&client, create_price).await.unwrap()
    };

    println!(
        "created a product {:?} at price {} {}",
        product.name.unwrap(),
        price.unit_amount.unwrap() / 100,
        price.currency.unwrap()
    );

    // finally, create a checkout session for this product / price
    let checkout_session = {
        let mut params = stripe::CreateCheckoutSession::new();
        params.cancel_url = Some("http://test.com/cancel");
        params.customer = Some(customer.id);
        params.mode = Some(stripe::CheckoutSessionMode::Payment);
        params.line_items = Some(items_payload);
        params.expand = &["line_items", "line_items.data.price.product"];

        stripe::CheckoutSession::create(&client, params)
            .await
            .unwrap()
    };

    let line_items = checkout_session.line_items;

    println!(
        "created a {} checkout session for {} {:?} for {} {} at {}",
        checkout_session.payment_status,
        line_items.data[0].quantity.unwrap(),
        match line_items.data[0]
            .price
            .as_ref()
            .unwrap()
            .product
            .as_ref()
            .unwrap()
        {
            stripe::Expandable::Object(p) => p.name.as_ref().unwrap(),
            _ => panic!("product not found"),
        },
        checkout_session.amount_subtotal.unwrap() / 100,
        line_items.data[0].price.as_ref().unwrap().currency.unwrap(),
        checkout_session.url.unwrap()
    );
    let response = Response::new(serde_json::json!({"status": "success"} ).to_string());
    Ok(response)
}
