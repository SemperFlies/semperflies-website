use crate::{
    error::{DataApiReturn, DataResponse},
    stripe::{shipping::get_shipping_info, shopping_cart_to_line_items, STRIPE_CLIENT},
    LOCALHOST,
};
use axum::{http::Response, response::IntoResponse, Json};
use std::{collections::HashMap, sync::LazyLock};
use stripe::{CreateCheckoutSessionDiscounts, CreateCheckoutSessionShippingOptions};
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
    // let mut shipping_items = get_shipping_info(&payload.items)
    //     .iter()
    //     .flat_map(|i| i.to_shipping_line_items())
    //     .collect::<Vec<stripe::CreateCheckoutSessionLineItems>>();
    let items = shopping_cart_to_line_items(payload.items);
    // items.append(&mut shipping_items);

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
            let msg = format!("failed to create customer: {e:?}");
            tracing::error!(msg);
            return Err(DataResponse::error(msg, None));
        }
    };

    warn!("created a customer with id: {}", customer.id);

    let checkout_session = match {
        let origin = std::env::var("ALLOWED_ORIGIN").unwrap_or_else(|_| {
            warn!("No allowed origin env var, falling back to localhost");
            format!("{}:{}", LOCALHOST, 3000)
        });
        let mut params = stripe::CreateCheckoutSession::new();
        let cancel_url = format!("{origin}/shopping_cart");
        params.cancel_url = Some(&cancel_url);

        params.customer = Some(customer.id);
        params.allow_promotion_codes = Some(true);
        params.mode = Some(stripe::CheckoutSessionMode::Payment);
        params.line_items = Some(items);
        params.expand = &["line_items", "line_items.data.price.product"];
        let success = format!("{origin}/patrol_gear");
        params.success_url = Some(&success);

        stripe::CheckoutSession::create(&client, params).await
    } {
        Ok(session) => session,
        Err(e) => {
            let msg = format!("failed to create checkout session: {e:?}");
            tracing::error!(msg);
            return Err(DataResponse::error(msg, None));
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
            _ => return Err(DataResponse::error("Product Object was not expanded", None,)),
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
