use std::{collections::HashMap, sync::LazyLock};
use stripe::{EventObject, EventType, Product, ProductId};
pub mod products;
pub mod shipping;

pub const STRIPE_CLIENT: LazyLock<stripe::Client> = LazyLock::new(|| {
    dotenv::dotenv().ok();
    let key = std::env::var("STRIPE_SECRET").expect("STRIPE_SECRET env var must not exist");
    stripe::Client::new(key)
});

pub fn shopping_cart_to_line_items(
    product_ids: HashMap<ProductId, usize>,
) -> Vec<stripe::CreateCheckoutSessionLineItems> {
    let mut items = vec![];
    let all_products = products::get_cached_products();

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

/// The server expects a .json file that contains all available products
/// In production, this is maintained by a cron job that runs the `save_products` binary
/// In development, this json file is written to manually by running the `save_products` binary
use axum::{
    async_trait,
    body::Body,
    extract::FromRequest,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use tracing::warn;

pub async fn handle_webhook(StripeEvent(event): StripeEvent) {
    match event.type_ {
        EventType::CheckoutSessionCompleted => {
            if let EventObject::CheckoutSession(stripe::CheckoutSession {
                id,
                amount_total,
                customer_details,
                line_items,
                shipping_details,
                customer_email,
                invoice,
                ..
            }) = event.data.object
            {
                warn!(
                    r#"Received checkout session completed webhook
                    id: {id:?}
                    invoice: {invoice:?}
                    amount_total: ${}
                    customer_details: {customer_details:?}
                    line_items: {line_items:?}
                    shipping_details: {shipping_details:?}
                    customer_email: {customer_email:?}
                    "#,
                    amount_total.unwrap() / 100
                );

                let c = STRIPE_CLIENT;
                let client = LazyLock::force(&c);
                match stripe::CheckoutSession::retrieve(
                    client,
                    &id,
                    &["line_items.data.price.product", "customer"],
                )
                .await
                {
                    Ok(session) => match checkout_session_email(session).await {
                        Ok(Email { subject, content }) => {
                            dotenv::dotenv().ok();
                            let from = &std::env::var("MAIL_FROM")
                                .unwrap_or("eklitsie@gmail.com".to_string());
                            let to = std::env::var("INVOICE_MAIL_TO")
                                .unwrap_or("voidkandy@gmail.com".to_string());
                            tracing::warn!("sending email to {to} from {from}");
                            let mail = sendgrid::Mail::new()
                                .add_from(from)
                                .add_text(&content)
                                .add_subject(&subject)
                                .add_to((to.as_str(), "Jamie").into());
                            let secret =
                                std::env::var("SENDGRID_SECRET").expect("No sendgrid secret");
                            match sendgrid::SGClient::new(secret).send(mail).await {
                                Ok(_) => warn!("SENT EMAIL"),
                                Err(e) => {
                                    warn!("FAILED TO SEND EMAIL: {e:#?}");
                                }
                            }
                        }
                        Err(e) => {
                            warn!("No mail returned from checkout session email: {e:#?}")
                        }
                    },
                    Err(e) => {
                        warn!("SESSION ERR: {e:?}");
                    }
                }
            }
        }
        EventType::AccountUpdated => {
            if let EventObject::Account(account) = event.data.object {
                warn!(
                    "Received account updated webhook for account: {:?}",
                    account.id
                );
            }
        }
        _ => warn!("Unknown event encountered in webhook: {:?}", event.type_),
    }
}

fn get_item_as_product<'p>(
    item: &stripe::CheckoutSessionItem,
    products: &'p products::CachedProducts,
) -> Option<&'p Product> {
    let id = match item.price.as_ref()?.product.as_ref()? {
        stripe::Expandable::Id(id) => id,
        stripe::Expandable::Object(prod) => &prod.id,
    };
    products.get(id)
}

