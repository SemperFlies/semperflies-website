use askama::Template;
use axum::response::Html;
use chrono::{Date, NaiveDate};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use serde::Deserialize;

use crate::components::carousel::{self, CarouselTemplate, Image};

#[derive(Template, Debug)]
#[template(path = "pages/fallen_brothers.html")]
pub struct FallenBrosTemplate {
    dedications: Vec<Dedication>,
}

#[derive(Debug)]
pub struct Dedication {
    pub name: String,
    pub bio: String,
    // insert
    pub birth: NaiveDate,
    // extract
    pub death: NaiveDate,
    pub carousel: CarouselTemplate,
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
            images.push(Image::new(&image_urls[i], "alt"))
        }
        let carousel = CarouselTemplate { images };

        dedications.push(Dedication {
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
// #---#

pub async fn fallen_brothers() -> Html<String> {
    let template = FallenBrosTemplate {
        dedications: generate_dedications(15),
    };
    match template.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}

impl FallenBrosTemplate {
    fn render_carousel(carousel: &CarouselTemplate) -> String {
        carousel
            .render()
            .unwrap_or("error rendering carousel".to_owned())
    }
}
