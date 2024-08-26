use askama::Template;
use axum::extract::State;
use axum::response::Html;
use axum::Extension;
use chrono::{Date, NaiveDate};
use jsonwebtoken::get_current_timestamp;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use serde::Deserialize;
use sqlx::{Pool, Postgres};

use crate::auth::middleware::SoftAuthExtension;
use crate::components::carousel::{self, CarouselTemplate, Image};
use crate::database::handles::DbData;
use crate::database::models::DBDedication;
use crate::state::SharedState;

#[derive(Template, Debug)]
#[template(path = "pages/dedications.html")]
pub struct DedicationsTemplate {
    dedications: Vec<Dedication>,
    admin: bool,
}

pub const DEDICATIONS: &str = "dedications";

#[derive(Debug)]
pub struct Dedication {
    pub id: uuid::Uuid,
    pub name: String,
    pub bio: String,
    // insert
    pub birth: NaiveDate,
    // extract
    pub death: NaiveDate,
    pub carousel: CarouselTemplate,
}

impl From<DBDedication> for Dedication {
    fn from(ded: DBDedication) -> Self {
        let carousel = CarouselTemplate {
            images: ded
                .img_urls
                .into_iter()
                .map(|url| Image {
                    src: url,
                    alt: String::new(),
                })
                .collect(),
        };
        Self {
            id: ded.id,
            name: ded.name,
            bio: ded.bio,
            birth: ded.birth,
            death: ded.death,
            carousel,
        }
    }
}

async fn get_dedications(pool: &Pool<Postgres>) -> anyhow::Result<Vec<Dedication>> {
    let dbdeds = DBDedication::get_multiple(pool).await?;
    Ok(dbdeds
        .into_iter()
        .map(|ded| Dedication::from(ded))
        .collect())
}

pub async fn dedications(
    State(data): State<SharedState>,
    Extension(soft_auth_ext): Extension<SoftAuthExtension>,
) -> Html<String> {
    let r = data.read().await;
    match get_dedications(&r.db).await {
        Ok(dedications) => {
            let template = DedicationsTemplate {
                dedications,
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

impl DedicationsTemplate {
    fn render_carousel(carousel: &CarouselTemplate) -> String {
        carousel
            .render()
            .unwrap_or("error rendering carousel".to_owned())
    }
}