#[derive(Debug)]
struct Email {
    subject: String,
    content: String,
}
/// This function needs to be passed the `CheckoutSession` that is received in order to populate line items
async fn checkout_session_email(session: stripe::CheckoutSession) -> anyhow::Result<Email> {
    dotenv::dotenv().ok();
    warn!(
        r#"
            ITEMS: {:#?}

            CUSTOMER: {:#?}
        "#,
        session.line_items, session.customer
    );
    let products = products::get_cached_products();
    let mut product_info_str = String::new();
    let donation_id = std::env::var("DONATION").expect("No DONATION env variable");

    if session.line_items.data.len() == 1
        && session.line_items.data[0].price.as_ref().is_some_and(|p| {
            if let Some(stripe::Expandable::Object(obj)) = &p.product {
                obj.id.as_str() == donation_id.as_str()
            } else {
                false
            }
        })
    {
        let content = {
            format!(
                "You got a donation of ${}!",
                session
                    .amount_total
                    .and_then(|cents| Some(format!("{}", cents / 100)))
                    .unwrap_or("NO AMT DATA".to_string()),
            )
        };
        return Ok(Email {
            subject: format!(
                "NEW DONATION {:?}",
                chrono::DateTime::from_timestamp(session.created, 0)
                    .map(|t| t.to_string())
                    .unwrap_or("NO TIMESTAMP".to_string())
            ),
            content,
        });
    }

    for item in session.line_items.data.iter() {
        match get_item_as_product(item, &products) {
            Some(product) => {
                product_info_str.push_str(&format!(
                    r#"
Id: {}
Name: {}
---
Quantity: {}
_____
"#,
                    product.id.to_string(),
                    product.name.as_ref().unwrap_or(&"NO NAME".to_string()),
                    item.quantity
                        .map(|s| format!("{s}"))
                        .unwrap_or("NO QUANTITY".to_string())
                ));
            }
            None => {
                product_info_str.push_str(&format!("NO INFO FOR PRODUCT ID: {}\n", item.id));
            }
        }
    }

    let (address_msg, contact_msg) = {
        match session.customer {
            Some(stripe::Expandable::Id(_)) => {
                tracing::error!("DID NOT EXPAND CUSTOMERS");
                None
            }
            Some(stripe::Expandable::Object(customer)) => {
                let addy = customer
                    .address
                    .and_then(|a| {
                        Some(format!(
                            r#"
    Line1: {}
    Line2: {}
    City: {}
    State: {}
    Zip: {}
                    "#,
                            a.line1.unwrap_or(String::new()),
                            a.line2.unwrap_or(String::new()),
                            a.city.unwrap_or(String::new()),
                            a.state.unwrap_or(String::new()),
                            a.postal_code.unwrap_or(String::new()),
                        ))
                    })
                    .unwrap_or("NO ADDRESS INFORMATION".to_string());

                let contact = format!(
                    r#"
    Name: {}
    Email: {}
                    "#,
                    customer.email.unwrap_or("NO EMAIL".to_string()),
                    customer.name.unwrap_or("NO NAME".to_string())
                );
                Some((addy, contact))
            }
            None => None,
        }
    }
    .unwrap_or((
        "NO ADDRESS INFORMATION".to_string(),
        "NO CONTACT INFORMATION".to_string(),
    ));

    let content = format!(
        r#"
You Got an Donor Order!
Order Total: ${}

PRODUCT INFO: 
{product_info_str}

CUSTOMER INFO:
CONTACT:
{contact_msg}
ADDRESS:
{address_msg}
        "#,
        session
            .amount_total
            .map(|i| (i / 100).to_string())
            .unwrap_or("NO TOTAL".to_string())
    );

    let subject = format!(
        "NEW ORDER {:?}",
        chrono::DateTime::from_timestamp(session.created, 0)
            .map(|t| t.to_string())
            .unwrap_or("NO TIMESTAMP".to_string())
    );
    let email = Email { content, subject };
    tracing::warn!("created email: {email:#?}");

    Ok(email)
}

pub struct StripeEvent(stripe::Event);

#[async_trait]
impl<S> FromRequest<S> for StripeEvent
where
    String: FromRequest<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        dotenv::dotenv().ok();
        let stripe_wh_secret = std::env::var("STRIPE_WH_SECRET").expect("no webhook secret?");
        let signature = if let Some(sig) = req.headers().get("stripe-signature") {
            sig.to_owned()
        } else {
            return Err(StatusCode::BAD_REQUEST.into_response());
        };

        let payload = String::from_request(req, state)
            .await
            .map_err(IntoResponse::into_response)?;

        let event = stripe::Webhook::construct_event(
            &payload,
            signature.to_str().unwrap(),
            &stripe_wh_secret,
        )
        .map_err(|e| {
            tracing::error!("problem with event: {e:#?}");
            StatusCode::BAD_REQUEST.into_response()
        })?;
        Ok(Self(event))
    }
}

mod tests {
    use sendgrid::SendgridError;
    #[tokio::test]
    async fn mail_test() {
        // let my_secret_key = std::env::var("SENDGRID_KEY").expect("need SENDGRID_KEY to test");
        use sendgrid::{Mail, SGClient};

        let mail = Mail::new()
            .add_from("eklitsie@gmail.com")
            .add_text("hi!")
            .add_subject("Hello")
            .add_to(("voidkandy@gmail.com", "Your Name").into());
        let response = SGClient::new("key!").send(mail).await.expect("failed");
    }
}
