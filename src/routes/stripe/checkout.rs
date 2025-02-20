use crate::{
    error::{DataApiReturn, DataResponse},
    stripe::{shopping_cart_to_line_items, STRIPE_CLIENT},
};
use axum::{
    extract::{Query, Request},
    http::Response,
    response::IntoResponse,
    Json,
};
use std::{collections::HashMap, sync::LazyLock};
use tracing::warn;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Address {
    line1: String,
    line2: String,
    #[serde(rename = "zipCode")]
    zip: String,
    state: String,
    city: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CustomerInfo {
    name: String,
    email: String,
    #[serde(flatten)]
    address: Address,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CheckoutPayload {
    customer: CustomerInfo,
    items: HashMap<stripe::ProductId, usize>,
}

pub async fn create_checkout(
    Json(payload): Json<CheckoutPayload>,
) -> anyhow::Result<impl IntoResponse, DataApiReturn> {
    warn!("got payload: {payload:#?}");

    let c = STRIPE_CLIENT;
    let client = LazyLock::force(&c);
    let items = shopping_cart_to_line_items(payload.items);

    let customer = match stripe::Customer::create(
        &client,
        stripe::CreateCustomer {
            name: Some(&payload.customer.name),
            email: Some(&payload.customer.email),
            address: Some(stripe::Address {
                country: Some(String::from("US")),
                city: Some(payload.customer.address.city),
                line1: Some(payload.customer.address.line1),
                line2: Some(payload.customer.address.line2),
                postal_code: Some(payload.customer.address.zip),
                state: Some(payload.customer.address.state),
            }),
            ..Default::default()
        },
    )
    .await
    {
        Ok(c) => c,
        Err(e) => {
            return Err(DataResponse::error(
                format!("failed to create customer: {e:?}"),
                None,
            ));
        }
    };

    warn!("created a customer with id: {}", customer.id);

    let checkout_session = match {
        let mut params = stripe::CreateCheckoutSession::new();
        params.cancel_url = Some("http://localhost:3000/shopping_cart");
        params.customer = Some(customer.id);
        params.mode = Some(stripe::CheckoutSessionMode::Payment);
        params.line_items = Some(items);
        params.expand = &["line_items", "line_items.data.price.product"];

        params.success_url = Some("http://localhost:3000/patrol_gear");

        stripe::CheckoutSession::create(&client, params).await
    } {
        Ok(session) => session,
        Err(e) => {
            return Err(DataResponse::error(
                format!("failed to create checkout session: {e:?}"),
                None,
            ));
        }
    };

    let line_items = checkout_session.line_items;

    if checkout_session.url.is_none() {
        return Err(DataResponse::error(
            "Checkout session created, but no url was provided",
            None,
        ));
    }

    warn!(
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
        checkout_session.url.as_ref().unwrap()
    );

    let response = Response::new(
        serde_json::json!({"status": "success", "url": checkout_session.url.unwrap()} ).to_string(),
    );
    Ok(response)
}
