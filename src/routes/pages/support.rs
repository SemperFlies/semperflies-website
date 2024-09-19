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
            resources.append(&mut builtin_support_resources());
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

fn builtin_support_resources() -> Vec<SupportResource> {
    let motivational_marine = SupportResource {
        id: Uuid::new_v4(),
        name: "The Motivational Marine".to_string(),
        description: r#"The Motivational Marine is dedicated to empowering individuals to break free from the confines of their minds and fully engage with their lives. 
Using evidence-based knowledge, we provide insightful coaching that reveals the often-overlooked aspects of how our minds work. 
Understanding is the first step to improvement—because you can't change what you don't know exists. 
Our mission is to illuminate these hidden facets, enabling you to live with intention, purpose, and clarity.
        "#.to_string(),
        phone: Some("(260)-466-8929".to_string()),
        facebook: Some("https://www.facebook.com/themotivationalmarine?mibextid=LQQJ4d".to_string()),
        linkedin: Some("https://www.linkedin.com/in/briangagye?utm_source=share&utm_campaign=share_via&utm_content=profile&utm_medium=ios_app".to_string()),
    logo: Some(Image {
            src: "public/assets/images/support/motivational_marine.webp".to_string(),
            alt: "the motivational marine logo".to_string(),
            subtitle: String::new(),
        }),
        email: None,
        instagram: None,
        missions: vec![],
        twitter: None,
        threads: None,
        youtube: None,
        physical_address: None,
        website_url: None,

    };

    let mission_22 = SupportResource {
        id: Uuid::new_v4(),
        name: "Mission 22".to_string(),
        description: r#"Mission 22 provides support to Veterans and their families when they need it most: right now. Through a comprehensive approach of outreach, events, and programs, we’re promoting long-term wellness and sustainable growth."#.to_string(),
        physical_address: Some(Address {
                line_2: Some("#910".to_string()),
                line_1: "649 N Larch St".to_string(),
                city: "Sisters".to_string(),
                state: "OR".to_string(),
                zip: "97759".to_string(),
            }),
        phone: Some("(503)-908-8505".to_string()),
        website_url: Some("https://mission22.com/".to_string()),
        logo: Some(Image {
            src: "public/assets/images/support/mission_22.webp".to_string(),
            alt: "the mission 22 logo".to_string(),
            subtitle: String::new(),
        }),
    linkedin: None,
        email: None,
        instagram: None,
        missions: vec![],
        twitter: None,
        threads: None,
        youtube: None,
        facebook: None,
    };

    vec![motivational_marine, mission_22]
}
