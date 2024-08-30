use askama::Template;
// use askama::Template;
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
    pub name: String,
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
            name: ded.name,
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
        Ok(mut dedications) => {
            dedications.append(&mut generate_dedications(10));
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

fn generate_dedications(amt: i32) -> Vec<Dedication> {
    let mut rng = thread_rng();
    let image_urls = vec![
        "public/assets/images/board_members/business.jpg".to_string(),
        "public/assets/images/board_members/business2.jpg".to_string(),
        "public/assets/images/board_members/old.jpg".to_string(),
    ];

    let mut dedications = Vec::new();

    for _ in 0..amt {
        let name = format!("Veteran {}", dedications.len() + 1);
        let bio = "Served in the military".to_string();
        let birth_year = rng.gen_range(1950..1990);
        let birth_month = rng.gen_range(1..13);
        let birth_day = rng.gen_range(1..29);
        let birth = NaiveDate::from_ymd_opt(birth_year, birth_month, birth_day).unwrap();

        let death_year = birth_year + rng.gen_range(20..70);
        let death_month = rng.gen_range(1..13);
        let death_day = rng.gen_range(1..29);
        let death = NaiveDate::from_ymd_opt(death_year, death_month, death_day).unwrap();

        let amt_imgs = rng.gen_range(0..=3);
        let mut images = vec![];
        for i in 0..amt_imgs {
            images.push(Image {
                src: image_urls[i].to_owned(),
                alt: "".to_string(),
                subtitle: "".to_string(),
            })
        }
        let carousel = CarouselTemplate {
            show_subtitles: false,
            images,
            auto_scroll: false,
        };

        dedications.push(Dedication {
            id: uuid::Uuid::new_v4(),
            name,
            bio,
            birth,
            death,
            carousel,
            // images,
        });
    }

    dedications
}
