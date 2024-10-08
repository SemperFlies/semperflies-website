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
        Ok(mut dedications) => {
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
        names:vec![ "Corporal Jason David Mileo".to_string()],
        birth: NaiveDate::from_ymd_opt(1982, 12, 14).unwrap(),
        death: NaiveDate::from_ymd_opt(2003, 4, 14).unwrap(),
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

    let fifth_platoon = Dedication {
        id: Uuid::new_v4(),
        names: vec![
            "SSgt Vincent Sabasteanski".to_string(),
            "SSgt David Galloway".to_string(),
            "SSgt Jeffrey Starling".to_string(),
            "Cpl Mark Baca".to_string(),
            "HM1 Jay Asis".to_string(),
            "GySgt James Paige".to_string(),
            "SSgt William Dame".to_string(),
        ],
        birth: NaiveDate::from_ymd_opt(1776, 11, 10).unwrap(),
        death: NaiveDate::from_ymd_opt(1999, 12, 9).unwrap(),
        bio: r#"On December 9, 1999 1st Force Reconnaissance Company suffered a major loss. A CH-46 was carrying 5th Platoon for a V.B.S.S (Visit Board Search Seizure). As the helicopter made the approach to the USNS Pecos the piolet became tangled in the netting causing it to flip upside down into the Pacific Ocean off the coast of Point Loma, Ca. This was a joint operation with the Navy SEALS. The SEALS had safety boats in the water and were able to rescue eleven survivors. The seven Warriors that lost their life’s that day paid the ultimate sacrifice in defense of our country. I still communicate with family of the fallen warriors. As a platoon we suffered mentally together and individually forever. The wives of the fallen Warriors showed us unmeasurable strength. Huge “Thank You” to the Navy SEALS for being so tactically proficient and bringing our Brothers aboard in the time of crisis."#.to_string(),
        carousel: CarouselTemplate {
            images: vec![Image {
                src: "public/assets/images/dedications/5th_platoon.webp".to_string(),
                alt: "a dedication to multiple solidiers".to_string(),
                subtitle: "".to_string(),
            }],
            auto_scroll: false,
            show_subtitles: false,
        },
    };

    let maxwell = Dedication {
        id: Uuid::new_v4(),
        names: vec![
            "Sergeant Jason Maxwel".to_string()
        ],
        birth: NaiveDate::from_ymd_opt(1978, 03, 04).unwrap(),
        death: NaiveDate::from_ymd_opt(2003, 10, 30).unwrap(),
        bio: r#"Sergeant Jason Maxwell was all heart & the epitome of a Force Recon Marine. I met him when we were standing by for the Iraq Invasion at Camp Commando in Kuwait. Our GP (general purpose) platoon size tents were right next to each other. Our platoons spent a lot of time together; it’s a small community anyway so a lot of us knew each other as a result from time in the unit. They were deployed to Iraq from Kāné Ohe Bay, Hawai’i, 4th Force Reconnaissance Co. Our platoon was out of Camp Pendleton, Ca, 1st Force Reconnaissance Co. After Combat Operations in Iraq our platoons returned back to our respective bases. I went out to Yuma, Az. to be an instructor at the Military Free Fall School, H.A.L.O. Shortly after becoming an instructor I looked up one day with the biggest smile and it was returned as Maxwell walked through the door to be a student. It was great to see him again; like I said small community. Maxwell did great progressing through the course. Another Force Recon Marine and myself were his instructors. Maxwell lost his life training to defend this country, an already accomplished Combat Veteran. He had a full malfunction on his parachute and left this life way too early. He will never be forgotten. 
RIP Warrior. 
S/F. 
ML&R. "#.to_string(),
        carousel: CarouselTemplate {
            images: vec![Image {
                src: "public/assets/images/dedications/maxwell.webp".to_string(),
                alt: "a dedication to maxwell".to_string(),
                subtitle: "".to_string(),
            }],
            auto_scroll: false,
            show_subtitles: false,
        },
    };
    vec![mileo, fifth_platoon, maxwell]
}
