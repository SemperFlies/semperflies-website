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
            dedications.append(&mut builtin_dedications());
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

fn builtin_dedications() -> Vec<Dedication> {
    let mileo = Dedication {
        id: Uuid::new_v4(),
        name: "Corporal Jason David Mileo".to_string(),
        birth: NaiveDate::from_ymd_opt(1982, 12, 14).unwrap(),
        death: NaiveDate::from_ymd_opt(2003, 4, 13).unwrap(),
        bio: r#"Corporal Jason David Mileo deployed to Iraq with 3rd Battalion 4th Marines in 2003. He fought along side his Marine Brothers during the Shock-N-Awe, the push on Baghdad, and he was in the city square when the statue of Saddam Hussein fell.
<br/>
<br/>
On April 14, 2003, Corporal Mileo bravely crawled into an elevated position on a night patrol so he could provide security over watch for his Marines. They were on a movement to contact patrol and had departed friendly lines with one thing in mind; contact. That evening there was an elevation in activity. Gunfire was being exchanged directly outside the walls of the Marines fortified position in downtown Baghdad. The gun fire continued intermittently throughout the late afternoon and into the dusk of night. Marine Scout Snipers (8541’s) from an elite unit were manning the most elevated position of the Marines stronghold. “The tragic death of Corporal Mileo was the result of several significant breakdowns in discipline, coordination and communication that set the stage for this horrific incident”.
<br/>
-Maj. Gen. J.N. Mattis, commander of the 1st Marine Division.
<br/>
<br/>
General Mattis also wrote:
<br/>
“Even though no one event or person was the catalyst for Corporal Mileo's death, one break in the chain of events may have spared his life." That night, Corporal Mileo was tragically mistaken for an enemy fighter and engaged by that Marine Scout Sniper Team. Everyone was doing what they were trained to do; believing he was an enemy target preparing a rooftop position, the snipers shot and killed him. “The devastation on the faces of every Marine that was present at his memorial the following morning can never be embodied in words. I’ve wished I can go back and say something, or I think I did.. I don’t remember. One second the memory is clear, the next it’s blank. But the faces, the faces of his Marine Brothers.. those will be burned into my mind. This moment redefined my entire life. The loss of that Warrior will have catastrophic effects on me for the rest of my life. I’ll never be able to leave that rooftop in my mind; life sentence.” -Marine Scout Sniper
<br/>
(Spotter/Jamie Martin Guajardo)
 "#.to_string(),
        carousel: CarouselTemplate { images: vec![Image {
            src: "public/assets/images/dedications/mileo.webp".to_string(),
            alt: "An image of a soldier".to_string(),
            subtitle: "".to_string(),
        }], auto_scroll: false, show_subtitles: false }
    };

    vec![mileo]
}

fn generate_dedications(amt: i32) -> Vec<Dedication> {
    let mut rng = thread_rng();
    let image_urls = vec![
        "public/assets/images/board_members/business.webp".to_string(),
        "public/assets/images/board_members/business2.webp".to_string(),
        "public/assets/images/board_members/old.".to_string(),
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
