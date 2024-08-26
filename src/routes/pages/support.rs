use anyhow::anyhow;
use askama::Template;
use axum::{extract::State, response::Html, Extension};
use rand::prelude::*;
use serde::Deserialize;
use sqlx::{Pool, Postgres};

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
        Ok(resources) => {
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
