use anyhow::anyhow;
use askama::Template;
use axum::{extract::State, response::Html, Extension};
use rand::prelude::*;
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use tracing::warn;
use uuid::Uuid;

use crate::{
    auth::middleware::SoftAuthExtension,
    components::carousel::Image,
    database::{
        handles::DbData,
        models::{DBAddress, DBImage, DBResource, DBResourceParams},
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
    pub logo: Option<Image>,
    pub description: String,
    pub missions: Vec<String>,
    pub phone: Option<String>,
    pub website_url: Option<String>,
    pub email: Option<String>,
    pub physical_address: Option<Address>,
    pub twitter: Option<String>,
    pub facebook: Option<String>,
    pub youtube: Option<String>,
    pub linkedin: Option<String>,
    pub threads: Option<String>,
    pub instagram: Option<String>,
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

impl From<(DBResource, Option<DBAddress>, Vec<DBImage>)> for SupportResource {
    fn from((res, add, imgs): (DBResource, Option<DBAddress>, Vec<DBImage>)) -> Self {
        if imgs.len() > 1 {
            warn!("this resource has more than one image, taking the 0th");
        }
        Self {
            id: res.id,
            logo: imgs.first().and_then(|dbimg| Some(dbimg.to_owned().into())),
            name: res.name,
            description: res.description,
            missions: res
                .missions
                .into_iter()
                .filter_map(|m| if m.trim().is_empty() { None } else { Some(m) })
                .collect(),
            phone: res.phone,
            email: res
                .email
                .and_then(|s| if s.trim().is_empty() { None } else { Some(s) }),
            website_url: res
                .website_url
                .and_then(|s| if s.trim().is_empty() { None } else { Some(s) }),
            physical_address: add.and_then(|a| Some(Address::from(a))),
            instagram: res
                .instagram
                .and_then(|s| if s.trim().is_empty() { None } else { Some(s) }),
            threads: res
                .threads
                .and_then(|s| if s.trim().is_empty() { None } else { Some(s) }),
            youtube: res
                .youtube
                .and_then(|s| if s.trim().is_empty() { None } else { Some(s) }),
            facebook: res
                .facebook
                .and_then(|s| if s.trim().is_empty() { None } else { Some(s) }),
            linkedin: res
                .linkedin
                .and_then(|s| if s.trim().is_empty() { None } else { Some(s) }),
            twitter: res
                .twitter
                .and_then(|s| if s.trim().is_empty() { None } else { Some(s) }),
        }
    }
}

async fn get_resources(pool: &Pool<Postgres>) -> anyhow::Result<Vec<SupportResource>> {
    let all_res_and_imgs =
        DBImage::get_multiple_with_images::<DBResource, DBResourceParams>(&pool).await?;
    let mut all = vec![];

    for (r, imgs) in all_res_and_imgs {
        let mut address = Option::<DBAddress>::None;
        if let Some(id) = r.address_id {
            let add = DBAddress::get_single_by(pool, id)
                .await?
                .ok_or(anyhow!("no address with id: {:?}", id))?;
            address = Some(add);
        }
        all.push(SupportResource::from((r, address, imgs)))
    }
    warn!("returning resources from database: {all:?}");

    Ok(all)
}

pub async fn support(
    State(data): State<SharedState>,
    Extension(soft_auth_ext): Extension<SoftAuthExtension>,
) -> Html<String> {
    let r = data.read().await;
    match get_resources(&r.db).await {
        Ok(mut resources) => {
            resources.append(&mut crate::database::builtins::builtin_support_resources());
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
