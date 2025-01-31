use askama::Template;
// use askama::Template;
use axum::extract::State;
use axum::response::Html;
use axum::Extension;
use chrono::NaiveDate;
use jsonwebtoken::get_current_timestamp;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::auth::middleware::SoftAuthExtension;
use crate::components::carousel::{self, CarouselTemplate, HasCarousel, Image};
use crate::database::handles::DbData;
use crate::database::models::{DBDedication, DBDedicationParams, DBImage};
use crate::state::SharedState;

#[derive(Template, Debug)]
#[template(path = "pages/dedications.html")]
pub struct DedicationsTemplate {
    dedications: Vec<Dedication>,
    admin: bool,
}

impl HasCarousel for DedicationsTemplate {}
pub const DEDICATIONS: &str = "dedications";

#[derive(Debug)]
pub struct Dedication {
    pub id: uuid::Uuid,
    pub names: Vec<String>,
    pub bio: String,
    // insert
    pub birth: NaiveDate,
    // extract
    pub death: NaiveDate,
    pub carousel: CarouselTemplate,
}

impl From<(DBDedication, Vec<DBImage>)> for Dedication {
    fn from((ded, images): (DBDedication, Vec<DBImage>)) -> Self {
        let images: Vec<Image> = images
            .into_iter()
            .filter_map(|i| {
                if ded.img_ids.contains(&i.id) {
                    Some(i.into())
                } else {
                    None
                }
            })
            .collect();

        let carousel = CarouselTemplate {
            show_subtitles: false,
            auto_scroll: false,
            images,
        };
        Self {
            id: ded.id,
            names: ded.names,
            bio: ded.bio,
            birth: ded.birth,
            death: ded.death,
            carousel,
        }
    }
}

async fn get_dedications(pool: &Pool<Postgres>) -> anyhow::Result<Vec<Dedication>> {
    let all_deds_and_imgs =
        DBImage::get_multiple_with_images::<DBDedication, DBDedicationParams>(&pool).await?;
    let mut all_deds = vec![];
    for (ded, imgs) in all_deds_and_imgs {
        all_deds.push(Dedication::from((ded, imgs)));
    }
    Ok(all_deds)
}

#[tracing::instrument(name = "dedications page", skip_all)]
pub async fn dedications(
    State(data): State<SharedState>,
    Extension(soft_auth_ext): Extension<SoftAuthExtension>,
) -> Html<String> {
    let r = data.read().await;
    match get_dedications(&r.db).await {
        Ok(mut got_dedications) => {
            let mut dedications = crate::database::builtins::builtin_dedications();
            dedications.append(&mut got_dedications);
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
