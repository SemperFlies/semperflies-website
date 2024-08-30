use anyhow::anyhow;
use askama::Template;
use axum::{extract::State, response::Html, Extension};
use rand::prelude::*;
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{
    auth::middleware::SoftAuthExtension,
    database::{
        handles::DbData,
        models::{DBAddress, DBResource},
    },
    state::SharedState,
};

#[derive(Template, Debug)]
#[template(path = "pages/support.html")]
pub struct SupportTemplate {
    resources: Vec<SupportResource>,
    admin: bool,
}

pub const SUPPORT: &str = "support";

#[derive(Debug)]
pub struct Address {
    pub line_2: Option<String>,
    pub line_1: String,
    pub city: String,
    pub state: String,
    pub zip: String,
}

#[derive(Debug)]
pub struct SupportResource {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub missions: Vec<String>,
    pub phone: Option<String>,
    pub website_url: Option<String>,
    pub email: Option<String>,
    pub physical_address: Option<Address>,
}

impl From<DBAddress> for Address {
    fn from(value: DBAddress) -> Self {
        Self {
            line_2: value.line_2,
            line_1: value.line_1,
            city: value.city,
            state: value.state,
            zip: value.zip,
        }
    }
}

impl From<(DBResource, Option<DBAddress>)> for SupportResource {
    fn from((res, add): (DBResource, Option<DBAddress>)) -> Self {
        Self {
            id: res.id,
            name: res.name,
            description: res.description,
            missions: res.missions,
            phone: res.phone,
            email: res.email,
            website_url: res.website_url,
            physical_address: add.and_then(|a| Some(Address::from(a))),
        }
    }
}

async fn get_resources(pool: &Pool<Postgres>) -> anyhow::Result<Vec<SupportResource>> {
    let res = DBResource::get_multiple(pool).await?;
    let mut all = vec![];

    for r in res {
        let mut address = Option::<DBAddress>::None;
        if let Some(id) = r.address_id {
            let add = DBAddress::get_single_by(pool, id)
                .await?
                .ok_or(anyhow!("no address with id: {:?}", id))?;
            address = Some(add);
        }
        all.push(SupportResource::from((r, address)))
    }

    Ok(all)
}

pub async fn support(
    State(data): State<SharedState>,
    Extension(soft_auth_ext): Extension<SoftAuthExtension>,
) -> Html<String> {
    let r = data.read().await;
    match get_resources(&r.db).await {
        Ok(mut resources) => {
            resources.append(&mut generate_support_resources());
            let template = SupportTemplate {
                resources,
                admin: soft_auth_ext.is_logged_in,
            };
            match template.render() {
                Ok(r) => Html(r),
                Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
            }
        }
        Err(err) => Html(format!("A database error occured: {:?}", err)),
    }
}
fn generate_support_resources() -> Vec<SupportResource> {
    let mut resources = Vec::new();

    let names = vec![
        "Red Cross",
        "Salvation Army",
        "Habitat for Humanity",
        "United Way",
        "Feeding America",
    ];

    let descriptions = vec![
        "Providing emergency assistance and disaster relief.",
        "Offering shelter, food, and social services.",
        "Building affordable housing and revitalizing communities.",
        "Supporting health, education, and financial stability programs.",
        "Fighting hunger and distributing food to those in need.",
    ];

    let missions = vec![
        vec!["Disaster Relief", "Blood Donation", "Health Services"],
        vec!["Homeless Services", "Rehabilitation", "Youth Programs"],
        vec![
            "Affordable Housing",
            "Home Repairs",
            "Neighborhood Revitalization",
        ],
        vec!["Education", "Income", "Health"],
        vec!["Food Pantries", "Meal Programs", "Nutrition Education"],
    ];

    let mut rng = rand::thread_rng();

    for _ in 0..5 {
        let name = names.choose(&mut rng).unwrap();
        let description = descriptions.choose(&mut rng).unwrap();
        let missions = missions
            .choose(&mut rng)
            .unwrap()
            .iter()
            .map(|m| m.to_string())
            .collect();
        let phone = if rng.gen_bool(0.7) {
            Some(format!(
                "1-800-{:03}-{:04}",
                rng.gen_range(100..999),
                rng.gen_range(1000..9999)
            ))
        } else {
            None
        };
        let email = if rng.gen_bool(0.8) {
            Some(format!(
                "info@{}.org",
                name.replace(" ", "_").to_lowercase()
            ))
        } else {
            None
        };
        let physical_address = if rng.gen_bool(0.6) {
            Some(Address {
                line_1: format!(
                    "{} Something {}",
                    rng.gen_range(100..999),
                    ["St", "Ave", "Blvd", "Rd"].choose(&mut rng).unwrap()
                ),
                line_2: None,
                city: ["New York", "Los Angeles", "Chicago", "Houston", "Phoenix"]
                    .choose(&mut rng)
                    .unwrap()
                    .to_string(),
                state: ["NY", "CA", "IL", "TX", "AZ"]
                    .choose(&mut rng)
                    .unwrap()
                    .to_string(),
                zip: format!("{:05}", rng.gen_range(10000..99999)),
            })
        } else {
            None
        };

        resources.push(SupportResource {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: description.to_string(),
            missions,
            phone,
            email,
            physical_address,
            website_url: Some(String::from("https://www.linkedin.com/feed/")),
        });
    }

    resources
}
